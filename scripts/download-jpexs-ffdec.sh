#!/usr/bin/env bash
# Download and extract the pinned JPEXS portable zip into src-tauri/resources/jpexs/
# Mirrors src-tauri/build.rs — keep URL and hash in sync when upgrading.

set -euo pipefail

RELEASE_TAG="version26.0.0"
ZIP_NAME="ffdec_26.0.0.zip"
URL="https://github.com/jindrapetrik/jpexs-decompiler/releases/download/${RELEASE_TAG}/${ZIP_NAME}"
EXPECTED_SHA256="e13509d0ed11c6d77bb1588701d803eb2c706e2ba4eaab8bc4477255cfc718f4"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${ROOT}/src-tauri/resources/jpexs"

sha256_file() {
  local f="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$f" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$f" | awk '{print $1}'
  else
    echo "error: need sha256sum or shasum in PATH" >&2
    exit 1
  fi
}

TMP="${TMPDIR:-/tmp}/stardelta-${ZIP_NAME}.$$"
cleanup() { rm -f "$TMP"; }
trap cleanup EXIT

if [[ -n "${STARDELTA_FFDEC_ZIP:-}" ]]; then
  cp "${STARDELTA_FFDEC_ZIP}" "$TMP"
else
  echo "Downloading ${URL} ..."
  curl -fL --retry 3 -o "$TMP" "${URL}"
fi

GOT="$(sha256_file "$TMP")"
if [[ "${GOT}" != "${EXPECTED_SHA256}" ]]; then
  echo "SHA-256 mismatch." >&2
  echo "  expected: ${EXPECTED_SHA256}" >&2
  echo "  got:      ${GOT}" >&2
  exit 1
fi

rm -rf "${OUT}"
mkdir -p "${OUT}"
echo "Extracting to ${OUT} ..."
unzip -q -o "$TMP" -d "${OUT}"

if [[ ! -f "${OUT}/ffdec.jar" ]]; then
  echo "error: ffdec.jar not found after extract" >&2
  exit 1
fi

echo "OK: ${OUT}/ffdec.jar"
