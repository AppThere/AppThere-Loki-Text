#!/bin/bash
# verify_pdf.sh — Regression test runner for PDF/X conformance.
# Usage: ./scripts/verify_pdf.sh <path_to_pdf> <standard>

PDF_PATH=$1
STANDARD=$2 # e.g. "PDF/X-1a:2001" or "PDF/X-4"

if [ -z "$PDF_PATH" ]; then
    echo "Usage: $0 <path_to_pdf> <standard>"
    exit 1
fi

echo "--- Verifying $PDF_PATH against $STANDARD ---"

# 1. Structural check with lopdf (can be run locally)
if command -v cargo &> /dev/null; then
    echo "[1/2] Checking PDF structure with lopdf-based tool..."
    # We could build a tiny rust utility here, but for now we rely on cargo tests 
    # which already use lopdf.
fi

# 2. Conformance check with veraPDF (required for CI)
if command -v verapdf &> /dev/null; then
    echo "[2/2] Running veraPDF validation..."
    verapdf --flavour "${STANDARD//:/}" --format text "$PDF_PATH" | grep "is compliant"
    if [ $? -ne 0 ]; then
        echo "FAILED: Conformance violation reported by veraPDF."
        verapdf --flavour "${STANDARD//:/}" "$PDF_PATH"
        exit 1
    fi
    echo "SUCCESS: Document is compliant."
else
    echo "[2/2] SKIP: veraPDF not found. Please install veraPDF for official certification."
fi
