import { test, expect, factory, poll } from '../../fixtures/test'
import { expectValidGatewayResponse, expectValidScoreUpdate } from '../../helpers/assertions'

/**
 * `/update-gateway-score` must record a transaction outcome for whichever layer picked the gateway.
 *
 * The regression these guard: the SRv3 producer admission check used to read the persisted
 * `routing_approach` text for a `"HEDGING"` / `"MULTI_OBJECTIVE"` substring, so any payment routed
 * by rules (`PRIORITY_LOGIC`), by the network, or by merchant preference was classified as an
 * on-policy SRv3 exploit. For a merchant with explore/exploit enabled its success and its failure
 * were then both dropped and the gateway's score never moved.
 *
 * How the score is observed: `/decide-gateway` with `rankingAlgorithm: SR_BASED_ROUTING` reads the
 * SRv3 moving window and returns it as `gateway_priority_map`, so a penalised gateway shows up as a
 * score below 1.0. Each test therefore drives a *rule-routed* decision (no ranking algorithm, so
 * the decider falls back to priority logic), reports its outcome, and reads the score back through
 * an SR-based decision.
 *
 * `explore-exploit-srv3` is toggled on because that is the merchant-facing switch that used to
 * swallow the feedback; `sr_v3_producer_isolation` has no API toggle and stays off here.
 */
