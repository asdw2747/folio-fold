# FolioFold 1.0.0

FolioFold is a native desktop application for PDF imposition and signature layout in physical bookbinding. It turns a normal reading-order PDF into print-ready folded sheets using pure-Rust PDF manipulation.

## Highlights

- **Booklet / saddle stitch** — single nested signature, 2-up duplex, padded to a multiple of four pages.
- **Multi-signature bookbinding** — 2–16 leaves per signature as presets, including 14 leaves / 28 pages, plus custom even-leaf sizes up to 32 leaves.
- **Quarto** — 4 pages per side, 2×2 head-to-head sheet layout.
- **Octavo** — 8 pages per side, 4×2 head-to-head sheet layout.
- **Duplex modes** — short-edge/tumble and long-edge/standard back-page rotation.
- **Lossless vector embedding** — source pages become Form XObjects; text, fonts, vectors, and images are preserved without rasterization.
- **Bookbinder geometry** — paper presets, custom sizes, fit/exact scaling, outer margin, spine gutter, and progressive creep compensation.
- **Printer marks** — fold guides, crop/trim marks, and signature/sheet sequence labels.
- **Interactive preview** — front/back physical sheet view, page navigation, rotation indicators, zoom/pan, and page inspection.
- **Safe export** — atomic writes, cancellation, separate front/back PDFs, and source-change detection.

## Downloads

- `FolioFold-1.0.0-x86_64.AppImage` — Linux x86_64.
- `FolioFold-1.0.0-x86_64.exe` — Windows x86_64, static CRT; no VC++ Redistributable required.
- `SHA256SUMS` — verify downloaded artifacts with `sha256sum -c SHA256SUMS`.

### Linux

```bash
chmod +x FolioFold-1.0.0-x86_64.AppImage
./FolioFold-1.0.0-x86_64.AppImage
```

Requires X11 or Wayland with OpenGL. The AppImage targets the glibc version of the build host; if your distribution is much older, use the source build instead.

### Windows

Run `FolioFold-1.0.0-x86_64.exe` directly. It is a self-contained GUI build with the static Microsoft C runtime.

## Source build

```bash
cargo build --release
cargo xwin build --release --target x86_64-pc-windows-msvc
./scripts/build-appimage.sh
```

## Known limitations

- Encrypted PDFs must be unprotected before import.
- Source artwork is cropped to CropBox; BleedBox/TrimBox are not preserved.
- Annotations, form fields, optional-content state, and catalog OutputIntents are not imposed.
- Quarto and Octavo sheets are independent folded sections.
- The preview is a structural page-number preview, not a rasterized content thumbnail.
- Always print a test sheet before a full binder run.

## License

FolioFold is released into the public domain under the Unlicense. Third-party crate and font notices are included in the repository and in the release archive.
