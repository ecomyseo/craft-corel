# Local zune-jpeg changes

Upstream: zune-jpeg 0.5.15, 2026-11-09.
All upstream MIT, Apache-2.0 and Zlib notices are retained.

A synthetic 32 by 24 RGB JPEG, baseline with three single-component scans,
4:2:0 sampling and optimized Huffman tables, is decoded correctly by Pillow
but the upstream decoder returns a green image (mean RGB error 88.5039).
The same pixels encoded with the upstream buffer adapter confirm this is
independent of Craft's encoder adapter.

In baseline single-component scans the number of data-unit rows depends on
the current component's vertical sampling factor, not on the frame MCU
height. The decoded row offset advances eight rows, without multiplying by
the component vertical factor a second time. The scan corrections in `src/mcu.rs` retain DCT, colour conversion and
interpolation mathematics. Additional small-dimension CMYK controls exposed
scratch allocation assuming component zero had the largest stride, and
component strides derived before frame sampling maxima were known when K
is the last component. Scratch uses the maximum stride in baseline and
progressive paths (`mcu.rs`, `mcu_prog.rs`); `misc.rs` first computes frame
sampling maxima before deriving any component stride. These changes affect
buffer geometry, without changing interpolation or colour mathematics.
A further constant-colour 17 by 9 CMYK baseline case showed scan-marker
prefetch reaching SOS/EOI while buffered bits still represented a later data
unit row. The baseline single-component path defers those prefetched markers
until its last row, while retaining restart-marker handling. An independent
Huffman boundary reader and Pillow decoding of the original encoded bytes
confirm the valid scan content; no ICC or encoded pixels were altered to
conceal the decoder defect.

Input dimensions, decoding budgets and format acceptance are unchanged.

Regression and independent Pillow results are recorded outside this vendor
folder in the Craft export evidence. This is a source patch, not a claim
that all JPEG corpuses or all Corel reference exports have been tested.
