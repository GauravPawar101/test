// Automatically converted from Haskell to Rust
// Generated on 2025-03-23 10:24:42

use crate::redis::cache::findByNameFromRedis;
use crate::redis::feature::{is_feature_enabled, RedisCompressionConfigCombined};
use masking::PeekInterface;
// Converted imports
// use gateway_decider::constants as c::{enable_elimination_v2, gateway_scoring_data, EnableExploreAndExploitOnSrv3, SrV3InputConfig, GatewayScoreFirstDimensionSoftTtl};
// use feedback::constants as c;
// use data::text::encoding as de::encode_utf8;
// use db::storage::types::merchant_account as merchant_account;
// use types::gateway_routing_input as etgri;
// use gateway_decider::utils::decode_and_log_error;
use crate::decider::gatewaydecider::gw_scoring::get_metric_entry_data;
// use feedback::utils as euler_transforms;
// use feedback::types::*;
// use feedback::types::txn_card_info;
// use eulerhs::prelude::*;
// use eulerhs::language::get_current_date_in_millis;
// use data::text as t;
// use feedback::utils::*;
// use feedback::gateway_selection_scoring_v3::flow;
// use feedback::gateway_elimination_scoring::flow;
// use eulerhs::language as el;
// use eulerhs::types as et;
// use eulerhs::tenant_redis_layer as rc;
use crate::app::get_tenant_app_state;
use crate::decider::gatewaydecider::constants::{self as DC, SR_V3_DEFAULT_INPUT_CONFIG};
use crate::decider::gatewaydecider::types as T;
use crate::decider::gatewaydecider::types::GatewayDeciderApproach;
use crate::decider::gatewaydecider::types::GatewayScoringData;
use crate::decider::gatewaydecider::types::{RoutingFlowType as RF, SrRoutingDimensions};
use crate::decider::gatewaydecider::utils::{
    self as GU, get_m_id, get_payment_method, get_sr_v3_latency_threshold,
};
use crate::feedback::gateway_selection_scoring_v3 as GSSV3;
use crate::feedback::types as FT;
use crate::feedback::utils as Fbu;
use crate::feedback::utils::GatewayScoringType as GST;
use crate::merchant_config_util as MerchantConfig;
use crate::redis::{feature as Cutover, types::ServiceConfigKey};
use crate::types::card::txn_card_info::TxnCardInfo;
use crate::types::gateway_routing_input::GatewaySuccessRateBasedRoutingInput;
use crate::types::merchant::id as MID;
use crate::types::merchant::merchant_account as MA;
use crate::types::merchant::merchant_account::MerchantAccount;
use std::collections::HashMap;
// use utils::redis::feature as cutover::is_feature_enabled;
// use prelude::{from_integral, foldable::length, map_m, error};
// use data::foldable::{for_, foldl};
// use data::text::is_infix_of;
// use data::byte_string::lazy as bsl;
// use data::text::encoding as te;
// use control::monad::extra::maybe_m;
// use control::category;
use crate::types::merchant as ETM;
// use utils::redis as redis;
// use db::common::types::payment_flows as pf;
// use utils::config::merchant_config as merchant_config;
// use gateway_decider::utils as gu::{get_sr_v3_latency_threshold, get_payment_method};
// use gateway_decider::types::{routing_flow_type, gateway_scoring_data};
// use gateway_decider::types as update_status;
// use types::tenant_config as tenant_config;
// use prelude::float;
// use utils::config::service_configuration as sc;
// use feedback::utils::*;
// use eulerhs::language as l;
// use data::aeson as a;
// use types::merchant as etm;

use crate::redis::feature::RedisDataStruct;
use crate::{
    feedback::{
        constants as C,
        gateway_elimination_scoring::flow as GEF,
        utils::{isPennyMandateRegTxn, isRecurringTxn, GatewayScoringType},
    },
    types::txn_details::types::{TransactionLatency, TxnDetail, TxnStatus, TxnStatus as TS},
};
use fred::types::SetOptions;
use serde::{Deserialize, Serialize};

use super::constants::{
    default_sr_v3_latency_threshold_in_secs, GsmBasedScoringFilterEnabledMerchant, SrV3InputConfig,
    UpdateGatewayScoreLockFlagTtl, UpdateScoreLockFeatureEnabledMerchant,
};
use super::utils::get_time_from_txn_created_in_mills;
use crate::logger;
use crate::types::payment::payment_method_type_const::*;
// Converted data types
// Original Haskell data type: GatewayLatencyForScoring
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GatewayLatencyForScoring {
    #[serde(rename = "default_latency_threshold")]
    pub default_latency_threshold: f64,

    #[serde(rename = "merchant_latency_gateway_wise_input")]
    pub merchant_latency_gateway_wise_input: Option<Vec<GatewayWiseLatencyInput>>,
}

// Original Haskell data type: GatewayWiseLatencyInput
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GatewayWiseLatencyInput {
    #[serde(rename = "gateway")]
    pub gateway: String,

    #[serde(rename = "paymentMethodType")]
    pub paymentMethodType: Option<String>,

    #[serde(rename = "paymentMethod")]
    pub paymentMethod: Option<String>,

    #[serde(rename = "latencyThreshold")]
    pub latencyThreshold: f64,
}

// Original Haskell data type: UpdateGatewayScoreRequest
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UpdateGatewayScoreRequest {
    #[serde(rename = "command")]
    pub command: GatewayScoringType,

    #[serde(rename = "txn_detail_id")]
    pub txn_detail_id: Option<String>,

    #[serde(rename = "txn_id")]
    pub txn_id: Option<String>,

    #[serde(rename = "merchant_id")]
    pub merchant_id: Option<String>,

    #[serde(rename = "order_id")]
    pub order_id: Option<String>,

    #[serde(rename = "txn_status")]
    pub txn_status: TxnStatus,
}

// Original Haskell data type: MetricEntry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricEntry {
    #[serde(rename = "n_value")]
    pub n_value: f32,

    #[serde(rename = "success_rate")]
    pub success_rate: f32,

    #[serde(rename = "sigma_factor")]
    pub sigma_factor: f32,

    #[serde(rename = "average_latency")]
    pub average_latency: f32,

    #[serde(rename = "tp99_latency")]
    pub tp99_latency: f32,

    #[serde(rename = "default_success_threshold")]
    pub default_success_threshold: f32,
}

