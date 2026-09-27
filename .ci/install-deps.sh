#!/usr/bin/env bash
# System packages and toolchains the engine's build needs.
#
# librdkafka (via rdkafka-sys) needs libcurl/sasl/zlib; the MySQL driver needs libmariadb and
# bindgen needs libclang; the Postgres track needs libpq. The engine's own CI installs the first
# four and leans on the runner image for the rest — they are listed here so this CI does not
# depend on the image's package set.
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive
export CARGO_INCREMENTAL=0
export CARGO_NET_RETRY=10
export RUSTUP_MAX_RETRIES=10
export PATH="$HOME/.cargo/bin:$PATH"

# GitHub's and Travis's images hand you a sudoer user; GitLab's runners and the Codespace run as
# root and have no sudo at all. Pick whichever this machine has.
SUDO=""
command -v sudo >/dev/null 2>&1 && SUDO="sudo"

$SUDO apt-get update
$SUDO apt-get install -y --no-install-recommends \
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

# `language: minimal` does not set Rust up, and the image's cargo may not even be on PATH. Bootstrap
# rustup when it is missing rather than assuming it: this is the step that has to survive a change
# of image.
if ! command -v rustup >/dev/null 2>&1; then
  echo "==> installing rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain none
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Pin the same toolchain the engine's own CI asks for (stable), and add nightly purely for rustfmt,
# which is what the project formats and format-checks with.
echo "==> pinning toolchains"
rustup set profile minimal
rustup toolchain install stable --profile minimal
rustup default stable
rustup toolchain install nightly --profile minimal --component rustfmt

cargo --version
rustc --version
cargo +nightly fmt --version
