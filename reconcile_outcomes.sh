#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

STRATEGY_CONFIG_PATH="${1:-configs/btc_ensemble.env}"

if [[ ! -f "$STRATEGY_CONFIG_PATH" ]]; then
  echo "Strategy config not found: $STRATEGY_CONFIG_PATH" >&2
  exit 1
fi

export STRATEGY_CONFIG="$STRATEGY_CONFIG_PATH"

echo "Running Polymarket official outcome reconciliation"
echo "Strategy config: $STRATEGY_CONFIG"

if [[ "${RELEASE:-0}" == "1" ]]; then
  cargo run --locked --release --bin reconcile_outcomes
else
  cargo run --locked --bin reconcile_outcomes
fi
