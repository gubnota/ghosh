//! Non-PTY exec. Each invocation owns one channel; both streams are drained even
//! after the capture limit. Closing SSH does not prove remote process termination.
use super::*;
use ghosh_core::runs::{ExecEnd, ExecOutput, Executor, bounded_append};
impl Executor for Connection {
    fn execute(
        &mut self,
        script: &str,
        timeout_s: u32,
        cancel: &Cancellation,
    ) -> Result<ExecOutput> {
        self.execute_streaming(script, timeout_s, cancel, &mut |_| {})
    }
    fn execute_streaming(
        &mut self,
        script: &str,
        timeout_s: u32,
        cancel: &Cancellation,
        on_output: &mut dyn FnMut(&[u8]),
    ) -> Result<ExecOutput> {
        cancel.check()?;
        if !self.reviewed
            || !self.session.authenticated()
            || !(1..=3600).contains(&timeout_s)
            || script.len() > 2 * 1024 * 1024
        {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let deadline = Instant::now() + Duration::from_secs(timeout_s.into());
        self.session.set_blocking(true);
        let mut channel = self.session.channel_session().map_err(|_| failed())?;
        channel.exec("sh -s").map_err(|_| failed())?;
        self.session.set_blocking(false);
        let mut offset = 0;
        let mut eof_sent = false;
        let mut output = vec![];
        let mut truncated = false;
        let end = loop {
            if cancel.check().is_err() {
                break ExecEnd::Cancelled;
            }
            if Instant::now() >= deadline {
                break ExecEnd::TimedOut;
            }
            if offset < script.len() {
                match channel.write(&script.as_bytes()[offset..]) {
                    Ok(0) => break ExecEnd::Disconnected,
                    Ok(length) => offset += length,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                    Err(_) => break ExecEnd::Disconnected,
                }
            } else if !eof_sent {
                match channel.send_eof() {
                    Ok(()) => eof_sent = true,
                    Err(e) if again(&e) => (),
                    Err(_) => break ExecEnd::Disconnected,
                }
            }
            let mut read_any = false;
            let mut broken = false;
            // A busy stdout must not starve stderr or cancellation.
            for stream in [0, 1] {
                let mut bytes = [0; 8192];
                match channel.stream(stream).read(&mut bytes) {
                    Ok(length) => {
                        read_any |= length > 0;
                        if length > 0 {
                            on_output(&bytes[..length]);
                        }
                        bounded_append(&mut output, &bytes[..length], &mut truncated);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                    Err(_) => broken = true,
                }
            }
            if broken {
                break ExecEnd::Disconnected;
            }
            if !read_any && channel.eof() {
                match channel.wait_close() {
                    Ok(()) => {
                        break channel
                            .exit_status()
                            .map(ExecEnd::Exited)
                            .unwrap_or(ExecEnd::Disconnected);
                    }
                    Err(e) if again(&e) => (),
                    Err(_) => break ExecEnd::Disconnected,
                }
            }
            if !read_any {
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        if !matches!(end, ExecEnd::Exited(_)) {
            let _ = channel.close();
            let _ = self
                .session
                .disconnect(None, "Ghosh execution stopped; outcome unknown", None);
        }
        Ok(ExecOutput {
            end,
            output,
            truncated,
        })
    }
}
