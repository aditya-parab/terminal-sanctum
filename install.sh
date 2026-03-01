#!/bin/bash
set -e

# --- 🛡️  TERMINAL SANCTUM: ROBUST DEPLOYMENT SCRIPT 🛡️  ---
# This script is designed for portability across Linux and macOS.

# Function to check for dependencies
check_dep() {
    if ! command -v "$1" &> /dev/null; then
        echo "❌ Error: Required dependency '$1' is not installed."
        exit 1
    fi
}

# Function to check environment health
check_environment() {
    # Check for cargo
    check_dep "cargo"

    # On macOS, check if Xcode license is agreed to (common failure point)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if ! /usr/bin/xcrun clang --version &> /dev/null; then
             echo "⚠️  Warning: macOS toolchain issue detected (likely Xcode license)."
             echo "Please run: 'sudo xcodebuild -license' to proceed with system-level compilation."
             echo "Alternatively, a containerized build is recommended."
             # We don't exit here as some users might have a custom LLVM setup,
             # but we warn them before the cargo build fails.
        fi
    fi
}

# 0. Initial Health Check
echo "--- [0/4] Environment Pre-flight ---"
check_environment

# 1. Purge Old Artifacts
echo "[1/4] Performing cache purge (cargo clean)..."
cargo clean --quiet

# 2. Hardened Validation
# We use a flag to allow skipping strict validation for "any machine" deployment
# if a developer already validated the code elsewhere.
SKIP_VALIDATE=${SKIP_VALIDATE:-false}

if [ "$SKIP_VALIDATE" = false ]; then
    echo "[2/4] Executing Zero-Warning Pipeline..."
    # We run clippy on ALL targets (including tests) to catch dead code in tests too.
    if ! cargo clippy --all-targets -- -D warnings; then
        echo "❌ Linter Failure: One or more warnings detected."
        echo "The Council Strategy requires Zero-Warning builds for deployment."
        exit 1
    fi

    if ! cargo fmt --all -- --check; then
        echo "❌ Formatting Failure: Code is not standardized."
        exit 1
    fi
else
    echo "[2/4] Skipping Hardened Validation (Manual Override active)..."
fi

# 3. Comprehensive Testing
echo "[3/4] Running 100% Logic & Integrity battery..."
if ! cargo test --quiet; then
    echo "❌ Test Failure: Logic regression detected."
    exit 1
fi

# 4. Nuclear Installation
echo "[4/4] Finalizing global link..."
# Use --locked to ensure reproducible builds from Cargo.lock
cargo install --path . --quiet --force --locked

echo ""
echo "✅ DEPLOYMENT SUCCESSFUL: v2.9.5-ULTIMATE is active."
echo "---"
echo "Command: 'sanctum' is synchronized."
