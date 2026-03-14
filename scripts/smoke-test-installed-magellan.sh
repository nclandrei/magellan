#!/usr/bin/env bash
set -euo pipefail

binary="${1:?usage: smoke-test-installed-magellan.sh /path/to/magellan-or-tarball}"
tmpdir=""

cleanup() {
  if [[ -n "${tmpdir}" && -d "${tmpdir}" ]]; then
    rm -rf "${tmpdir}"
  fi
}
trap cleanup EXIT

if [[ -f "${binary}" && "${binary}" == *.tar.xz ]]; then
  tmpdir="$(mktemp -d)"
  tar -xJf "${binary}" -C "${tmpdir}"
  binary="$(find "${tmpdir}" -type f -name magellan | head -n 1)"
  if [[ -z "${binary}" || ! -x "${binary}" ]]; then
    echo "could not find extracted magellan binary in ${tmpdir}" >&2
    exit 1
  fi
fi

"${binary}" --help >/dev/null
"${binary}" schema >/dev/null
"${binary}" example --preset walkthrough >/dev/null
"${binary}" prompt --agent-type codex >/dev/null
