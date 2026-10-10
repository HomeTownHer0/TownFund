#!/usr/bin/env bash
# Build the Devnet (Demo Day) version of TownFund.
# Same program as ./scripts/build.sh, plus the `demo` feature, which also
# accepts a 10-minute project so a full run fits in a live demo.
set -euo pipefail
cd "$(dirname "$0")/.."
anchor build --tools-version v1.54 --arch v0 -- --features demo
cargo test --features demo
echo "Built target/deploy/townfund.so with the demo feature."
