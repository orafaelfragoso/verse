#!/usr/bin/env bash
# Concatenate the Verse spec parts into one Markdown file and render HTML/PDF.
#   ./build.sh                 -> ../dist/verse-spec.md (+ .html, .pdf)
#   OUT=/tmp/x.md ./build.sh   -> another target
#   NO_RENDER=1 ./build.sh     -> Markdown only
set -euo pipefail
cd "$(dirname "$0")"
OUT="${OUT:-$PWD/../dist/verse-spec.md}"
mkdir -p "$(dirname "$OUT")"

PARTS=(
  01-philosophy.md 02-tour.md 03-lexical.md 04-types.md 05-functions.md
  06-control-flow.md 07-pattern-matching.md 08-errors.md 09-collections.md
  10-traits-generics.md 11-concurrency.md 12-modules.md 13-resources.md
  14-testing-docs.md 15-stdlib.md 16-tooling.md
  grammar.ebnf.md            # slot 17 (17-grammar.md is only a stub)
  18-design-notes.md 19-comparison.md 20-examples.md 21-review-log.md
)

tmp="$(mktemp)"
# Front matter: the region of 00-README.md between the spec markers.
awk '/^<!-- spec:end/{f=0} f{print} /^<!-- spec:begin/{f=1}' 00-README.md > "$tmp"
for p in "${PARTS[@]}"; do cat "$p" >> "$tmp"; done
# Parts link to each other as `NN-name.md#anchor` (see tools/crosslink.py);
# in the single combined file those become in-page `#anchor` links again.
sed -E -i 's/\]\((([0-9]{2}-[a-z-]+)|grammar\.ebnf)\.md#/](#/g' "$tmp"
mv "$tmp" "$OUT"
echo "wrote $OUT ($(wc -l < "$OUT") lines)"

[[ -n "${NO_RENDER:-}" ]] && exit 0

PY="${VERSE_PY:-$HOME/.cache/verse-mdenv/bin/python}"
if ! "$PY" -c 'import markdown, pygments' 2>/dev/null; then
  python3 -m venv "$HOME/.cache/verse-mdenv"
  "$HOME/.cache/verse-mdenv/bin/pip" -q install markdown pygments
  PY="$HOME/.cache/verse-mdenv/bin/python"
fi
HTML="${OUT%.md}.html"; PDF="${OUT%.md}.pdf"
"$PY" tools/render.py "$OUT" "$HTML"
echo "wrote $HTML"
if command -v google-chrome >/dev/null; then
  google-chrome --headless --no-sandbox --disable-gpu --no-pdf-header-footer \
    --print-to-pdf="$PDF" "file://$HTML" 2>/dev/null
  echo "wrote $PDF"
fi
