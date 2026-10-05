#![allow(non_snake_case)]

use std::fmt;
use zstio_tv_lib::{fail, log::{self, Subsystem}};

const CASES: [&str; 5] = ["port", "spawn", "hostname", "chain", "panic"];

#[derive(Debug)]
struct ConfigError { Key: &'static str, Inner: serde_json::Error }

impl fmt::Display for ConfigError {
    fn fmt(&self, F: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(F, "invalid value for {}", self.Key)
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { Some(&self.Inner) }
}

async fn Port() {
    let Held = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let Addr = Held.local_addr().unwrap();
    if let Err(E) = tokio::net::TcpListener::bind(Addr).await {
        fail!(Subsystem::Server, E, "vote server could not bind {}", Addr);
    }
}

async fn Spawn() {
    if let Err(E) = tokio::process::Command::new("/nonexistent/cloudflared").spawn() {
        fail!(Subsystem::Cloudflared, E, "could not spawn cloudflared");
    }
}

async fn Hostname() {
    let Client = reqwest::Client::new();
    if let Err(E) = Client.get("https://vote.zstio-tv.invalid/_health").send().await {
        fail!(Subsystem::Cloudflared, E, "vote.zstio-tv.invalid not routed to this machine");
    }
}

async fn Chain() {
    let Inner = serde_json::from_str::<u16>("\"ten thousand\"").unwrap_err();
    let E = ConfigError { Key: "voting_port", Inner };
    fail!(Subsystem::Core, E, "could not load configuration");
}

fn Panic() {
    let Ports: Vec<u16> = Vec::new();
    let _ = Ports[0];
}

async fn Run(Case: &str) {
    match Case {
        "port" => Port().await,
        "spawn" => Spawn().await,
        "hostname" => Hostname().await,
        "chain" => Chain().await,
        "panic" => Panic(),
        Other => {
            eprintln!("unknown case {:?}, pick one of: {}", Other, CASES.join(", "));
            std::process::exit(2);
        }
    }
}

fn main() {
    log::_InstallPanicHook();
    let Case = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());
    let Rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    Rt.block_on(async {
        if Case == "all" {
            for C in CASES {
                Run(C).await;
            }
        } else {
            Run(&Case).await;
        }
    });
}
