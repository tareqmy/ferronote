# Ferronote — Logo Files

**Concept:** the shell prompt `>_` with a motion-ghost chevron. Reads as "terminal" and "fast-forward" at once.

## Files

| File | Use |
|---|---|
| `ferronote-logo-dark.svg` / `.png` | Primary lockup on dark backgrounds (README hero, website) |
| `ferronote-logo-light.svg` / `.png` | Lockup for light backgrounds (docs) |
| `ferronote-icon.svg` | Icon, dark backgrounds |
| `ferronote-icon-light.svg` | Icon, light backgrounds |
| `ferronote-icon-mono-black.svg` / `-white.svg` | Single-color versions (also recolorable via `currentColor` → edit the fill) |
| `ferronote-icon-{16..512}.png` | Transparent PNGs (GitHub avatar: 512, app icon: 256) |
| `favicon.ico` | 16 + 32 + 48 multi-size favicon |

## Palette

- Rust orange (dark bg): `#E24E1B`
- Rust orange (light bg): `#CE422B`
- Cream: `#F2ECE4`
- Ink: `#201B17`

## Notes

- Wordmark is outlined (Liberation Mono Bold converted to paths) — no font dependency.
- Ghost chevron uses 38% opacity; in single-color contexts use the mono versions where it's solid.
- ASCII fallback for terminal splash: `»_`
