//! Native PTY ownership and bounded transport. No transcript persistence.
use ghosh_core::{contracts::*, terminal::*};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
    sync::{
        Mutex,
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    thread,
    time::{Duration, Instant},
};

const CHUNK: usize = 16 * 1024;
const QUEUE: usize = 8;
const MAX_SESSIONS: usize = 8;

fn failure(_: impl std::fmt::Display) -> AppError {
    AppError::new(ErrorCode::Internal)
}
fn size(size: TerminalSize) -> Result<PtySize> {
    let size = size.validate()?;
    Ok(PtySize {
        rows: size.rows,
        cols: size.cols,
        pixel_width: 0,
        pixel_height: 0,
    })
}
struct LocalPty {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    input: SyncSender<Vec<u8>>,
    output: Receiver<Vec<u8>>,
    eof: bool,
    write_failed: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl LocalPty {
    fn stop(&mut self) {
        // try_wait caches a reaped status. Never signal a pid after reaping it.
        if self.child.try_wait().ok().flatten().is_some() {
            return;
        }
        #[cfg(unix)]
        if let Some(pid) = self.child.process_id() {
            // The unreaped child reserves its pid during this cleanup. Signal its
            // foreground job as well as its own group; don't signal the app group.
            if let Some(group) = self
                .master
                .process_group_leader()
                .filter(|group| *group > 1)
            {
                unsafe {
                    libc::kill(-group, libc::SIGHUP);
                }
            }
            unsafe {
                libc::kill(-(pid as i32), libc::SIGHUP);
            }
        }
        let _ = self.child.kill();
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            if Instant::now() >= deadline {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        #[cfg(unix)]
        if let Some(pid) = self.child.process_id() {
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }
        let _ = self.child.wait();
    }
}
impl Drop for LocalPty {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Pty for LocalPty {
    fn input(&mut self, bytes: Vec<u8>) -> Result<()> {
        self.input
            .try_send(bytes)
            .map_err(|_| AppError::new(ErrorCode::Capacity))
    }
    fn resize(&mut self, dimensions: TerminalSize) -> Result<()> {
        self.master.resize(size(dimensions)?).map_err(failure)
    }
    fn poll(&mut self) -> Result<PtyRead> {
        let bytes = match self.output.try_recv() {
            Ok(bytes) => bytes,
            Err(TryRecvError::Empty) => Vec::new(),
            Err(TryRecvError::Disconnected) => {
                self.eof = true;
                Vec::new()
            }
        };
        let exit = self.child.try_wait().map_err(failure)?;
        Ok(PtyRead {
            bytes,
            exited: exit.is_some() && self.eof,
            exit_code: exit.map(|value| value.exit_code()),
            failed: self.write_failed.load(std::sync::atomic::Ordering::Acquire),
        })
    }
}
struct Session {
    pty: Box<dyn Pty>,
    pending: Option<TerminalPacket>,
    sequence: u32,
    acknowledged: u32,
}
#[derive(Default)]
pub struct Terminals {
    sessions: Mutex<HashMap<String, Session>>,
}
impl Terminals {
    /// Explicit local UI action only. Never use a package path or command here.
    pub fn open_local(&self, dimensions: TerminalSize) -> Result<TerminalSession> {
        let dimensions = size(dimensions)?;
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && path.is_dir())
            .ok_or_else(|| AppError::new(ErrorCode::InvalidPath))?;
        let shell = if cfg!(target_os = "macos") {
            "/bin/zsh"
        } else {
            "/bin/bash"
        };
        let mut command = CommandBuilder::new(shell);
        command.arg("-l");
        command.cwd(&home);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        self.spawn(
            command,
            dimensions,
            "Local machine".into(),
            shell.into(),
            home.to_string_lossy().into(),
        )
    }
    fn spawn(
        &self,
        command: CommandBuilder,
        dimensions: PtySize,
        target: String,
        shell: String,
        cwd: String,
    ) -> Result<TerminalSession> {
        let mut sessions = self.sessions.lock().map_err(failure)?;
        if sessions.len() >= MAX_SESSIONS {
            return Err(AppError::new(ErrorCode::Capacity));
        }
        let pair = native_pty_system().openpty(dimensions).map_err(failure)?;
        let mut reader = pair.master.try_clone_reader().map_err(failure)?;
        let mut writer = pair.master.take_writer().map_err(failure)?;
        let child = pair.slave.spawn_command(command).map_err(failure)?;
        drop(pair.slave);
        let (send_output, output) = mpsc::sync_channel(QUEUE);
        let (input, receive_input) = mpsc::sync_channel::<Vec<u8>>(4);
        let write_failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = write_failed.clone();
        // At most 8 queued chunks + one in-flight read + one unacknowledged
        // packet. The PTY producer blocks rather than losing terminal output.
        thread::spawn(move || {
            let mut buffer = [0; CHUNK];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if send_output.send(buffer[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });
        thread::spawn(move || {
            while let Ok(bytes) = receive_input.recv() {
                if writer
                    .write_all(&bytes)
                    .and_then(|()| writer.flush())
                    .is_err()
                {
                    flag.store(true, std::sync::atomic::Ordering::Release);
                    break;
                }
            }
        });
        Self::insert(
            &mut sessions,
            Box::new(LocalPty {
                master: pair.master,
                child,
                input,
                output,
                eof: false,
                write_failed,
            }),
            target,
            shell,
            cwd,
        )
    }
    /// Inject a native-owned adapter; no transport-specific dependencies here.
    pub fn attach(
        &self,
        pty: Box<dyn Pty>,
        target: String,
        shell: String,
        cwd: String,
    ) -> Result<TerminalSession> {
        let mut sessions = self.sessions.lock().map_err(failure)?;
        if sessions.len() >= MAX_SESSIONS {
            return Err(AppError::new(ErrorCode::Capacity));
        }
        Self::insert(&mut sessions, pty, target, shell, cwd)
    }
    fn insert(
        sessions: &mut HashMap<String, Session>,
        pty: Box<dyn Pty>,
        target: String,
        shell: String,
        cwd: String,
    ) -> Result<TerminalSession> {
        let id = uuid::Uuid::new_v4().to_string();
        sessions.insert(
            id.clone(),
            Session {
                pty,
                pending: None,
                sequence: 0,
                acknowledged: 0,
            },
        );
        Ok(TerminalSession {
            id,
            target,
            shell,
            cwd,
        })
    }
    pub fn input(&self, id: &str, bytes: Vec<u8>) -> Result<()> {
        if bytes.is_empty() || bytes.len() > CHUNK {
            return Err(AppError::new(ErrorCode::InvalidDocument));
        }
        let mut sessions = self.sessions.lock().map_err(failure)?;
        sessions
            .get_mut(id)
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .pty
            .input(bytes)
    }
    pub fn resize(&self, id: &str, dimensions: TerminalSize) -> Result<()> {
        dimensions.validate()?;
        let mut sessions = self.sessions.lock().map_err(failure)?;
        sessions
            .get_mut(id)
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .pty
            .resize(dimensions)
    }
    pub fn poll(&self, id: &str, ack: u32) -> Result<TerminalPacket> {
        let mut sessions = self.sessions.lock().map_err(failure)?;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?;
        if ack != session.acknowledged {
            if session
                .pending
                .as_ref()
                .is_none_or(|packet| packet.sequence != ack)
            {
                return Err(AppError::new(ErrorCode::Conflict));
            }
            session.acknowledged = ack;
            session.pending = None;
        }
        if let Some(packet) = &session.pending {
            return Ok(packet.clone());
        }
        let PtyRead {
            bytes,
            exited,
            exit_code,
            failed,
        } = session.pty.poll()?;
        if !bytes.is_empty() || exited || failed {
            session.sequence = session
                .sequence
                .checked_add(1)
                .ok_or_else(|| AppError::new(ErrorCode::Capacity))?;
            let packet = TerminalPacket {
                sequence: session.sequence,
                bytes,
                exited,
                exit_code,
                failed,
            };
            session.pending = Some(packet.clone());
            return Ok(packet);
        }
        Ok(TerminalPacket {
            sequence: session.acknowledged,
            bytes,
            exited: false,
            exit_code: None,
            failed: false,
        })
    }
    pub fn close(&self, id: &str) -> Result<()> {
        // Drop/termination outside the map lock so other sessions remain usable.
        let session = self.sessions.lock().map_err(failure)?.remove(id);
        drop(session);
        Ok(())
    }
    pub fn close_all(&self) {
        let sessions = self
            .sessions
            .lock()
            .map(|mut sessions| std::mem::take(&mut *sessions));
        drop(sessions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_pty_resize_replay_utf8_and_exit() {
        let terminals = Terminals::default();
        let mut command = CommandBuilder::new("/bin/sh");
        command.arg("-c");
        command.arg("read go; test -t 0 && test -t 1 && printf 'GHOSH_PTY_OK\\n'; stty size; printf '\\033[31mhello 世界\\033[0m\\n'; exit 7");
        let session = terminals
            .spawn(
                command,
                size(TerminalSize { rows: 24, cols: 80 }).unwrap(),
                "Test".into(),
                "/bin/sh".into(),
                "/".into(),
            )
            .unwrap();
        terminals
            .resize(
                &session.id,
                TerminalSize {
                    rows: 37,
                    cols: 101,
                },
            )
            .unwrap();
        assert!(terminals.poll(&session.id, 99).is_err());
        terminals.input(&session.id, b"go\n".to_vec()).unwrap();
        let mut ack = 0;
        let mut result = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(Instant::now() < deadline);
            let packet = terminals.poll(&session.id, ack).unwrap();
            if packet.sequence != ack {
                assert_eq!(
                    terminals.poll(&session.id, ack).unwrap().bytes,
                    packet.bytes
                );
                result.extend(&packet.bytes);
                ack = packet.sequence;
            }
            if packet.exited {
                assert_eq!(packet.exit_code, Some(7));
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        let result = String::from_utf8(result).unwrap();
        assert!(result.contains("GHOSH_PTY_OK"));
        assert!(result.contains("37 101"));
        assert!(result.contains("\u{1b}[31mhello 世界"));
        terminals.close(&session.id).unwrap();
        assert!(terminals.poll(&session.id, ack).is_err());
    }
    #[test]
    fn bounds_and_close_busy_producer() {
        let terminals = Terminals::default();
        assert!(
            terminals
                .open_local(TerminalSize { rows: 0, cols: 80 })
                .is_err()
        );
        let mut command = CommandBuilder::new("/usr/bin/yes");
        command.arg("GHOSH_DISPOSABLE_FLOOD");
        let session = terminals
            .spawn(
                command,
                size(TerminalSize { rows: 24, cols: 80 }).unwrap(),
                "Test".into(),
                "yes".into(),
                "/".into(),
            )
            .unwrap();
        assert!(terminals.input(&session.id, vec![0; CHUNK + 1]).is_err());
        thread::sleep(Duration::from_millis(80));
        let deadline = Instant::now();
        terminals.close(&session.id).unwrap();
        assert!(deadline.elapsed() < Duration::from_secs(2));
    }
}
