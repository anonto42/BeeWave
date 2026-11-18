#!/usr/bin/env bash
# Regenerate every logo asset from the one source, web/src/shared/brand/crest.svg (colours are CSS variables
# with fallbacks; here they are baked). Needs inkscape and ImageMagick (both are in `nix develop`).
#   assets/logo/crest-primary.svg     dark bee on a light page
#   assets/logo/crest-reversed.svg    light bee on a dark page
#   web/public/icon.svg + PNGs        the app icon (reversed crest on the app's dark colour)
#   web/public/favicon.svg + .ico     the browser-tab icon (same crest, drawn larger)
set -euo pipefail
cd "$(dirname "$0")/.."
SRC=web/src/shared/brand/crest.svg
DARK='#2B3035'
LIGHT='#FFFFFF'
PUB=web/public
OUT=assets/logo
mkdir -p "$OUT" "$PUB"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

bake() { sed "s/var(--crest-ink,#2B3035)/$1/g; s/var(--crest-paper,#FFFFFF)/$2/g" "$SRC"; }
bake "$DARK" "$LIGHT" > "$OUT/crest-primary.svg"
bake "$LIGHT" "$DARK" > "$OUT/crest-reversed.svg"

# App icon: the reversed crest on the dark square, inside the 80 % safe zone so it also works as a maskable icon.
inner=$(sed -e 's/^<svg[^>]*>//' -e 's/<\/svg>$//' "$OUT/crest-reversed.svg")
cat > "$PUB/icon.svg" <<EOF
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><rect width="512" height="512" fill="$DARK"/><g transform="translate(70 39) scale(1.55)">$inner</g></svg>
EOF

# Favicon: the same crest, drawn larger (no maskable margin) so it stays readable at 16-32 px.
cat > "$PUB/favicon.svg" <<EOF
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><rect width="512" height="512" fill="$DARK"/><g transform="translate(-21 -68) scale(2.3)">$inner</g></svg>
EOF

png() { # size, output: render, then squeeze (flat colours: a small palette is lossless here)
  inkscape "$PUB/icon.svg" --export-type=png --export-width="$1" --export-height="$1" --export-filename="$TMP/r.png" >/dev/null 2>&1
  convert "$TMP/r.png" -strip -colors 16 -define png:compression-level=9 -define png:exclude-chunk=all PNG8:"$2"
}
png 192 "$PUB/icon-192.png"
png 512 "$PUB/icon-512.png"
png 180 "$PUB/apple-touch-icon.png"
# maskable: full-bleed square (the OS applies its own mask), same artwork
png 512 "$PUB/icon-maskable-512.png"
for s in 16 32 48; do inkscape "$PUB/favicon.svg" --export-type=png --export-width=$s --export-height=$s --export-filename="$TMP/f$s.png" >/dev/null 2>&1; done
convert "$TMP/f16.png" "$TMP/f32.png" "$TMP/f48.png" -colors 16 "$PUB/favicon.ico"
ls -l "$OUT" "$PUB" | grep -v '^total'
