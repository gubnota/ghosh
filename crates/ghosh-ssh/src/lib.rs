//! Native SSH transport. Workspace policy and credentials remain in the caller.
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD},
};
use ghosh_core::{contracts::*, ssh::*};
use sha2::{Digest, Sha256};
use ssh2::{CheckResult, HostKeyType, KnownHostFileKind, KnownHosts, MethodType, Session};
use std::{
    net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(5);
static RESOLVERS: AtomicUsize = AtomicUsize::new(0);
fn failed() -> AppError {
    AppError::new(ErrorCode::ConnectionFailed)
}

pub struct Endpoint {
    pub host: String,
    pub port: u16,
    pub user: String,
}
/// Public server key bytes, retained natively between review and authentication.
#[derive(Clone)]
pub struct HostRecord {
    key: Vec<u8>,
    pub preview: HostKeyPreview,
}
pub struct Connection {
    session: Session,
    pub host: HostRecord,
    reviewed: bool,
}

fn addresses(endpoint: &Endpoint, cancel: &Cancellation) -> Result<Vec<SocketAddr>> {
    cancel.check()?;
    if let Ok(ip) = endpoint.host.parse::<IpAddr>() {
        return Ok(vec![SocketAddr::new(ip, endpoint.port)]);
    }
    // libc DNS is not cancellable. Bound outstanding resolver threads, and
    // stop waiting at the deadline without allowing an unbounded thread leak.
    RESOLVERS
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
            (n < 2).then_some(n + 1)
        })
        .map_err(|_| AppError::new(ErrorCode::Capacity))?;
    let (sender, receiver) = mpsc::sync_channel(1);
    let host = endpoint.host.clone();
    let port = endpoint.port;
    if std::thread::Builder::new()
        .name("ssh-dns".into())
        .spawn(move || {
            let result = (host.as_str(), port)
                .to_socket_addrs()
                .map(|items| items.take(16).collect());
            let _ = sender.send(result);
            RESOLVERS.fetch_sub(1, Ordering::SeqCst);
        })
        .is_err()
    {
        RESOLVERS.fetch_sub(1, Ordering::SeqCst);
        return Err(failed());
    }
    let deadline = Instant::now() + TIMEOUT;
    loop {
        cancel.check()?;
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(result) => return result.map_err(|_| failed()),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(failed()),
            Err(_) if Instant::now() >= deadline => return Err(failed()),
            Err(_) => (),
        }
    }
}

fn load_hosts(session: &Session, known_hosts: &[String]) -> Result<KnownHosts> {
    let mut hosts = session.known_hosts().map_err(|_| failed())?;
    for text in known_hosts {
        for line in text
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        {
            // Unsupported directives and damaged trust stores fail closed.
            hosts
                .read_str(line, KnownHostFileKind::OpenSSH)
                .map_err(|_| failed())?;
        }
    }
    Ok(hosts)
}

fn host_preferences(
    session: &Session,
    endpoint: &Endpoint,
    known_hosts: &[String],
) -> Result<String> {
    let hosts = load_hosts(session, known_hosts)?;
    let mut known = Vec::new();
    for host in hosts.hosts().map_err(|_| failed())? {
        let key = STANDARD.decode(host.key()).map_err(|_| failed())?;
        if matches!(
            hosts.check_port(&endpoint.host, endpoint.port, &key),
            CheckResult::Match
        ) {
            let length = key.get(..4).ok_or_else(failed)?;
            let length = u32::from_be_bytes(length.try_into().map_err(|_| failed())?) as usize;
            let algorithm = key.get(4..4 + length).ok_or_else(failed)?;
            known.push(
                std::str::from_utf8(algorithm)
                    .map_err(|_| failed())?
                    .to_string(),
            );
        }
    }
    let preferred = [
        "ssh-ed25519",
        "ecdsa-sha2-nistp256",
        "ecdsa-sha2-nistp384",
        "ecdsa-sha2-nistp521",
        "rsa-sha2-512",
        "rsa-sha2-256",
    ];
    let choices: Vec<_> = preferred
        .into_iter()
        .filter(|algorithm| {
            known.is_empty()
                || known.iter().any(|key| {
                    key == algorithm || (key == "ssh-rsa" && algorithm.starts_with("rsa-sha2-"))
                })
        })
        .collect();
    if choices.is_empty() {
        return Err(failed());
    }
    Ok(choices.join(","))
}
fn trust(
    session: &Session,
    endpoint: &Endpoint,
    key: &[u8],
    known_hosts: &[String],
) -> Result<HostTrust> {
    let hosts = load_hosts(session, known_hosts)?;
    match hosts.check_port(&endpoint.host, endpoint.port, key) {
        CheckResult::Match => Ok(HostTrust::Known),
        CheckResult::NotFound => Ok(HostTrust::Unknown),
        CheckResult::Mismatch => Err(AppError::new(ErrorCode::HostKeyChanged)),
        CheckResult::Failure => Err(failed()),
    }
}

fn match_review(review: Option<&HostRecord>, key: &[u8], allow_unknown: bool) -> Result<()> {
    if let Some(review) = review {
        if review.key != key {
            return Err(AppError::new(ErrorCode::HostKeyChanged));
        }
        if review.preview.trust == HostTrust::Unknown && !allow_unknown {
            return Err(AppError::new(ErrorCode::Denied));
        }
    }
    Ok(())
}

