use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn e<E: std::fmt::Display>(x: E) -> String {
    x.to_string()
}

/// Run a child program without showing a console window (Windows only).
pub fn hide(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}

/// Hidden, and with no input or output attached (for the long-running servers).
pub fn quiet(cmd: &mut Command) {
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    hide(cmd);
}

pub fn port_open(port: u16) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok()
}

/// Process ids of a running program, by image name (for example "worldserver.exe").
pub fn pids(image: &str) -> Vec<u32> {
    let mut c = Command::new("tasklist");
    c.arg("/FI").arg(format!("IMAGENAME eq {}", image)).arg("/FO").arg("CSV").arg("/NH");
    hide(&mut c);
    let out = match c.output() {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.split("\",\"").collect();
            if parts.len() >= 2 && parts[0].trim_matches('"').eq_ignore_ascii_case(image) {
                parts[1].trim_matches('"').parse().ok()
            } else {
                None
            }
        })
        .collect()
}

pub fn is_running(image: &str) -> bool {
    !pids(image).is_empty()
}

/// Path with forward slashes (the server config files want that).
pub fn norm(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// Compare dotted numbers: is `a` newer than `b`? ("0.2.0" vs "0.1.9")
pub fn newer(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> { s.trim().trim_start_matches('v').split('.').map(|p| p.trim().parse().unwrap_or(0)).collect() };
    let (x, y) = (parse(a), parse(b));
    for i in 0..x.len().max(y.len()) {
        let (p, q) = (*x.get(i).unwrap_or(&0), *y.get(i).unwrap_or(&0));
        if p != q {
            return p > q;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_compare() {
        assert!(newer("0.2.0", "0.1.9"));
        assert!(newer("1.0", "0.9.9"));
        assert!(!newer("0.1.0", "0.1.0"));
        assert!(!newer("0.1.0", "0.1.1"));
    }
}
