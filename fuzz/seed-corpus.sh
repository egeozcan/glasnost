#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Seeds fuzz/corpus/<target>/ from the synthetic fixtures (our own bytes).
# Run from the repository root after `cargo run -p glasnost-tools --bin glasnost-fixtures`.
set -eu
src=tests/fixtures/synthetic
for t in mix shp tmp vxl hva pal pcx csf idx aud lzo lcw mappack; do
    mkdir -p "fuzz/corpus/$t"
    [ -d "$src/$t" ] && cp "$src/$t"/* "fuzz/corpus/$t/"
done
mkdir -p fuzz/corpus/sniff
cp "$src"/*/* fuzz/corpus/sniff/
