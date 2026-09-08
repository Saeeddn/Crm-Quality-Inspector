#!/usr/bin/env bash
# scripts/clean-target.sh
#
# Why this script exists:
#   Cargo's `target/` directory grows unbounded — every incremental build keeps
#   old .pdb files (98MB+ each on Windows debug), stale .rlib artifacts, and
#   incremental cache. After a few months of active development this hits 5-10GB
#   even though .gitignore excludes it. The user hit 8.7GB once; this script is
#   the one-liner to nuke it safely.
#
# Usage:
#   bash scripts/clean-target.sh           # clean and rebuild-free
#   bash scripts/clean-target.sh --rebuild # clean and run `cargo build` afterwards
#
# Cross-platform:
#   - git-bash (Windows): primary target
#   - macOS / Linux: works as-is

set -euo pipefail

cd "$(dirname "$0")/.."  # always run from project root

if [[ ! -d target ]]; then
  echo "✓ target/ already clean (does not exist)."
  exit 0
fi

size_before=$(du -sh target 2>/dev/null | awk '{print $1}')
echo "→ Cleaning target/ (current size: ${size_before})..."

# `cargo clean` first — handles file locks gracefully (some Windows sessions
# have Explorer indexing locks on .pdb files that a bare `rm -rf` can't unlock).
if cargo clean 2>&1 | tail -5; then
  echo "✓ cargo clean succeeded."
else
  # cargo clean failed (likely file lock). Fall back to direct rm.
  echo "⚠️  cargo clean failed — falling back to rm -rf (file lock likely)."
  rm -rf target
fi

if [[ -d target ]]; then
  echo "❌ target/ still exists — investigate before proceeding."
  exit 1
fi

size_after=$(du -sh . 2>/dev/null | awk '{print $1}')
echo "✓ Done. Project size now: ${size_after}"

if [[ "${1:-}" == "--rebuild" ]]; then
  echo "→ Rebuilding with cargo build..."
  cargo build
fi