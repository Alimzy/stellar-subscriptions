#!/usr/bin/env bash
# Run from inside the repo folder. DRY_RUN=1 prints titles only.
set -euo pipefail
mk() { # title label body
  if [ "${DRY_RUN:-0}" = "1" ]; then echo "[$2] $1"; else gh issue create --title "$1" --label "$2" --body "$(printf "%b" "$3")"; fi
}
if [ "${DRY_RUN:-0}" != "1" ]; then
  for l in trivial medium high ci security docs; do gh label create "$l" --force >/dev/null; done
fi
mk "docs: add a sequence diagram to ARCHITECTURE.md" "trivial,docs" "Add a Mermaid sequence diagram of subscribe -> charge -> cancel.\n\nAcceptance: renders on GitHub."
mk "docs: document the approve-then-subscribe flow with CLI examples" "trivial,docs" "Add copy-pasteable stellar-cli examples to the README.\n\nAcceptance: commands verified on testnet."
mk "test: add test for create_plan with i128::MAX amount" "trivial" "Add a unit test that a very large amount is accepted and stored unchanged."
mk "test: assert emitted events in unit tests" "trivial" "Use env.events().all() to assert topics and data for each of the five events."
mk "ci: add a code coverage job (cargo-llvm-cov)" "medium,ci" "Upload an lcov report as an artifact. Do not gate on a threshold yet."
mk "feat: add get_plan_subscribers pagination view" "medium" "Add a paginated view listing sub ids per plan. Needs a new storage index and tests."
mk "feat: add merchant-initiated plan reactivation" "medium" "Add reactivate_plan (merchant auth) with tests. Update docs/FEATURE-STATUS.md."
mk "feat: add subscriber-initiated pause/resume" "medium" "Add pause/resume with tests; define charge behavior while paused and document it."
mk "security: threat model document" "medium,security" "Write docs/THREAT-MODEL.md covering malicious merchant, malicious keeper, and token edge cases."
mk "feat: optional back-charging with a per-call cap" "high" "Allow charging up to N missed periods per call, configurable per plan. Must keep allowance cap semantics and add property tests."
mk "feat: keeper tip paid by subscriber" "high" "Optional small tip per charge to the caller. Needs design note, tests, and a limitations update."
mk "feat: plan price change with subscriber consent" "high" "Let merchants propose a new price; subscribers must opt in before it applies. Design in the issue first."
