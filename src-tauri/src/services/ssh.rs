use super::*;
use ghosh_core::{operations::JobKind, policy::SecretAccess, ssh::*, workspace::model::AuthMethod};
use ghosh_ssh::{Connection, Endpoint, HostRecord};
use std::{
    io::Read,
    time::{Duration, Instant},
};

pub(super) struct Review {
    ticket: String,
    request: NodeConnectionRequest,
    session: String,
    identity: IdentityPreview,
    host: HostRecord,
    agent_socket: Option<PathBuf>,
    created: Instant,
}
fn denied() -> AppError {
    AppError::new(ErrorCode::Denied)
}
fn endpoint(node: &Node) -> Endpoint {
    Endpoint {
        host: node.host.clone(),
        port: node.port,
        user: node.user.clone(),
    }
}
fn require_file_identity(identity: &IdentityPreview) -> Result<()> {
    let message = match (identity.key_id.as_ref(), identity.status) {
        (Some(_), Some(KeyStatus::Available)) => return Ok(()),
        (None, _) => {
            "No SSH file key is assigned. In SSH Keys, choose a workspace or global default, or edit this node and select a File key."
        }
        (_, Some(KeyStatus::Changed)) => {
            "The selected SSH key file changed since registration. In SSH Keys, relink the same identity to verify it again, or explicitly select another key in Nodes."
        }
        (_, Some(KeyStatus::Unsafe)) => {
            "The selected SSH key path is unsafe. Choose a regular file owned by you with permissions 0600 or 0400, without symlinks or hard links, then relink it in SSH Keys."
        }
        _ => {
            "The selected SSH key reference or file is missing. Relink it in SSH Keys or edit this node to select an available File key. An explicit missing assignment does not fall back to another identity."
        }
    };
    Err(AppError {
        code: ErrorCode::KeyUnavailable,
        message: message.into(),
    })
}
fn prepare(
    active: &Active,
    request: &NodeConnectionRequest,
    keys: &KeysSnapshot,
) -> Result<(Node, IdentityPreview)> {
    let snapshot = active.workspace.snapshot();
    if !snapshot.trusted || snapshot.read_only || snapshot.external_changed {
        return Err(denied());
    }
    let node = active
        .workspace
        .node_details(Principal::LocalUi, &request.context)?
        .into_iter()
        .find(|node| node.id == request.node_id)
        .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
    if node.tunnel != "direct"
        || node.jump.as_ref().is_some_and(|id| !id.is_empty())
        || matches!(node.auth.method, AuthMethod::Key) && node.schema != 2
        || matches!(node.auth.method, AuthMethod::Password)
            && (node.schema != 2 || node.auth.password_ref.is_some())
    {
        return Err(AppError::new(ErrorCode::UnsupportedConnection));
    }
    let default = active
        .workspace
        .workspace_key(Principal::LocalUi, &request.context)?;
    let identity = select_identity(&node, default.as_deref(), keys);
    if matches!(node.auth.method, AuthMethod::Key) {
        require_file_identity(&identity)?;
    }
    Ok((node, identity))
}

/// Existing OpenSSH trust is read only. Do not execute ssh_config/ProxyCommand,
/// edit known_hosts, or silently accept malformed/oversized trust files.
fn known_hosts() -> Result<Vec<String>> {
    let home = std::env::var_os("HOME").ok_or_else(denied)?;
    let mut result = Vec::new();
    for relative in [".ssh/known_hosts", ".orbstack/ssh/known_hosts"] {
        let path = PathBuf::from(&home).join(relative);
        #[cfg(unix)]
        use std::os::unix::fs::OpenOptionsExt;
        let mut options = fs::OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        options.custom_flags(libc::O_NONBLOCK);
        let file = match options.open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return Err(AppError::new(ErrorCode::ConnectionFailed)),
        };
        if !file.metadata()?.is_file() {
            return Err(AppError::new(ErrorCode::ConnectionFailed));
        }
        let mut text = String::new();
        file.take(1024 * 1024 + 1)
            .read_to_string(&mut text)
            .map_err(|_| AppError::new(ErrorCode::ConnectionFailed))?;
        if text.len() > 1024 * 1024 {
            return Err(AppError::new(ErrorCode::Capacity));
        }
        result.push(text);
    }
    Ok(result)
}