// Original Haskell data type: SrMetrics
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SrMetrics {
    #[serde(rename = "dimension")]
    pub dimension: String,

    #[serde(rename = "value")]
    pub value: MetricEntry,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MerchantSrMetrics {
    #[serde(rename = "merchant_id")]
    pub merchant_id: String,

    #[serde(rename = "sr_metrics")]
    pub sr_metrics: Vec<SrMetrics>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResetGatewayScoreRequest {
    #[serde(rename = "gateway")]
    pub gateway: String,

    #[serde(rename = "eliminationThreshold")]
    pub eliminationThreshold: f64,

    #[serde(rename = "gatewayEliminationThreshold")]
    pub gatewayEliminationThreshold: Option<f64>,

    #[serde(rename = "eliminationMaxCount")]
    pub eliminationMaxCount: i32,

    #[serde(rename = "gatewayReferenceId")]
    pub gatewayReferenceId: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResetGatewayScoreBulkRequest {
    #[serde(rename = "txn_detail_id")]
    pub txn_detail_id: Option<String>,

    #[serde(rename = "txn_id")]
    pub txn_id: Option<String>,

    #[serde(rename = "merchant_id")]
    pub merchant_id: Option<String>,

    #[serde(rename = "order_id")]
    pub order_id: Option<String>,

    #[serde(rename = "resetGatewayScoreReqArr")]
    pub reset_gateway_score_req_arr: Vec<ResetGatewayScoreRequest>,
}

#[derive(Debug, Serialize)]
struct GatewayScoreAnalyticsDetails<'a> {
    routing_approach: String,
    gateway_scoring_type: String,
    message: &'a str,
}

fn serialize_gateway_score_analytics_details(
    routing_approach: &impl std::fmt::Debug,
    gateway_scoring_type: &impl std::fmt::Debug,
    message: &'static str,
) -> Option<String> {
    crate::analytics::serialize_details(&GatewayScoreAnalyticsDetails {
        routing_approach: format!("{routing_approach:?}"),
        gateway_scoring_type: format!("{gateway_scoring_type:?}"),
        message,
    })
}

pub fn default_gw_latency_check_in_mins() -> GatewayLatencyForScoring {
    GatewayLatencyForScoring {
        default_latency_threshold: 10.0,
        merchant_latency_gateway_wise_input: None,
    }
}

pub fn txn_success_states() -> Vec<TxnStatus> {
    vec![
        TS::Charged,
        TS::Authorized,
        TS::CODInitiated,
        TS::Voided,
        TS::VoidInitiated,
        TS::CaptureInitiated,
        TS::CaptureFailed,
        TS::VoidFailed,
        TS::AutoRefunded,
        TS::PartialCharged,
        TS::ToBeCharged,
    ]
}

pub fn txn_failure_states() -> Vec<TxnStatus> {
    vec![
        TxnStatus::AuthenticationFailed,
        TxnStatus::AuthorizationFailed,
        TxnStatus::JuspayDeclined,
        TxnStatus::Failure,
    ]
}

pub async fn check_and_send_should_update_gateway_score(
    lock_key: String,
    lock_key_ttl: i32,
    redis_comp_config: Option<RedisCompressionConfigCombined>,
) -> bool {
    let app_state = get_tenant_app_state().await;
    let is_set_either = app_state
        .redis_conn
        .setXWithOption(
            lock_key.as_str(),
            "true",
            lock_key_ttl as i64,
            SetOptions::NX,
            redis_comp_config,
            RedisDataStruct::STRING,
        )
        .await;

    is_set_either.unwrap_or(false)
}

pub fn is_transaction_success(txn_status: TxnStatus) -> bool {
    txn_success_states().contains(&txn_status)
}

pub fn is_transaction_failure(txn_status: TxnStatus) -> bool {
    txn_failure_states().contains(&txn_status)
}

pub fn isGwLatencyWithinConfiguredThreshold(
    txn_latency: Option<f64>,
    merchant_latency_threshold: Option<f64>,
) -> bool {
    logger::debug!(
        action = "txn_latency_within_threshold",
        tag = "txn_latency_within_threshold",
        "Latency & Threshold: {:?} {:?}",
        txn_latency,
        merchant_latency_threshold
    );
    if let Some((latency, threshold)) = txn_latency.zip(merchant_latency_threshold) {
        latency <= threshold
    } else {
        true
    }
}

pub async fn get_gateway_scoring_type(
    txn_detail: TxnDetail,
    txn_card_info: TxnCardInfo,
    flag: bool,
) -> GatewayScoringType {
    if flag {
        return GatewayScoringType::PenaliseSrv3;
    }

    let txn_status = txn_detail.status.clone();
    let merchant_id = txn_detail.merchantId.clone();
    let is_success = is_transaction_success(txn_status.clone());
    let is_failure = is_transaction_failure(txn_status.clone());
    let time_difference = get_time_from_txn_created_in_mills(txn_detail.clone());
    let merchant_sr_v3_input_config =
        findByNameFromRedis(SrV3InputConfig(get_m_id(merchant_id)).get_key()).await;
    let pmt = txn_card_info.paymentMethodType;
    let pm = get_payment_method(
        pmt.to_string(),
        txn_card_info.paymentMethod,
        txn_detail.sourceObject.unwrap_or_default(),
    );
    // Extract the new parameters from txn_card_info

    let sr_routing_dimensions = SrRoutingDimensions {
        card_network: txn_card_info
            .cardSwitchProvider
            .as_ref()
            .map(|s| s.peek().to_string()),
        card_isin: txn_card_info.card_isin.clone(),
        currency: Some(txn_detail.currency.to_string()),
        country: txn_detail.country.as_ref().map(|c| c.to_string()),
        auth_type: txn_card_info.authType.as_ref().map(|a| a.to_string()),
    };

    let maybe_latency_threshold = get_sr_v3_latency_threshold(
        merchant_sr_v3_input_config.clone(),
        &pmt,
        &pm,
        &sr_routing_dimensions,
    );

    let time_difference_threshold = match maybe_latency_threshold {
        None => {
            let default_sr_v3_input_config =
                findByNameFromRedis(SR_V3_DEFAULT_INPUT_CONFIG.get_key()).await;
            let maybe_default_latency_threshold = get_sr_v3_latency_threshold(
                default_sr_v3_input_config,
                &pmt,
                &pm,
                &sr_routing_dimensions,
            );
            maybe_default_latency_threshold.unwrap_or(default_sr_v3_latency_threshold_in_secs())
        }
        Some(latency_threshold) => latency_threshold,
    };

    logger::debug!(
        action = "sr_v3_latency_threshold",
        tag = "sr_v3_latency_threshold",
        "Latency Threshold: {} Time Difference: {}",
        time_difference_threshold,
        time_difference
    );

    if is_success {
        GatewayScoringType::Reward
    } else if is_failure {
        GatewayScoringType::PenaliseSrv3
    } else if time_difference < ((time_difference_threshold * 1000.0) as u128) {
        GatewayScoringType::Penalise
    } else {
        GatewayScoringType::PenaliseSrv3
    }
}

pub fn update_gateway_score_lock(
    gateway_scoring_type: GatewayScoringType,
    txn_uuid: String,
    gateway: String,
) -> String {
    match gateway_scoring_type {
        GatewayScoringType::Penalise => {
            format!("gateway_scores_lock_PENALISE_{}_{}", txn_uuid, gateway)
        }
        GatewayScoringType::PenaliseSrv3 => {
            format!("gateway_scores_lock_PENALISE_SRV3_{}_{}", txn_uuid, gateway)
        }
        GatewayScoringType::Reward => {
            format!("gateway_scores_lock_REWARD_{}_{}", txn_uuid, gateway)
        }
    }
}

/// Returns true when the GSM rule indicates the failure is user/issuer-originated
/// and the gateway itself is healthy, meaning the gateway should not be penalized.
///
/// Matching is done on `unified_message` (the UE-code family):
///
///   "Issue with Payment Method details" (UE_1000) – card/user error: expired card,
///     wrong CVV, insufficient funds, blocked card, FRM decline on card data.
///     The gateway processed the request correctly — penalising it would be wrong.
///
///   All other unified_messages (UE_2000 Issue with Configurations, UE_3000 Technical
///   issue with PSP, UE_4000 Issue with Integration, "Something went wrong") indicate
///   a gateway, config, or integration fault and receive full penalty.
///
/// step_up_possible == true overrides everything: the gateway is functional but the
/// transaction needs a 3-D Secure step-up.
fn is_gateway_healthy_failure(gsm_info: &crate::gsm::GsmInfo) -> bool {
    if gsm_info.step_up_possible {
        return true;
    }
    matches!(
        gsm_info.unified_message.as_deref(),
        Some("Issue with Payment Method details")
    )
}

pub fn invalid_request_error(detail: &str, e: &impl std::fmt::Display) -> T::ErrorResponse {
    T::ErrorResponse {
        status: "400".to_string(),
        error_code: "INVALID_REQUEST".to_string(),
        error_message: format!("Failed to extract {}: {}", detail, e),
        priority_logic_tag: None,
        routing_approach: None,
        filter_wise_gateways: None,
        error_info: T::UnifiedError {
            code: "INVALID_REQUEST".to_string(),
            user_message: "Invalid request data provided".to_string(),
            developer_message: format!("Error extracting {}: {}", detail, e),
        },
        priority_logic_output: None,
        is_dynamic_mga_enabled: false,
    }
}

/// What `check_and_update_gateway_score_` returns when the payment has no scoring context (it was
/// never routed by the decider, and no context was stored for it), so no score was updated.
pub const SCORE_UPDATE_SKIPPED: &str = "Skipped";

pub async fn check_and_update_gateway_score_(
    api_payload: FT::UpdateScorePayload,
) -> Result<String, T::ErrorResponse> {
    // Emit AB test outcome unconditionally — before any early returns.
    // emit_if_in_flight only needs payment_id, merchant_id, and is_success; it does
    // not depend on gateway scoring data or GSM lookups, so it must not be gated on
    // either. Moving it here ensures the outcome is recorded even when:
    //   (a) the GSM scoring filter skips gateway penalisation, or
    //   (b) GatewayScoringData is absent from Redis (expired/not cached).
    let is_success = is_transaction_success(api_payload.status.clone());
    crate::decider::gatewaydecider::ab_test::emit_if_in_flight(
        &api_payload.payment_id,
        &api_payload.merchant_id,
        is_success,
    )
    .await;

    // GSM-based scoring filter: skip penalization for failures where the gateway
    // is healthy (user/issuer-originated errors). Gated per merchant so it can be
    // rolled back instantly without a deploy.
    if let Some(error_info) = api_payload.error_info.as_ref() {
        let gsm_filter_enabled = is_feature_enabled(
            GsmBasedScoringFilterEnabledMerchant.get_key(),
            api_payload.merchant_id.clone(),
            C::kvRedis(),
        )
        .await;
        logger::debug!(
            action = "GSM_SCORING_FILTER_CHECK",
            tag = "GSM_SCORING_FILTER_CHECK",
            "GSM scoring filter: enabled={} merchant={} gateway={} flow={} sub_flow={} \
             error_code={:?} error_message={:?}",
            gsm_filter_enabled,
            api_payload.merchant_id,
            api_payload.gateway,
            error_info.flow,
            error_info.sub_flow,
            error_info.error_code,
            error_info.error_message,
        );
        if gsm_filter_enabled {
            // Connector must be the gateway that processed the payment.
            let effective_error_info = crate::gsm::GsmErrorInfo {
                connector: api_payload.gateway.clone(),
                ..error_info.clone()
            };
            let gsm_lookup_result = crate::gsm::lookup(&effective_error_info);
            logger::debug!(
                action = "GSM_SCORING_FILTER_LOOKUP",
                tag = "GSM_SCORING_FILTER_LOOKUP",
                "GSM lookup: connector={} -> {:?}",
                api_payload.gateway,
                gsm_lookup_result
                    .as_ref()
                    .map(|g| (&g.unified_message, &g.decision)),
            );
            if let Some(gsm_info) = gsm_lookup_result {
                if is_gateway_healthy_failure(&gsm_info) {
                    logger::info!(
                        action = "GSM_SCORING_FILTER_SKIP",
                        tag = "GSM_SCORING_FILTER_SKIP",
                        "Skipping gateway penalization for merchant={} gateway={} \
                         unified_message={:?} step_up_possible={}: user-originated failure",
                        api_payload.merchant_id,
                        api_payload.gateway,
                        gsm_info.unified_message,
                        gsm_info.step_up_possible,
                    );
                    return Ok("Success".to_string());
                }
            }
        }
    } else {
        logger::debug!(
            action = "GSM_SCORING_FILTER_NO_ERROR_INFO",
            tag = "GSM_SCORING_FILTER_NO_ERROR_INFO",
            "GSM scoring filter skipped: no error_info in payload for merchant={} gateway={}",
            api_payload.merchant_id,
            api_payload.gateway,
        );
    }

    let redis_key = format!(
        "{}{}",
        C::GATEWAY_SCORING_DATA,
        api_payload.clone().payment_id
    );
    let app_state = get_tenant_app_state().await;

    // Attempt to fetch gateway scoring data from Redis
    let m_gateway_scoring_data: Result<
        GatewayScoringData,
        error_stack::Report<redis_interface::errors::RedisError>,
    > = app_state
        .redis_conn
        .get_key(&redis_key, "GatewayScoringData")
        .await;

    match m_gateway_scoring_data {
        Ok(gateway_scoring_data) => {
            // Extract transaction details and card info from the API payload
            let txn_detail: TxnDetail = match Fbu::get_txn_detail_from_api_payload(
                api_payload.clone(),
                gateway_scoring_data.clone(),
            ) {
                Ok(detail) => detail,
                Err(e) => {
                    return Err(invalid_request_error("transaction details", &e));
                }
            };
            let txn_card_info: TxnCardInfo = Fbu::get_txn_card_info_from_api_payload(
                api_payload.clone(),
                gateway_scoring_data.clone(),
            );

            let log_message = "update_gateway_score";
            let enforce_failure = api_payload.enforce_dynamic_routing_failure.unwrap_or(false);

            // Call the function to check and update the gateway score
            check_and_update_gateway_score(
                txn_detail,
                txn_card_info,
                log_message,
                enforce_failure,
                api_payload.gateway_reference_id.clone(),
                api_payload.txn_latency.clone(),
                crate::analytics::AnalyticsRoute::UpdateGatewayScore,
                crate::analytics::AnalyticsFlowContext::new(
                    crate::analytics::ApiFlow::DynamicRouting,
                    crate::analytics::FlowType::UpdateGatewayScoreScoreSnapshot,
                ),
                None,
            )
            .await;

            // Return success response
            Ok("Success".to_string())
        }
        Err(e) => {
            // With compression on, a missing key also reads as `GetFailed`, which is otherwise a
            // read or decode failure. Only a key confirmed absent is skipped.
            let scoring_data_absent = match e.current_context() {
                redis_interface::errors::RedisError::NotFound => true,
                redis_interface::errors::RedisError::GetFailed => {
                    matches!(app_state.redis_conn.key_exists(&redis_key).await, Ok(false))
                }
                _ => false,
            };
            if scoring_data_absent {
                logger::info!(
                    action = "GATEWAY_SCORING_DATA_NOT_FOUND_SKIP",
                    tag = "GATEWAY_SCORING_DATA_NOT_FOUND_SKIP",
                    "No GatewayScoringData in redis for merchant={} gateway={} payment_id={}; \
                     skipping score update: {}",
                    api_payload.merchant_id,
                    api_payload.gateway,
                    api_payload.payment_id,
                    e,
                );
                return Ok(SCORE_UPDATE_SKIPPED.to_string());
            }
            Err(T::ErrorResponse {
                status: "400".to_string(),
                error_code: "GATEWAY_SCORING_DATA_NOT_FOUND".to_string(),
                error_message: "GatewayScoringData is not found in redis".to_string(),
                priority_logic_tag: None,
                routing_approach: None,
                filter_wise_gateways: None,
                error_info: T::UnifiedError {
                    code: "GATEWAY_SCORING_DATA_NOT_FOUND".to_string(),
                    user_message:
                        "GatewayScoringData is not in redis. Please create the transaction."
                            .to_string(),
                    developer_message: e.to_string(),
                },
                priority_logic_output: None,
                is_dynamic_mga_enabled: false,
            })
        }
    }
}

pub async fn check_and_update_gateway_score(
    txn_detail: TxnDetail,
    txn_card_info: TxnCardInfo,
    log_message: &str,
    enforce_failure: bool,
    gateway_reference_id: Option<String>,
    txn_latency: Option<TransactionLatency>,
    analytics_route: crate::analytics::AnalyticsRoute,
    score_snapshot_flow: crate::analytics::AnalyticsFlowContext,
    redis_comp_config: Option<RedisCompressionConfigCombined>,
) -> () {
    // Get gateway scoring type
    let gateway_scoring_type =
        get_gateway_scoring_type(txn_detail.clone(), txn_card_info.clone(), enforce_failure).await;

    let gateway_in_string = txn_detail.gateway.clone().unwrap_or_default();

    // Create update score lock key
    let update_score_lock_key = update_gateway_score_lock(
        gateway_scoring_type.clone(),
        txn_detail.txnUuid.clone(), // This is Maybe type in haskell and here it is not Option
        gateway_in_string,          // Convert Option to String
    );

    let lock_key_ttl = findByNameFromRedis(UpdateGatewayScoreLockFlagTtl.get_key())
        .await
        .unwrap_or(300);

    let should_compute_gw_score = check_and_send_should_update_gateway_score(
        update_score_lock_key,
        lock_key_ttl,
        redis_comp_config.clone(),
    )
    .await;

    // Check if feature is enabled for merchant
    let feature_enabled = is_feature_enabled(
        UpdateScoreLockFeatureEnabledMerchant.get_key(),
        get_m_id(txn_detail.merchantId.clone()),
        "kv_redis".to_string(),
    )
    .await;

    // Logging and score update logic
    if feature_enabled {
        if should_compute_gw_score {
            logger::debug!(
                action = "UPDATE_GATEWAY_SCORE_LOCK",
                tag = "UPDATE_GATEWAY_SCORE_LOCK",
                "Updating Gateway Score in {} flow with status as {:?} and scoring type as {:?}",
                log_message,
                txn_detail.status,
                gateway_scoring_type
            );
            update_gateway_score(
                gateway_scoring_type.clone(),
                txn_detail.clone(),
                txn_card_info.clone(),
                gateway_reference_id.clone(),
                txn_latency.clone(),
                analytics_route,
                score_snapshot_flow,
                redis_comp_config.clone(),
            )
            .await;
        }
    } else {
        logger::debug!(
            action = "GW_SCORE_LOCK_FEATURE_NOT_ENABLED",
            tag = "GW_SCORE_LOCK_FEATURE_NOT_ENABLED",
            "Updating Gateway Score in {} flow with status as {:?} and scoring type as {:?}",
            log_message,
            txn_detail.status,
            gateway_scoring_type
        );
        update_gateway_score(
            gateway_scoring_type.clone(),
            txn_detail,
            txn_card_info.clone(),
            gateway_reference_id.clone(),
            txn_latency.clone(),
            analytics_route,
            score_snapshot_flow,
            redis_comp_config,
        )
        .await;
    }
}

// Original Haskell function: updateGatewayScore
pub async fn update_gateway_score(
    gateway_scoring_type: GatewayScoringType,
    txn_detail: TxnDetail,
    txn_card_info: TxnCardInfo,
    gateway_reference_id: Option<String>,
    txn_latency: Option<TransactionLatency>,
    analytics_route: crate::analytics::AnalyticsRoute,
    score_snapshot_flow: crate::analytics::AnalyticsFlowContext,
    redis_compression_config: Option<RedisCompressionConfigCombined>,
) -> () {
    let mer_acc: MerchantAccount =
        MA::load_merchant_by_merchant_id(MID::merchant_id_to_text(txn_detail.clone().merchantId))
            .await
            .expect("Merchant account not found");

    //let mer_acc =
    let routing_approach = get_routing_approach(txn_detail.clone());
    logger::debug!(
        action = "routing_approach_value",
        tag = "routing_approach_value",
        "{:?}",
        routing_approach
    );

    let m_source_object = if txn_card_info.paymentMethodType == UPI {
        txn_detail.sourceObject.clone()
    } else {
        Some(txn_card_info.paymentMethod.clone())
    };

    //let is_pm_and_pmt_present = Fbu::isTrueString(txn_card_info.paymentMethod) && txn_card_info.paymentMethodType.is_some();
    // Which layer picked the gateway, read from the decision's own metadata and typed: rule-based,
    // network and merchant-preference picks are all valid answers, and none is an SR variant.
    let decided_approach = get_typed_routing_approach(txn_detail.clone());

    let srv3_producer_isolation_enabled = Cutover::is_feature_enabled(
        C::SrV3ProducerIsolation.get_key(),
        MID::merchant_id_to_text(txn_detail.clone().merchantId),
        C::kvRedis(),
    )
    .await;

    let explore_exploit_enabled = Cutover::is_feature_enabled(
        DC::EnableExploreAndExploitOnSrv3(txn_card_info.clone().paymentMethodType.to_string())
            .get_key(),
        MID::merchant_id_to_text(txn_detail.clone().merchantId),
        C::kvRedis(),
    )
    .await;

    // The legacy dimension (elimination / outage / SR v1+v2) follows the scoring type and the
    // transaction's status only — never which layer routed the payment.
    let should_update_gateway_score = if gateway_scoring_type.clone() == GST::PenaliseSrv3 {
        false
    } else if gateway_scoring_type.clone() == GST::Penalise {
        is_transaction_pending(txn_detail.clone().status)
    } else {
        true
    };

    // The SRv3 producer takes every scoring type except `Penalise`, the legacy in-flight signal.
    let should_update_srv3_gateway_score = gateway_scoring_type.clone() != GST::Penalise;

    let redis_key = format!("{}{}", C::GATEWAY_SCORING_DATA, txn_detail.clone().txnUuid);
    let app_state = get_tenant_app_state().await;

    // One read of the scoring context, shared by every dimension below, and deliberately not gated
    // on the SRv3 producer checks: which layer picked a gateway says nothing about the gateway's
    // health. Reading it here also gives the latency window and the snapshot the payment's real SR
    // metrics instead of fall-back defaults.
    let any_dimension_due = should_update_gateway_score || should_update_srv3_gateway_score;
    let mb_gateway_scoring_data: Option<GatewayScoringData> = if any_dimension_due {
        app_state
            .redis_conn
            .get_key(&redis_key, "GatewayScoringData")
            .await
            .ok()
    } else {
        None
    };

    let m_metric_entry: Option<MetricEntry> = match mb_gateway_scoring_data.clone() {
        None => {
            let merchant_id_str = MID::merchant_id_to_text(txn_detail.clone().merchantId);
            let pmt_str = txn_card_info.paymentMethodType.to_string();
            let txn_obj_type_str = txn_detail
                .txnObjectType
                .clone()
                .map(|t| t.to_string())
                .unwrap_or_default();
            let card_type_str = txn_card_info.card_type.clone().map(|t| t.to_string());
            get_metric_entry_data(
                merchant_id_str,
                pmt_str,
                m_source_object.clone(),
                txn_obj_type_str,
                card_type_str,
                false,
                None,
            )
            .await
        }
        Some(gateway_scoring_data) => {
            let merchant_id_str = MID::merchant_id_to_text(txn_detail.clone().merchantId);
            let pmt_str = txn_card_info.paymentMethodType.to_string();
            let txn_obj_type_str = txn_detail
                .txnObjectType
                .clone()
                .map(|t| t.to_string())
                .unwrap_or_default();
            let card_type_str = txn_card_info.card_type.clone().map(|t| t.to_string());
            get_metric_entry_data(
                merchant_id_str,
                pmt_str,
                m_source_object.clone(),
                txn_obj_type_str,
                card_type_str,
                gateway_scoring_data.isGriEnabledForElimination,
                gateway_scoring_data.gatewayReferenceId,
            )
            .await
        }
    };

    let is_update_within_window = is_update_within_latency_window(
        txn_detail.clone(),
        txn_card_info.clone(),
        gateway_scoring_type.clone(),
        mer_acc.clone(),
        txn_latency.clone(),
        m_metric_entry.clone(),
    )
    .await;

    // The SRv3 producer's two admission gates are about estimator quality, not about whether the
    // outcome is worth recording, so they apply to the SRv3 dimension alone.
    let should_record_srv3_post_update = srv3_producer_admits_outcome(
        decided_approach.as_ref(),
        srv3_producer_isolation_enabled,
        explore_exploit_enabled,
    ) && should_update_srv3_gateway_score
        && is_update_within_window;

    // Producer isolation turned a would-be SRv3 update away. That is the configured behaviour, so
    // it is counted rather than warned about: the counter is how we learn whether the trade-off is
    // costing real merchants anything, without changing what is recorded.
    if should_update_srv3_gateway_score
        && is_update_within_window
        && !should_record_srv3_post_update
        && srv3_write_isolated_from_producer(
            decided_approach.as_ref(),
            srv3_producer_isolation_enabled,
        )
    {
        let approach = match decided_approach.as_ref() {
            Some(approach) => approach.to_string(),
            None => "UNRECORDED".to_string(),
        };
        crate::metrics::SRV3_OUTCOME_ISOLATED_COUNTER
            .with_label_values(&[approach.as_str()])
            .inc();
        logger::info!(
            action = "SRV3_OUTCOME_ISOLATED",
            tag = "SRV3_OUTCOME_ISOLATED",
            "Outcome not written to the SRv3 score: producer isolation is on and {} did not pick \
             the gateway",
            approach
        );
    }
    if !should_record_srv3_post_update {
        if let Some(metric_entry) = m_metric_entry.clone() {
            crate::analytics::DomainAnalyticsEvent::record_score_snapshot(
                score_snapshot_flow,
                Some(MID::merchant_id_to_text(txn_detail.clone().merchantId)),
                Some(txn_card_info.paymentMethodType.to_string()),
                Some(m_source_object.clone().unwrap_or_default()),
                txn_card_info
                    .cardSwitchProvider
                    .as_ref()
                    .map(|provider| provider.peek().to_string()),
                txn_card_info.card_isin.clone(),
                Some(txn_detail.currency.to_string()),
                txn_detail
                    .country
                    .as_ref()
                    .map(|country| country.to_string()),
                txn_card_info
                    .authType
                    .as_ref()
                    .map(|auth_type| auth_type.to_string()),
                txn_detail.gateway.clone().or(gateway_reference_id.clone()),
                Some(metric_entry.success_rate.into()),
                Some(metric_entry.sigma_factor.into()),
                Some(metric_entry.average_latency.into()),
                Some(metric_entry.tp99_latency.into()),
                Some(metric_entry.n_value as i64),
                analytics_route,
                serialize_gateway_score_analytics_details(
                    &routing_approach,
                    &gateway_scoring_type,
                    "Gateway score updated successfully",
                ),
                Some(txn_detail.txnUuid.clone()),
                None,
                None,
                None,
                Some("score_updated".to_string()),
            );
        }
    }

    if should_record_srv3_post_update {
        logger::debug!(
            action = "updateGatewayScore",
            tag = "updateGatewayScore",
            "Updating sr v3 score for the txn with scoring type as {:?} and status as {:?}",
            gateway_scoring_type,
            txn_detail.status
        );
        GSSV3::flow::update_sr_v3_score(
            gateway_scoring_type.clone(),
            txn_detail.clone(),
            txn_card_info.clone(),
            mer_acc.clone(),
            mb_gateway_scoring_data.clone(),
            gateway_reference_id.clone(),
        )
        .await;

        if let Some(gateway_scoring_data) = mb_gateway_scoring_data.clone() {
            let gateway_name = txn_detail.gateway.clone().unwrap_or_default();
            if !gateway_name.is_empty() {
                let merchant_id = MID::merchant_id_to_text(txn_detail.clone().merchantId);
                let pmt_str = txn_card_info.paymentMethodType.to_string();
                let pm_str = m_source_object.clone().unwrap_or_default();
                let sr_routing_dimensions = SrRoutingDimensions {
                    card_network: txn_card_info
                        .cardSwitchProvider
                        .as_ref()
                        .map(|s| s.peek().to_string()),
                    card_isin: txn_card_info.card_isin.clone(),
                    currency: Some(txn_detail.currency.to_string()),
                    country: txn_detail.country.as_ref().map(|c| c.to_string()),
                    auth_type: txn_card_info.authType.as_ref().map(|a| a.to_string()),
                };
                let merchant_sr_v3_input_config =
                    findByNameFromRedis(SrV3InputConfig(merchant_id.clone()).get_key()).await;
                let default_sr_v3_input_config =
                    findByNameFromRedis(SR_V3_DEFAULT_INPUT_CONFIG.get_key()).await;
                let bucket_size = GU::get_sr_v3_bucket_size(
                    merchant_sr_v3_input_config,
                    &pmt_str,
                    &pm_str,
                    &sr_routing_dimensions,
                    true,
                )
                .or_else(|| {
                    GU::get_sr_v3_bucket_size(
                        default_sr_v3_input_config,
                        &pmt_str,
                        &pm_str,
                        &sr_routing_dimensions,
                        true,
                    )
                })
                .unwrap_or(DC::DEFAULT_SR_V3_BASED_BUCKET_SIZE);

                let gateway_ref_id_map =
                    HashMap::from([(gateway_name.clone(), gateway_reference_id.clone())]);
                let redis_key_map = GU::get_unified_key(
                    gateway_scoring_data,
                    None,
                    T::ScoreKeyType::SrV3Key,
                    false,
                    gateway_ref_id_map,
                )
                .await;

                if let Some(redis_key) = redis_key_map.get(&gateway_name) {
                    let score_value =
                        crate::decider::gatewaydecider::gw_scoring::get_score_from_redis(
                            bucket_size,
                            redis_key,
                        )
                        .await;
                    crate::analytics::DomainAnalyticsEvent::record_score_snapshot(
                        score_snapshot_flow,
                        Some(merchant_id),
                        Some(pmt_str),
                        Some(pm_str),
                        sr_routing_dimensions.card_network.clone(),
                        sr_routing_dimensions.card_isin.clone(),
                        sr_routing_dimensions.currency.clone(),
                        sr_routing_dimensions.country.clone(),
                        sr_routing_dimensions.auth_type.clone(),
                        Some(gateway_name),
                        Some(score_value),
                        None,
                        None,
                        None,
                        Some(bucket_size as i64),
                        analytics_route,
                        serialize_gateway_score_analytics_details(
                            &routing_approach,
                            &gateway_scoring_type,
                            "Gateway score snapshot derived from sr_v3 redis score",
                        ),
                        Some(txn_detail.txnUuid.clone()),
                        None,
                        None,
                        None,
                        Some("score_updated".to_string()),
                    );
                }
            }
        }
    }

    if should_update_gateway_score && is_update_within_window {
        let mer_acc_p_id: ETM::id::MerchantPId = mer_acc.id;
        let m_pf_mc_config = MerchantConfig::getMerchantConfigEntityLevelLookupConfig().await;
        logger::debug!(tag = "GatewayScoringData", "{:?}", mb_gateway_scoring_data);
        match mb_gateway_scoring_data {
            None => {
                logger::error!(
                    action = "GATEWAY_SCORING_DATA_NOT_FOUND_FOR_ELIMINATION",
                    tag = "GATEWAY_SCORING_DATA_NOT_FOUND_FOR_ELIMINATION",
                    "Gateway scoring data is not found in redis"
                );
            }
            Some(gateway_scoring_data) => {
                logger::debug!(
                    action = "Downtime-EmailNotification",
                    tag = "Downtime-EmailNotification",
                    "Proceed to updateKeyScoreForKeysFromConsumer"
                );
                let key_array = GEF::getAllUnifiedKeys(
                    txn_detail.clone(),
                    txn_card_info.clone(),
                    mer_acc_p_id,
                    m_pf_mc_config.clone(),
                    mer_acc.clone(),
                    gateway_scoring_data.clone(),
                    gateway_reference_id.clone(),
                )
                .await;
                logger::debug!(
                    action = "Downtime-EmailNotification",
                    tag = "Downtime-EmailNotification",
                    "{:?}",
                    key_array
                );
                for key in key_array {
                    tokio::spawn(GEF::updateKeyScoreForKeysFromConsumer(
                        txn_detail.clone(),
                        txn_card_info.clone(),
                        gateway_scoring_type.clone(),
                        gateway_scoring_data.clone(),
                        mer_acc_p_id,
                        mer_acc.clone(),
                        key,
                        redis_compression_config.clone(),
                    ));
                }
            }
        }
        Fbu::log_gateway_score_type(
            gateway_scoring_type,
            RF::EliminationFlow,
            txn_detail.clone(),
        );
    } else {
        if !should_update_gateway_score {
            logger::debug!(
                tag = "updateGatewayScore",
                action = "updateGatewayScore",
                "Gateway Scoring Type {:?} does not match with txn status {:?}",
                gateway_scoring_type,
                txn_detail.status
            );
        }
        // if !is_pm_and_pmt_present {
        // logger::debug!(
        //     tag = "updateGatewayScore",
        //     "Payment Method or Payment Method Type is null for txn_detail.id {}",
        //     txn_detail._id.as_deref().unwrap_or("")
        // );
        // }
        if !is_update_within_window {
            logger::debug!(
                tag = "updateGatewayScore",
                action = "updateGatewayScore",
                "Update GW Score call received outside Update Window"
            );
        }
    }
}

// Original Haskell function: getRoutingApproach
pub fn get_routing_approach(txn_detail: TxnDetail) -> Option<String> {
    let internal_meta: Option<FT::InternalMetadata> = get_value_from_meta_data(&txn_detail);
    match internal_meta {
        Some(meta) => Some(meta.internal_tracking_info.routing_approach),
        None => None,
    }
}

/// The payment's routing approach as a typed value. `None` means "no usable approach recorded":
/// the metadata is absent, or the text is not a `GatewayDeciderApproach` variant — `/routing/hybrid`'s
/// rule layer stamps its own `STATIC_ROUTING`. Callers must read that as an unknown selector.
pub fn get_typed_routing_approach(txn_detail: TxnDetail) -> Option<GatewayDeciderApproach> {
    get_routing_approach(txn_detail)
        .as_deref()
        .and_then(|text| text.parse::<GatewayDeciderApproach>().ok())
}

// Original Haskell function: getValueFromMetaData
pub fn get_value_from_meta_data<T: serde::de::DeserializeOwned>(
    txn_detail: &TxnDetail,
) -> Option<T> {
    let metadata = txn_detail.internalMetadata.clone()?;
    serde_json::from_str(metadata.peek()).ok()
}

/// Whether the SRv3 producer may accept this outcome. Pure, so the gate is testable without
/// Redis or a database. Replaces the old `contains("V3")` / `contains("HEDGING")` substring pair
/// with the typed classification on [`GatewayDeciderApproach`], and the two flags gate opposite
/// things:
///
///   * `sr_v3_producer_isolation` (on) keeps foreign outcomes out of the moving window. An
///     unrecorded approach counts as foreign: "we cannot tell" is not evidence of SRv3.
///   * `explore_exploit` (on) keeps on-policy samples out. An unrecorded approach counts as
///     explore, for the mirror-image reason — an unidentifiable selector is no proof of an
///     on-policy exploit, and dropping it discards a real gateway-health signal.
pub fn srv3_producer_admits_outcome(
    decided_approach: Option<&GatewayDeciderApproach>,
    srv3_producer_isolation_enabled: bool,
    explore_exploit_enabled: bool,
) -> bool {
    let is_srv3_produced = decided_approach.is_some_and(|approach| approach.is_srv3_scored());
    let is_explore = match decided_approach {
        Some(approach) => approach.is_explore_sample(),
        None => true,
    };
    (!srv3_producer_isolation_enabled || is_srv3_produced)
        && (!explore_exploit_enabled || is_explore)
}

/// Whether `sr_v3_producer_isolation` is the reason an otherwise-due SRv3 write did not happen:
/// the payment was picked by some other layer (rules, the network, merchant preference, or a
/// selector that was not recorded) and this merchant has isolation switched on.
///
/// That drop is deliberate — see [`srv3_producer_admits_outcome`] — which is exactly why it needs a
/// counter. Without one, "how many merchants are on the losing side of this trade-off" is a guess;
/// with it, the answer is a rate. Never called when the SRv3 dimension was not due anyway, so the
/// number means "outcomes that would have been recorded".
pub fn srv3_write_isolated_from_producer(
    decided_approach: Option<&GatewayDeciderApproach>,
    srv3_producer_isolation_enabled: bool,
) -> bool {
    srv3_producer_isolation_enabled
        && !decided_approach.is_some_and(|approach| approach.is_srv3_scored())
}

// Original Haskell function: isUpdateWithinLatencyWindow
pub async fn is_update_within_latency_window(
    txn_detail: TxnDetail,
    txn_card_info: TxnCardInfo,
    gateway_scoring_type: GatewayScoringType,
    mer_acc: MerchantAccount,
    txn_latency: Option<TransactionLatency>,
    m_metric_entry: Option<MetricEntry>,
) -> bool {
    match gateway_scoring_type {
        GatewayScoringType::Penalise => true,
        _ => {
            let exempt_for_mandate_txn = checkExemptIfMandateTxn(&txn_detail, &txn_card_info).await;
            if exempt_for_mandate_txn
            // || txn_detail
            //     .gateway
            //     .as_ref()
            //     .map_or(true, |gw| exempt_gws.contains(gw))
            {
                true
            } else {
                // let m_auto_refund_conflict_threshold_in_mins: Option<i32> = None; // Placeholder for actual implementation
                let gw_latency_check_threshold =
                    findByNameFromRedis(C::GatewayScoreLatencyCheckInMins.get_key())
                        .await
                        .unwrap_or(C::defaultGatewayScoreLatencyCheckInMins());
                let gw_wise_latency_threshold = get_gateway_wise_latency(
                    &default_gw_latency_check_in_mins(),
                    &txn_card_info.paymentMethodType.to_string(),
                    &GU::get_payment_method(
                        txn_card_info.paymentMethodType.to_string(),
                        txn_card_info.paymentMethod.clone(),
                        txn_detail.sourceObject.clone().unwrap_or_default(),
                    ),
                    &txn_detail.gateway.clone().unwrap_or_default(), // Convert Option to String
                );
                logger::info!(
                    action = "gw_wise_latency_threshold",
                    tag = "gw_wise_latency_threshold",
                    "gw_wise_latency_threshold: {}",
                    gw_wise_latency_threshold
                );

                // check if the transaction latency calculated by orchestration is within the configured threshold
                let is_gw_latency_within_threshold = isGwLatencyWithinConfiguredThreshold(
                    txn_latency.and_then(|m| m.gateway_latency),
                    GatewaySuccessRateBasedRoutingInput::from_str(
                        &mer_acc.gatewaySuccessRateBasedDeciderInput,
                    )
                    .ok()
                    .and_then(|m| m.txnLatency.and_then(|l| l.gatewayLatency)),
                );
                // Cutover::findByNameFromRedis(C.gatewayScoreLatencyCheckInMins)
                //     .await
                //     .unwrap_or(C.defaultGatewayScoreLatencyCheckInMins);
                let _merchant_id = MID::merchant_id_to_text(txn_detail.merchantId.clone());
                let pmt = txn_card_info.paymentMethodType;
                let _pm = GU::get_payment_method(
                    pmt,
                    txn_card_info.paymentMethod,
                    txn_detail.sourceObject.clone().unwrap_or_default(),
                );

                let gw_score_update_latency =
                    Fbu::get_time_from_txn_created_in_mills(txn_detail.clone());
                let gw_latency_check_threshold_ =
                    gw_wise_latency_threshold.min(gw_latency_check_threshold as f64);
                let gw_latency_check_threshold = match m_metric_entry {
                    Some(metric_entry) => {
                        gw_latency_check_threshold_.min(metric_entry.tp99_latency.into())
                    }
                    None => gw_latency_check_threshold_,
                };
                logger::debug!(
                    action = "gwLatencyCheckThreshold",
                    tag = "gwLatencyCheckThreshold",
                    "gwLatencyCheckThreshold: {}",
                    gw_latency_check_threshold
                );
                (gw_score_update_latency < (gw_latency_check_threshold * 60000.0) as u128)
                    && is_gw_latency_within_threshold
            }
        }
    }
}

async fn checkExemptIfMandateTxn(txn_detail: &TxnDetail, txn_card_info: &TxnCardInfo) -> bool {
    let is_recurring = isRecurringTxn(txn_detail.txnObjectType.clone());
    let is_nb_pmt = txn_card_info.paymentMethodType == (NB);
    let is_penny_reg_txn = isPennyMandateRegTxn(txn_detail.clone());
    is_recurring || (is_nb_pmt && is_penny_reg_txn)
}

// Original Haskell function: isTransactionPending
pub fn is_transaction_pending(txn_status: TxnStatus) -> bool {
    txn_status == TS::PendingVBV || txn_status == TS::Started
}

// Helper function to filter by gateway only
fn filter_upto_gw<'a>(
    latency_input: &'a [GatewayWiseLatencyInput],
    gw: &'a str,
) -> Option<&'a GatewayWiseLatencyInput> {
    latency_input
        .iter()
        .find(|x| x.gateway == gw && x.paymentMethodType.is_none() && x.paymentMethod.is_none())
}

