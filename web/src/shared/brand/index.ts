import crest from "./crest.svg?raw";

/**
 * The BeeWave crest as inline SVG at `size` px. Its ink and paper colours follow the page (`--crest-ink` and
 * `--crest-paper`, set by `.crest` in style.css), so it is right in the dark and the light theme.
 */
export const crestSvg = (size = 30): string => crest.replace("<svg ", `<svg class="crest" width="${size}" height="${size}" `);
