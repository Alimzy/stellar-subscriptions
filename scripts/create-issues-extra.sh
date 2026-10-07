#!/usr/bin/env bash
# Run from inside the repo folder, after create-issues.sh. DRY_RUN=1 prints titles only.
set -euo pipefail
mk() {
  if [ "${DRY_RUN:-0}" = "1" ]; then echo "[$2] $1"; else gh issue create --title "$1" --label "$2" --body "$(printf "%b" "$3")"; fi
}
mk "test: stateful model-based property tests for the subscription state machine" "high" "Add a proptest state-machine test: random sequences of create_plan, subscribe, charge, cancel, deactivate_plan and time advances, checked against a simple in-memory model.\n\nInvariants: contract balance is always 0; merchant receives exactly amount x successful charges; a cancelled subscription is never charged; total pulled never exceeds the allowance.\n\nAcceptance: runs in CI under 2 minutes; any shrunk failure prints a readable action sequence."
mk "feat: keeper bot that charges due subscriptions on testnet" "high" "Add a small keeper (TypeScript or Rust) in keeper/ that lists subscriptions, calls is_due, and sends charge for due ones. Must handle failures (insufficient allowance, plan deactivated) without crashing and log each outcome.\n\nAcceptance: README section with run instructions; demonstrated against the deployed testnet contract; no secrets committed; unit tests for the due-selection logic."
