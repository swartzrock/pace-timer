# FIGlet fonts

The `fonts/*.flf` files are the complete 289-font catalog selected by
`pace-ts/src/common/fonts.ts`, copied from the `figlet` npm package version
1.5.2, published by Patrick Gillespie:

- Project: https://github.com/patorjk/figlet.js
- Package archive: https://registry.npmjs.org/figlet/-/figlet-1.5.2.tgz
- Package license: [FIGLET-LICENSE.txt](FIGLET-LICENSE.txt) (MIT)

Individual fonts retain their authors' copyright, permission, and attribution
notices in their original comment headers. The original `.flf` files are
preserved byte for byte. Some contain legacy non-UTF-8 bytes; their accompanying
`.utf8.flf` copies use the same UTF-8 replacement decoding as Node's
`fs.readFileSync(path, 'utf8')`. Rust embeds these decoded copies when needed so
the rendered output matches the original program. The original copies preserve
the source notices for inspection.

The renderer in `src/fonts.rs` ports FIGlet.js 1.5.2's default horizontal and
vertical layout rules. The FIGlet.js attribution header is retained there.
No npm package, network access, or external font directory is required at
runtime.

`fonts/snapshots/*.txt` are characterization fixtures generated before the
Rust implementation using the original package's `textSync` with
`horizontalLayout: 'default'` and `verticalLayout: 'default'`. Each font's
catalog fixture renders `23 Quick foxes`; additional fixtures cover `55:55`,
punctuation, and multiline text for the timer fonts and Standard. Whitespace
and row padding are intentional. Additional fixtures cover extended glyphs
and right-to-left font direction.