test.describe('Gateway score feedback by routing approach (API)', () => {
  /** `/decide-gateway` with no ranking algorithm: the decider falls back to priority logic. */
  const ruleRoutedDecide = async (api: any, merchantId: string, paymentId: string) => {
    const response = await api.raw('POST', '/decide-gateway', {
      body: {
        merchantId: merchantId,
        eligibleGatewayList: factory.connectorNames('stripe', 'adyen'),
        eliminationEnabled: true,
        paymentInfo: factory.paymentInfo({
          paymentId,
          paymentMethodType: 'CARD',
          paymentMethod: 'VISA',
        }),
      },
    })
    expectValidGatewayResponse(response.body)
    // Precondition: this really is a rule-based (priority-logic) decision, not an SR one.
    expect(response.body.routing_approach).toBe('PRIORITY_LOGIC')
    return response.body
  }

  /**
   * `/decide-gateway` on the SRv3 path, which reads back the moving window the feedback wrote.
   * Hedging must be 0 for this to be a plain read: under exploration the decider replaces the
   * scores with 1.0 rather than reading them.
   */
  const srScoreDecision = async (api: any, merchantId: string) => {
    const response = await api.raw('POST', '/decide-gateway', {
      body: factory.srDecideGatewayRequest({
        merchantId,
        eligibleGatewayList: factory.connectorNames('stripe', 'adyen'),
        paymentInfo: { paymentMethodType: 'CARD', paymentMethod: 'VISA' },
      }),
    })
    expectValidGatewayResponse(response.body)
    return response
  }

  const scoresOf = (response: { body: any }) => response.body.gateway_priority_map as Record<string, number>

  const lowestOf = (scores: Record<string, number>, gateways: Set<string>) =>
    Math.min(...[...gateways].map((gateway) => scores[gateway]))

  /**
   * One decide + one feedback call. Reports the outcome for the gateway the decision actually
   * picked, which is what a real orchestrator does.
   */
  const decideThenReport = async (
    api: any,
    merchantId: string,
    status: 'FAILURE' | 'CHARGED',
    prefix: string,
  ) => {
    const paymentId = factory.paymentId(prefix)
    const decision = await ruleRoutedDecide(api, merchantId, paymentId)
    const update = await api.updateGatewayScore({
      merchantId,
      gateway: decision.decided_gateway,
      paymentId,
      status,
    })
    expectValidScoreUpdate(update.body)
    return decision.decided_gateway as string
  }

  // A 4-slot window means one outcome moves a gateway's score by a quarter, so a single outcome is
  // already visible and four consecutive successes flush a window back to 1.0.
  const BUCKET = { defaultBucketSize: 4, defaultHedgingPercent: 0 }

  test('a rule-routed failure penalises the gateway score', async ({ api, merchant }) => {
    const m = merchant.id
    await api.setMerchantFeature(m, 'explore-exploit-srv3', true)
    await api.createSuccessRateConfig(m, BUCKET)

    const failed = new Set<string>()
    for (let i = 0; i < 3; i++) {
      failed.add(await decideThenReport(api, m, 'FAILURE', `rule_fail_${i}`))
    }

    const settled = await poll(
      () => srScoreDecision(api, m),
      (response) => lowestOf(scoresOf(response), failed) < 1.0,
      {
        message: `Expected a rule-routed failure to penalise one of ${[...failed].join(', ')}`,
        timeout: 15_000,
        interval: 1_000,
      },
    )
    expect(lowestOf(scoresOf(settled), failed)).toBeLessThan(1.0)
  })

  test('a rule-routed success rewards the gateway score', async ({ api, merchant }) => {
    const m = merchant.id
    await api.setMerchantFeature(m, 'explore-exploit-srv3', true)
    await api.createSuccessRateConfig(m, BUCKET)

    // Drive the scores down first, so a reward is observable as a rise.
    const penalised = new Set<string>()
    for (let i = 0; i < 3; i++) {
      penalised.add(await decideThenReport(api, m, 'FAILURE', `rule_penalise_${i}`))
    }
    const afterFailures = await poll(
      () => srScoreDecision(api, m),
      (response) => lowestOf(scoresOf(response), penalised) < 1.0,
      {
        message: `Expected rule-routed failures to penalise one of ${[...penalised].join(', ')}`,
        timeout: 15_000,
        interval: 1_000,
      },
    )
    const total = (response: { body: any }) =>
      [...penalised].reduce((sum, gateway) => sum + scoresOf(response)[gateway], 0)
    const penalisedTotal = total(afterFailures)

    // Successes then have to push the same scores back up: each one raises its own gateway by a
    // quarter of a window, and no window here is full of failures, so the total has to grow.
    for (let i = 0; i < 6; i++) {
      await decideThenReport(api, m, 'CHARGED', `rule_reward_${i}`)
    }

    const afterSuccesses = await poll(
      () => srScoreDecision(api, m),
      (response) => total(response) > penalisedTotal,
      {
        message: 'Expected rule-routed successes to reward the gateway scores',
        timeout: 15_000,
        interval: 1_000,
      },
    )
    expect(total(afterSuccesses)).toBeGreaterThan(penalisedTotal)
  })

  /**
   * The unchanged half of the contract: an SRv3 payment the scorer exploited on-policy still does
   * NOT feed the producer while explore/exploit is on, because feeding it would bias the very
   * estimates the scorer relies on.
   */
  test('an on-policy SRv3 failure is still not recorded under explore/exploit', async ({ api, merchant }) => {
    const m = merchant.id
    await api.setMerchantFeature(m, 'explore-exploit-srv3', true)
    // Hedging 0 => every SRv3 decision is a plain exploit (SR_SELECTION_V3_ROUTING).
    await api.createSuccessRateConfig(m, BUCKET)

    // Control: prove feedback for this merchant does land, so "unchanged" below cannot pass merely
    // because nothing was recorded in the first place.
    const control = await decideThenReport(api, m, 'FAILURE', 'control')
    await poll(
      () => srScoreDecision(api, m),
      (response) => scoresOf(response)[control] < 1.0,
      {
        message: 'Expected the control rule-routed failure to be recorded',
        timeout: 15_000,
        interval: 1_000,
      },
    )

    const before = scoresOf(await srScoreDecision(api, m))

    for (let i = 0; i < 3; i++) {
      const paymentId = factory.paymentId(`srv3_exploit_${i}`)
      const decide = await api.decideGateway(
        factory.srDecideGatewayRequest({
          merchantId: m,
          eligibleGatewayList: factory.connectorNames('stripe', 'adyen'),
          paymentInfo: { paymentId, paymentMethodType: 'CARD', paymentMethod: 'VISA' },
        }),
      )
      expectValidGatewayResponse(decide.body)
      expect(decide.body.routing_approach).toBe('SR_SELECTION_V3_ROUTING')
      const update = await api.updateGatewayScore({
        merchantId: m,
        gateway: decide.body.decided_gateway,
        paymentId,
        status: 'FAILURE',
      })
      expectValidScoreUpdate(update.body)
    }

    // Nothing should have moved. Settle for longer than the control needed before concluding so.
    await new Promise((resolve) => setTimeout(resolve, 5_000))

    expect(scoresOf(await srScoreDecision(api, m))).toEqual(before)
  })

  /**
   * ...while an SRv3 *hedged* payment — a genuine explore sample — still is recorded, so the
   * explore gate keeps admitting what it used to.
   */
  test('a hedged SRv3 failure is still recorded under explore/exploit', async ({ api, merchant }) => {
    const m = merchant.id
    await api.setMerchantFeature(m, 'explore-exploit-srv3', true)
    // Hedging 100 => every SRv3 decision is an explore sample (SR_V3_HEDGING).
    await api.createSuccessRateConfig(m, { defaultBucketSize: 4, defaultHedgingPercent: 100 })

    const failed = new Set<string>()
    for (let i = 0; i < 3; i++) {
      const paymentId = factory.paymentId(`srv3_hedge_${i}`)
      const decide = await api.decideGateway(
        factory.srDecideGatewayRequest({
          merchantId: m,
          eligibleGatewayList: factory.connectorNames('stripe', 'adyen'),
          paymentInfo: { paymentId, paymentMethodType: 'CARD', paymentMethod: 'VISA' },
        }),
      )
      expectValidGatewayResponse(decide.body)
      expect(decide.body.routing_approach).toBe('SR_V3_HEDGING')
      const update = await api.updateGatewayScore({
        merchantId: m,
        gateway: decide.body.decided_gateway,
        paymentId,
        status: 'FAILURE',
      })
      expectValidScoreUpdate(update.body)
      failed.add(decide.body.decided_gateway)
    }

    // Read the window with hedging off, otherwise the explore path replaces the scores with 1.0.
    await api.updateSuccessRateConfig(m, BUCKET)
    const settled = await poll(
      () => srScoreDecision(api, m),
      (response) => lowestOf(scoresOf(response), failed) < 1.0,
      {
        message: `Expected a hedged SRv3 failure to penalise one of ${[...failed].join(', ')}`,
        timeout: 15_000,
        interval: 1_000,
      },
    )
    expect(lowestOf(scoresOf(settled), failed)).toBeLessThan(1.0)
  })
})
