#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
SWIFT_PACKAGE="${ROOT_DIR}/native/apple/BridgeKitStoreKit"
PROFILE="${1:-debug}"

swift build -c "${PROFILE}" --package-path "${SWIFT_PACKAGE}"

echo "Built BridgeKitStoreKit (${PROFILE}) at:"
echo "  ${SWIFT_PACKAGE}/.build/${PROFILE}/libBridgeKitStoreKit.dylib"
