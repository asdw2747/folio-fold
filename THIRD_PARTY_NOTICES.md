# Third-Party Notices

**FolioFold 1.0.0** is released under the Unlicense. See [`LICENSE`](LICENSE).

FolioFold is a native Rust desktop application. It statically links the Rust crates listed below and bundles the default egui font set plus the FolioFold application icon in the application binary. It does not invoke external PDF tools; all PDF parsing, page embedding, imposition, and writing are performed in-process by `lopdf`.

This table was generated from the resolved dependency graph in `Cargo.lock` (334 third-party packages, including platform-specific dependencies). Where a crate offers a choice of licenses, FolioFold relies on the permissive option unless a component is specifically noted below. Full license texts are in [`LICENSES/`](LICENSES/).

## Rust crates

| Crate | Version | License (SPDX) | Direct dependency |
| --- | --- | --- | --- |
| `accesskit` | 0.25.2 | MIT OR Apache-2.0 |  |
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |  |
| `aes` | 0.9.4 | MIT OR Apache-2.0 |  |
| `ahash` | 0.8.12 | MIT OR Apache-2.0 |  |
| `alloc-no-stdlib` | 2.0.4 | BSD-3-Clause |  |
| `alloc-stdlib` | 0.2.4 | BSD-3-Clause |  |
| `android-activity` | 0.6.1 | MIT OR Apache-2.0 |  |
| `android-properties` | 0.2.2 | MIT |  |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | yes |
| `arboard` | 3.7.0 | MIT OR Apache-2.0 |  |
| `arrayvec` | 0.7.8 | MIT OR Apache-2.0 |  |
| `as-raw-xcb-connection` | 1.0.1 | MIT OR Apache-2.0 |  |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT |  |
| `autocfg` | 1.5.1 | Apache-2.0 OR MIT |  |
| `bit-set` | 0.10.0 | Apache-2.0 OR MIT |  |
| `bit-vec` | 0.9.1 | Apache-2.0 OR MIT |  |
| `bitflags` | 1.3.2 | MIT/Apache-2.0 |  |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 |  |
| `block-buffer` | 0.12.1 | MIT OR Apache-2.0 |  |
| `block-padding` | 0.4.2 | MIT OR Apache-2.0 |  |
| `block2` | 0.5.1 | MIT |  |
| `block2` | 0.6.2 | MIT |  |
| `brotli-decompressor` | 5.0.3 | BSD-3-Clause/MIT |  |
| `bumpalo` | 3.20.4 | MIT OR Apache-2.0 |  |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT |  |
| `bytemuck_derive` | 1.12.1 | Zlib OR Apache-2.0 OR MIT |  |
| `byteorder-lite` | 0.1.0 | Unlicense OR MIT |  |
| `bytes` | 1.12.1 | MIT |  |
| `calloop` | 0.13.0 | MIT |  |
| `calloop-wayland-source` | 0.3.0 | MIT |  |
| `cbc` | 0.2.1 | MIT OR Apache-2.0 |  |
| `cc` | 1.11.2 | MIT OR Apache-2.0 |  |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 |  |
| `cfg_aliases` | 0.2.2 | MIT |  |
| `cgl` | 0.3.2 | MIT / Apache-2.0 |  |
| `chacha20` | 0.10.2 | MIT OR Apache-2.0 |  |
| `cipher` | 0.5.2 | MIT OR Apache-2.0 |  |
| `clipboard-win` | 5.4.3 | BSL-1.0 |  |
| `codespan-reporting` | 0.13.1 | Apache-2.0 |  |
| `color` | 0.3.3 | Apache-2.0 OR MIT |  |
| `combine` | 4.6.9 | MIT |  |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT |  |
| `const-oid` | 0.10.3 | Apache-2.0 OR MIT |  |
| `core-foundation` | 0.9.4 | MIT OR Apache-2.0 |  |
| `core-foundation-sys` | 0.8.7 | MIT OR Apache-2.0 |  |
| `core-graphics` | 0.23.2 | MIT OR Apache-2.0 |  |
| `core-graphics-types` | 0.1.3 | MIT OR Apache-2.0 |  |
| `core_detect` | 1.0.0 | MIT/Apache-2.0 |  |
| `cpubits` | 0.1.1 | MIT OR Apache-2.0 |  |
| `cpufeatures` | 0.3.1 | MIT OR Apache-2.0 |  |
| `crc32fast` | 1.5.5 | MIT OR Apache-2.0 |  |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 |  |
| `crunchy` | 0.2.4 | MIT |  |
| `crypto-common` | 0.2.2 | MIT OR Apache-2.0 |  |
| `cursor-icon` | 1.2.0 | MIT OR Apache-2.0 OR Zlib |  |
| `digest` | 0.11.3 | MIT OR Apache-2.0 |  |
| `dispatch` | 0.2.0 | MIT |  |
| `dispatch2` | 0.3.1 | Zlib OR Apache-2.0 OR MIT |  |
| `dlib` | 0.5.3 | MIT |  |
| `document-features` | 0.2.12 | MIT OR Apache-2.0 |  |
| `downcast-rs` | 1.2.1 | MIT/Apache-2.0 |  |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT |  |
| `ecb` | 0.2.1 | MIT OR Apache-2.0 |  |
| `ecolor` | 0.37.0 | MIT OR Apache-2.0 |  |
| `eframe` | 0.37.0 | MIT OR Apache-2.0 | yes |
| `egui` | 0.37.0 | MIT OR Apache-2.0 | yes |
| `egui-wgpu` | 0.37.0 | MIT OR Apache-2.0 |  |
| `egui-winit` | 0.37.0 | MIT OR Apache-2.0 |  |
| `egui_glow` | 0.37.0 | MIT OR Apache-2.0 |  |
| `either` | 1.19.0 | MIT OR Apache-2.0 |  |
| `emath` | 0.37.0 | MIT OR Apache-2.0 |  |
| `encoding_rs` | 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause |  |
| `epaint` | 0.37.0 | MIT OR Apache-2.0 |  |
| `epaint_default_fonts` | 0.37.0 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 |  |
| `equivalent` | 1.0.3 | Apache-2.0 OR MIT |  |
| `errno` | 0.3.15 | MIT OR Apache-2.0 |  |
| `error-code` | 3.4.0 | BSL-1.0 |  |
| `euclid` | 0.22.14 | MIT OR Apache-2.0 |  |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT |  |
| `fax` | 0.2.7 | MIT |  |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 |  |
| `fearless_simd` | 0.4.1 | Apache-2.0 OR MIT |  |
| `find-msvc-tools` | 0.1.15 | MIT OR Apache-2.0 |  |
| `fixedbitset` | 0.5.9 | MIT OR Apache-2.0 |  |
| `flate2` | 1.1.11 | MIT OR Apache-2.0 |  |
| `foldhash` | 0.1.5 | Zlib |  |
| `foldhash` | 0.2.0 | Zlib |  |
| `font-types` | 0.12.7 | MIT OR Apache-2.0 |  |
| `foreign-types` | 0.5.0 | MIT/Apache-2.0 |  |
| `foreign-types-macros` | 0.2.4 | MIT/Apache-2.0 |  |
| `foreign-types-shared` | 0.3.1 | MIT/Apache-2.0 |  |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 |  |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 |  |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 |  |
| `gethostname` | 1.1.0 | Apache-2.0 |  |
| `getrandom` | 0.3.4 | MIT OR Apache-2.0 |  |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 |  |
| `gl_generator` | 0.14.0 | Apache-2.0 |  |
| `glifo` | 0.3.0 | Apache-2.0 OR MIT |  |
| `glow` | 0.17.0 | MIT OR Apache-2.0 OR Zlib |  |
| `glutin` | 0.32.3 | Apache-2.0 |  |
| `glutin-winit` | 0.5.0 | MIT |  |
| `glutin_egl_sys` | 0.7.1 | Apache-2.0 |  |
| `glutin_glx_sys` | 0.6.1 | Apache-2.0 |  |
| `glutin_wgl_sys` | 0.6.1 | Apache-2.0 |  |
| `guillotiere` | 0.7.0 | MIT/Apache-2.0 |  |
| `half` | 2.7.1 | MIT OR Apache-2.0 |  |
| `harfrust` | 0.12.0 | MIT |  |
| `hashbrown` | 0.15.5 | MIT OR Apache-2.0 |  |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 |  |
| `hermit-abi` | 0.5.3 | MIT OR Apache-2.0 |  |
| `hybrid-array` | 0.4.15 | MIT OR Apache-2.0 |  |
| `image` | 0.25.10 | MIT OR Apache-2.0 | yes |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT |  |
| `inout` | 0.2.6 | MIT OR Apache-2.0 |  |
| `itertools` | 0.15.0 | MIT OR Apache-2.0 |  |
| `itoa` | 1.0.19 | MIT OR Apache-2.0 |  |
| `jni` | 0.22.4 | MIT OR Apache-2.0 |  |
| `jni-macros` | 0.22.4 | MIT OR Apache-2.0 |  |
| `jni-sys` | 0.3.1 | MIT OR Apache-2.0 |  |
| `jni-sys` | 0.4.1 | MIT OR Apache-2.0 |  |
| `jni-sys-macros` | 0.4.1 | MIT OR Apache-2.0 |  |
| `jobserver` | 0.1.35 | MIT OR Apache-2.0 |  |
| `js-sys` | 0.3.107 | MIT OR Apache-2.0 |  |
| `khronos_api` | 3.1.0 | Apache-2.0 |  |
| `kurbo` | 0.13.1 | Apache-2.0 OR MIT |  |
| `libc` | 0.2.191 | MIT OR Apache-2.0 |  |
| `libloading` | 0.8.9 | ISC |  |
| `libm` | 0.2.16 | MIT |  |
| `libredox` | 0.1.25 | MIT |  |
| `linebender_resource_handle` | 0.1.1 | Apache-2.0 OR MIT |  |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |  |
| `linux-raw-sys` | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |  |
| `litrs` | 1.0.1 | MIT OR Apache-2.0 |  |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 |  |
| `log` | 0.4.35 | MIT OR Apache-2.0 |  |
| `lopdf` | 0.47.0 | MIT | yes |
| `md-5` | 0.11.0 | MIT OR Apache-2.0 |  |
| `memchr` | 2.8.3 | Unlicense OR MIT |  |
| `memmap2` | 0.9.12 | MIT OR Apache-2.0 |  |
| `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 |  |
| `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 |  |
| `moxcms` | 0.8.1 | BSD-3-Clause OR Apache-2.0 |  |
| `multiversion_no_op` | 1.0.0 | Apache-2.0 OR MIT |  |
| `naga` | 30.0.2 | MIT OR Apache-2.0 |  |
| `naga-types` | 30.0.2 | MIT OR Apache-2.0 |  |
| `ndk` | 0.9.0 | MIT OR Apache-2.0 |  |
| `ndk-context` | 0.1.1 | MIT OR Apache-2.0 |  |
| `ndk-sys` | 0.6.0+11769913 | MIT OR Apache-2.0 |  |
| `nohash-hasher` | 0.2.0 | Apache-2.0 OR MIT |  |
| `nom` | 8.0.0 | MIT |  |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 |  |
| `num_enum` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |  |
| `num_enum_derive` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |  |
| `objc-sys` | 0.3.5 | MIT |  |
| `objc2` | 0.5.3 | MIT |  |
| `objc2` | 0.6.5 | MIT |  |
| `objc2-app-kit` | 0.2.2 | MIT |  |
| `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT |  |
| `objc2-cloud-kit` | 0.2.2 | MIT |  |
| `objc2-contacts` | 0.2.2 | MIT |  |
| `objc2-core-data` | 0.2.2 | MIT |  |
| `objc2-core-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT |  |
| `objc2-core-graphics` | 0.3.2 | Zlib OR Apache-2.0 OR MIT |  |
| `objc2-core-image` | 0.2.2 | MIT |  |
| `objc2-core-location` | 0.2.2 | MIT |  |
| `objc2-encode` | 4.1.0 | MIT |  |
| `objc2-foundation` | 0.2.2 | MIT |  |
| `objc2-foundation` | 0.3.2 | MIT |  |
| `objc2-io-surface` | 0.3.2 | Zlib OR Apache-2.0 OR MIT |  |
| `objc2-link-presentation` | 0.2.2 | MIT |  |
| `objc2-metal` | 0.2.2 | MIT |  |
| `objc2-quartz-core` | 0.2.2 | MIT |  |
| `objc2-symbols` | 0.2.2 | MIT |  |
| `objc2-ui-kit` | 0.2.2 | MIT |  |
| `objc2-ui-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT |  |
| `objc2-uniform-type-identifiers` | 0.2.2 | MIT |  |
| `objc2-uniform-type-identifiers` | 0.3.2 | Zlib OR Apache-2.0 OR MIT |  |
| `objc2-user-notifications` | 0.2.2 | MIT |  |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 |  |
| `orbclient` | 0.3.55 | MIT |  |
| `os_pipe` | 1.2.3 | MIT |  |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 |  |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 |  |
| `peniko` | 0.6.1 | Apache-2.0 OR MIT |  |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 |  |
| `petgraph` | 0.8.3 | MIT OR Apache-2.0 |  |
| `pin-project` | 1.1.13 | Apache-2.0 OR MIT |  |
| `pin-project-internal` | 1.1.13 | Apache-2.0 OR MIT |  |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT |  |
| `pkg-config` | 0.3.34 | MIT OR Apache-2.0 |  |
| `plain` | 0.2.3 | MIT/Apache-2.0 |  |
| `png` | 0.18.1 | MIT OR Apache-2.0 |  |
| `polling` | 3.11.0 | Apache-2.0 OR MIT |  |
| `pollster` | 1.0.1 | Apache-2.0/MIT |  |
| `polycool` | 0.4.0 | MIT OR Apache-2.0 |  |
| `portable-atomic` | 1.15.0 | Apache-2.0 OR MIT |  |
| `portable-atomic-util` | 0.2.8 | Apache-2.0 OR MIT |  |
| `proc-macro-crate` | 3.5.0 | MIT OR Apache-2.0 |  |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 |  |
| `profiling` | 1.0.18 | MIT OR Apache-2.0 |  |
| `pxfm` | 0.1.30 | BSD-3-Clause OR Apache-2.0 |  |
| `quick-error` | 2.0.1 | MIT/Apache-2.0 |  |
| `quick-xml` | 0.41.0 | MIT |  |
| `quote` | 1.0.47 | MIT OR Apache-2.0 |  |
| `r-efi` | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |  |
| `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |  |
| `rand` | 0.10.4 | MIT OR Apache-2.0 |  |
| `rand_core` | 0.10.1 | MIT OR Apache-2.0 |  |
| `rangemap` | 1.8.0 | MIT/Apache-2.0 |  |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib |  |
| `read-fonts` | 0.41.0 | MIT OR Apache-2.0 |  |
| `redox_syscall` | 0.4.1 | MIT |  |
| `redox_syscall` | 0.5.18 | MIT |  |
| `redox_syscall` | 0.9.5 | MIT |  |
| `renderdoc-sys` | 1.1.0 | MIT OR Apache-2.0 |  |
| `rfd` | 0.18.0 | MIT | yes |
| `rustc-hash` | 1.1.0 | Apache-2.0/MIT |  |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT |  |
| `rustc_version` | 0.4.1 | MIT OR Apache-2.0 |  |
| `rustix` | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |  |
| `rustix` | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |  |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 |  |
| `same-file` | 1.0.6 | Unlicense/MIT |  |
| `scoped-tls` | 1.0.1 | MIT/Apache-2.0 |  |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 |  |
| `self_cell` | 1.3.0 | Apache-2.0 OR GPL-2.0-only |  |
| `semver` | 1.0.28 | MIT OR Apache-2.0 |  |
| `send_wrapper` | 0.6.0 | MIT/Apache-2.0 |  |
| `serde` | 1.0.229 | MIT OR Apache-2.0 |  |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 |  |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 |  |
| `serde_spanned` | 1.1.3 | MIT OR Apache-2.0 |  |
| `sha2` | 0.11.1 | MIT OR Apache-2.0 |  |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 |  |
| `simd-adler32` | 0.3.10 | MIT |  |
| `simd_cesu8` | 1.2.0 | Apache-2.0 OR MIT |  |
| `simdutf8` | 0.1.5 | MIT OR Apache-2.0 |  |
| `skrifa` | 0.44.0 | MIT OR Apache-2.0 |  |
| `slab` | 0.4.12 | MIT |  |
| `slotmap` | 1.1.1 | Zlib |  |
| `smallvec` | 1.16.5 | MIT OR Apache-2.0 |  |
| `smithay-client-toolkit` | 0.19.2 | MIT |  |
| `smol_str` | 0.2.2 | MIT OR Apache-2.0 |  |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 |  |
| `stringprep` | 0.1.5 | MIT/Apache-2.0 |  |
| `syn` | 2.0.119 | MIT OR Apache-2.0 |  |
| `syn` | 3.0.10 | MIT OR Apache-2.0 |  |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | yes |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 |  |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 |  |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 |  |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 |  |
| `tiff` | 0.11.3 | MIT |  |
| `tinyvec` | 1.13.3 | Zlib OR Apache-2.0 OR MIT |  |
| `tokio` | 1.54.1 | MIT |  |
| `toml` | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 |  |
| `toml_datetime` | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 |  |
| `toml_edit` | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 |  |
| `toml_parser` | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 |  |
| `toml_writer` | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 |  |
| `tracing` | 0.1.44 | MIT |  |
| `tracing-core` | 0.1.36 | MIT |  |
| `tree_magic_mini` | 3.2.2 | MIT |  |
| `type-map` | 0.5.1 | MIT/Apache-2.0 |  |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 |  |
| `unicode-bidi` | 0.3.18 | MIT OR Apache-2.0 |  |
| `unicode-general-category` | 1.1.0 | Apache-2.0 |  |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |  |
| `unicode-normalization` | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |  |
| `unicode-properties` | 0.1.5 | (MIT OR Apache-2.0) AND Unicode-3.0 |  |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 |  |
| `unicode-width` | 0.2.3 | MIT OR Apache-2.0 |  |
| `uuid` | 1.29.2 | Apache-2.0 OR MIT |  |
| `vello_common` | 0.2.0 | Apache-2.0 OR MIT |  |
| `vello_cpu` | 0.2.0 | Apache-2.0 OR MIT |  |
| `version_check` | 0.9.5 | MIT/Apache-2.0 |  |
| `walkdir` | 2.5.0 | Unlicense/MIT |  |
| `wasip2` | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |  |
| `wasm-bindgen` | 0.2.130 | MIT OR Apache-2.0 |  |
| `wasm-bindgen-futures` | 0.4.80 | MIT OR Apache-2.0 |  |
| `wasm-bindgen-macro` | 0.2.130 | MIT OR Apache-2.0 |  |
| `wasm-bindgen-macro-support` | 0.2.130 | MIT OR Apache-2.0 |  |
| `wasm-bindgen-shared` | 0.2.130 | MIT OR Apache-2.0 |  |
| `wayland-backend` | 0.3.17 | MIT |  |
| `wayland-client` | 0.31.15 | MIT |  |
| `wayland-csd-frame` | 0.3.0 | MIT |  |
| `wayland-cursor` | 0.31.14 | MIT |  |
| `wayland-protocols` | 0.32.13 | MIT |  |
| `wayland-protocols-plasma` | 0.3.12 | MIT |  |
| `wayland-protocols-wlr` | 0.3.12 | MIT |  |
| `wayland-scanner` | 0.31.11 | MIT |  |
| `wayland-sys` | 0.31.11 | MIT |  |
| `web-sys` | 0.3.107 | MIT OR Apache-2.0 |  |
| `web-time` | 1.1.0 | MIT OR Apache-2.0 |  |
| `weezl` | 0.1.12 | MIT OR Apache-2.0 |  |
| `weezl` | 0.2.1 | MIT OR Apache-2.0 |  |
| `wgpu` | 30.0.2 | MIT OR Apache-2.0 |  |
| `wgpu-core` | 30.0.2 | MIT OR Apache-2.0 |  |
| `wgpu-core-deps-windows-linux-android` | 30.0.2 | MIT OR Apache-2.0 |  |
| `wgpu-hal` | 30.0.2 | MIT OR Apache-2.0 |  |
| `wgpu-naga-bridge` | 30.0.2 | MIT OR Apache-2.0 |  |
| `wgpu-types` | 30.0.2 | MIT OR Apache-2.0 |  |
| `winapi-util` | 0.1.11 | Unlicense OR MIT |  |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 |  |
| `windows-sys` | 0.52.0 | MIT OR Apache-2.0 |  |
| `windows-sys` | 0.59.0 | MIT OR Apache-2.0 |  |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 |  |
| `windows-targets` | 0.52.6 | MIT OR Apache-2.0 |  |
| `windows_aarch64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 |  |
| `windows_aarch64_msvc` | 0.52.6 | MIT OR Apache-2.0 |  |
| `windows_i686_gnu` | 0.52.6 | MIT OR Apache-2.0 |  |
| `windows_i686_gnullvm` | 0.52.6 | MIT OR Apache-2.0 |  |
| `windows_i686_msvc` | 0.52.6 | MIT OR Apache-2.0 |  |
| `windows_x86_64_gnu` | 0.52.6 | MIT OR Apache-2.0 |  |
| `windows_x86_64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 |  |
| `windows_x86_64_msvc` | 0.52.6 | MIT OR Apache-2.0 |  |
| `winit` | 0.30.13 | Apache-2.0 |  |
| `winnow` | 1.0.5 | MIT |  |
| `winresource` | 0.1.34 | MIT | yes |
| `wit-bindgen` | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |  |
| `wl-clipboard-rs` | 0.9.5 | MIT OR Apache-2.0 |  |
| `x11-dl` | 2.21.0 | MIT |  |
| `x11rb` | 0.13.2 | MIT OR Apache-2.0 |  |
| `x11rb-protocol` | 0.13.2 | MIT OR Apache-2.0 |  |
| `xcursor` | 0.3.11 | MIT |  |
| `xkbcommon-dl` | 0.4.2 | MIT |  |
| `xkeysym` | 0.2.1 | MIT OR Apache-2.0 OR Zlib |  |
| `xml-rs` | 0.8.29 | MIT |  |
| `zerocopy` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |  |
| `zerocopy-derive` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |  |
| `zlib-rs` | 0.6.8 | Zlib |  |
| `zune-core` | 0.5.3 | MIT OR Apache-2.0 OR Zlib |  |
| `zune-jpeg` | 0.5.15 | MIT OR Apache-2.0 OR Zlib |  |

