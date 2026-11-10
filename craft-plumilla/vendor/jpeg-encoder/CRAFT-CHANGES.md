# Local jpeg-encoder changes

Upstream: jpeg-encoder 0.6.1. Crates.io archive SHA-256:
`b454d911ac55068f53495488d8ccd0646eaa540c033a28ee15b07838afafb01f`.
Date: 2026-11-09.

The original MIT and Apache-2.0 notices and the IJG notice in `src/fdct.rs`
are retained. This software is based in part on the work of the Independent
JPEG Group. No DCT, quantization, sampling, Huffman or colour mathematics
are changed.

`ImageBuffer` has default fallible guards, preserving existing implementors.
Both pixel input paths propagate guard errors, as do DCT block rows, Huffman
frequency scans and output scans. This permits cancellation with ordinary
errors during progressive and optimized encoding, including its preprocessing
before entropy writes. Row and coefficient allocations use `try_reserve_exact`.

Craft regression tests live in `crates/io/src/raster_export.rs`. They compare
baseline/progressive and optimized encoding, actual decoded pixels, ICC,
sampling metadata and cancellation before commit. The upstream source and
notices are included intact except for `src/image_buffer.rs` and
`src/encoder.rs`; this note records the local delta.
