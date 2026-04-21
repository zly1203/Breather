//! Installs (or keeps up to date) the Claude Code hook so Breather
//! receives Stop events. Called on every app startup — idempotent.
//!
//! Flow:
//!   1. If ~/.claude/ doesn't exist, Claude Code isn't installed. Skip.
//!   2. Write the current hook.sh to ~/.breather/hook.sh (overwrite).
//!   3. Ensure ~/.claude/settings.json has our Stop hook registered.
//!      Back up the file before modifying.

use std::fs;
use std::path::PathBuf;
use serde_json::{json, Value};

/// The hook script content, embedded at compile time.
const HOOK_SCRIPT: &str = include_str!("../../hook/hook.sh");

#[derive(Debug, PartialEq)]
pub enum InstallOutcome {
    /// Claude Code isn't installed; nothing to do.
    ClaudeNotFound,
    /// Hook was already registered correctly.
    AlreadyInstalled,
    /// We just installed or updated the hook.
    Installed,
    /// Something went wrong; don't block app startup.
    Error(String),
}

pub fn ensure_hook_installed() -> InstallOutcome {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return InstallOutcome::Error("no home dir".into()),
    };

    let claude_dir = home.join(".claude");
    if !claude_dir.exists() {
        return InstallOutcome::ClaudeNotFound;
    }

    let hook_path = home.join(".breather").join("hook.sh");
    if let Err(e) = write_hook_script(&hook_path) {
        return InstallOutcome::Error(format!("write hook: {}", e));
    }

    let settings_path = claude_dir.join("settings.json");
    match register_in_settings(&settings_path, &hook_path) {
        Ok(true) => InstallOutcome::Installed,
        Ok(false) => InstallOutcome::AlreadyInstalled,
        Err(e) => InstallOutcome::Error(format!("settings: {}", e)),
    }
}

fn write_hook_script(hook_path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = hook_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(hook_path, HOOK_SCRIPT)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(hook_path)?.permissions();
        p.set_mode(0o755);
        fs::set_permissions(hook_path, p)?;
    }
    Ok(())
}

/// Returns Ok(true) if settings.json was modified, Ok(false) if already correct.
fn register_in_settings(settings_path: &PathBuf, hook_path: &PathBuf) -> std::io::Result<bool> {
    let hook_cmd = hook_path.to_string_lossy().to_string();

    let mut settings: Value = if settings_path.exists() {
        let text = fs::read_to_string(settings_path)?;
        serde_json::from_str(&text).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    // Navigate/create hooks.Stop.
    if !settings.is_object() {
        settings = json!({});
    }
    let hooks = settings.as_object_mut().unwrap()
        .entry("hooks").or_insert_with(|| json!({}));
    if !hooks.is_object() {
        *hooks = json!({});
    }
    let stop = hooks.as_object_mut().unwrap()
        .entry("Stop").or_insert_with(|| json!([]));
    if !stop.is_array() {
        *stop = json!([]);
    }
    let stop_arr = stop.as_array_mut().unwrap();

    // Is our exact command already present?
    let already = stop_arr.iter().any(|group| {
        group.get("hooks")
            .and_then(|hs| hs.as_array())
            .map(|hs| hs.iter().any(|h| {
                h.get("command").and_then(|c| c.as_str()) == Some(hook_cmd.as_str())
            }))
            .unwrap_or(false)
    });
    if already {
        return Ok(false);
    }

    // Strip out any stale Breather hooks (old ~/.vibe-break/hook.sh or similar)
    // so we don't leave duplicates across renames.
    for group in stop_arr.iter_mut() {
        if let Some(hs) = group.get_mut("hooks").and_then(|h| h.as_array_mut()) {
            hs.retain(|h| {
                match h.get("command").and_then(|c| c.as_str()) {
                    Some(c) => !is_stale_breather_hook(c, &hook_cmd),
                    None => true,
                }
            });
        }
    }
    stop_arr.retain(|g| {
        g.get("hooks").and_then(|h| h.as_array()).map(|a| !a.is_empty()).unwrap_or(true)
    });

    // Add our hook group.
    stop_arr.push(json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": hook_cmd}]
    }));

    // Backup before writing.
    if settings_path.exists() {
        let backup = settings_path.with_extension(format!(
            "json.bak.{}",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        ));
        let _ = fs::copy(settings_path, &backup);
    }

    // Atomic write: tmp → rename.
    let tmp = settings_path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(&settings)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(&tmp, text + "\n")?;
    fs::rename(&tmp, settings_path)?;

    Ok(true)
}