impl Services {
    pub(super) fn runner_identity(
        &self,
        request: &NodeConnectionRequest,
        ticket: &str,
        cancel: &Cancellation,
    ) -> Result<serde_json::Value> {
        let guard = self.active.lock().map_err(poisoned)?;
        let active = guard.as_ref().ok_or_else(denied)?;
        let keys = self.keys()?;
        let (node, identity) = prepare(active, request, &keys)?;
        match node.auth.method {
            AuthMethod::Key => {
                let access = SecretAccess::authorize(
                    Principal::NativeService,
                    &request.context.workspace_id,
                    true,
                )?;
                let private = self.with_keys(|keys| {
                    keys.resolve(
                        &access,
                        identity.key_id.as_deref().ok_or_else(denied)?,
                        &identity.registry_revision,
                    )
                })?;
                Ok(
                    serde_json::json!({"ssh_key": std::str::from_utf8(&private).map_err(|_| denied())?}),
                )
            }
            // Pin the selected agent key through a public IdentityFile with
            // IdentitiesOnly. OpenSSH still obtains signing from that agent.
            AuthMethod::Agent => {
                let (socket, public) = {
                    let reviews = self.ssh_reviews.lock().map_err(poisoned)?;
                    let review = reviews
                        .iter()
                        .find(|r| r.ticket == ticket && r.request.node_id == request.node_id)
                        .ok_or_else(denied)?;
                    (
                        review.agent_socket.clone().ok_or_else(denied)?,
                        review.identity.public.clone().ok_or_else(denied)?,
                    )
                };
                let public = ghosh_ssh::agent_public_record(socket.clone(), public, cancel)?;
                Ok(serde_json::json!({"agent_socket": socket, "public_key": public}))
            }
            AuthMethod::Password => Ok(serde_json::json!({"password_auth": true})),
        }
    }
    pub fn connection_preview(
        &self,
        request: NodeConnectionRequest,
    ) -> Result<NodeConnectionPreview> {
        let job = self.jobs.begin(JobKind::NodeConnect, true)?;
        let result = (|| {
            let (node, mut identity, session) = {
                let guard = self.active.lock().map_err(poisoned)?;
                let active = guard.as_ref().ok_or_else(denied)?;
                let (node, identity) = prepare(active, &request, &self.keys()?)?;
                (node, identity, active.context.session_id.clone())
            };
            let agent_socket = if matches!(node.auth.method, AuthMethod::Agent) {
                let socket = node
                    .auth
                    .agent_socket
                    .as_ref()
                    .map(PathBuf::from)
                    .or_else(|| std::env::var_os("SSH_AUTH_SOCK").map(PathBuf::from))
                    .filter(|socket| socket.is_absolute())
                    .ok_or_else(|| AppError::new(ErrorCode::AgentUnavailable))?;
                identity.public = Some(ghosh_ssh::agent_identity(
                    socket.clone(),
                    job.cancellation(),
                )?);
                identity.status = Some(KeyStatus::Available);
                Some(socket)
            } else {
                None
            };
            let connection = Connection::open(
                &endpoint(&node),
                &known_hosts()?,
                None,
                false,
                job.cancellation(),
            )?;
            let guard = self.active.lock().map_err(poisoned)?;
            let active = guard.as_ref().ok_or_else(denied)?;
            prepare(active, &request, &self.keys()?)?;
            if active.context.session_id != session {
                return Err(denied());
            }
            let mut reviews = self.ssh_reviews.lock().map_err(poisoned)?;
            reviews.retain(|review| review.created.elapsed() < Duration::from_secs(120));
            if reviews.len() >= 8 {
                return Err(AppError::new(ErrorCode::Capacity));
            }
            let ticket = uuid_ticket();
            let preview = NodeConnectionPreview {
                ticket: ticket.clone(),
                node_id: node.id,
                host: node.host,
                port: node.port,
                user: node.user,
                identity: identity.clone(),
                host_key: connection.host.preview.clone(),
                credential_prompt: if matches!(node.auth.method, AuthMethod::Password) {
                    Some("SSH password".into())
                } else {
                    self.keys()?
                        .keys
                        .iter()
                        .find(|key| Some(&key.id) == identity.key_id.as_ref())
                        .filter(|key| key.encrypted)
                        .map(|_| "Key passphrase".into())
                },
            };
            job.cancellation().check()?;
            reviews.push(Review {
                ticket,
                request,
                session,
                identity,
                host: connection.host,
                agent_socket,
                created: Instant::now(),
            });
            Ok(preview)
        })();
        job.finish(&result)?;
        result
    }
    pub fn connection_test(&self, request: NodeConnectionCommit) -> Result<NodeConnectionResult> {
        let job = self.jobs.begin(JobKind::NodeConnect, true)?;
        let start = Instant::now();
        let result = self.with_connection(request, job.cancellation(), |connection, node| {
            Ok(NodeConnectionResult {
                node_id: node.id,
                host_key: connection.host.preview.clone(),
                elapsed_ms: start.elapsed().as_millis().min(u32::MAX as u128) as u32,
            })
        });
        job.finish(&result)?;
        result
    }
    pub fn terminal_open_node(
        &self,
        request: NodeTerminalRequest,
    ) -> Result<ghosh_core::terminal::TerminalSession> {
        request.dimensions.validate()?;
        let job = self.jobs.begin(JobKind::NodeConnect, true)?;
        let result = self.with_connection(
            request.connection,
            job.cancellation(),
            |connection, node| {
                let pty = connection.terminal(request.dimensions)?;
                job.cancellation().check()?;
                self.terminals.attach(
                    pty,
                    format!("{} · {}@{}:{}", node.id, node.user, node.host, node.port),
                    "SSH login shell".into(),
                    "~".into(),
                )
            },
        );
        job.finish(&result)?;
        result
    }
    pub(super) fn with_connection<T>(
        &self,
        mut request: NodeConnectionCommit,
        cancel: &Cancellation,
        effect: impl FnOnce(Connection, Node) -> Result<T>,
    ) -> Result<T> {
        // Move the secret out immediately; every return path clears native memory.
        let credential = request
            .credential
            .take()
            .map(|value| ghosh_core::secrets::SecretValue::new(value.into_bytes()));
        if credential
            .as_ref()
            .is_some_and(|value| value.expose().len() > 4096 || value.expose().contains(&0))
        {
            return Err(AppError::new(ErrorCode::InvalidDocument));
        }
        let review = {
            let mut reviews = self.ssh_reviews.lock().map_err(poisoned)?;
            let index = reviews
                .iter()
                .position(|review| review.ticket == request.ticket)
                .ok_or_else(denied)?;
            reviews.remove(index)
        };
        if review.created.elapsed() >= Duration::from_secs(120) {
            return Err(denied());
        }
        let node = {
            let guard = self.active.lock().map_err(poisoned)?;
            self.validate_connection_review(guard.as_ref().ok_or_else(denied)?, &review)?
        };
        let connection = Connection::open(
            &endpoint(&node),
            &known_hosts()?,
            Some(&review.host),
            request.allow_unknown,
            cancel,
        )?;
        // Serialize revocation, credential disclosure and terminal attachment.
        // DNS/TCP/handshake happen outside this lock; auth/setup have transport timeouts.
        let guard = self.active.lock().map_err(poisoned)?;
        let active = guard.as_ref().ok_or_else(denied)?;
        self.validate_connection_review(active, &review)?;
        let credential_text = credential
            .as_ref()
            .map(|value| std::str::from_utf8(value.expose()).map_err(|_| denied()))
            .transpose()?;
        match node.auth.method {
            AuthMethod::Key => {
                let access = SecretAccess::authorize(
                    Principal::NativeService,
                    &review.request.context.workspace_id,
                    true,
                )?;
                let private = self.with_keys(|keys| {
                    keys.resolve(
                        &access,
                        review.identity.key_id.as_deref().ok_or_else(denied)?,
                        &review.identity.registry_revision,
                    )
                })?;
                connection.authenticate_file(&node.user, &private, credential_text, cancel)?;
            }
            AuthMethod::Agent => connection.authenticate_agent(
                &node.user,
                review.agent_socket.ok_or_else(denied)?,
                review.identity.public.ok_or_else(denied)?,
                cancel,
            )?,
            AuthMethod::Password => connection.authenticate_password(
                &node.user,
                credential_text
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| AppError::new(ErrorCode::AuthenticationFailed))?,
                cancel,
            )?,
        }
        cancel.check()?;
        effect(connection, node)
    }
    pub(super) fn validate_run_ticket(
        &self,
        connection: &NodeConnectionCommit,
        context: &RevisionRequest,
        node: &str,
    ) -> Result<()> {
        let reviews = self.ssh_reviews.lock().map_err(poisoned)?;
        let review = reviews
            .iter()
            .find(|review| review.ticket == connection.ticket)
            .ok_or_else(denied)?;
        if review.request.context.workspace_id != context.workspace_id
            || review.request.context.revision != context.revision
            || review.request.node_id != node
            || review.created.elapsed() >= Duration::from_secs(120)
        {
            return Err(denied());
        }
        Ok(())
    }
    fn validate_connection_review(&self, active: &Active, review: &Review) -> Result<Node> {
        let keys = self.keys()?;
        let (node, identity) = prepare(active, &review.request, &keys)?;
        if active.context.session_id != review.session
            || identity.registry_revision != review.identity.registry_revision
            || identity.key_id != review.identity.key_id
        {
            return Err(denied());
        }
        Ok(node)
    }
}
fn uuid_ticket() -> String {
    // Existing native session UUID generator; random IDs contain no credentials.
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghosh_core::workspace::model::Auth;

    #[test]
    fn file_identity_errors_explain_assignment_and_file_recovery() {
        let mut identity = IdentityPreview {
            source: "global".into(),
            key_id: None,
            public: None,
            status: Some(KeyStatus::Missing),
            registry_revision: "test".into(),
        };
        assert!(
            require_file_identity(&identity)
                .unwrap_err()
                .message
                .contains("No SSH file key is assigned")
        );
        identity.key_id = Some("test".into());
        for (status, expected) in [
            (KeyStatus::Missing, "reference or file is missing"),
            (KeyStatus::Changed, "changed since registration"),
            (KeyStatus::Unsafe, "path is unsafe"),
        ] {
            identity.status = Some(status);
            let error = require_file_identity(&identity).unwrap_err();
            assert_eq!(error.code, ErrorCode::KeyUnavailable);
            assert!(error.message.contains(expected));
        }
        identity.status = Some(KeyStatus::Available);
        require_file_identity(&identity).unwrap();
    }

    /// Explicit opt-in only. Uses existing verified host trust and a referenced
    /// current-user key. Creates only disposable local workspace/registry files.
    #[test]
    #[ignore = "Authenticates to explicitly authorized test nodes; run manually"]
    fn authorized_nodes_native_authentication() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let service =
            Services::new(root.join("data"), Arc::new(DocumentOpenQueue::default())).unwrap();
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        let path = std::env::var("GHOSH_SMOKE_KEY")
            .unwrap_or_else(|_| home.join(".ssh/id_ed25519").to_string_lossy().into());
        let preview = service
            .preview_key(KeyImportRequest {
                path,
                label: "Authorized smoke identity".into(),
                storage: KeyStorage::Reference,
            })
            .unwrap();
        let keys = service
            .commit_key(KeyCommitRequest {
                preview,
                revision: service.keys().unwrap().revision,
            })
            .unwrap();
        let key_id = keys.keys[0].id.clone();
        let selected = std::env::var("GHOSH_SMOKE_NODE").ok();
        let use_agent = std::env::var_os("GHOSH_SMOKE_AGENT").is_some();
        let credential = std::env::var("GHOSH_SMOKE_PASSPHRASE").ok();
        let nodes = [("debian", "debian.orb.local"), ("node3", "node3.dabaga.cc")]
            .into_iter()
            .filter(|(id, _)| selected.as_deref().is_none_or(|selected| selected == *id))
            .collect::<Vec<_>>();
        assert!(
            !nodes.is_empty(),
            "GHOSH_SMOKE_NODE must be debian or node3"
        );
        let mut workspace = Workspace::create(
            &root,
            "SSH smoke",
            &root.join("locks"),
            &Cancellation::default(),
        )
        .unwrap();
        let snapshot = workspace.snapshot();
        let request = WorkspaceKeyRequest {
            context: RevisionRequest {
                workspace_id: snapshot.id,
                revision: snapshot.revision,
            },
            default_key: Some(key_id.clone()),
        };
        let preview = workspace
            .preview_workspace_key(Principal::LocalUi, &request)
            .unwrap();
        workspace
            .commit_workspace_key(
                Principal::LocalUi,
                &WorkspaceKeyCommit {
                    request,
                    digest: preview.digest,
                },
                &Cancellation::default(),
            )
            .unwrap();
        for &(id, host) in &nodes {
            let snapshot = workspace.snapshot();
            let mutation = NodeMutationRequest {
                context: RevisionRequest {
                    workspace_id: snapshot.id,
                    revision: snapshot.revision,
                },
                change: NodeChange::Create {
                    node: Node {
                        schema: 2,
                        id: id.into(),
                        host: host.into(),
                        port: 22,
                        user: "vm".into(),
                        auth: Auth {
                            method: if use_agent {
                                AuthMethod::Agent
                            } else {
                                AuthMethod::Key
                            },
                            key_ref: None,
                            key_id: if use_agent {
                                None
                            } else {
                                Some(key_id.clone())
                            },
                            password_ref: None,
                            agent_socket: None,
                        },
                        labels: vec![],
                        tunnel: "direct".into(),
                        jump: None,
                    },
                },
            };
            let preview = workspace
                .preview_node(Principal::LocalUi, &mutation)
                .unwrap();
            workspace
                .commit_node(
                    Principal::LocalUi,
                    &NodeCommitRequest {
                        mutation,
                        digest: preview.digest,
                    },
                    &Cancellation::default(),
                )
                .unwrap();
        }
        let snapshot = workspace.snapshot();
        workspace
            .set_trusted(Principal::LocalUi, &snapshot.id, &snapshot.revision, true)
            .unwrap();
        let watch = WorkspaceWatch::start(workspace.root(), || {}).unwrap();
        *service.active.lock().unwrap() = Some(Active {
            workspace,
            context: ApprovalContext::new(snapshot.id.clone(), snapshot.revision.clone()),
            _watch: watch,
        });
        for &(id, _) in &nodes {
            let review = service
                .connection_preview(NodeConnectionRequest {
                    context: RevisionRequest {
                        workspace_id: snapshot.id.clone(),
                        revision: snapshot.revision.clone(),
                    },
                    node_id: id.into(),
                })
                .unwrap_or_else(|error| panic!("{id}: preview {}", error.message));
            assert_eq!(
                review.host_key.trust,
                HostTrust::Known,
                "{id}: existing trust required"
            );
            let ticket = review.ticket;
            let result = service
                .connection_test(NodeConnectionCommit {
                    ticket: ticket.clone(),
                    allow_unknown: false,
                    credential: credential.clone(),
                })
                .unwrap_or_else(|error| panic!("{id}: authenticate {}", error.message));
            assert_eq!(result.node_id, id);
            eprintln!(
                "{id}: native authenticated; {} {}; {} ms",
                result.host_key.algorithm, result.host_key.fingerprint, result.elapsed_ms
            );
            assert!(
                service
                    .connection_test(NodeConnectionCommit {
                        ticket,
                        allow_unknown: false,
                        credential: credential.clone(),
                    })
                    .is_err()
            );
            let preview = service
                .connection_preview(NodeConnectionRequest {
                    context: RevisionRequest {
                        workspace_id: snapshot.id.clone(),
                        revision: snapshot.revision.clone(),
                    },
                    node_id: id.into(),
                })
                .unwrap();
            let terminal = service
                .terminal_open_node(NodeTerminalRequest {
                    connection: NodeConnectionCommit {
                        ticket: preview.ticket.clone(),
                        allow_unknown: false,
                        credential: credential.clone(),
                    },
                    dimensions: ghosh_core::terminal::TerminalSize {
                        rows: 37,
                        cols: 101,
                    },
                })
                .unwrap_or_else(|error| panic!("{id}: terminal {}", error.message));
            assert!(
                service
                    .terminal_open_node(NodeTerminalRequest {
                        connection: NodeConnectionCommit {
                            ticket: preview.ticket,
                            allow_unknown: false,
                            credential: None
                        },
                        dimensions: ghosh_core::terminal::TerminalSize {
                            rows: 37,
                            cols: 101
                        },
                    })
                    .is_err(),
                "terminal ticket must not replay"
            );
            service
                .terminals
                .resize(
                    &terminal.id,
                    ghosh_core::terminal::TerminalSize {
                        rows: 41,
                        cols: 111,
                    },
                )
                .unwrap();
            let first = service.terminals.poll(&terminal.id, 0).unwrap();
            let mut ack = first.sequence;
            let mut output = first.bytes;
            service.terminals.input(&terminal.id, b"unset HISTFILE; test -t 0 && test -t 1 && printf 'GHOSH_REMOTE_PTY_OK\\n'; stty size; printf '\\033[31mhello ".iter().copied().chain("世界".bytes()).chain(b"\\033[0m\\n'; exit 7\n".iter().copied()).collect()).unwrap();
            let deadline = Instant::now() + Duration::from_secs(15);
            loop {
                assert!(Instant::now() < deadline, "{id}: terminal deadline");
                let packet = service.terminals.poll(&terminal.id, ack).unwrap();
                assert!(!packet.failed, "{id}: terminal transport failed");
                if packet.sequence != ack {
                    assert_eq!(
                        service.terminals.poll(&terminal.id, ack).unwrap().bytes,
                        packet.bytes,
                        "replay must be identical"
                    );
                    output.extend(&packet.bytes);
                    assert!(output.len() < 262144, "bounded smoke output");
                    ack = packet.sequence;
                }
                if packet.exited {
                    assert_eq!(packet.exit_code, Some(7));
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let output = String::from_utf8_lossy(&output);
            assert!(output.contains("GHOSH_REMOTE_PTY_OK"), "{id}: TTY probe");
            assert!(output.contains("41 111"), "{id}: remote resize");
            assert!(
                output.contains("\u{1b}[31mhello 世界"),
                "{id}: ANSI and UTF8"
            );
            let closing = Instant::now();
            service.terminals.close(&terminal.id).unwrap();
            assert!(closing.elapsed() < Duration::from_secs(2));
            assert!(service.terminals.poll(&terminal.id, ack).is_err());
            eprintln!(
                "{id}: native remote PTY, resize, replay, ANSI/UTF8, exit 7 and cleanup passed"
            );
            let snapshot = service.snapshot().unwrap().unwrap();
            let preview = service
                .connection_preview(NodeConnectionRequest {
                    context: RevisionRequest {
                        workspace_id: snapshot.id,
                        revision: snapshot.revision,
                    },
                    node_id: id.into(),
                })
                .unwrap();
            service.with_connection(NodeConnectionCommit { ticket: preview.ticket, allow_unknown: false, credential: credential.clone() }, &Cancellation::default(), |mut connection, _| {
                use ghosh_core::runs::*;
                let out = connection.execute("printf 'EXEC_SMOKE\\n'; printf 'stderr-smoke\\n' >&2; yes x | head -c 131072; yes e | head -c 131072 >&2; exit 7\n", 10, &Cancellation::default())?;
                assert_eq!(out.end, ExecEnd::Exited(7)); assert!(out.truncated); assert_eq!(out.output.len(), OUTPUT_LIMIT);
                assert!(String::from_utf8_lossy(&out.output).contains("EXEC_SMOKE"));
                Ok(())
            }).unwrap();
            eprintln!("{id}: native exec drains both streams, bounds output and preserves exit 7");
        }
        use ghosh_core::{automation::*, runs::*, workspace::automation::*};
        let service = Arc::new(service);
        let snapshot = service.snapshot().unwrap().unwrap();
        let mutation = AutomationMutation { context: RevisionRequest { workspace_id: snapshot.id, revision: snapshot.revision }, document: AutomationDocument { id: "authorized-facts".into(), kind: AutomationKind::Playbook, source: "schema: 1\nname: Authorized read-only facts\ntasks:\n  - name: Facts\n    check:\n      command: 'uname -s; uptime; printf exec-stderr >&2'\n    shell:\n      command: 'false'\n    timeout_s: 15\n".into() }, replace: false, remove: false };
        let preview = service.automation_preview(mutation.clone()).unwrap();
        let snapshot = service
            .automation_commit(AutomationCommit {
                mutation,
                digest: preview.digest,
            })
            .unwrap();
        let context = RevisionRequest {
            workspace_id: snapshot.id,
            revision: snapshot.revision,
        };
        let targets = nodes
            .iter()
            .map(|(id, _)| {
                let preview = service
                    .connection_preview(NodeConnectionRequest {
                        context: context.clone(),
                        node_id: (*id).into(),
                    })
                    .unwrap();
                RunTarget {
                    node_id: (*id).into(),
                    connection: Some(NodeConnectionCommit {
                        ticket: preview.ticket,
                        allow_unknown: false,
                        credential: credential.clone(),
                    }),
                }
            })
            .collect();
        let initial = service
            .run_start(RunRequest {
                automation: AutomationReadRequest {
                    context: context.clone(),
                    id: "authorized-facts".into(),
                    kind: AutomationKind::Playbook,
                },
                mode: RunMode::Check,
                targets,
                input: "{}".into(),
                controller_python: None,
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(45);
        while service.run_active().unwrap() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(20));
        }
        let report = service
            .runs_list(context)
            .unwrap()
            .into_iter()
            .find(|report| report.id == initial.id)
            .unwrap();
        assert_eq!(report.state, RunState::Succeeded, "{:?}", report.results);
        assert_eq!(report.results.len(), nodes.len());
        assert!(report.results.iter().all(|result| result.phase == "check"
            && result.output.contains("Linux")
            && result.output.contains("exec-stderr")));
        if let Ok(python) = std::env::var("GHOSH_CONTROLLER_PYTHON") {
            let snapshot = service.snapshot().unwrap().unwrap();
            let mutation = AutomationMutation { context: RevisionRequest { workspace_id: snapshot.id, revision: snapshot.revision }, document: AutomationDocument { id: "ansible-facts".into(), kind: AutomationKind::Playbook, source: "- name: Authorized upstream facts\n  hosts: all\n  gather_facts: false\n  tasks:\n    - name: Read-only facts\n      ansible.builtin.shell: uname -s; uptime; printf exec-stderr >&2\n      changed_when: false\n      check_mode: false\n".into() }, replace: false, remove: false };
            let preview = service.automation_preview(mutation.clone()).unwrap();
            let snapshot = service
                .automation_commit(AutomationCommit {
                    mutation,
                    digest: preview.digest,
                })
                .unwrap();
            let context = RevisionRequest {
                workspace_id: snapshot.id,
                revision: snapshot.revision,
            };
            let targets = nodes
                .iter()
                .map(|(id, _)| {
                    let preview = service
                        .connection_preview(NodeConnectionRequest {
                            context: context.clone(),
                            node_id: (*id).into(),
                        })
                        .unwrap();
                    RunTarget {
                        node_id: (*id).into(),
                        connection: Some(NodeConnectionCommit {
                            ticket: preview.ticket,
                            allow_unknown: false,
                            credential: credential.clone(),
                        }),
                    }
                })
                .collect();
            let started = service
                .run_start(RunRequest {
                    automation: AutomationReadRequest {
                        context: context.clone(),
                        id: "ansible-facts".into(),
                        kind: AutomationKind::Playbook,
                    },
                    mode: RunMode::Check,
                    targets,
                    input: "{}".into(),
                    controller_python: Some(python),
                })
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(90);
            while service.run_active().unwrap() {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(20));
            }
            let report = service
                .runs_list(context)
                .unwrap()
                .into_iter()
                .find(|r| r.id == started.id)
                .unwrap();
            assert_eq!(report.state, RunState::Succeeded, "{:?}", report.results);
            assert_eq!(report.results.len(), nodes.len(), "{:?}", report.results);
            assert!(
                report
                    .results
                    .iter()
                    .all(|r| r.output.contains("Linux") && r.output.contains("exec-stderr"))
            );
            eprintln!(
                "upstream Ansible with exact OpenSSH host pins passed on {} node(s)",
                nodes.len()
            );
        }
        eprintln!(
            "saved native scenario → SSH → bounded persisted report passed for {} node(s)",
            nodes.len()
        );
    }
}
