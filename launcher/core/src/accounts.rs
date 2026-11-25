use num_bigint::BigUint;
use rand::RngCore;
use sha1::{Digest, Sha1};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::util::{e, hide};

/// SRP6 verifier exactly as AzerothCore makes it (checked against its SRP6.cpp and AccountMgr.cpp):
/// v = 7 ^ H(salt || H(UPPER(user) ":" UPPER(pass))) mod N, numbers little-endian, 32 bytes.
pub fn srp_verifier(user: &str, pass: &str, salt: &[u8; 32]) -> [u8; 32] {
    let inner = Sha1::digest(format!("{}:{}", user.to_ascii_uppercase(), pass.to_ascii_uppercase()).as_bytes());
    let mut h = Sha1::new();
    h.update(salt);
    h.update(inner);
    let x = BigUint::from_bytes_le(&h.finalize());
    let n = BigUint::parse_bytes(b"894B645E89E1535BBDAD5B8B290650530801B18EBFBF5E8FAB3C82872A3E9BB7", 16).unwrap();
    let v = BigUint::from(7u32).modpow(&x, &n);
    let bytes = v.to_bytes_le();
    let mut out = [0u8; 32];
    out[..bytes.len()].copy_from_slice(&bytes);
    out
}

pub fn valid_user(u: &str) -> Result<(), String> {
    if u.len() < 2 || u.len() > 16 || !u.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("The account name must be 2 to 16 letters or numbers (no spaces or symbols).".into());
    }
    Ok(())
}

pub fn valid_pass(p: &str) -> Result<(), String> {
    if p.is_empty() || p.len() > 16 || !p.chars().all(|c| c.is_ascii_graphic()) {
        return Err("The password must be 1 to 16 characters, plain letters, numbers or symbols, no spaces.".into());
    }
    Ok(())
}

/// Run one SQL statement through the proven mysql.exe (root, no password, local only). Returns the output lines.
pub fn sql(root: &Path, query: &str) -> Result<String, String> {
    let exe = root.join("mysql").join("bin").join("mysql.exe");
    let mut c = Command::new(exe);
    c.args(["-u", "root", "--skip-password", "-N", "-B", "-e", query]);
    c.stdin(Stdio::null());
    hide(&mut c);
    let out = c.output().map_err(|x| format!("Could not run mysql.exe: {}", x))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn tables_ready(root: &Path) -> Result<(), String> {
    let n = sql(
        root,
        "SELECT COUNT(*) FROM information_schema.tables WHERE (table_schema='acore_auth' AND table_name='account') OR (table_schema='acore_characters' AND table_name='characters')",
    )?;
    if n.trim() == "2" {
        Ok(())
    } else {
        Err("The realm database is not built yet. Press Install (it builds the database on its first start).".into())
    }
}

pub fn db_ready(root: &Path) -> bool {
    tables_ready(root).is_ok()
}

fn account_id(root: &Path, user: &str) -> Result<Option<u32>, String> {
    let r = sql(root, &format!("SELECT id FROM acore_auth.account WHERE username='{}'", user.to_ascii_uppercase()))?;
    Ok(r.trim().parse().ok())
}

pub fn create_account(root: &Path, user: &str, pass: &str, gm: bool) -> Result<(), String> {
    valid_user(user)?;
    valid_pass(pass)?;
    tables_ready(root)?;
    if account_id(root, user)?.is_some() {
        return Err("That account name already exists.".into());
    }
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let ver = srp_verifier(user, pass, &salt);
    let u = user.to_ascii_uppercase();
    sql(
        root,
        &format!("INSERT INTO acore_auth.account (username, salt, verifier) VALUES ('{}', X'{}', X'{}')", u, hex::encode(salt), hex::encode(ver)),
    )?;
    if gm {
        sql(
            root,
            &format!("INSERT INTO acore_auth.account_access (id, gmlevel, RealmID) SELECT id, 3, -1 FROM acore_auth.account WHERE username='{}'", u),
        )?;
    }
    Ok(())
}

/// Checks the password the same way the login server does. Returns the account id.
fn verify_password(root: &Path, user: &str, pass: &str) -> Result<u32, String> {
    valid_user(user)?;
    valid_pass(pass)?;
    tables_ready(root)?;
    let u = user.to_ascii_uppercase();
    let row = sql(root, &format!("SELECT id, HEX(salt), HEX(verifier) FROM acore_auth.account WHERE username='{}'", u))?;
    let parts: Vec<&str> = row.split('\t').collect();
    if parts.len() < 3 {
        return Err("Wrong account name or password.".into());
    }
    let salt_bytes = hex::decode(parts[1]).map_err(e)?;
    let mut salt = [0u8; 32];
    if salt_bytes.len() != 32 {
        return Err("Wrong account name or password.".into());
    }
    salt.copy_from_slice(&salt_bytes);
    let want = hex::encode(srp_verifier(user, pass, &salt));
    if !want.eq_ignore_ascii_case(parts[2]) {
        return Err("Wrong account name or password.".into());
    }
    parts[0].trim().parse().map_err(|_| "Wrong account name or password.".to_string())
}

fn character_count(root: &Path, account: u32) -> Result<u32, String> {
    let n = sql(root, &format!("SELECT COUNT(*) FROM acore_characters.characters WHERE account = {}", account))?;
    Ok(n.trim().parse().unwrap_or(0))
}

/// Deletes the account only when the password is right and it has no characters (so no orphan data is left behind).
pub fn delete_account(root: &Path, user: &str, pass: &str) -> Result<(), String> {
    let id = verify_password(root, user, pass)?;
    if character_count(root, id)? > 0 {
        return Err("This account still has characters. Delete them in the game first (character select screen), then delete the account.".into());
    }
    let u = user.to_ascii_uppercase();
    sql(root, &format!("DELETE FROM acore_auth.account_access WHERE id IN (SELECT id FROM (SELECT id FROM acore_auth.account WHERE username='{}') t)", u))?;
    sql(root, &format!("DELETE FROM acore_auth.account WHERE username='{}'", u))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verifier_is_stable_and_case_insensitive() {
        let salt = [7u8; 32];
        let a = srp_verifier("Test", "Secret1", &salt);
        let b = srp_verifier("TEST", "SECRET1", &salt);
        assert_eq!(a, b);
        assert_ne!(a, srp_verifier("TEST", "SECRET2", &salt));
        assert_ne!(a, [0u8; 32]);
    }
    #[test]
    fn name_checks() {
        assert!(valid_user("Ali9").is_ok());
        assert!(valid_user("a'b").is_err());
        assert!(valid_user("x").is_err());
        assert!(valid_pass("pass word").is_err());
        assert!(valid_pass("").is_err());
    }
}
