#!/usr/bin/env bash
# End-to-end testnet demo. DRY_RUN=1 prints commands without running them.
set -euo pipefail
NET=testnet
WASM=target/wasm32v1-none/release/subscriptions.wasm
run() { echo "+ $*"; [ "${DRY_RUN:-0}" = "1" ] || "$@"; }
cap() { echo "+ $*" >&2; if [ "${DRY_RUN:-0}" = "1" ]; then echo "DRY_RUN_VALUE"; else "$@"; fi; }

run stellar contract build --package subscriptions
run sha256sum "$WASM"

for k in merchant subscriber; do
  run stellar keys generate "$k" --network $NET --fund 2>/dev/null || true
done
MERCHANT=$(cap stellar keys address merchant)
SUBSCRIBER=$(cap stellar keys address subscriber)
TOKEN=$(cap stellar contract id asset --asset native --network $NET)

CONTRACT=$(cap stellar contract deploy --wasm "$WASM" --source merchant --network $NET)
echo "CONTRACT_ID=$CONTRACT"

LEDGER=$(cap curl -s -X POST https://soroban-testnet.stellar.org \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getLatestLedger"}')
if [ "${DRY_RUN:-0}" = "1" ]; then EXP=1000000; else
  EXP=$(echo "$LEDGER" | python3 -c 'import sys,json;print(json.load(sys.stdin)["result"]["sequence"]+20000)'); fi

run stellar contract invoke --id "$TOKEN" --source subscriber --network $NET -- \
  approve --from "$SUBSCRIBER" --spender "$CONTRACT" --amount 100000000 --expiration_ledger "$EXP"
run stellar contract invoke --id "$CONTRACT" --source merchant --network $NET -- \
  create_plan --merchant "$MERCHANT" --token "$TOKEN" --amount 10000000 --period 60
run stellar contract invoke --id "$CONTRACT" --source subscriber --network $NET -- \
  subscribe --subscriber "$SUBSCRIBER" --plan_id 1
[ "${DRY_RUN:-0}" = "1" ] || sleep 65
run stellar contract invoke --id "$CONTRACT" --source merchant --network $NET -- charge --sub_id 1
echo "Done. Put CONTRACT_ID and the tx links in README.md."
