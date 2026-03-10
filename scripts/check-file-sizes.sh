#!/bin/bash
# check-file-sizes.sh - Enforce the 300-line maximum per source file.
#
# Usage:
#   ./scripts/check-file-sizes.sh rust [file1.rs file2.rs ...]
#   ./scripts/check-file-sizes.sh typescript [file1.ts file2.tsx ...]

MAX_LINES=300
LANG=${1:-rust}
shift || true

VIOLATIONS=0
CHECKED=0

check_file() {
    local file="$1"

    # Skip test files, generated files, and build artifacts
    case "$file" in
        *.test.*|*generated*|*/gen/*|*target/*) return ;;
    esac

    [ -f "$file" ] || return

    # Count total lines using wc (simple, reliable)
    TOTAL=$(wc -l < "$file")

    CHECKED=$((CHECKED + 1))

    if [ "$TOTAL" -gt "$MAX_LINES" ]; then
        echo "FAIL $file: $TOTAL lines (max $MAX_LINES)"
        VIOLATIONS=$((VIOLATIONS + 1))
    else
        echo "OK   $file: $TOTAL lines"
    fi
}

if [ "$#" -gt 0 ]; then
    for file in "$@"; do
        check_file "$file"
    done
elif [ "$LANG" = "rust" ]; then
    TMPFILE=$(mktemp)
    find src-tauri/formats -name "*.rs" -type f | sort > "$TMPFILE"
    while IFS= read -r file; do
        check_file "$file"
    done < "$TMPFILE"
    rm -f "$TMPFILE"
elif [ "$LANG" = "typescript" ]; then
    TMPFILE=$(mktemp)
    find src -name "*.ts" -type f -o -name "*.tsx" -type f | sort > "$TMPFILE"
    while IFS= read -r file; do
        check_file "$file"
    done < "$TMPFILE"
    rm -f "$TMPFILE"
fi

echo ""
if [ "$VIOLATIONS" -gt 0 ]; then
    echo "FAIL: $VIOLATIONS file(s) exceed the $MAX_LINES line limit."
    echo "      Split large files into smaller focused modules."
    exit 1
fi

echo "PASS: All $CHECKED files are within the $MAX_LINES line limit."
exit 0
