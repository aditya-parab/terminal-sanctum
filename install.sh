#!/bin/bash
set -e

echo "--- 🛡️  TERMINAL SANCTUM: STANDARD DEPLOYMENT 🛡️  ---"

# 1. Purge Old Artifacts
echo "[1/4] Performing cache purge (cargo clean)..."
cargo clean --quiet

# 2. Hardened Validation
echo "[2/4] Executing Zero-Warning Pipeline..."
# We run clippy on ALL targets (including tests) to catch dead code in tests too.
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check

# 3. Comprehensive Testing
echo "[3/4] Running 100% Logic & Integrity battery..."
cargo test --quiet

# 4. Nuclear Installation
echo "[4/4] Finalizing global link..."
cargo install --path . --quiet --force

echo ""
echo "✅ DEPLOYMENT SUCCESSFUL: v2.9.5-ULTIMATE is active."
echo "---"
echo "Command: 'sanctum' is synchronized."
