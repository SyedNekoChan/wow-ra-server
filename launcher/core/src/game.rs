use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::net::sha256_file;
use crate::util::{e, hide, is_running};

pub const PATCH_NAME: &str = "patch-enUS-4.MPQ";
const BACKUP_SUFFIX: &str = ".launcher-backup"; // same suffix as the PowerShell launcher, so its undo still works

pub fn validate_client(dir: &Path) -> Result<(), String> {
    if !dir.join("Wow.exe").exists() {
        return Err("I cannot find Wow.exe in that folder. Pick the folder that holds Wow.exe and the Data folder.".into());
    }
    if !dir.join("Data").join("enUS").is_dir() {
        return Err("There is no Data\\enUS folder in there. This realm needs an English (enUS) 3.3.5a client.".into());
    }
    Ok(())
}

fn backup_once(file: &Path) -> Result<(), String> {
    let backup = format!("{}{}", file.display(), BACKUP_SUFFIX);
    if file.exists() && !Path::new(&backup).exists() {
        fs::copy(file, &backup).map_err(e)?;
    }
    Ok(())
}

/// Copies our patch into Data\enUS (only when it differs) and points realmlist.wtf at the local server.
/// The original files are backed up once (*.launcher-backup).
pub fn install_patch(root: &Path, client: &Path) -> Result<(), String> {
    validate_client(client)?;
    if is_running("Wow.exe") {
        return Err("The game is running. Close it first.".into());
    }
    let src = root.join("patch").join(PATCH_NAME);
    if !src.exists() {
        return Err(format!("Our patch is missing from the server folder ({}). Press Install or Update.", src.display()));
    }
    let dest_dir = client.join("Data").join("enUS");
    let dest = dest_dir.join(PATCH_NAME);
    let same = dest.exists() && sha256_file(&src)? == sha256_file(&dest)?;
    if !same {
        backup_once(&dest)?;
        fs::copy(&src, &dest).map_err(|x| format!("Could not copy the patch: {}", x))?;
    }
    let realm = dest_dir.join("realmlist.wtf");
    let wanted = "set realmlist 127.0.0.1\r\n";
    if fs::read_to_string(&realm).map(|t| t != wanted).unwrap_or(true) {
        backup_once(&realm)?;
        fs::write(&realm, wanted).map_err(e)?;
    }
    Ok(())
}

const AUTOLOGIN_PS1: &str = r#"param([int]$ProcessId,[string]$User,[int]$Delay)
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName Microsoft.VisualBasic
function Esc([string]$s){ $o=''; foreach($c in $s.ToCharArray()){ if('+^%~(){}[]'.Contains([string]$c)){ $o+='{'+$c+'}' } else { $o+=[string]$c } }; return $o }
$pw = $env:RA_LAUNCHER_PW
$end = (Get-Date).AddSeconds(90)
while ((Get-Date) -lt $end) {
  $p = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
  if (-not $p) { exit 1 }
  if ($p.MainWindowHandle -ne 0) { break }
  Start-Sleep -Milliseconds 500
}
Start-Sleep -Seconds $Delay
[Microsoft.VisualBasic.Interaction]::AppActivate($ProcessId)
Start-Sleep -Milliseconds 600
[System.Windows.Forms.SendKeys]::SendWait((Esc $User))
Start-Sleep -Milliseconds 200
[System.Windows.Forms.SendKeys]::SendWait('{TAB}')
Start-Sleep -Milliseconds 200
[System.Windows.Forms.SendKeys]::SendWait((Esc $pw))
Start-Sleep -Milliseconds 200
[System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
"#;

/// Opens Wow.exe. With auto-login, a hidden helper types the account name and password into the login screen
/// (experimental: the 3.3.5a client has no login command line). The password goes through an environment variable, never a file.
pub fn launch_game(client: &Path, user: &str, pass: &str, autologin: bool, delay: u64) -> Result<Child, String> {
    let child = Command::new(client.join("Wow.exe")).current_dir(client).spawn().map_err(|x| format!("Could not start Wow.exe: {}", x))?;
    if autologin {
        let script = std::env::temp_dir().join("ra-realm-autologin.ps1");
        fs::write(&script, AUTOLOGIN_PS1).map_err(e)?;
        let mut c = Command::new("powershell.exe");
        c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(&script)
            .arg("-ProcessId")
            .arg(child.id().to_string())
            .arg("-User")
            .arg(user)
            .arg("-Delay")
            .arg(delay.to_string())
            .env("RA_LAUNCHER_PW", pass);
        c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        hide(&mut c);
        let _ = c.spawn();
    }
    Ok(child)
}
