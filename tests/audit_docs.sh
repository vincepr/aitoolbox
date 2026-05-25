#!/usr/bin/env bash
set -euo pipefail

state_file="$(mktemp)"
rm -f "${state_file}"

out_first="$(AUDIT_STATE_FILE="${state_file}" AUDIT_FINGERPRINT="a" ./scripts/audit-docs.sh)"
[[ "${out_first}" == "first audit" ]]

out_skip="$(AUDIT_STATE_FILE="${state_file}" AUDIT_FINGERPRINT="a" ./scripts/audit-docs.sh)"
[[ "${out_skip}" == "skip audit" ]]

out_reaudit="$(AUDIT_STATE_FILE="${state_file}" AUDIT_FINGERPRINT="b" ./scripts/audit-docs.sh)"
[[ "${out_reaudit}" == "re-audit" ]]

echo "audit_docs.sh: PASS"
