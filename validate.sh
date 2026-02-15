#!/bin/bash
set -e

echo "--- 🛠️  VALIDATION PIPELINE 🛠️  ---"

# 1. Formatting Check
echo "Step 1: Checking Formatting (cargo fmt)..."
cargo fmt --all -- --check

# 2. Static Analysis
echo "Step 2: Running Linter (cargo clippy)..."
cargo clippy -- -D warnings

# 3. Functional Testing
echo "Step 3: Executing Test Suite (cargo test)..."
cargo test

echo "--- ✅ VALIDATION SUCCESSFUL: PRODUCT IS STABLE ✅ ---"