// Helper function to filter by gateway and payment method type
fn filter_upto_pmt<'a>(
    latency_input: &'a [GatewayWiseLatencyInput],
    gw: &'a str,
    pmt: &'a str,
) -> Option<&'a GatewayWiseLatencyInput> {
    latency_input.iter().find(|x| {
        x.gateway == gw
            && x.paymentMethodType
                .as_ref()
                .map_or("".to_string(), |s| s.clone())
                == pmt
            && x.paymentMethod.is_none()
    })
}

// Helper function to filter by gateway, payment method type, and payment method
fn filter_upto_pm<'a>(
    latency_input: &'a [GatewayWiseLatencyInput],
    gw: &'a str,
    pmt: &'a str,
    pm: &'a str,
) -> Option<&'a GatewayWiseLatencyInput> {
    latency_input.iter().find(|x| {
        x.gateway == gw
            && x.paymentMethodType
                .as_ref()
                .map_or("".to_string(), |s| s.clone())
                == pmt
            && x.paymentMethod
                .as_ref()
                .map_or("".to_string(), |s| s.clone())
                == pm
    })
}

// Helper function to get gateway latency threshold
fn get_gw_latency_threshold(
    merchant_latency_gateway_wise_input: &Option<Vec<GatewayWiseLatencyInput>>,
) -> Option<&GatewayWiseLatencyInput> {
    match merchant_latency_gateway_wise_input {
        None => None,
        Some(_latency_input) => {
            // This will be called with specific parameters in the main function
            // For now, return None as the actual filtering happens in the main function
            None
        }
    }
}