## Embedded fonts and artwork

The GUI uses the default fonts shipped by `epaint_default_fonts` 0.37.0 (part of egui):

| Font | License | License text |
| --- | --- | --- |
| Ubuntu Light | Ubuntu Font License 1.0 | `LICENSES/Ubuntu-font-1.0.txt` |
| Noto Emoji | SIL Open Font License 1.1 | `LICENSES/OFL-1.1.txt` |
| Hack | MIT / Bitstream Vera | `LICENSES/Hack-font.txt` |
| emoji-icon-font | MIT | `LICENSES/emoji-icon-font-MIT.txt` |
| egui-icons | egui MIT / Apache-2.0 | `LICENSES/MIT.txt`, `LICENSES/Apache-2.0.txt` |

The FolioFold application icon (`assets/icon.svg`, `assets/icon.png`, `assets/icon.ico`) is original FolioFold artwork and is released with the rest of the project under the Unlicense. The PNG is embedded with `include_bytes!`; the Windows icon is embedded as a PE resource at build time. No external icon file is required at runtime.

## License choices and special notices

- `self_cell` is offered under `Apache-2.0 OR GPL-2.0-only`; FolioFold uses the **Apache-2.0** option.
- `r-efi` is offered under `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; FolioFold uses the **MIT/Apache-2.0** options.
- `encoding_rs` includes BSD-3-Clause (WHATWG) code; the BSD-3-Clause text is included in `LICENSES/BSD-3-Clause.txt`.
- `unicode-ident`, `unicode-properties`, and related data crates include Unicode-3.0 data; the Unicode license text is included in `LICENSES/Unicode-3.0.txt`.
- The Rust standard library is distributed under `MIT OR Apache-2.0`. The Windows build statically links the Microsoft C runtime under the Microsoft Visual Studio license terms; it does not require the VC++ Redistributable.
- `epaint_default_fonts` combines MIT/Apache-2.0 code with SIL OFL-1.1 and Ubuntu-font-1.0 font data. The font notices above apply.
- `winresource` is used only at Windows build time to embed the application icon and version metadata. It is not linked into the runtime application.

## Full license texts

The following texts are included in `LICENSES/`:

- `LICENSES/0BSD.txt`
- `LICENSES/Apache-2.0.txt`
- `LICENSES/BSD-2-Clause.txt`
- `LICENSES/BSD-3-Clause.txt`
- `LICENSES/Hack-font.txt`
- `LICENSES/ISC.txt`
- `LICENSES/MIT.txt`
- `LICENSES/MPL-2.0.txt`
- `LICENSES/OFL-1.1.txt`
- `LICENSES/Ubuntu-font-1.0.txt`
- `LICENSES/Unicode-3.0.txt`
- `LICENSES/Unlicense.txt`
- `LICENSES/Zlib.txt`
- `LICENSES/emoji-icon-font-MIT.txt`

For any license not reproduced here, the SPDX identifier above refers to the standard license text published by SPDX and the corresponding upstream package.
