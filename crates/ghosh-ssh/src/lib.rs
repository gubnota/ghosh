//! Native SSH transport. Workspace policy and credentials remain in the caller.
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD},
};
use ghosh_core::ssh_keys::PublicKeyMetadata;
use ghosh_core::terminal::{Pty, PtyRead, TerminalSize};
use ghosh_core::{contracts::*, ssh::*};
use sha2::{Digest, Sha256};
use ssh2::{CheckResult, HostKeyType, KnownHostFileKind, KnownHosts, MethodType, Session};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(5);
static RESOLVERS: AtomicUsize = AtomicUsize::new(0);
static AGENT_WORKERS: AtomicUsize = AtomicUsize::new(0);
fn failed() -> AppError {
    AppError::new(ErrorCode::ConnectionFailed)
}

fn agent_work<T: Send + 'static>(
    cancel: &Cancellation,
    work: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    cancel.check()?;
    AGENT_WORKERS
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
            (n < 2).then_some(n + 1)
        })
        .map_err(|_| AppError::new(ErrorCode::Capacity))?;
    let (send, receive) = mpsc::sync_channel(1);
    if std::thread::Builder::new()
        .name("ssh-agent".into())
        .spawn(move || {
            let result = work();
            let _ = send.send(result);
            AGENT_WORKERS.fetch_sub(1, Ordering::SeqCst);
        })
        .is_err()
    {
        AGENT_WORKERS.fetch_sub(1, Ordering::SeqCst);
        return Err(failed());
    }
    let deadline = Instant::now() + TIMEOUT;
    loop {
        cancel.check()?;
        match receive.recv_timeout(Duration::from_millis(50)) {
            Ok(result) => return result,
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(failed()),
            Err(_) if Instant::now() >= deadline => return Err(failed()),
            Err(_) => (),
        }
    }
}
fn agent(session: &Session, socket: &Path) -> Result<ssh2::Agent> {
    let mut agent = session.agent().map_err(|_| failed())?;
    agent.set_identity_path(socket).map_err(|_| failed())?;
    agent
        .connect()
        .map_err(|_| AppError::new(ErrorCode::KeyUnavailable))?;
    agent
        .list_identities()
        .map_err(|_| AppError::new(ErrorCode::KeyUnavailable))?;
    Ok(agent)
}
fn public_key(blob: &[u8]) -> Result<PublicKeyMetadata> {
    let length = blob.get(..4).ok_or_else(failed)?;
    let length = u32::from_be_bytes(length.try_into().map_err(|_| failed())?) as usize;
    let algorithm =
        std::str::from_utf8(blob.get(4..4 + length).ok_or_else(failed)?).map_err(|_| failed())?;
    if !matches!(
        algorithm,
        "ssh-ed25519"
            | "ssh-rsa"
            | "ecdsa-sha2-nistp256"
            | "ecdsa-sha2-nistp384"
            | "ecdsa-sha2-nistp521"
    ) {
        return Err(AppError::new(ErrorCode::KeyUnavailable));
    }
    Ok(PublicKeyMetadata {
        algorithm: algorithm.into(),
        fingerprint: format!("SHA256:{}", STANDARD_NO_PAD.encode(Sha256::digest(blob))),
    })
}
/// List only public metadata. Authentication later pins this exact identity;
/// no automatic retries with other agent identities are allowed.
pub fn agent_identity(socket: PathBuf, cancel: &Cancellation) -> Result<PublicKeyMetadata> {
    agent_work(cancel, move || {
        let session = Session::new().map_err(|_| failed())?;
        let agent = agent(&session, &socket)?;
        let keys = agent.identities().map_err(|_| failed())?;
        keys.iter()
            .find_map(|key| public_key(key.blob()).ok())
            .ok_or_else(|| AppError::new(ErrorCode::KeyUnavailable))
    })
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
        passphrase: Option<&str>,
        cancel: &Cancellation,
    ) -> Result<()> {
        if !self.reviewed {
            return Err(AppError::new(ErrorCode::Denied));
        }
        cancel.check()?;
        let text =
            std::str::from_utf8(private).map_err(|_| AppError::new(ErrorCode::KeyUnavailable))?;
        self.session
            .userauth_pubkey_memory(user, None, text, passphrase)
            .map_err(|_| AppError::new(ErrorCode::AuthenticationFailed))?;
        cancel.check()?;
        if !self.session.authenticated() {
            return Err(AppError::new(ErrorCode::AuthenticationFailed));
        }
        Ok(())
    }

    pub fn terminal(self, dimensions: TerminalSize) -> Result<Box<dyn Pty>> {
        let dimensions = dimensions.validate()?;
        if !self.reviewed || !self.session.authenticated() {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let mut channel = self.session.channel_session().map_err(|_| failed())?;
        channel
            .handle_extended_data(ssh2::ExtendedData::Merge)
            .map_err(|_| failed())?;
        channel
            .request_pty(
                "xterm-256color",
                None,
                Some((dimensions.cols.into(), dimensions.rows.into(), 0, 0)),
            )
            .map_err(|_| failed())?;
        channel.shell().map_err(|_| failed())?;
        self.session.set_blocking(false);
        Ok(Box::new(RemotePty {
            channel,
            _session: self.session,
            input: VecDeque::new(),
            offset: 0,
            resize: None,
            next_resize: None,
            failed: false,
            closed: false,
            exit_code: None,
        }))
    }

    pub fn authenticate_agent(
        &self,
        user: &str,
        socket: PathBuf,
        reviewed: PublicKeyMetadata,
        cancel: &Cancellation,
    ) -> Result<()> {
        if !self.reviewed {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let session = self.session.clone();
        let user = user.to_owned();
        agent_work(cancel, move || {
            let agent = agent(&session, &socket)?;
            let keys = agent.identities().map_err(|_| failed())?;
            let key = keys
                .iter()
                .find(|key| public_key(key.blob()).is_ok_and(|public| public == reviewed))
                .ok_or_else(|| AppError::new(ErrorCode::KeyUnavailable))?;
            agent
                .userauth(&user, key)
                .map_err(|_| AppError::new(ErrorCode::AuthenticationFailed))?;
            if !session.authenticated() {
                return Err(AppError::new(ErrorCode::AuthenticationFailed));
            }
            Ok(())
        })
    }

    pub fn authenticate_password(
        &self,
        user: &str,
        password: &str,
        cancel: &Cancellation,
    ) -> Result<()> {
        if !self.reviewed {
            return Err(AppError::new(ErrorCode::Denied));
        }
        cancel.check()?;
        self.session
            .userauth_password(user, password)
            .map_err(|_| AppError::new(ErrorCode::AuthenticationFailed))?;
        cancel.check()?;
        if !self.session.authenticated() {
            return Err(AppError::new(ErrorCode::AuthenticationFailed));
        }
        Ok(())
    }
}

// A single nonblocking channel owns its session. Cloned blocking channel readers
// share libssh2's lock and can otherwise prevent input/resize indefinitely.
struct RemotePty {
    channel: ssh2::Channel,
    _session: Session,
    input: VecDeque<Vec<u8>>,
    offset: usize,
    resize: Option<TerminalSize>,
    next_resize: Option<TerminalSize>,
    failed: bool,
    closed: bool,
    exit_code: Option<u32>,
}
fn again(error: &ssh2::Error) -> bool {
    error.code() == ssh2::ErrorCode::Session(-37)
}
impl Pty for RemotePty {
    fn input(&mut self, bytes: Vec<u8>) -> Result<()> {
        if self.failed || self.closed {
            return Err(failed());
        }
        if bytes.is_empty() || bytes.len() > 16384 {
            return Err(AppError::new(ErrorCode::InvalidDocument));
        }
        if self.input.len() >= 4 {
            return Err(AppError::new(ErrorCode::Capacity));
        }
        self.input.push_back(bytes);
        Ok(())
    }
    fn resize(&mut self, dimensions: TerminalSize) -> Result<()> {
        self.next_resize = Some(dimensions.validate()?);
        Ok(())
    }
    fn poll(&mut self) -> Result<PtyRead> {
        if !self.failed && !self.closed {
            // Retain exactly the same write buffer and resize arguments across
            // EAGAIN; coalesce subsequent resizes separately until it completes.
            if let Some(bytes) = self.input.front() {
                match self.channel.write(&bytes[self.offset..]) {
                    Ok(0) => self.failed = true,
                    Ok(n) => {
                        self.offset += n;
                        if self.offset == bytes.len() {
                            self.input.pop_front();
                            self.offset = 0;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                    Err(_) => self.failed = true,
                }
            }
            if self.resize.is_none() {
                self.resize = self.next_resize.take();
            }
            if let Some(size) = self.resize {
                match self
                    .channel
                    .request_pty_size(size.cols.into(), size.rows.into(), None, None)
                {
                    Ok(()) => self.resize = None,
                    Err(e) if again(&e) => (),
                    Err(_) => self.failed = true,
                }
            }
        }
        let mut bytes = vec![0; 16384];
        let length = if self.failed || self.closed {
            0
        } else {
            match self.channel.read(&mut bytes) {
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => 0,
                Err(_) => {
                    self.failed = true;
                    0
                }
            }
        };
        bytes.truncate(length);
        if !self.closed && !self.failed && self.channel.eof() {
            match self.channel.wait_close() {
                Ok(()) => {
                    self.closed = true;
                    self.exit_code = self
                        .channel
                        .exit_status()
                        .ok()
                        .and_then(|code| code.try_into().ok());
                }
                Err(e) if again(&e) => (),
                Err(_) => self.failed = true,
            }
        }
        Ok(PtyRead {
            bytes,
            exited: self.closed,
            exit_code: self.exit_code,
            failed: self.failed,
        })
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