// Main function to get gateway-wise latency
pub fn get_gateway_wise_latency(
    gateway_latency_threshold: &GatewayLatencyForScoring,
    pmt: &str,
    pm: &str,
    gw: &str,
) -> f64 {
    let _m_gateway_wise_input =
        get_gw_latency_threshold(&gateway_latency_threshold.merchant_latency_gateway_wise_input);

    // Log the input (similar to EL.logDebugV in Haskell)
    logger::debug!(
        action = "get_gateway_wise_latency",
        tag = "get_gateway_wise_latency",
        "mGatewayWiseInput: {:?}",
        gateway_latency_threshold.merchant_latency_gateway_wise_input
    );

    match &gateway_latency_threshold.merchant_latency_gateway_wise_input {
        Some(gw_wise_input) => {
            // Try to find the most specific match first, then fall back to less specific
            if let Some(result) = filter_upto_pm(gw_wise_input, gw, pmt, pm) {
                result.latencyThreshold
            } else if let Some(result) = filter_upto_pmt(gw_wise_input, gw, pmt) {
                result.latencyThreshold
            } else if let Some(result) = filter_upto_gw(gw_wise_input, gw) {
                result.latencyThreshold
            } else {
                gateway_latency_threshold.default_latency_threshold
            }
        }
        None => gateway_latency_threshold.default_latency_threshold,
    }
}

