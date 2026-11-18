<img src="../assets/logo/crest-primary.svg" alt="BeeWave logo" width="96" align="right">

# Branding

The logo is the **crest**: a bee flying above a wave. It comes from the design reference `A · Crest` (concept A).

## Colours
| Name | Value | Use |
|---|---|---|
| Ink | `#2B3035` | the bee and wave on a light page; the app's dark background |
| Paper | `#FFFFFF` | the page behind the primary logo; the bee's wings and stripes |

Two colourways: **primary** (ink on paper, `assets/logo/crest-primary.svg`) for light pages, and **reversed** (paper on ink,
`assets/logo/crest-reversed.svg`) for dark ones. The wings, stripes and eye are "paper" coloured, so the logo should sit on a
flat background of that colour.

## Where it is used
| Place | File | Notes |
|---|---|---|
| Top bar of the app | `web/src/shared/brand/crest.svg` (inline) | logo only (the name is kept for screen readers); follows the theme: `--crest-ink` is the text colour, `--crest-paper` the bar colour |
| Page before the app starts | `web/public/icon.svg` | shown by `index.html` until the app has drawn |
| Browser tab | `web/public/favicon.svg` and `favicon.ico` (16, 32, 48 px) | the same crest, drawn larger so it reads at tab size |
| Installed PWA / home screen | `icon-192.png`, `icon-512.png`, `icon-maskable-512.png` (in `web/public`, listed in the manifest in `vite.config.ts`) | the maskable icon keeps the art inside the safe zone |
| iPhone home screen | `apple-touch-icon.png` (180 px) | |
| README and docs | `assets/logo/crest-primary.svg`, `crest-reversed.svg` | the README switches with the reader's colour scheme |

The firmware has no display, so the logo does not appear on the device. Its Bluetooth name stays `BeeWave`.

## Size
The source SVG is about 1.3 KB. The generated PNGs use a 16-colour palette (the art is flat colour, so nothing is lost):
the 512 px icon is about 4 KB, the 192 px icon 1.5 KB, and the whole set is under 20 KB.

## Changing or regenerating it
Edit **one** file, `web/src/shared/brand/crest.svg` (its colours are CSS variables with the brand colours as fallbacks), then run

```
make logo
```

which rewrites `assets/logo/*.svg` and every icon in `web/public` (needs `inkscape` and ImageMagick; both are in `nix develop`).
Do not edit the generated files by hand.
