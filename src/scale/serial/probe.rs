use std::io::{ErrorKind, Read};
use std::time::{Duration, Instant};

use super::detect::{ProbeOutcome, ScaleProbe};
use crate::scale::{append_raw, parse_weight};

#[derive(Debug, Clone)]
pub struct SerialPortProbe {
    timeout: Duration,
    default_unit: String,
}

impl SerialPortProbe {
    pub fn new(timeout: Duration, default_unit: &str) -> Self {
        Self {
            timeout,
            default_unit: default_unit.to_string(),
        }
    }

    fn probe_serial(&self, device: &str, baud: u32) -> Result<ProbeOutcome, String> {
        let timeout = self.timeout;
        let port = serialport::new(device, baud)
            .timeout(timeout)
            .open()
            .map_err(|err| err.to_string());

        match port {
            Ok(mut port) => self.probe_reader(&mut port),
            Err(err) if should_try_unix_pty_fallback(&err) => self.probe_unix_pty(device),
            Err(err) => Err(err),
        }
    }

    fn probe_reader(&self, reader: &mut impl Read) -> Result<ProbeOutcome, String> {
        let timeout = self.timeout;
        let deadline = Instant::now() + timeout;
        let mut buf = [0_u8; 256];
        let mut raw = String::new();
        let mut has_data = false;

        while Instant::now() < deadline {
            match reader.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    has_data = true;
                    raw = append_raw(&raw, &String::from_utf8_lossy(&buf[..n]), 240);
                    if parse_weight(&raw, &self.default_unit).is_some() {
                        return Ok(ProbeOutcome::parsed_weight());
                    }
                }
                Err(err) if err.kind() == ErrorKind::TimedOut => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(err) => return Err(err.to_string()),
            }
        }

        if has_data {
            Ok(ProbeOutcome::data())
        } else {
            Ok(ProbeOutcome::empty())
        }
    }

    #[cfg(unix)]
    fn probe_unix_pty(&self, device: &str) -> Result<ProbeOutcome, String> {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(device)
            .map_err(|err| err.to_string())?;

        self.probe_reader(&mut file)
    }

    #[cfg(not(unix))]
    fn probe_unix_pty(&self, _device: &str) -> Result<ProbeOutcome, String> {
        Ok(ProbeOutcome::empty())
    }
}

impl ScaleProbe for SerialPortProbe {
    fn probe(&mut self, device: &str, baud: u32) -> Result<ProbeOutcome, String> {
        self.probe_serial(device, baud)
    }
}

fn should_try_unix_pty_fallback(err: &str) -> bool {
    let err = err.to_ascii_lowercase();
    err.contains("not a typewriter") || err.contains("inappropriate ioctl for device")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{SerialPortProbe, should_try_unix_pty_fallback};

    #[test]
    fn stores_timeout_and_default_unit() {
        let probe = SerialPortProbe::new(Duration::from_millis(800), "kg");

        assert_eq!(probe.timeout, Duration::from_millis(800));
        assert_eq!(probe.default_unit, "kg");
    }

    #[test]
    fn detects_unix_pty_fallback_errors() {
        assert!(should_try_unix_pty_fallback("Not a typewriter"));
        assert!(should_try_unix_pty_fallback(
            "inappropriate ioctl for device"
        ));
        assert!(!should_try_unix_pty_fallback("permission denied"));
    }
}
