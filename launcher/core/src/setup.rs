use std::fs;
use std::path::{Path, PathBuf};

use crate::net::{self, ServerRelease};
use crate::util::{e, is_running, norm};
use crate::{Emit, Progress};

// Official places, the same ones proven by hand (1-get-files.bat and launcher option 9).
pub const MYSQL_URL: &str = "https://cdn.mysql.com/archives/mysql-8.4/mysql-8.4.9-winx64.zip";
pub const DATA_URL: &str = "https://github.com/wowgaming/client-data/releases/download/v20.0/Data.zip";
pub const DATA_SHA256: &str = "a3d4df635ae6c2c8f08052c32a79e0f806955150ad36b014a823dd08a32a4610";

pub fn package_present(root: &Path) -> bool {
    root.join("authserver.exe").exists() && root.join("worldserver.exe").exists()
}

pub fn is_setup_done(root: &Path) -> bool {
    package_present(root) && root.join("mysql").join("bin").join("mysqld.exe").exists() && root.join("Data").join("maps").exists()
}

pub fn installed_version(root: &Path) -> Option<String> {
    fs::read_to_string(root.join("version.txt")).ok().map(|t| t.trim().to_string()).filter(|t| !t.is_empty())
}

/// MySQL needs the Visual C++ runtime files (shipped in our package) next to mysqld.exe.
pub fn copy_vc_runtime(root: &Path) -> Result<(), String> {
    let bin = root.join("mysql").join("bin");
    if bin.join("vcruntime140_1.dll").exists() {
        return Ok(());
    }
    let mut copied = 0;
    for entry in fs::read_dir(root).map_err(e)? {
        let entry = entry.map_err(e)?;
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.contains("140") && name.ends_with(".dll") {
            fs::copy(entry.path(), bin.join(entry.file_name())).map_err(e)?;
            copied += 1;
        }
    }
    if copied == 0 {
        return Err("The Visual C++ runtime files (*140*.dll) are missing from the server folder. Press Install to fetch the server package.".into());
    }
    Ok(())
}

/// Creates worldserver.conf and authserver.conf from the .dist files (if missing) and sets the paths
/// this package needs. Same keys as the PowerShell launcher (verified in the .dist files earlier).
pub fn prepare_config(root: &Path) -> Result<(), String> {
    let data_dir = norm(&root.join("Data"));
    let source_dir = norm(&root.join("sql-source"));
    let mysql_exe = norm(&root.join("mysql").join("bin").join("mysql.exe"));
    for name in ["worldserver", "authserver"] {
        let conf = root.join("configs").join(format!("{}.conf", name));
        let dist = root.join("configs").join(format!("{}.conf.dist", name));
        if !conf.exists() {
            fs::copy(&dist, &conf).map_err(|x| format!("{}: {}", dist.display(), x))?;
        }
        let text = fs::read_to_string(&conf).map_err(e)?;
        let mut settings = vec![("SourceDirectory", source_dir.clone()), ("MySQLExecutable", mysql_exe.clone())];
        if name == "worldserver" {
            settings.push(("DataDir", data_dir.clone()));
        }
        fs::write(&conf, set_keys(&text, &settings)).map_err(e)?;
    }
    Ok(())
}

/// Replaces lines that start with `Key =` by `Key = "value"`; everything else stays as it was.
pub fn set_keys(text: &str, settings: &[(&str, String)]) -> String {
    let mut out = String::with_capacity(text.len() + 64);
    for line in text.split_inclusive('\n') {
        let eol = if line.ends_with("\r\n") { "\r\n" } else if line.ends_with('\n') { "\n" } else { "" };
        let mut replaced = false;
        for (key, value) in settings {
            if let Some(rest) = line.strip_prefix(key) {
                if rest.trim_start_matches([' ', '\t']).starts_with('=') {
                    out.push_str(&format!("{} = \"{}\"{}", key, value, eol));
                    replaced = true;
                    break;
                }
            }
        }
        if !replaced {
            out.push_str(line);
        }
    }
    out
}

fn move_inner_dir(tmp: &Path, final_dir: &Path, marker: &str) -> Result<(), String> {
    let inner = fs::read_dir(tmp)
        .map_err(e)?
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .find(|p| p.is_dir() && p.join(marker).exists());
    match inner {
        Some(p) => fs::rename(p, final_dir).map_err(e),
        None => Err("The zip has a layout I did not expect.".into()),
    }
}

fn clear_temp(p: &Path) {
    // Only our own temporary unpack folders (mysql-extract, data-extract) are ever removed.
    if p.exists() {
        let _ = fs::remove_dir_all(p);
    }
}

/// Download and unpack our server package (programs, DLLs, configs, sql, patch, launcher scripts).
/// Never touches mysql\, Data\ or the .conf files you already have.
pub fn install_package(root: &Path, rel: &ServerRelease, emit: Emit, base: f32, weight: f32) -> Result<(), String> {
    let zip = root.join("downloads").join("package.zip");
    if rel.sha256.is_none() {
        let _ = fs::remove_file(&zip); // no hash to compare: never trust an older copy
    }
    let want = rel.sha256.clone();
    let good = move |p: &Path| match &want {
        Some(h) => net::sha256_file(p).map(|x| x.eq_ignore_ascii_case(h)).unwrap_or(false),
        None => net::zip_ok(p),
    };
    net::download(&rel.url, &zip, "package", &format!("Server package {}", rel.version), base, weight * 0.7, emit, &good)?;
    let skip = |p: &str| {
        let low = p.to_lowercase();
        low.starts_with("mysql/") || low.starts_with("data/") || (low.starts_with("configs/") && low.ends_with(".conf")) || low == "launcher-config.txt"
    };
    net::extract_zip(&zip, root, "package", &format!("Unpacking server package {}", rel.version), base + weight * 0.7, weight * 0.3, emit, &skip)?;
    fs::write(root.join("version.txt"), &rel.version).map_err(e)?;
    Ok(())
}

