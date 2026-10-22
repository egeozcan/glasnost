//! `glasnost-turn`: the network lab's STUN and TURN server and its NAT
//! probe (M9 N8c, `glasnost_netlab`). Test only; see
//! docs/development.md, "Network lab (N8c)".
//!
//! - `glasnost-turn serve --listen IP:PORT ... --relay-ip IP [options]`:
//!   serves until Ctrl-C or SIGTERM, with a counts line every few seconds.
//! - `glasnost-turn probe --server IP:PORT --server IP:PORT`: whether the
//!   NAT in front of this machine maps one socket to one outside address
//!   for both servers. Exit code 0 when every server answered.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    use std::process::ExitCode;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = || {
        eprintln!("usage: {}", glasnost_netlab::USAGE);
        ExitCode::from(2)
    };
    match args.first().map(String::as_str) {
        Some("serve") => match glasnost_netlab::parse(&args[1..]) {
            Ok(o) => match glasnost_netlab::serve(&o) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("glasnost-turn: {e}");
                    ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("glasnost-turn serve: {e}");
                usage()
            }
        },
        Some("probe") => match glasnost_netlab::parse_probe(&args[1..]) {
            Ok((bind, servers, timeout)) => match glasnost_netlab::probe(bind, &servers, timeout) {
                Ok(p) => {
                    println!("{}", p.line());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    println!("probe failed: {e}");
                    ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("glasnost-turn probe: {e}");
                usage()
            }
        },
        _ => usage(),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
