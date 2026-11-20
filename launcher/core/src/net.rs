use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::util::e;
use crate::{Emit, Progress};

pub const REPO: &str = "SyedNekoChan/wow-ra-server";
pub const SERVER_TAG_PREFIX: &str = "server-v";
pub const PACKAGE_ASSET: &str = "wow-ra-server-package.zip";
pub const PACKAGE_SHA_ASSET: &str = "wow-ra-server-package.zip.sha256";
pub const MAX_TRIES: u32 = 300;

pub fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (RA-Realm-Launcher)")
        .build()
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut f = File::open(path).map_err(e)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf).map_err(e)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

pub fn zip_ok(path: &Path) -> bool {
    match File::open(path) {
        Ok(f) => zip::ZipArchive::new(f).map(|a| a.len() > 0).unwrap_or(false),
        Err(_) => false,
    }
}

/// One download attempt. Resumes a partial file. Ok(()) when the whole file is on disk.
fn attempt(url: &str, dest: &Path, stage: &str, label: &str, base: f32, weight: f32, emit: Emit) -> Result<(), String> {
    let existing = fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    let mut req = agent().get(url);
    if existing > 0 {
        req = req.set("Range", &format!("bytes={}-", existing));
    }
    let resp = match req.call() {
        Ok(r) => r,
        Err(ureq::Error::Status(416, _)) => return Ok(()), // nothing left to fetch: the file is complete
        Err(err) => return Err(err.to_string()),
    };
    let status = resp.status();
    let remaining: u64 = resp.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
    let (mut file, mut done) = if status == 206 {
        (OpenOptions::new().append(true).open(dest).map_err(e)?, existing)
    } else {
        (File::create(dest).map_err(e)?, 0u64)
    };
    let total = if remaining > 0 { remaining + done } else { 0 };
    let mut reader = resp.into_reader();
    let mut buf = vec![0u8; 64 * 1024];
    let mut last = Instant::now();
    loop {
        let n = reader.read(&mut buf).map_err(e)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(e)?;
        done += n as u64;
        if last.elapsed() > Duration::from_millis(250) {
            last = Instant::now();
            let frac = if total > 0 { done as f32 / total as f32 } else { 0.0 };
            emit(Progress::new(stage, label, done, total, base + weight * frac, ""));
        }
    }
    file.flush().map_err(e)?;
    if total > 0 && done < total {
        return Err("the connection closed early".into());
    }
    Ok(())
}

/// Download with resume and retries (300 tries, 3 seconds apart), then check the file.
pub fn download(
    url: &str,
    dest: &Path,
    stage: &str,
    label: &str,
    base: f32,
    weight: f32,
    emit: Emit,
    is_good: &dyn Fn(&Path) -> bool,
) -> Result<(), String> {
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).map_err(e)?;
    }
    emit(Progress::new(stage, label, 0, 0, base, "Checking what is already downloaded ..."));
    if dest.exists() && is_good(dest) {
        emit(Progress::new(stage, label, 1, 1, base + weight, "Already downloaded and checked."));
        return Ok(());
    }
    let mut tries = 0u32;
    loop {
        match attempt(url, dest, stage, label, base, weight, emit) {
            Ok(()) => break,
            Err(err) => {
                tries += 1;
                if tries >= MAX_TRIES {
                    return Err(format!("Giving up on {} after {} attempts ({}). Run it again later: it continues where it stopped.", label, MAX_TRIES, err));
                }
                emit(Progress::new(stage, label, 0, 0, base, &format!("Connection dropped, retrying (attempt {} of {}) ...", tries, MAX_TRIES)));
                std::thread::sleep(Duration::from_secs(3));
            }
        }
    }
    emit(Progress::new(stage, label, 1, 1, base + weight, "Checking the download ..."));
    if is_good(dest) {
        return Ok(());
    }
    // Our own damaged download: remove it so the next try starts clean.
    let _ = fs::remove_file(dest);
    Err(format!("The downloaded file for {} failed its check, so I removed it. Press the button again to download it again.", label))
}

/// Unzip into `dest`, entry by entry, with progress. `skip` gets the path inside the zip.
pub fn extract_zip(
    zip_path: &Path,
    dest: &Path,
    stage: &str,
    label: &str,
    base: f32,
    weight: f32,
    emit: Emit,
    skip: &dyn Fn(&str) -> bool,
) -> Result<(), String> {
    let f = File::open(zip_path).map_err(e)?;
    let mut ar = zip::ZipArchive::new(f).map_err(e)?;
    let n = ar.len();
    let mut last = Instant::now();
    for i in 0..n {
        let mut entry = ar.by_index(i).map_err(e)?;
        let rel = match entry.enclosed_name() {
            Some(p) => p,
            None => continue,
        };
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let out = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(e)?;
            continue;
        }
        if skip(&rel_str) {
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(e)?;
        }
        let mut o = File::create(&out).map_err(|x| format!("{}: {}", rel_str, x))?;
        std::io::copy(&mut entry, &mut o).map_err(|x| format!("{}: {}", rel_str, x))?;
        if last.elapsed() > Duration::from_millis(250) {
            last = Instant::now();
            emit(Progress::new(stage, label, i as u64, n as u64, base + weight * (i as f32 / n as f32), ""));
        }
    }
    emit(Progress::new(stage, label, n as u64, n as u64, base + weight, ""));
    Ok(())
}

// ---------------- GitHub Releases ----------------

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GhAsset>,
}

#[derive(Clone)]
pub struct ServerRelease {
    pub version: String,
    pub url: String,
    pub size: u64,
    pub sha256: Option<String>,
}

/// The newest published release whose tag starts with "server-v" and that carries the package zip.
pub fn latest_server_release() -> Result<Option<ServerRelease>, String> {
    let api = format!("https://api.github.com/repos/{}/releases?per_page=30", REPO);
    let rels: Vec<GhRelease> = agent()
        .get(&api)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|x| format!("Could not reach GitHub: {}", x))?
        .into_json()
        .map_err(e)?;
    let mut best: Option<ServerRelease> = None;
    for r in rels {
        if r.draft || r.prerelease || !r.tag_name.starts_with(SERVER_TAG_PREFIX) {
            continue;
        }
        let version = r.tag_name[SERVER_TAG_PREFIX.len()..].to_string();
        let pkg = match r.assets.iter().find(|a| a.name == PACKAGE_ASSET) {
            Some(a) => a,
            None => continue,
        };
        let sha256 = r.assets.iter().find(|a| a.name == PACKAGE_SHA_ASSET).and_then(|a| {
            agent().get(&a.browser_download_url).call().ok().and_then(|resp| resp.into_string().ok()).and_then(|t| {
                let h: String = t.trim().chars().take(64).collect();
                if h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()) {
                    Some(h.to_lowercase())
                } else {
                    None
                }
            })
        });
        let cand = ServerRelease { version, url: pkg.browser_download_url.clone(), size: pkg.size, sha256 };
        best = match best {
            Some(b) if !crate::util::newer(&cand.version, &b.version) => Some(b),
            _ => Some(cand),
        };
    }
    Ok(best)
}
