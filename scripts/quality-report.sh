#!/bin/bash
# quality-report.sh - Generate a full code quality report for the project.
#
# Usage:
#   ./scripts/quality-report.sh [--save]
#
# With --save, writes output to quality-reports/YYYY-MM-DD.txt.

set -uo pipefail

SAVE=false
if [[ "${1:-}" == "--save" ]]; then
    SAVE=true
fi

REPORT=""
append() { REPORT="$REPORT
$1"; }

append "=== AppThere Loki Quality Report ==="
append "Generated: $(date)"
append ""

# ── Rust metrics ─────────────────────────────────────────────────────────────
append "## Rust Metrics"

append ""
append "### File Size Compliance (formats/)"
if OUTPUT=$(bash scripts/check-file-sizes.sh rust 2>&1); then
    append "PASS"
else
    append "$OUTPUT"
fi

append ""
append "### Clippy Warnings"
if command -v cargo &>/dev/null; then
    WARN_COUNT=$(cargo clippy -p common-core -p odt-format --all-targets 2>&1 | grep -c "^warning" || true)
    append "Warnings: $WARN_COUNT"
else
    append "cargo not found"
fi

append ""
append "### Format Check"
if command -v cargo &>/dev/null; then
    if cargo fmt --all -- --check 2>/dev/null; then
        append "PASS: all Rust files are formatted"
    else
        append "FAIL: some Rust files need formatting (run: cargo fmt --all)"
    fi
else
    append "cargo not found"
fi

# ── TypeScript metrics ────────────────────────────────────────────────────────
append ""
append "## TypeScript Metrics"

append ""
append "### File Size Compliance (src/)"
if OUTPUT=$(bash scripts/check-file-sizes.sh typescript 2>&1); then
    append "PASS"
else
    append "$OUTPUT"
fi

append ""
append "### Type Check"
if command -v npx &>/dev/null && [ -f tsconfig.json ]; then
    if npx tsc --noEmit 2>&1 | tail -3; then
        append ""
    fi
else
    append "TypeScript not configured"
fi

append ""
append "=== End Report ==="

echo "$REPORT"

if $SAVE; then
    mkdir -p quality-reports
    OUT="quality-reports/$(date +%Y-%m-%d).txt"
    echo "$REPORT" > "$OUT"
    echo "Report saved to $OUT"
fi