#[cfg(test)]
mod tests {
    use super::is_gateway_healthy_failure;
    use crate::gsm::GsmInfo;

    fn gsm_info(unified_message: Option<&str>, step_up_possible: bool) -> GsmInfo {
        GsmInfo {
            decision: "do_default".to_string(),
            step_up_possible,
            clear_pan_possible: false,
            alternate_network_possible: false,
            unified_code: None,
            unified_message: unified_message.map(str::to_string),
            error_category: None,
            standardised_code: None,
            description: None,
            user_guidance_message: None,
        }
    }

    // ── Skip-penalty cases (UE_1000 — Issue with Payment Method details) ─────

    #[test]
    fn payment_method_details_expired_card_skips_penalty() {
        // e.g. adyen code 6: Expired Card
        assert!(is_gateway_healthy_failure(&gsm_info(
            Some("Issue with Payment Method details"),
            false
        )));
    }

    #[test]
    fn payment_method_details_wrong_cvv_skips_penalty() {
        // e.g. adyen code 24: CVC Declined
        assert!(is_gateway_healthy_failure(&gsm_info(
            Some("Issue with Payment Method details"),
            false
        )));
    }

    #[test]
    fn payment_method_details_frm_skips_penalty() {
        // e.g. adyen code 5: Blocked Card (frm_decline + do_default → UE_1000)
        assert!(is_gateway_healthy_failure(&gsm_info(
            Some("Issue with Payment Method details"),
            false
        )));
    }

