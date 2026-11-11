#!/usr/bin/env bash
# Ubuntu 24.04 hosted-runner APT bootstrap for required native Vulkan proofs.
# Run on disposable GitHub-hosted runners, not on developer machines.
set -euo pipefail

if [[ "$#" -eq 0 ]]; then
  echo 'No Ubuntu packages were specified' >&2
  exit 64
fi

mirrors=/etc/apt/apt-mirrors.txt
if [[ ! -f "$mirrors" ]]; then
  echo "::error::Expected GitHub Ubuntu runner mirror list is missing: $mirrors" >&2
  exit 1
fi

echo 'Original Ubuntu APT mirrors:'
cat "$mirrors"

# The Azure-first mirror repeatedly failed DNS on the 2026-11-11 hosted runners.
# Only change it when the known official Ubuntu archive fallback is present.
if ! grep -Eq '^https?://archive\.ubuntu\.com/ubuntu/?[[:space:]]*$' "$mirrors"; then
  echo '::error::Official archive.ubuntu.com fallback is absent; refusing to rewrite unknown APT sources' >&2
  exit 1
fi
if grep -Fq 'azure.archive.ubuntu.com' "$mirrors"; then
  sudo sed -i '/azure\.archive\.ubuntu\.com/d' "$mirrors"
fi

echo 'Effective Ubuntu APT mirrors:'
cat "$mirrors"

apt_options=(
  -o Acquire::Retries=1
  -o Acquire::http::Timeout=15
  -o Acquire::https::Timeout=15
)
# In CI, an index-download warning must not be mistaken for a successful update.
# Bound both operations; any failure must fail the full Vulkan proof job.
timeout --signal=TERM --kill-after=10s 300s sudo apt-get "${apt_options[@]}" update --error-on=any
timeout --signal=TERM --kill-after=10s 300s sudo apt-get "${apt_options[@]}" install --yes "$@"
