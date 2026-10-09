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
# The INI reader gets a few invented documents of our own.
mkdir -p fuzz/corpus/ini
printf '[A]\nk=1\nk=2\nFiller0=1\n[A]\nx=$1F\n' > fuzz/corpus/ini/dups
printf '[B]\r\n  key = 12.5%%,0.125,abc ; c\r\n=x\n[x=5\n' > fuzz/corpus/ini/forms
printf '[C]\nt=1, 2 ,3\nr=0,0,30\nc=300,-1,a\nl=,,a, b,\nd=0.1\n' > fuzz/corpus/ini/tuples
# The JSONC reader gets our synthetic ruleset.
mkdir -p fuzz/corpus/jsonc
cp data/synthetic/ruleset/*.jsonc fuzz/corpus/jsonc/
