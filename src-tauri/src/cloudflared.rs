#![allow(non_snake_case)]

use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
#[cfg(unix)]
#[allow(unused_imports)]
use std::os::unix::process::CommandExt;
use crate::log::{Subsystem::Cloudflared, Level::*};

static _DAEMON_PID: AtomicI32 = AtomicI32::new(0);

pub fn _StopDaemon() {
    let Pid = _DAEMON_PID.swap(0, Ordering::Relaxed);
    if Pid <= 0 {
        return;
    }
    #[cfg(unix)]
    unsafe { libc::kill(-Pid, libc::SIGKILL); }
}

async fn _KillChild(Child: &mut tokio::process::Child) {
    if let Some(Pid) = Child.id() {
        #[cfg(unix)]
        unsafe { libc::kill(-(Pid as i32), libc::SIGKILL); }
    }
    let _ = Child.kill().await;
    let _ = Child.wait().await;
    log!(Cloudflared, Debug, "process killed");
}

#[derive(serde::Serialize, Clone)]
pub struct HostnameVerifyResult {
    #[serde(rename = "Management")]
    pub management: bool,
    #[serde(rename = "Voting")]
    pub voting: bool,
    #[serde(rename = "Error", skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

async fn _WaitForTunnel(Child: &mut tokio::process::Child, WaitForConfig: bool) -> Result<(), String> {
    let Stderr = Child.stderr.take().ok_or("No stderr")?;
    let mut Lines = BufReader::new(Stderr).lines();
    let mut LastLine: String = String::new();
    let mut Registered = false;

    log!(Cloudflared, Debug, "waiting for tunnel registration (config={})", WaitForConfig);

    let Result = tokio::time::timeout(std::time::Duration::from_secs(30), async {
        while let Ok(Some(Line)) = Lines.next_line().await {
            log!(Cloudflared, Debug, "stderr: {}", Line);
            let Lower = Line.to_lowercase();

            if Lower.contains("unauthorized") || Lower.contains("tunnel not found") || Lower.contains("invalid token") {
                return Err(Line);
            }

            if Lower.contains("registered tunnel connection") || Lower.contains("connection registered") {
                Registered = true;
                if !WaitForConfig { return Ok(()); }
            }

            if WaitForConfig && Registered && Lower.contains("updated to new configuration") {
                return Ok(());
            }

            LastLine = Line;
        }
        Err(if LastLine.is_empty() { "cloudflared exited".to_string() } else { LastLine })
    })
    .await;

    match Result {
        Ok(Ok(())) => { log!(Cloudflared, Info, "tunnel ready (registered={} config={})", Registered, WaitForConfig); Ok(()) },
        Ok(Err(Msg)) => { log!(Cloudflared, Error, "tunnel failed: {}", Msg); Err(Msg) },
        Err(_) => { log!(Cloudflared, Error, "tunnel timed out (30s)"); Err("Tunnel connection timed out".to_string()) },
    }
}

fn _Nonce() -> String {
    use rand::RngExt;
    let Bytes: [u8; 16] = rand::rng().random();
    hex::encode(Bytes)
}

pub fn NormalizeHostname(Raw: &str) -> String {
    let Trimmed = Raw.trim();
    let Bare = Trimmed.strip_prefix("https://").or_else(|| Trimmed.strip_prefix("http://")).unwrap_or(Trimmed);
    Bare.split('/').next().unwrap_or("").to_lowercase()
}

async fn _Probe(Port: u16, Nonce: String) -> Result<tokio::task::JoinHandle<()>, std::io::Error> {
    let Listener = tokio::net::TcpListener::bind(("127.0.0.1", Port)).await?;
    let App = axum::Router::new().route("/_health", axum::routing::get(move || async move { Nonce }));
    Ok(tokio::spawn(async move { let _ = axum::serve(Listener, App).await; }))
}

async fn _CheckHostname(Client: &reqwest::Client, Hostname: &str, Expected: &str) -> bool {
    if Hostname.is_empty() {
        log!(Cloudflared, Warn, "empty hostname");
        return false;
    }

    let Url = format!("https://{}/_health", Hostname);
    let mut Reason = String::new();

    for Attempt in 1..=10 {
        log!(Cloudflared, Debug, "probe {} attempt {}", Url, Attempt);
        match Client.get(&Url).send().await {
            Ok(Res) => {
                let Status = Res.status();
                match Res.text().await {
                    Ok(Body) if Body.trim() == Expected => {
                        log!(Cloudflared, Info, "{} reached through tunnel", Hostname);
                        return true;
                    }
                    Ok(Body) => Reason = format!("status {} body {:?}", Status, Body.chars().take(80).collect::<String>()),
                    Err(E) => Reason = format!("status {} read error: {}", Status, E),
                }
            }
            Err(E) => Reason = E.to_string(),
        }
        if Attempt < 10 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    log!(Cloudflared, Warn, "{} not routed to this machine: {}", Hostname, Reason);
    false
}

pub async fn _VerifyHostnames(
    BinPath: &Path,
    TunnelToken: &str,
    ManagementHostname: &str,
    VotingHostname: &str,
) -> HostnameVerifyResult {
    let ManagementHostname = NormalizeHostname(ManagementHostname);
    let VotingHostname = NormalizeHostname(VotingHostname);
    log!(Cloudflared, Info, "verify hostnames: mgmt={} vote={}", ManagementHostname, VotingHostname);

    let MgmtNonce = _Nonce();
    let VoteNonce = _Nonce();

    let MgmtProbe = match _Probe(9999, MgmtNonce.clone()).await {
        Ok(H) => H,
        Err(E) => {
            fail!(Cloudflared, E, "could not bind management probe on 127.0.0.1:9999");
            return HostnameVerifyResult { management: false, voting: false, error: Some(format!("Port 9999 unavailable: {}", E)) };
        }
    };
    let VoteProbe = match _Probe(10000, VoteNonce.clone()).await {
        Ok(H) => H,
        Err(E) => {
            MgmtProbe.abort();
            fail!(Cloudflared, E, "could not bind voting probe on 127.0.0.1:10000");
            return HostnameVerifyResult { management: false, voting: false, error: Some(format!("Port 10000 unavailable: {}", E)) };
        }
    };

    log!(Cloudflared, Debug, "spawning {:?}", BinPath);

    let mut Cmd = Command::new(BinPath);
    Cmd.args(["tunnel", "run", "--token", TunnelToken])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    #[cfg(unix)]
    unsafe { Cmd.pre_exec(|| { libc::setpgid(0, 0); Ok(()) }); }

    let mut Child = match Cmd.spawn() {
        Ok(C) => C,
        Err(E) => {
            MgmtProbe.abort();
            VoteProbe.abort();
            fail!(Cloudflared, E, "could not spawn cloudflared");
            return HostnameVerifyResult { management: false, voting: false, error: Some(format!("Failed to start cloudflared: {}", E)) };
        }
    };

    if let Err(E) = _WaitForTunnel(&mut Child, true).await {
        MgmtProbe.abort();
        VoteProbe.abort();
        _KillChild(&mut Child).await;
        return HostnameVerifyResult { management: false, voting: false, error: Some(E) };
    }

    let Client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let (MgmtOk, VoteOk) = tokio::join!(
        _CheckHostname(&Client, &ManagementHostname, &MgmtNonce),
        _CheckHostname(&Client, &VotingHostname, &VoteNonce),
    );

    MgmtProbe.abort();
    VoteProbe.abort();

    log!(Cloudflared, Info, "verify results: mgmt={} vote={}", MgmtOk, VoteOk);
    _KillChild(&mut Child).await;

    HostnameVerifyResult { management: MgmtOk, voting: VoteOk, error: None }
}

pub async fn _RunDaemon(BinPath: &Path, Token: &str) {
    log!(Cloudflared, Info, "starting tunnel daemon");

    let mut Cmd = Command::new(BinPath);
    Cmd.args(["tunnel", "run", "--token", Token])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    #[cfg(unix)]
    unsafe { Cmd.pre_exec(|| { libc::setpgid(0, 0); Ok(()) }); }

    let mut Child = match Cmd.spawn() {
        Ok(C) => C,
        Err(E) => { log!(Cloudflared, Error, "daemon spawn failed: {}", E); return; }
    };

    if let Some(Pid) = Child.id() {
        _DAEMON_PID.store(Pid as i32, Ordering::Relaxed);
    }

    let Stderr = match Child.stderr.take() {
        Some(S) => S,
        None => { log!(Cloudflared, Error, "no stderr"); return; }
    };

    let mut Lines = BufReader::new(Stderr).lines();
    let mut Ready = false;

    while let Ok(Some(Line)) = Lines.next_line().await {
        let Lower = Line.to_lowercase();

        if !Ready {
            if Lower.contains("unauthorized") || Lower.contains("tunnel not found") || Lower.contains("invalid token") {
                log!(Cloudflared, Error, "tunnel failed: {}", Line);
                _KillChild(&mut Child).await;
                return;
            }
            if Lower.contains("updated to new configuration") {
                Ready = true;
                log!(Cloudflared, Info, "tunnel daemon running");
            }
        }

        log!(Cloudflared, Debug, "stderr: {}", Line);
    }

    let _ = Child.wait().await;
    _DAEMON_PID.store(0, Ordering::Relaxed);
    log!(Cloudflared, Warn, "tunnel daemon exited");
}

pub async fn _VerifyToken(BinPath: &Path, Token: &str) -> Result<(), String> {
    log!(Cloudflared, Info, "verifying token");

    let mut Cmd = Command::new(BinPath);
    Cmd.args(["tunnel", "run", "--token", Token])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    #[cfg(unix)]
    unsafe { Cmd.pre_exec(|| { libc::setpgid(0, 0); Ok(()) }); }

    let mut Child = Cmd.spawn()
        .map_err(|E| format!("Failed to start cloudflared: {}", E))?;

    let Result = _WaitForTunnel(&mut Child, false).await;
    _KillChild(&mut Child).await;
    Result
}
