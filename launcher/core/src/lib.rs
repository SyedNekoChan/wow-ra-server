//! RA Realm Launcher: everything the launcher does, without any window code (so it can be tested on its own).

mod accounts;
mod game;
mod net;
mod server;
mod settings;
mod setup;
mod util;

use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

pub use accounts::{Character, LoginResult};
pub use settings::Settings;

pub const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Clone)]
pub struct Progress {
    pub stage: String,
    pub label: String,
    pub done: u64,
    pub total: u64,
    pub overall: f32,
    pub note: String,
}

impl Progress {
    pub fn new(stage: &str, label: &str, done: u64, total: u64, overall: f32, note: &str) -> Self {
        Progress { stage: stage.into(), label: label.into(), done, total, overall, note: note.into() }
    }
}

#[derive(Serialize, Clone)]
pub struct PlayEvent {
    pub phase: String,
    pub message: String,
}

pub type Emit<'a> = &'a dyn Fn(Progress);

#[derive(Serialize)]
pub struct Status {
    pub client_dir: Option<String>,
    pub server_dir: String,
    pub setup_done: bool,
    pub installed_version: Option<String>,
    pub autologin: bool,
    pub mysql_running: bool,
    pub server_running: bool,
    pub launcher_version: String,
}

#[derive(Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub installed: Option<String>,
    pub latest: Option<String>,
    pub size: u64,
    pub error: Option<String>,
}

fn root() -> PathBuf {
    Settings::load().server_root()
}

pub fn status() -> Status {
    let s = Settings::load();
    let r = s.server_root();
    Status {
        client_dir: s.client_dir.clone(),
        server_dir: r.display().to_string(),
        setup_done: setup::is_setup_done(&r),
        installed_version: setup::installed_version(&r),
        autologin: s.autologin,
        mysql_running: util::port_open(3306),
        server_running: util::is_running("worldserver.exe"),
        launcher_version: LAUNCHER_VERSION.into(),
    }
}

pub fn set_client_folder(path: &str) -> Result<(), String> {
    let p = PathBuf::from(path.trim().trim_matches('"'));
    game::validate_client(&p)?;
    let mut s = Settings::load();
    s.client_dir = Some(p.display().to_string());
    s.save()
}

pub fn set_server_folder(path: &str) -> Result<(), String> {
    let p = PathBuf::from(path.trim().trim_matches('"'));
    if !p.is_dir() {
        return Err("That folder does not exist.".into());
    }
    let mut s = Settings::load();
    s.server_dir = Some(p.display().to_string());
    s.save()
}

pub fn set_autologin(on: bool) -> Result<(), String> {
    let mut s = Settings::load();
    s.autologin = on;
    s.save()
}

pub fn check_update() -> UpdateInfo {
    let r = root();
    let installed = setup::installed_version(&r);
    match net::latest_server_release() {
        Ok(Some(rel)) => {
            let available = match &installed {
                Some(i) => util::newer(&rel.version, i),
                None => !setup::package_present(&r),
            };
            UpdateInfo { available, installed, latest: Some(rel.version), size: rel.size, error: None }
        }
        Ok(None) => UpdateInfo { available: false, installed, latest: None, size: 0, error: Some("No server release on GitHub yet.".into()) },
        Err(err) => UpdateInfo { available: false, installed, latest: None, size: 0, error: Some(err) },
    }
}

/// First-time install: server package, MySQL, game data, configs, database build, client patch.
pub fn run_setup(emit: Emit) -> Result<(), String> {
    let s = Settings::load();
    let client = s.client_dir.as_ref().map(PathBuf::from);
    setup::run_setup(&s.server_root(), emit, client.as_deref())
}

/// Update: newer server package (programs, SQL, patch). Keeps MySQL data, game data and your .conf files.
pub fn apply_update(emit: Emit) -> Result<(), String> {
    let s = Settings::load();
    let r = s.server_root();
    if util::is_running("worldserver.exe") || util::is_running("authserver.exe") {
        return Err("The server is running. Close the game first.".into());
    }
    emit(Progress::new("package", "Update", 0, 0, 0.0, "Looking for the newest server release ..."));
    let rel = net::latest_server_release()?.ok_or("No server release on GitHub yet.")?;
    setup::install_package(&r, &rel, emit, 0.0, 70.0)?;
    emit(Progress::new("config", "Update", 0, 0, 72.0, "Preparing config files ..."));
    setup::prepare_config(&r)?;
    // new SQL files are applied by the server's own updater at its next start: start once now so Play is quick
    setup::first_start_if_needed(&r, emit, 72.0)?;
    if let Some(c) = s.client_dir.as_ref() {
        emit(Progress::new("patch", "Update", 0, 0, 95.0, "Updating the client patch ..."));
        game::install_patch(&r, &PathBuf::from(c))?;
    }
    emit(Progress::new("done", "Ready", 1, 1, 100.0, "Updated."));
    Ok(())
}

pub fn login(user: &str, pass: &str) -> Result<LoginResult, String> {
    let r = root();
    server::ensure_mysql(&r)?;
    accounts::login(&r, user, pass)
}

pub fn characters(user: &str) -> Result<Vec<Character>, String> {
    let r = root();
    server::ensure_mysql(&r)?;
    accounts::characters(&r, &user.to_ascii_uppercase())
}

pub fn create_account(user: &str, pass: &str, gm: bool) -> Result<(), String> {
    let r = root();
    server::ensure_mysql(&r)?;
    accounts::create_account(&r, user, pass, gm)
}

pub fn delete_account(user: &str, pass: &str) -> Result<(), String> {
    let r = root();
    server::ensure_mysql(&r)?;
    accounts::delete_account(&r, user, pass)
}

/// Start the realm, open the game, wait until it closes, stop the realm. Reports each phase.
pub fn play(user: &str, pass: &str, ev: &dyn Fn(PlayEvent)) -> Result<(), String> {
    let s = Settings::load();
    let r = s.server_root();
    let client = PathBuf::from(s.client_dir.clone().ok_or("Choose your game folder first.")?);
    let say = |phase: &str, m: &str| ev(PlayEvent { phase: phase.into(), message: m.into() });
    if util::is_running("Wow.exe") {
        return Err("The game is already running. Close it first.".into());
    }
    // the same password check the login server will do
    server::ensure_mysql(&r)?;
    accounts::login(&r, user, pass)?;
    say("starting", "Updating your game files ...");
    game::install_patch(&r, &client)?;
    say("starting", "Starting the realm ...");
    server::start_realm(&r, &|m| say("starting", m))?;
    say("ready", "The realm is online. Opening the game ...");
    let mut child = game::launch_game(&client, user, pass, s.autologin, s.autologin_delay)?;
    say("ingame", "In game. Closing the game stops the realm.");
    let _ = child.wait();
    say("stopping", "The game closed. Saving and stopping the realm ...");
    std::thread::sleep(Duration::from_secs(5));
    let stop = server::stop_realm(&r);
    say("stopped", "Realm stopped.");
    stop
}

pub fn stop_realm() -> Result<(), String> {
    server::stop_realm(&root())
}

/// Used when the launcher window closes: stop the realm if it runs, then MySQL.
pub fn shutdown_all() {
    let r = root();
    let _ = server::stop_realm(&r);
    server::stop_mysql(&r);
}
