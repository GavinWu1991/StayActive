#!/usr/bin/env bash
# Validate that a Git tag (vX.Y.Z) matches src-tauri/tauri.conf.json version (X.Y.Z).
set -euo pipefail

TAG="${RELEASE_TAG:-${GITHUB_REF_NAME:-}}"
CONF="src-tauri/tauri.conf.json"

if [ -z "$TAG" ]; then
  echo "RELEASE_TAG or GITHUB_REF_NAME is required" >&2
  exit 1
fi

if [[ ! "$TAG" =~ ^v ]]; then
  echo "Release tag must start with 'v' (got: $TAG)" >&2
  exit 1
fi

TAG_VERSION="${TAG#v}"
CONF_VERSION="$(python3 -c "import json; print(json.load(open('$CONF'))['version'])")"

if [ "$TAG_VERSION" != "$CONF_VERSION" ]; then
  echo "Tag version mismatch: tag=$TAG (version $TAG_VERSION) vs $CONF version=$CONF_VERSION" >&2
  echo "Bump src-tauri/tauri.conf.json (and package.json if needed) before tagging." >&2
  exit 1
fi

echo "PASS: tag $TAG matches app version $CONF_VERSION"
