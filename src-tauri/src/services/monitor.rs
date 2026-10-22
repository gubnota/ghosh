//! One ephemeral Monitor connection, independent of automation and terminal ports.
use super::*;
use ghosh_core::{monitor::*, terminal::PtyRead};
use std::{
    io::Read,
    process::{Child, ChildStderr, ChildStdout, Command, Stdio},
    time::{Duration, Instant},
};
struct Process {
    child: Child,
    stdout: ChildStdout,
    stderr: ChildStderr,
}
impl Drop for Process {
    fn drop(&mut self) {
        // Every local probe/log command owns an isolated process group.
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.child.id() as i32), libc::SIGKILL);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[derive(Default)]
struct LocalMonitor {
    processes: [Option<Process>; 2],
}
impl MonitorTransport for LocalMonitor {
    fn start(&mut self, slot: usize, command: &str) -> Result<()> {
        self.stop(slot);
        let mut cmd = Command::new("/bin/sh");
        cmd.arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(home) = std::env::var_os("HOME") {
            cmd.current_dir(home);
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::new(ErrorCode::Internal))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::new(ErrorCode::Internal))?;
        let process = Process {
            child,
            stdout,
            stderr,
        };
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            for fd in [process.stdout.as_raw_fd(), process.stderr.as_raw_fd()] {
                let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
                if flags < 0
                    || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
                {
                    return Err(AppError::new(ErrorCode::Io));
                }
            }
        }
        self.processes[slot] = Some(process);
        Ok(())
    }
    fn poll(&mut self, slot: usize) -> Result<PtyRead> {
        let process = self.processes[slot]
            .as_mut()
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        let mut bytes = Vec::new();
        let mut eof = true;
        for pipe in [
            &mut process.stdout as &mut dyn Read,
            &mut process.stderr as &mut dyn Read,
        ] {
            let mut buffer = [0; 8192];
            match pipe.read(&mut buffer) {
                Ok(n) => {
                    bytes.extend_from_slice(&buffer[..n]);
                    eof &= n == 0;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => eof = false,
                Err(e) => return Err(e.into()),
            }
        }
        let status = process.child.try_wait()?;
        Ok(PtyRead {
            bytes,
            exited: eof && status.is_some(),
            exit_code: status
                .and_then(|s| s.code())
                .and_then(|n| n.try_into().ok()),
            failed: false,
        })
    }
    fn stop(&mut self, slot: usize) {
        self.processes[slot] = None;
    }
}
pub(super) struct Monitor {
    session: MonitorSession,
    context: RevisionRequest,
    transport: Box<dyn MonitorTransport>,
    output: Vec<u8>,
    truncated: bool,
    running: bool,
    exit_code: Option<u32>,
    metrics: MonitorMetrics,
    probe: Option<Instant>,
    probe_output: Vec<u8>,
    next_probe: Instant,
}
impl Monitor {
    fn new(context: RevisionRequest, target: String, transport: Box<dyn MonitorTransport>) -> Self {
        Self {
            session: MonitorSession {
                id: uuid::Uuid::new_v4().to_string(),
                target,
            },
            context,
            transport,
            output: vec![],
            truncated: false,
            running: false,
            exit_code: None,
            metrics: MonitorMetrics::default(),
            probe: None,
            probe_output: vec![],
            next_probe: Instant::now(),
        }
    }
}
impl Services {
    fn monitor_context(&self, context: &RevisionRequest) -> Result<()> {
        let guard = self.active.lock().map_err(poisoned)?;
        let active = guard
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?;
        let snapshot = active.workspace.snapshot();
        if !snapshot.trusted || snapshot.external_changed || snapshot.read_only {
            return Err(AppError::new(ErrorCode::Denied));
        }
        active.workspace.node_details(Principal::LocalUi, context)?;
        Ok(())
    }
    pub fn monitor_open(&self, request: MonitorOpen) -> Result<MonitorSession> {
        self.monitor_context(&request.context)?;
        let attach =
            |transport: Box<dyn MonitorTransport>, target: String| -> Result<MonitorSession> {
                let monitor = Monitor::new(request.context.clone(), target, transport);
                let session = monitor.session.clone();
                let mut guard = self.monitor.lock().map_err(poisoned)?;
                if guard.is_some() {
                    return Err(AppError::new(ErrorCode::Busy));
                }
                *guard = Some(monitor);
                Ok(session)
            };
        if let Some(connection) = request.connection {
            self.with_connection(connection, &Cancellation::default(), |connection, node| {
                attach(connection.monitor()?, node.id)
            })
        } else {
            // Hold workspace first, matching revocation lock order.
            let active = self.active.lock().map_err(poisoned)?;
            active
                .as_ref()
                .ok_or_else(|| AppError::new(ErrorCode::Denied))?
                .workspace
                .node_details(Principal::LocalUi, &request.context)?;
            attach(Box::<LocalMonitor>::default(), "Local machine".into())
        }
    }
    pub fn monitor_shutdown(&self) {
        if let Ok(mut guard) = self.monitor.lock() {
            *guard = None;
        }
    }
    pub fn monitor_close(&self, id: String) -> Result<()> {
        let mut guard = self.monitor.lock().map_err(poisoned)?;
        if guard.as_ref().is_some_and(|m| m.session.id == id) {
            *guard = None;
        }
        Ok(())
    }
    pub fn monitor_start(&self, id: String, command: String) -> Result<()> {
        if command.trim().is_empty() || command.len() > 16384 || command.contains('\0') {
            return Err(AppError::new(ErrorCode::InvalidDocument));
        }
        let context = self
            .monitor
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .filter(|m| m.session.id == id)
            .map(|m| m.context.clone())
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        self.monitor_context(&context)?;
        let _active = self.active.lock().map_err(poisoned)?;
        let mut guard = self.monitor.lock().map_err(poisoned)?;
        let m = guard
            .as_mut()
            .filter(|m| m.session.id == id)
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        if m.running {
            return Err(AppError::new(ErrorCode::Busy));
        }
        m.transport.start(0, &command)?;
        m.output.clear();
        m.truncated = false;
        m.exit_code = None;
        m.running = true;
        Ok(())
    }
    pub fn monitor_poll(&self, id: String, visible: bool) -> Result<MonitorPacket> {
        let context = self
            .monitor
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .filter(|m| m.session.id == id)
            .map(|m| m.context.clone())
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        if let Err(error) = self.monitor_context(&context) {
            self.monitor_close(id)?;
            return Err(error);
        }
        let _active = self.active.lock().map_err(poisoned)?;
        let mut guard = self.monitor.lock().map_err(poisoned)?;
        let m = guard
            .as_mut()
            .filter(|m| m.session.id == id)
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        if m.running {
            for _ in 0..4 {
                let packet = m.transport.poll(0)?;
                let empty = packet.bytes.is_empty();
                m.output.extend_from_slice(&packet.bytes);
                if m.output.len() > 65536 {
                    m.output.drain(..m.output.len() - 65536);
                    m.truncated = true;
                }
                if packet.exited || packet.failed {
                    m.running = false;
                    m.exit_code = packet.exit_code;
                    m.transport.stop(0);
                    break;
                }
                if empty {
                    break;
                }
            }
        }
        if !visible && m.probe.take().is_some() {
            m.transport.stop(1);
            m.probe_output.clear();
        }
        if visible && m.probe.is_none() && Instant::now() >= m.next_probe {
            m.transport.start(1, METRICS_COMMAND)?;
            m.probe = Some(Instant::now());
            m.probe_output.clear();
            m.next_probe = Instant::now() + Duration::from_secs(5);
        }
        if let Some(started) = m.probe {
            let packet = m.transport.poll(1)?;
            if m.probe_output.len() + packet.bytes.len() <= 16384 {
                m.probe_output.extend_from_slice(&packet.bytes);
            }
            if packet.exited || packet.failed || started.elapsed() > Duration::from_secs(4) {
                m.metrics = parse_metrics(&String::from_utf8_lossy(&m.probe_output));
                m.transport.stop(1);
                m.probe = None;
            }
        }
        Ok(MonitorPacket {
            output: String::from_utf8_lossy(&m.output).into_owned(),
            truncated: m.truncated,
            running: m.running,
            exit_code: m.exit_code,
            metrics: m.metrics.clone(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_probes_stream_and_stop_without_waiting_for_child() {
        let mut port = LocalMonitor::default();
        port.start(0, "printf monitor-sentinel; sleep 30").unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut output = vec![];
        while Instant::now() < deadline {
            let p = port.poll(0).unwrap();
            output.extend(p.bytes);
            if !output.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(output, b"monitor-sentinel");
        port.start(1, METRICS_COMMAND).unwrap();
        let mut metrics = vec![];
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(Instant::now() < deadline);
            let p = port.poll(1).unwrap();
            metrics.extend(p.bytes);
            if p.exited {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            parse_metrics(&String::from_utf8_lossy(&metrics))
                .disk_percent
                .is_some()
        );
        let started = Instant::now();
        port.stop(0);
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
