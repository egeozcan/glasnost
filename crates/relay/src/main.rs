//! `glasnost-relay`: runs the relay and lobby server.

use glasnost_relay::Limits;

const HELP: &str = "\
glasnost-relay: the Glasnost relay and lobby server (WebSocket)

Usage: glasnost-relay [--listen ADDR] [--max-rooms N] [--max-connections N]

  --listen ADDR          the address to listen on (default 127.0.0.1:8787)
  --max-rooms N          the most rooms at once (default 256)
  --max-connections N    the most connections at once (default 1024)
  --help                 this text

Clients connect to ws://ADDR/. The relay speaks no TLS: put it behind a
proxy that does (wss://) when it faces the internet.";

/// The default address.
// nocite: our own netcode
const LISTEN: &str = "127.0.0.1:8787";

fn number(flag: &str, value: Option<String>) -> Result<usize, String> {
    let value = value.ok_or_else(|| format!("{flag} needs a number"))?;
    value
        .parse()
        .ok()
        .filter(|&n| n > 0)
        .ok_or_else(|| format!("{flag}: not a positive number: {value}"))
}

fn parse() -> Result<Option<(String, Limits)>, String> {
    let mut listen = LISTEN.to_owned();
    let mut limits = Limits::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--listen" => listen = args.next().ok_or("--listen needs an address")?,
            "--max-rooms" => limits.max_rooms = number("--max-rooms", args.next())?,
            "--max-connections" => {
                limits.max_connections = number("--max-connections", args.next())?;
            }
            other => return Err(format!("unknown argument: {other} (see --help)")),
        }
    }
    Ok(Some((listen, limits)))
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let (listen, limits) = match parse() {
        Ok(Some(v)) => v,
        Ok(None) => {
            println!("{HELP}");
            return std::process::ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("glasnost-relay: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    match glasnost_relay::bind(&listen, limits).await {
        Ok(relay) => {
            println!("glasnost-relay listening on ws://{}/", relay.addr);
            relay.wait().await;
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("glasnost-relay: cannot listen on {listen}: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
