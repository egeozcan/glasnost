//! Hostile-input smoke test (stable, runs in CI): the reader and every
//! value form get synthetic documents, their prefixes and mutated copies.
//! Deeper fuzzing runs with cargo-fuzz (`fuzz/`, target `ini`).

use glasnost_fmt::synth::SynthRng;
use glasnost_ini::hostile::run;

/// Invented documents that touch every lexical rule and value form.
const SEEDS: &[&str] = &[
    "[A]\nk=1\nk=2\nFiller0=1\n[A]\nx=$1F\n",
    "\u{feff}[B]\r\n  key = 12.5%,0.125,abc ; c\r\n=x\n[x=5\n",
    "[C]\nv=1e39\nw=-0x5h\nt=1, 2 ,3\nr=0,0,30\nc=300,-1,a\nl=,,a, b,\n",
    "          k=   v    ;\n[]\nZdxeLZgd=B\nCost=A\n[D]\nq=99999999999\n",
    "[E]\nd=0.000000000000000000000000000000000000000001\ne=1e-99999999999\nf=7e999\n",
];

const MUTATIONS: usize = 400;

/// Bytes that matter to the reader, inserted by the mutator.
const INSERTS: &[u8] = b"=,;[]%$h.e-\r\n\0 ";

#[test]
fn reader_survives_mutated_documents() {
    let mut rng = SynthRng::new(0x1A1);
    for seed in SEEDS {
        let bytes = seed.as_bytes();
        run(bytes);
        for cut in 0..bytes.len() {
            run(&bytes[..cut]);
        }
        for _ in 0..MUTATIONS {
            let mut b = bytes.to_vec();
            let n = 1 + ((rng.next_u64() as u32) % 4) as usize;
            for _ in 0..n {
                let i = (rng.next_u64() as u32) as usize % b.len().max(1);
                match (rng.next_u64() as u32) % 3 {
                    0 if !b.is_empty() => b[i] = (rng.next_u64() as u32) as u8,
                    1 => {
                        let pick = INSERTS[(rng.next_u64() as usize) % INSERTS.len()];
                        b.insert(i.min(b.len()), pick);
                    }
                    _ if !b.is_empty() => {
                        b.remove(i);
                    }
                    _ => {}
                }
            }
            run(&b);
        }
    }
}

#[test]
fn reader_survives_long_and_repetitive_input() {
    let mut many = String::from("[S]\n");
    for i in 0..3000 {
        many.push_str(&format!("K{}=v{i}\n", i % 7));
    }
    run(many.as_bytes());
    let long = format!("[L]\nk={}\n", "9".repeat(5000));
    run(long.as_bytes());
    run(&[0xFF; 2048]);
}