    #[test]
    fn step_up_possible_skips_penalty_regardless_of_message() {
        // e.g. paypal DECLINED_SCA_REQUIRED — gateway functional, needs 3DS
        assert!(is_gateway_healthy_failure(&gsm_info(None, true)));
    }

    #[test]
    fn step_up_possible_overrides_technical_psp_message() {
        // step_up_possible takes priority even when unified_message signals a PSP issue
        assert!(is_gateway_healthy_failure(&gsm_info(
            Some("Technical issue with PSP"),
            true
        )));
    }

    // ── Full-penalty cases ────────────────────────────────────────────────────

    #[test]
    fn issue_with_configurations_applies_full_penalty() {
        // UE_2000: acquirer not configured, API key expired, missing merchant key
        assert!(!is_gateway_healthy_failure(&gsm_info(
            Some("Issue with Configurations"),
            false
        )));
    }

    #[test]
    fn technical_issue_with_psp_applies_full_penalty() {
        // UE_3000: gateway/PSP down, refused, transaction not permitted
        assert!(!is_gateway_healthy_failure(&gsm_info(
            Some("Technical issue with PSP"),
            false
        )));
    }

    #[test]
    fn issue_with_integration_applies_full_penalty() {
        // UE_4000: missing required fields, integration misconfiguration
        assert!(!is_gateway_healthy_failure(&gsm_info(
            Some("Issue with Integration"),
            false
        )));
    }

