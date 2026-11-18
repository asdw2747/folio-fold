#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
ARCH="${ARCH:-x86_64}"
BIN="$ROOT/target/release/folio-fold"
APPDIR="$ROOT/target/appimage/FolioFold.AppDir"
OUT="$ROOT/dist/FolioFold-${VERSION}-${ARCH}.AppImage"

if [ ! -x "$BIN" ]; then
    echo "missing $BIN; run: cargo build --release" >&2
    exit 1
fi
if ! command -v appimagetool >/dev/null 2>&1; then
    echo "appimagetool was not found on PATH" >&2
    exit 1
fi

rm -rf "$APPDIR" "$OUT"
mkdir -p \
    "$APPDIR/usr/bin" \
    "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps" \
    "$APPDIR/usr/share/doc/folio-fold" \
    "$APPDIR/usr/share/licenses/folio-fold"

install -m755 "$BIN" "$APPDIR/usr/bin/folio-fold"
install -m755 "$ROOT/packaging/appimage/AppRun" "$APPDIR/AppRun"
install -m644 "$ROOT/packaging/appimage/folio-fold.desktop" \
    "$APPDIR/folio-fold.desktop"
install -m644 "$ROOT/assets/icon-256.png" "$APPDIR/folio-fold.png"
install -m644 "$ROOT/assets/icon-256.png" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps/folio-fold.png"
install -m644 "$ROOT/packaging/appimage/folio-fold.desktop" \
    "$APPDIR/usr/share/applications/folio-fold.desktop"
install -m644 "$ROOT/LICENSE" \
    "$APPDIR/usr/share/licenses/folio-fold/LICENSE"
install -m644 "$ROOT/THIRD_PARTY_NOTICES.md" \
    "$APPDIR/usr/share/doc/folio-fold/THIRD_PARTY_NOTICES.md"
cp -R "$ROOT/LICENSES" \
    "$APPDIR/usr/share/licenses/folio-fold/LICENSES"

cp "$APPDIR/folio-fold.png" "$APPDIR/.DirIcon"
mkdir -p "$ROOT/dist"
ARCH="$ARCH" appimagetool --no-appstream "$APPDIR" "$OUT"
chmod 755 "$OUT"
echo "Built $OUT"
