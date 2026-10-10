#!/usr/bin/env bash
# Build TownFund and run its tests.
#   --tools-version v1.54 : platform-tools that ship with Solana CLI 4.1.2
#   --arch v0             : classic SBPF format; loads in LiteSVM and on Devnet
set -euo pipefail
cd "$(dirname "$0")/.."
anchor build --tools-version v1.54 --arch v0
cargo test
