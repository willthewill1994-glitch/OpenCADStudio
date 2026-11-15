# Native named plot styles: audit and regression coverage

Audited OpenCADStudio `0372abff846cbf2cd8f2243a61b1d8738d7c08dc`.

## Architecture and format

The application is Rust/Iced. `document` is `opencaddocument` at `ae40a59`,
which reexports `acadrust` at `f946e8d`. Its `plot_style` module owns the
CTB/STB model, parser and native serializer. OpenCADStudio's
`src/io/plot_style.rs` owns file discovery, session registration, built-ins
and application rendering adapters. No new third-party code was copied:
the existing MPL-2.0 document/codec dependencies remain dependencies of this
GPL-3.0 application.

The native container has a 60-byte header, a `PIAFILEVERSION_2.0` signature,
a `CTBVER1` or `STBVER1` tag, compressed-payload Adler-32, uncompressed and
compressed lengths, and a zlib text payload. STBs use named `plot_style`
entries; CTBs use ACI rows. The codec already tolerates reported container
inconsistencies and supports Windows-1252 text. These warnings are retained
in `load_warnings`; malformed input still fails through the shared codec.

`encode_stb` previously wrote just zlib text, with `aci_table_available=TRUE`
and unordered entries, despite the shared native STB serializer being
available. It now delegates to the same serializer as disk `save`, with a
native header and deterministic style ordering.

## Drawing and PDF path

`header.plotstyle_mode == false` denotes a named-style drawing. Entity
`plotstyle_flags` encode ByLayer (0), ByBlock (1), Normal (2), and an explicit
style handle (3). Layers carry `plotstyle_handle`; handles resolve through
`ACAD_PLOTSTYLENAME`, preferably from the named objects dictionary, then the
header pointer. The underlying DWG reader/writer already preserves these
fields and the dictionary-with-default objects.

`plot_scene_content` gathers geometry and `plot_entity_styles` resolves the
entity's style name. `PlotContent.entity_plot_styles` carries those names
into the shared PDF renderer used for export, preview and printing. Each
named style selects an entry with Normal fallback. The new changes:

- apply named colours, screening and lineweights to true-colour objects as
  well as indexed objects, while CTBs retain their true-colour exclusion;
- take caps, joins and fill patterns from the selected named entry;
- resolve grayscale against actual object RGB, including individual text
  runs/glyphs;
- associate hatches through `HatchModel.entity`, not the pattern name;
- keep object properties when Normal is absent, rather than selecting an
  arbitrary named style from a hash map.

The old “STB not supported yet” message exists only as translation catalog
history in this revision; it is no longer an active loader rejection.
An installation showing it predates the audited implementation.

## Fixtures and checks

`tests/fixtures/plot/named-pens.stb` is an independently constructed native
container, not a user file or a fixture produced by the serializer under
test. `named-pens.txt` is its UTF-8 payload. It defines Normal, a blue
0.5 mm pen with butt caps/miter joins, and a grayscale 50% pen. The payload
checksum and lengths match the container.

Unit tests cover native load/encode/reload, accented names and filenames,
true-colour grayscale, CTB exclusion, absent Normal and PDF true-colour
fills/hatches. An application integration test generates a DWG with named
style placeholders and layer/entity assignments, reloads it, resolves
ByLayer/ByBlock/Normal/explicit styles and checks emitted PDF RGB/cap
operators. No proprietary test data or new library license is needed.

## Remaining fidelity limits

Expanded wires and hatches now carry `effective_plot_style` per occurrence.
The block cache resolves explicit/Normal, ByLayer (including layer 0), and
ByBlock through nested INSERT contexts. Style grouping and both root and
nested prototype keys include named-style identity. Clones, stub expansion,
clipped geometry and projected hatch pattern wires preserve the metadata.
The PDF renderer prefers occurrence metadata over the legacy handle map.
A separate integration regression checks nested children and repeated inserts
whose display properties coincide but whose plot pens differ.

This change does **not** establish complete AutoCAD plot fidelity.
The table's linetype/adaptive-linetype settings are also not fully mapped
by the existing vector renderer. Physical/virtual pens and dithering are
not PDF device features. SVG and on-screen “display plot styles” need their
own consistency audit; the fixes here target the shared PDF renderer.

The user's `Houle_ech-20à50.stb` is not required to develop these regressions.
A paired user STB/DWG and an AutoCAD reference PDF would be useful only for
subsequent fidelity validation, especially block inheritance and linetypes.

## Validation setup

The monolithic `cargo test --lib native_stb` build exceeded the environment's
memory limit and was killed with exit 137 before executing tests. The new
codec unit tests and DWG/PDF checks therefore live in the existing integration
runner. They call the same public resolver that production uses; the block
checks exercise the real cache. They do not mock the codec or the PDF writer.
The retry uses `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1` to bound build memory.
The complete pre-existing internal unit suite is not claimed as verified.

Executed successfully on Linux, Rust 1.100.0 (19 tests total):

| Command filter (`cargo test --test integration …`) | Passed |
| --- | ---: |
| `stb_` | 6 |
| `block_hatch_export` | 7 |
| `block_text_draw_depth` | 1 |
| `pdf_export_text_check` | 1 |
| `xclip_plot_test` | 4 |

The native executable also built successfully with
`CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --bin OpenCADStudio`.
Its `--help` entry point exited successfully. Existing unrelated compiler
warnings remain; no release, Windows, wasm, or AutoCAD-reference build is
claimed by these checks.

## Changed files

- `benches/hatch_pattern_segments.rs`
- `docs/development/named-plot-styles.md`
- `src/app/update/file.rs`
- `src/entities/multileader.rs`
- `src/io/pdf_export.rs`
- `src/io/plot_style.rs`
- `src/io/svg_export.rs`
- `src/modules/draw/draw/hatch.rs`
- `src/scene/cache/block_cache.rs`
- `src/scene/convert/tessellate.rs`
- `src/scene/entity.rs`
- `src/scene/mod.rs`
- `src/scene/model/hatch_model.rs`
- `src/scene/model/wire_model.rs`
- `src/scene/pipeline/wipeout_gpu_tests.rs`
- `src/scene/preview.rs`
- `src/scene/project.rs`
- `src/scene/render_graph.rs`
- `src/scene/text/complex_lt.rs`
- `src/scene/view/render.rs`
- `src/ui/properties.rs`
- `tests/block_hatch_export.rs`
- `tests/fixtures/plot/named-pens.stb`
- `tests/fixtures/plot/named-pens.txt`
- `tests/integration.rs`
- `tests/stb_dwg_pdf.rs`
- `tests/stb_named_blocks.rs`
- `tests/stb_native_codec.rs`
