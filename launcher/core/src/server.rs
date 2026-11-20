use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use crate::util::{e, hide, is_running, pids, port_open, quiet};

fn wait_port(port: u16, seconds: u64) -> Result<(), String> {
    let end = Instant::now() + Duration::from_secs(seconds);
    while Instant::now() < end {
        if port_open(port) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    Err(format!("Nothing answered on port {} in time.", port))
}

/// Starts MySQL if needed (first time: prepares the data folder and creates the acore user and databases).
pub fn ensure_mysql(root: &Path) -> Result<(), String> {
    let mysql_dir = root.join("mysql");
    let mysqld = mysql_dir.join("bin").join("mysqld.exe");
    if !mysqld.exists() {
        return Err("MySQL is not installed yet. Press Install first.".into());
    }
    crate::setup::copy_vc_runtime(root)?;
    if !mysql_dir.join("data").exists() {
        let mut c = Command::new(&mysqld);
        c.args(["--initialize-insecure", "--console"]).current_dir(&mysql_dir);
        quiet(&mut c);
        let st = c.status().map_err(e)?;
        if !st.success() {
            return Err("Preparing the database failed.".into());
        }
    }
    if !port_open(3306) {
        let mut c = Command::new(&mysqld);
        c.args(["--console", "--bind-address=127.0.0.1"]).current_dir(&mysql_dir);
        quiet(&mut c);
        c.spawn().map_err(e)?;
        wait_port(3306, 90).map_err(|_| "MySQL did not start in time.".to_string())?;
    }
    let found = crate::accounts::sql(root, "SHOW DATABASES LIKE 'acore_world'")?;
    if found.trim().is_empty() {
        let script = root.join("sql-source").join("data").join("sql").join("create").join("create_mysql.sql");
        let f = File::open(&script).map_err(|x| format!("{}: {}", script.display(), x))?;
        let mut c = Command::new(mysql_dir.join("bin").join("mysql.exe"));
        c.args(["-u", "root", "--skip-password"]).stdin(Stdio::from(f)).stdout(Stdio::null()).stderr(Stdio::null());
        hide(&mut c);
        c.status().map_err(e)?;
        let again = crate::accounts::sql(root, "SHOW DATABASES LIKE 'acore_world'")?;
        if again.trim().is_empty() {
            return Err("Creating the databases failed.".into());
        }
    }
    Ok(())
}

fn world_log_ready(root: &Path, since: SystemTime) -> bool {
    let log = root.join("Server.log");
    let fresh = fs::metadata(&log).and_then(|m| m.modified()).map(|t| t >= since).unwrap_or(false);
    if !fresh {
        return false;
    }
    match fs::read(&log) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).contains("World Initialized In"),
        Err(_) => false,
    }
}

/// Starts the login server and the world server hidden and waits until the world has finished loading.
pub fn start_realm(root: &Path, note: &dyn Fn(&str)) -> Result<(), String> {
    ensure_mysql(root)?;
    if !root.join("configs").join("worldserver.conf").exists() {
        crate::setup::prepare_config(root)?;
    }
    if !is_running("authserver.exe") {
        note("Starting the login server ...");
        let mut c = Command::new(root.join("authserver.exe"));
        c.current_dir(root);
        quiet(&mut c);
        c.spawn().map_err(|x| format!("authserver.exe: {}", x))?;
        std::thread::sleep(Duration::from_secs(8));
    }
    let was_running = is_running("worldserver.exe");
    let since = SystemTime::now() - Duration::from_secs(2);
    if !was_running {
        note("Starting the world server ...");
        let mut c = Command::new(root.join("worldserver.exe"));
        c.current_dir(root);
        quiet(&mut c);
        c.spawn().map_err(|x| format!("worldserver.exe: {}", x))?;
    }
    note("Waiting for the world to load (the very first start takes several minutes) ...");
    let end = Instant::now() + Duration::from_secs(1500);
    while Instant::now() < end {
        if !is_running("worldserver.exe") {
            return Err("The world server stopped unexpectedly. Look at Server.log and Errors.log in the server folder.".into());
        }
        if (was_running || world_log_ready(root, since)) && port_open(8085) && port_open(3724) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(3));
    }
    Err("The server did not become ready in time.".into())
}

fn send_ctrl_c(root: &Path, pid: u32) -> Result<(), String> {
    let helper = root.join("send-ctrl-c.ps1");
    if !helper.exists() {
        return Err("send-ctrl-c.ps1 is missing in the server folder.".into());
    }
    let mut c = Command::new("powershell.exe");
    c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]).arg(&helper).arg("-ProcessId").arg(pid.to_string());
    c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    hide(&mut c);
    c.status().map_err(e)?;
    Ok(())
}

/// Clean stop of the world server, then the login server (same method as the PowerShell launcher).
pub fn stop_realm(root: &Path) -> Result<(), String> {
    for image in ["worldserver.exe", "authserver.exe"] {
        for pid in pids(image) {
            send_ctrl_c(root, pid)?;
        }
        let end = Instant::now() + Duration::from_secs(90);
        while is_running(image) && Instant::now() < end {
            std::thread::sleep(Duration::from_secs(1));
        }
        if is_running(image) {
            return Err(format!("{} did not stop within 90 seconds. It is still running.", image));
        }
    }
    Ok(())
}

pub fn stop_mysql(root: &Path) {
    if !port_open(3306) {
        return;
    }
    let admin = root.join("mysql").join("bin").join("mysqladmin.exe");
    let mut c = Command::new(admin);
    c.args(["-u", "root", "--skip-password", "shutdown"]);
    c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    hide(&mut c);
    let _ = c.status();
    let end = Instant::now() + Duration::from_secs(60);
    while port_open(3306) && Instant::now() < end {
        std::thread::sleep(Duration::from_secs(1));
    }
}
