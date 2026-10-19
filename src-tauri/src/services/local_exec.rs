//! Local controller execution with bounded nonblocking pipes and process-group
//! cleanup. The same engine is used for local and SSH scenarios.
use ghosh_core::{contracts::*, runs::*};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
pub(super) struct LocalExecutor;
impl Executor for LocalExecutor {
    fn execute(
        &mut self,
        script: &str,
        timeout_s: u32,
        cancel: &Cancellation,
    ) -> Result<ExecOutput> {
        cancel.check()?;
        let mut command = Command::new("/bin/sh");
        command
            .arg("-s")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(home) = std::env::var_os("HOME") {
            command.current_dir(home);
        }
        run_command(command, script, timeout_s, cancel, &mut |_| Ok(()))
    }
}
/// Stream structured controller events while retaining bounded diagnostics and
/// the same process-group cleanup used for local scripts.
pub(super) fn run_command(
    mut command: Command,
    script: &str,
    timeout_s: u32,
    cancel: &Cancellation,
    on_stdout: &mut dyn FnMut(&[u8]) -> Result<()>,
) -> Result<ExecOutput> {
    cancel.check()?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let result = (|| {
        let mut input = child.stdin.take();
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::new(ErrorCode::Internal))?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::new(ErrorCode::Internal))?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            for fd in [
                input.as_ref().unwrap().as_raw_fd(),
                stdout.as_raw_fd(),
                stderr.as_raw_fd(),
            ] {
                // Pipes are exclusively owned by this invocation.
                let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
                if flags < 0
                    || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
                {
                    return Err(AppError::new(ErrorCode::Io));
                }
            }
        }
        let mut offset = 0;
        let mut output = vec![];
        let mut truncated = false;
        let mut out_eof = false;
        let mut err_eof = false;
        let deadline = Instant::now() + Duration::from_secs(timeout_s.into());
        let end = 'execution: loop {
            if cancel.check().is_err() {
                break ExecEnd::Cancelled;
            }
            if Instant::now() >= deadline {
                break ExecEnd::TimedOut;
            }
            if let Some(stdin) = &mut input {
                match stdin.write(&script.as_bytes()[offset..]) {
                    Ok(length) => {
                        offset += length;
                        if offset == script.len() {
                            input = None;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                    Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => input = None,
                    Err(_) => break 'execution ExecEnd::Disconnected,
                }
            }
            for (index, (stream, eof)) in [
                (&mut stdout as &mut dyn Read, &mut out_eof),
                (&mut stderr as &mut dyn Read, &mut err_eof),
            ]
            .into_iter()
            .enumerate()
            {
                let mut bytes = [0; 8192];
                match stream.read(&mut bytes) {
                    Ok(0) => *eof = true,
                    Ok(length) => {
                        if index == 0 {
                            on_stdout(&bytes[..length])?;
                        }
                        bounded_append(&mut output, &bytes[..length], &mut truncated);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                    Err(_) => break 'execution ExecEnd::Disconnected,
                }
            }
            let status = match child.try_wait() {
                Ok(status) => status,
                Err(_) => break ExecEnd::Disconnected,
            };
            if let Some(status) = status
                && out_eof
                && err_eof
            {
                break ExecEnd::Exited(status.code().unwrap_or(128));
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        Ok(ExecOutput {
            end,
            output,
            truncated,
        })
    })();
    // Give Runner time to cancel its pexpect-owned subprocess before the
    // final group reap. Ordinary shells still terminate immediately.
    if !matches!(
        &result,
        Ok(ExecOutput {
            end: ExecEnd::Exited(_),
            ..
        })
    ) {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGTERM);
        }
        let grace = Instant::now() + Duration::from_secs(4);
        while Instant::now() < grace {
            if child.try_wait().ok().flatten().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    // Includes descendants that inherited pipes, on success/error/timeout/cancel.
    #[cfg(unix)]
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.kill();
    let _ = child.wait();
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drains_both_streams_bounds_output_and_reaps_timeout() {
        let mut executor = LocalExecutor;
        let out = executor
            .execute(
                "printf 'hello\\n'; printf 'error\\n' >&2; exit 7",
                5,
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(out.end, ExecEnd::Exited(7));
        assert!(String::from_utf8_lossy(&out.output).contains("error"));
        let out = executor
            .execute(
                "yes x | head -c 262144; yes e | head -c 262144 >&2",
                5,
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(out.end, ExecEnd::Exited(0));
        assert!(out.truncated);
        assert_eq!(out.output.len(), OUTPUT_LIMIT);
        let start = Instant::now();
        let out = executor
            .execute("sleep 30 & wait", 1, &Cancellation::default())
            .unwrap();
        assert_eq!(out.end, ExecEnd::TimedOut);
        assert!(start.elapsed() < Duration::from_secs(3));
        let cancel = Cancellation::default();
        let signal = cancel.clone();
        let worker =
            std::thread::spawn(move || LocalExecutor.execute("sleep 30", 60, &signal).unwrap());
        std::thread::sleep(Duration::from_millis(30));
        cancel.cancel();
        assert_eq!(worker.join().unwrap().end, ExecEnd::Cancelled);
    }
}