/// Heuristic: detect hook paths from old Breather/Vibe Break installs so they
/// get cleaned up when we install the current one. We deliberately match only
/// paths under common Breather-named dirs to avoid touching unrelated hooks.
fn is_stale_breather_hook(command: &str, current: &str) -> bool {
    if command == current {
        return false; // we handle "already correct" above
    }
    command.contains("/.breather/hook.sh") || command.contains("/.vibe-break/hook.sh")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let d = std::env::temp_dir().join(format!("breather-hook-{}-{}", label, nanos));
        fs::create_dir_all(&d).ok();
        d
    }

    #[test]
    fn installs_into_empty_settings() {
        let dir = unique_dir("empty");
        let settings = dir.join("settings.json");
        let hook = dir.join("hook.sh");
        let modified = register_in_settings(&settings, &hook).unwrap();
        assert!(modified);
        let text = fs::read_to_string(&settings).unwrap();
        assert!(text.contains(&hook.to_string_lossy().to_string()));
    }

    #[test]
    fn second_install_is_noop() {
        let dir = unique_dir("noop");
        let settings = dir.join("settings.json");
        let hook = dir.join("hook.sh");
        assert!(register_in_settings(&settings, &hook).unwrap());
        assert!(!register_in_settings(&settings, &hook).unwrap(),
            "already correct should be no-op");
    }

    #[test]
    fn upgrades_old_vibe_break_path() {
        let dir = unique_dir("upgrade");
        let settings = dir.join("settings.json");
        let old_hook = "/Users/test/.vibe-break/hook.sh";
        fs::write(&settings, serde_json::to_string_pretty(&json!({
            "hooks": {
                "Stop": [{
                    "matcher": "",
                    "hooks": [{"type": "command", "command": old_hook}]
                }]
            }
        })).unwrap()).unwrap();

        let hook = dir.join("hook.sh");
        assert!(register_in_settings(&settings, &hook).unwrap());
        let text = fs::read_to_string(&settings).unwrap();
        assert!(!text.contains(".vibe-break"), "old hook should have been removed");
        assert!(text.contains(&hook.to_string_lossy().to_string()));
    }

    #[test]
    fn preserves_unrelated_hooks() {
        let dir = unique_dir("preserve");
        let settings = dir.join("settings.json");
        let unrelated = "/Users/test/my-other-tool/hook.sh";
        fs::write(&settings, serde_json::to_string_pretty(&json!({
            "hooks": {
                "PostToolUse": [{
                    "matcher": "",
                    "hooks": [{"type": "command", "command": unrelated}]
                }]
            }
        })).unwrap()).unwrap();

        let hook = dir.join("hook.sh");
        register_in_settings(&settings, &hook).unwrap();
        let text = fs::read_to_string(&settings).unwrap();
        assert!(text.contains(unrelated), "unrelated hook must be preserved");
    }

    #[test]
    fn gracefully_handles_malformed_settings() {
        let dir = unique_dir("malformed");
        let settings = dir.join("settings.json");
        fs::write(&settings, "this is not json").unwrap();
        let hook = dir.join("hook.sh");
        // Should not panic; should overwrite with fresh structure.
        let result = register_in_settings(&settings, &hook);
        assert!(result.is_ok());
    }

    #[test]
    fn write_hook_script_sets_executable() {
        let dir = unique_dir("script");
        let hook = dir.join("hook.sh");
        write_hook_script(&hook).unwrap();
        assert!(hook.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&hook).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o755);
        }
    }
}