    #[test]
    fn something_went_wrong_applies_full_penalty() {
        // e.g. HTTP 401 response — unknown/unclassified gateway fault
        assert!(!is_gateway_healthy_failure(&gsm_info(
            Some("Something went wrong"),
            false
        )));
    }

    #[test]
    fn no_unified_message_applies_full_penalty() {
        // GSM rule matched but unified_message field is absent — safe default
        assert!(!is_gateway_healthy_failure(&gsm_info(None, false)));
    }

    #[test]
    fn unknown_message_applies_full_penalty() {
        // Forward-compatibility: unrecognised future messages default to full penalty
        assert!(!is_gateway_healthy_failure(&gsm_info(
            Some("some_future_message"),
            false
        )));
    }

    // ── Routing-approach producer/explore gates ─────────────────────────────
    //
    // `srv3_producer_admits_outcome(approach, isolation, explore)` is the whole SRv3 admission gate.
    // The two flags gate opposite things, so the table below states both columns for every kind of
    // selector rather than one flag at a time.
    use super::srv3_producer_admits_outcome;
    use crate::decider::gatewaydecider::types::GatewayDeciderApproach as Approach;

    /// (selector, isolation, explore) -> admitted.
    const ADMISSION: &[(Option<Approach>, bool, bool, bool)] = &[
        // SRv3 hedging and the cost/volume nudges are produced by SRv3 *and* off-policy, so they
        // clear either flag.
        (Some(Approach::SrV3Hedging), true, true, true),
        (Some(Approach::SrV3DowntimeHedging), true, true, true),
        (Some(Approach::SrSelectionMultiObjective), true, true, true),
        (
            Some(Approach::SrSelectionVolumeCommitment),
            true,
            true,
            true,
        ),
        // A plain SRv3 pick is produced by SRv3 but on-policy: isolation admits it, explore does not.
        (Some(Approach::SrSelectionV3Routing), true, true, false),
        (Some(Approach::SrV3DowntimeRouting), false, true, false),
        // Foreign producers: nothing no SR scorer chose may enter an isolated producer, but as
        // off-policy samples they are exactly what the explore gate is for. This is the fix — a
        // rule-routed payment used to be dropped here and never moved its gateway's score.
        (Some(Approach::PriorityLogic), true, true, false),
        (Some(Approach::PriorityLogic), false, true, true),
        (Some(Approach::PlDowntimeRouting), false, true, true),
        (Some(Approach::NtwBasedRouting), false, true, true),
        (Some(Approach::MerchantPreference), false, true, true),
        (Some(Approach::Default), false, true, true),
        // SRv1/v2 have their own producer, so they stay on-policy for SRv3 either way.
        (Some(Approach::SrSelectionV2Routing), true, true, false),
        (Some(Approach::SrSelectionV2Routing), false, true, false),
        (Some(Approach::SrV2Hedging), false, true, true),
    ];