pub fn run_setup(root: &Path, emit: Emit, install_patch_to: Option<&Path>) -> Result<(), String> {
    fs::create_dir_all(root).map_err(e)?;
    if is_running("worldserver.exe") || is_running("authserver.exe") {
        return Err("The server is running. Close the game first.".into());
    }
    let mysql_dir = root.join("mysql");
    let data_dir = root.join("Data");
    if mysql_dir.exists() && !mysql_dir.join("bin").join("mysqld.exe").exists() {
        return Err("A folder named mysql exists in the server folder but has no bin\\mysqld.exe. Rename or move it, then press Install again.".into());
    }
    if data_dir.exists() && !data_dir.join("maps").exists() {
        return Err("A folder named Data exists in the server folder but has no maps folder. Rename or move it, then press Install again.".into());
    }

    // 1. our server package from the newest GitHub Release
    if !package_present(root) {
        emit(Progress::new("package", "Server package", 0, 0, 0.0, "Looking for the newest server release on GitHub ..."));
        let rel = net::latest_server_release()?.ok_or("There is no server release on GitHub yet (a release with a tag starting with server-v).")?;
        install_package(root, &rel, emit, 0.0, 6.0)?;
    }

    // 2. MySQL
    if !mysql_dir.join("bin").join("mysqld.exe").exists() {
        let zip = root.join("downloads").join("mysql.zip");
        net::download(MYSQL_URL, &zip, "mysql", "MySQL 8.4.9", 6.0, 10.0, emit, &|p| net::zip_ok(p))?;
        let tmp = root.join("mysql-extract");
        clear_temp(&tmp);
        net::extract_zip(&zip, &tmp, "mysql", "Unpacking MySQL", 16.0, 4.0, emit, &|_| false)?;
        move_inner_dir(&tmp, &mysql_dir, "bin/mysqld.exe")?;
        clear_temp(&tmp);
    }

    // 3. game data (Data v20, checked by sha256)
    if !data_dir.join("maps").exists() {
        let zip = root.join("downloads").join("Data.zip");
        net::download(DATA_URL, &zip, "data", "Game data (Data v20)", 20.0, 48.0, emit, &|p| {
            net::sha256_file(p).map(|h| h.eq_ignore_ascii_case(DATA_SHA256)).unwrap_or(false)
        })?;
        let tmp = root.join("data-extract");
        clear_temp(&tmp);
        net::extract_zip(&zip, &tmp, "data", "Unpacking game data", 68.0, 20.0, emit, &|_| false)?;
        if tmp.join("Data").join("maps").exists() {
            fs::rename(tmp.join("Data"), &data_dir).map_err(e)?;
        } else if tmp.join("maps").exists() {
            fs::rename(&tmp, &data_dir).map_err(e)?;
        } else {
            return Err("The game data zip has a layout I did not expect.".into());
        }
        clear_temp(&tmp);
    }

    // 4. runtime files, configs
    emit(Progress::new("config", "Preparing the server", 0, 0, 88.0, "Copying runtime files and preparing the config files ..."));
    copy_vc_runtime(root)?;
    prepare_config(root)?;

    // 5. first start builds the databases (and applies our SQL)
    first_start_if_needed(root, emit, 88.0)?;

    // 6. the client patch
    if let Some(client) = install_patch_to {
        emit(Progress::new("patch", "Client patch", 0, 0, 99.0, "Installing our patch into your game folder ..."));
        crate::game::install_patch(root, client)?;
    }
    emit(Progress::new("done", "Ready", 1, 1, 100.0, "Everything is installed."));
    Ok(())
}

/// The databases are built by the servers on their first start (about 5 minutes). Do it once, here.
pub fn first_start_if_needed(root: &Path, emit: Emit, base: f32) -> Result<(), String> {
    crate::server::ensure_mysql(root)?;
    if crate::accounts::db_ready(root) {
        return Ok(());
    }
    let note = |m: &str| emit(Progress::new("firststart", "Building the realm database", 0, 0, base + 5.0, m));
    note("First start: building the realm database. This takes about 5 minutes, once ...");
    crate::server::start_realm(root, &note)?;
    note("Database built. Stopping the server ...");
    crate::server::stop_realm(root)?;
    if !crate::accounts::db_ready(root) {
        return Err("The databases still look empty after the first start. Look at Errors.log in the server folder.".into());
    }
    Ok(())
}

#[allow(dead_code)]
pub fn dl_dir(root: &Path) -> PathBuf {
    root.join("downloads")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_lines_are_replaced_and_endings_kept() {
        let text = "# comment\r\nDataDir = \".\"\r\nSourceDirectory = \"\"\r\nOther = 1\r\n";
        let out = set_keys(text, &[("DataDir", "D:/x/Data".to_string()), ("SourceDirectory", "D:/x/sql".to_string())]);
        assert_eq!(out, "# comment\r\nDataDir = \"D:/x/Data\"\r\nSourceDirectory = \"D:/x/sql\"\r\nOther = 1\r\n");
    }
    #[test]
    fn longer_key_is_not_touched() {
        let out = set_keys("DataDirExtra = 1\n", &[("DataDir", "x".to_string())]);
        assert_eq!(out, "DataDirExtra = 1\n");
    }
}
