#!/usr/bin/env bash
# System and toolchain packages the engine's build needs.
#
# librdkafka (via rdkafka-sys) needs libcurl/sasl/zlib; the MySQL driver needs libmariadb and
# bindgen needs libclang; the Postgres track needs libpq. The engine's own CI installs the first
# four and leans on the runner image for the rest — they are listed here so this CI does not
# depend on the image's package set.
set -euo pipefail

sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  pkg-config \
  libcurl4-openssl-dev \
  libsasl2-dev \
  zlib1g-dev \
  libpq-dev \
  libmariadb-dev \
  libclang-dev \
  cmake \
  postgresql-client \
  redis-tools

export CARGO_INCREMENTAL=0
export CARGO_NET_RETRY=10
export RUSTUP_MAX_RETRIES=10

# Travis ships rustup but no guarantee about which toolchain. Pin stable for the build (the same
# toolchain the engine's own CI asks for) and add nightly purely for rustfmt.
rustup set profile minimal
rustup toolchain install stable --profile minimal --component clippy
rustup default stable
rustup toolchain install nightly --profile minimal --component rustfmt
cargo --version
rustc --version
cargo +nightly fmt --version
