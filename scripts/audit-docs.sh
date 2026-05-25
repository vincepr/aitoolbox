#!/usr/bin/env bash
set -euo pipefail

state_file="${AUDIT_STATE_FILE:-docs/.audit-state.json}"
mode="${1:-run}"

compute_fingerprint() {
  if [[ -n "${AUDIT_FINGERPRINT:-}" ]]; then
    printf '%s' "${AUDIT_FINGERPRINT}"
    return
  fi

  local targets=(
    docs/superpowers/specs
    docs/superpowers/plans
    crates/knowledge-core/src
    crates/knowledge-cli/src
  )

  local files=()
  local target
  for target in "${targets[@]}"; do
    if [[ -d "${target}" ]]; then
      while IFS= read -r path; do
        files+=("${path}")
      done < <(find "${target}" -type f | sort)
    fi
  done

  if [[ "${#files[@]}" -eq 0 ]]; then
    printf '%s' "empty"
    return
  fi

  sha256sum "${files[@]}" | sha256sum | awk '{print $1}'
}

load_last_fingerprint() {
  if [[ ! -f "${state_file}" ]]; then
    printf '%s' ""
    return
  fi
  awk -F '"' '/"last_fingerprint"/ {print $4; exit}' "${state_file}" || true
}

write_state() {
  local fingerprint="$1"
  mkdir -p "$(dirname "${state_file}")"
  cat > "${state_file}" <<JSON
{
  "last_fingerprint": "${fingerprint}",
  "updated_by": "scripts/audit-docs.sh"
}
JSON
}

current_fingerprint="$(compute_fingerprint)"
last_fingerprint="$(load_last_fingerprint)"

if [[ -z "${last_fingerprint}" ]]; then
  write_state "${current_fingerprint}"
  echo "first audit"
  exit 0
fi

if [[ "${last_fingerprint}" == "${current_fingerprint}" ]]; then
  echo "skip audit"
  exit 0
fi

write_state "${current_fingerprint}"
echo "re-audit"
