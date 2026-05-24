#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   scripts/migrate-knowledge-db.sh <from-version> <to-version>
# If versions are omitted, script reads installed CLI version and latest GitHub release.

repo="vincepr/aitoolbox"

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${repo}/releases/latest" \
    | sed -n 's/.*"tag_name": "v\([^"]*\)".*/\1/p' \
    | head -n1
}

get_local_version() {
  if command -v knowledge-cli >/dev/null 2>&1; then
    knowledge-cli version
  else
    echo ""
  fi
}

major_of() {
  local v="$1"
  echo "$v" | awk -F. '{print $1}'
}

from_version="${1:-$(get_local_version)}"
to_version="${2:-$(get_latest_version)}"

if [[ -z "$from_version" ]]; then
  echo "knowledge-cli is not installed; no DB migration needed yet."
  exit 0
fi

if [[ -z "$to_version" ]]; then
  echo "failed to resolve latest release version" >&2
  exit 1
fi

from_major="$(major_of "$from_version")"
to_major="$(major_of "$to_version")"

if [[ "$from_major" == "$to_major" ]]; then
  echo "no major version change (${from_version} -> ${to_version}); no DB migration required."
  exit 0
fi

echo "major version change detected (${from_version} -> ${to_version})."
echo "no major migration steps are defined yet."
echo "create migration logic in scripts/migrate-knowledge-db.sh before proceeding."
exit 2
