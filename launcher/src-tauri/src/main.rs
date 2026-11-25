#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

use launcher_core as core;

// Only one long job (install, update, play) at a time.
static BUSY: AtomicBool = AtomicBool::new(false);

struct BusyGuard;
impl BusyGuard {
    fn take() -> Result<BusyGuard, String> {
        if BUSY.swap(true, Ordering::SeqCst) {
            Err("Another job is already running. Wait for it to finish.".into())
        } else {
            Ok(BusyGuard)
        }
    }
}
impl Drop for BusyGuard {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::SeqCst);
    }
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|x| x.to_string())?
}

#[tauri::command]
fn get_status() -> core::Status {
    core::status()
}

#[tauri::command]
fn set_client_folder(path: String) -> Result<core::Status, String> {
    core::set_client_folder(&path)?;
    Ok(core::status())
}

#[tauri::command]
fn set_server_folder(path: String) -> Result<core::Status, String> {
    core::set_server_folder(&path)?;
    Ok(core::status())
}

#[tauri::command]
async fn check_update() -> Result<core::UpdateInfo, String> {
    blocking(|| Ok(core::check_update())).await
}

#[tauri::command]
async fn run_setup(app: tauri::AppHandle) -> Result<(), String> {
    let guard = BusyGuard::take()?;
    let r = blocking(move || {
        let _g = guard;
        core::run_setup(&|p| {
            let _ = app.emit("progress", p);
        })
    })
    .await;
    r
}

#[tauri::command]
async fn apply_update(app: tauri::AppHandle) -> Result<(), String> {
    let guard = BusyGuard::take()?;
    blocking(move || {
        let _g = guard;
        core::apply_update(&|p| {
            let _ = app.emit("progress", p);
        })
    })
    .await
}

#[tauri::command]
async fn create_account(user: String, password: String, gm: bool) -> Result<(), String> {
    blocking(move || core::create_account(&user, &password, gm)).await
}

#[tauri::command]
async fn delete_account(user: String, password: String) -> Result<(), String> {
    blocking(move || core::delete_account(&user, &password)).await
}

#[tauri::command]
async fn play(app: tauri::AppHandle) -> Result<(), String> {
    let guard = BusyGuard::take()?;
    blocking(move || {
        let _g = guard;
        core::play(&|ev| {
            let _ = app.emit("play", ev);
        })
    })
    .await
}

#[tauri::command]
async fn stop_realm() -> Result<(), String> {
    blocking(core::stop_realm).await
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_status,
            set_client_folder,
            set_server_folder,
            check_update,
            run_setup,
            apply_update,
            create_account,
            delete_account,
            play,
            stop_realm
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let handle = window.app_handle().clone();
                std::thread::spawn(move || {
                    core::shutdown_all();
                    handle.exit(0);
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running the launcher");
}
