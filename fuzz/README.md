# Fuzzing

cargo-fuzz targets for every parser in `glasnost-fmt`. Each target calls
`glasnost_fmt::hostile::run`, which drives the parser and every decoder over
the input and ignores errors: a panic, overflow trap, hang or runaway
allocation is a finding.

```sh
rustup toolchain install nightly
cargo +nightly install cargo-fuzz
./fuzz/seed-corpus.sh                       # seeds corpora from our synthetic fixtures
cargo +nightly fuzz run shp -- -max_total_time=60
cargo +nightly fuzz list                    # all targets
```

Corpora and crash artifacts are git-ignored. They hold only bytes derived
from our synthetic fixtures; never seed them from a real install. A stable
smoke test with the same entry points runs in CI:
`cargo test -p glasnost-fmt --test hostile`.