impl Connection {
    /// Handshake and host verification only; no credential is sent here.
    pub fn open(
        endpoint: &Endpoint,
        known_hosts: &[String],
        review: Option<&HostRecord>,
        allow_unknown: bool,
        cancel: &Cancellation,
    ) -> Result<Self> {
        let candidates = addresses(endpoint, cancel)?;
        let deadline = Instant::now() + TIMEOUT;
        let mut socket = None;
        for address in candidates {
            cancel.check()?;
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                break;
            };
            if let Ok(stream) = TcpStream::connect_timeout(&address, remaining) {
                socket = Some(stream);
                break;
            }
        }
        let socket = socket.ok_or_else(failed)?;
        socket
            .set_read_timeout(Some(TIMEOUT))
            .map_err(|_| failed())?;
        socket
            .set_write_timeout(Some(TIMEOUT))
            .map_err(|_| failed())?;
        let mut session = Session::new().map_err(|_| failed())?;
        session.set_timeout(TIMEOUT.as_millis() as u32);
        session
            .method_pref(
                MethodType::HostKey,
                &host_preferences(&session, endpoint, known_hosts)?,
            )
            .map_err(|_| failed())?;
        session.set_tcp_stream(socket);
        cancel.check()?;
        session.handshake().map_err(|_| failed())?;
        cancel.check()?;
        let (key, kind) = session.host_key().ok_or_else(failed)?;
        let trust = trust(&session, endpoint, key, known_hosts)?;
        match_review(review, key, allow_unknown)?;
        let algorithm = match kind {
            HostKeyType::Ed25519 => "ssh-ed25519",
            HostKeyType::Rsa => "ssh-rsa",
            HostKeyType::Ecdsa256 => "ecdsa-sha2-nistp256",
            HostKeyType::Ecdsa384 => "ecdsa-sha2-nistp384",
            HostKeyType::Ecdsa521 => "ecdsa-sha2-nistp521",
            _ => return Err(failed()),
        };
        let host = HostRecord {
            key: key.to_vec(),
            preview: HostKeyPreview {
                algorithm: algorithm.into(),
                fingerprint: format!("SHA256:{}", STANDARD_NO_PAD.encode(Sha256::digest(key))),
                trust,
            },
        };
        Ok(Self {
            session,
            host,
            reviewed: review.is_some(),
        })
    }

    /// Only a previously reviewed connection can be authenticated by the caller.
    /// No agent, ssh_config, shell command, password or identity fallback occurs.
    pub fn authenticate_file(
        &self,
        user: &str,
        private: &[u8],
        cancel: &Cancellation,
    ) -> Result<()> {
        if !self.reviewed {
            return Err(AppError::new(ErrorCode::Denied));
        }
        cancel.check()?;
        let text =
            std::str::from_utf8(private).map_err(|_| AppError::new(ErrorCode::KeyUnavailable))?;
        self.session
            .userauth_pubkey_memory(user, None, text, None)
            .map_err(|_| AppError::new(ErrorCode::AuthenticationFailed))?;
        cancel.check()?;
        if !self.session.authenticated() {
            return Err(AppError::new(ErrorCode::AuthenticationFailed));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_key_and_unapproved_unknown_are_rejected() {
        let key = vec![1, 2, 3];
        let review = HostRecord {
            key: key.clone(),
            preview: HostKeyPreview {
                algorithm: "ssh-ed25519".into(),
                fingerprint: "public".into(),
                trust: HostTrust::Unknown,
            },
        };
        assert_eq!(
            match_review(Some(&review), &key, false).unwrap_err().code,
            ErrorCode::Denied
        );
        assert!(match_review(Some(&review), &key, true).is_ok());
        assert_eq!(
            match_review(Some(&review), &[4], true).unwrap_err().code,
            ErrorCode::HostKeyChanged
        );
    }
    #[test]
    fn known_hosts_checks_nonstandard_ports_and_refuses_mismatch() {
        let session = Session::new().unwrap();
        let endpoint = Endpoint {
            host: "example.test".into(),
            port: 2222,
            user: "vm".into(),
        };
        let mut key = vec![0, 0, 0, 11];
        key.extend_from_slice(b"ssh-ed25519");
        key.extend_from_slice(&[0, 0, 0, 32]);
        key.extend_from_slice(&[1; 32]);
        let line = format!(
            "[example.test]:2222 ssh-ed25519 {}",
            base64::engine::general_purpose::STANDARD.encode(&key)
        );
        assert_eq!(
            trust(&session, &endpoint, &key, std::slice::from_ref(&line)).unwrap(),
            HostTrust::Known
        );
        assert_eq!(
            trust(&session, &endpoint, &key, &[]).unwrap(),
            HostTrust::Unknown
        );
        assert_eq!(
            host_preferences(&session, &endpoint, std::slice::from_ref(&line)).unwrap(),
            "ssh-ed25519"
        );
        let other = Endpoint {
            host: "other.test".into(),
            port: 22,
            user: "vm".into(),
        };
        assert!(
            host_preferences(&session, &other, std::slice::from_ref(&line))
                .unwrap()
                .starts_with("ssh-ed25519,")
        );
        let mut changed = key.clone();
        *changed.last_mut().unwrap() = 2;
        assert_eq!(
            trust(&session, &endpoint, &changed, &[line])
                .unwrap_err()
                .code,
            ErrorCode::HostKeyChanged
        );
    }
}
