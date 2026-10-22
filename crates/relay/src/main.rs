//! `glasnost-relay`: runs the relay and lobby server.

use glasnost_relay::Limits;

const HELP: &str = "\
glasnost-relay: the Glasnost relay and lobby server (WebSocket)

Usage: glasnost-relay [--listen ADDR] [--max-rooms N] [--max-connections N]
                      [--grace SECONDS] [--stats SECONDS]

  --listen ADDR          the address to listen on (default 127.0.0.1:8787)
  --max-rooms N          the most rooms at once (default 256)
  --max-connections N    the most connections at once (default 1024)
  --grace SECONDS        how long a dropped player has to reconnect before
                         their house surrenders, in game time (default 60)
  --stats SECONDS        print one line of counts every SECONDS (default 60,
                         0 turns it off)
  --version              print the build and stop
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

/// What the command line asked for.
enum Command {
    Help,
    Version,
    Run(String, Limits, u64),
}

/// Seconds between statistics lines unless `--stats` says otherwise.
// nocite: our own netcode
const STATS_SECONDS: u64 = 60;

fn parse() -> Result<Command, String> {
    let mut listen = LISTEN.to_owned();
    let mut stats = STATS_SECONDS;
    let mut limits = Limits::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-V" => return Ok(Command::Version),
            "--stats" => {
                let v = args.next().ok_or("--stats needs a number of seconds")?;
                stats = v
                    .parse()
                    .map_err(|_| format!("--stats: not a number of seconds: {v}"))?;
            }
            "--listen" => listen = args.next().ok_or("--listen needs an address")?,
            "--max-rooms" => limits.max_rooms = number("--max-rooms", args.next())?,
            "--max-connections" => {
                limits.max_connections = number("--max-connections", args.next())?;
            }
            "--grace" => {
                let s = number("--grace", args.next())? as u64;
                limits.grace_ms = s.saturating_mul(1000); // nocite: milliseconds in a second
            }
            other => return Err(format!("unknown argument: {other} (see --help)")),
        }
    }
    Ok(Command::Run(listen, limits, stats))
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let (listen, limits, stats) = match parse() {
        Ok(Command::Run(listen, limits, stats)) => (listen, limits, stats),
        Ok(Command::Help) => {
            println!("{HELP}");
            return std::process::ExitCode::SUCCESS;
        }
        Ok(Command::Version) => {
            println!("glasnost-relay {}", glasnost_net::BUILD);
            return std::process::ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("glasnost-relay: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    let report = (stats > 0).then(|| glasnost_relay::Report {
        every: std::time::Duration::from_secs(stats),
        sink: Box::new(|line| println!("{line}")),
    });
    let entropy = Box::new(glasnost_relay::OsEntropy);
    match glasnost_relay::bind_reporting(&listen, limits, entropy, report).await {
        Ok(relay) => {
            println!("glasnost-relay listening on ws://{}/", relay.addr);
            println!("build {}", glasnost_net::BUILD);
            relay.wait().await;
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("glasnost-relay: cannot listen on {listen}: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