    #[test]
    fn srv3_producer_admission_matches_the_table() {
        for (approach, isolation, explore, admitted) in ADMISSION {
            assert_eq!(
                srv3_producer_admits_outcome(approach.as_ref(), *isolation, *explore),
                *admitted,
                "{:?} with isolation={} explore={}",
                approach,
                isolation,
                explore
            );
        }
    }

    #[test]
    fn isolation_reports_only_what_it_excluded() {
        // Isolation is the reason, so the predicate is true exactly for a non-SRv3 selector while the
        // flag is on — false once the flag is off, and false for SRv3's own outcomes.
        for (approach, isolation, isolated) in [
            (Some(Approach::PriorityLogic), true, true),
            (Some(Approach::MerchantPreference), true, true),
            (None, true, true),
            (Some(Approach::PriorityLogic), false, false),
            (Some(Approach::SrSelectionV3Routing), true, false),
            (Some(Approach::SrV3Hedging), true, false),
        ] {
            assert_eq!(
                super::srv3_write_isolated_from_producer(approach.as_ref(), isolation),
                isolated,
                "{:?} with isolation={}",
                approach,
                isolation
            );
        }
    }

    #[test]
    fn both_flags_off_admit_every_outcome() {
        // The default configuration: feedback is recorded regardless of which layer routed it.
        for approach in [
            None,
            Some(Approach::PriorityLogic),
            Some(Approach::SrSelectionV3Routing),
            Some(Approach::SrSelectionV2Routing),
        ] {
            assert!(srv3_producer_admits_outcome(
                approach.as_ref(),
                false,
                false
            ));
        }
    }

    #[test]
    fn an_unrecorded_selector_is_explore_but_not_srv3_produced() {
        // /routing/hybrid's rule layer stamps its own approach, and a legacy call may record none.
        assert!(srv3_producer_admits_outcome(None, false, true));
        assert!(!srv3_producer_admits_outcome(None, true, true));
    }
}

// Helper function to filter by gateway only
