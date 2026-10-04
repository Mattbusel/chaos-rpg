//! Where CHAOS RPG keeps its files (saves, scores, settings).
//!
//! Every file used to be written next to the program. That breaks when the
//! program lives somewhere read-only, and after `cargo install` it scatters
//! JSON files into `~/.cargo/bin`. Saving failed silently in those cases.
//!
//! Now each file is looked up in this order:
//!
//! 1. `$CHAOS_RPG_DATA_DIR/<name>` if that variable is set.
//! 2. `<folder of the program>/<name>` if that file already exists, so
//!    existing saves and portable installs (a zip with `chaos_config.toml`
//!    next to the exe) keep working unchanged.
//! 3. The per-user data folder from the [`directories`] crate, created on
//!    first use:
//!    - Windows: `%APPDATA%\chaos-rpg\data\`
//!    - macOS: `~/Library/Application Support/chaos-rpg/`
//!    - Linux: `$XDG_DATA_HOME/chaos-rpg/` (usually `~/.local/share/chaos-rpg/`)
//! 4. The current folder, if no home folder can be found.
//!
//! ```
//! use chaos_rpg_core::paths::data_file;
//! let p = data_file("chaos_rpg_scores.json");
//! assert!(p.ends_with("chaos_rpg_scores.json"));
//! ```

use std::path::{Path, PathBuf};

/// Environment variable that forces every file into one folder.
pub const DATA_DIR_ENV: &str = "CHAOS_RPG_DATA_DIR";

/// The per-user data folder (rule 3 above), if the OS reports a home folder.
pub fn user_data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "chaos-rpg").map(|d| d.data_dir().to_path_buf())
}

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
}

/// The pure lookup behind [`data_file`], with every input passed in.
///
/// Does not touch the file system except to check whether
/// `exe_dir/name` exists.
pub fn resolve(
    name: &str,
    override_dir: Option<&Path>,
    exe_dir: Option<&Path>,
    user_dir: Option<&Path>,
) -> PathBuf {
    if let Some(dir) = override_dir {
        return dir.join(name);
    }
    if let Some(dir) = exe_dir {
        let legacy = dir.join(name);
        if legacy.exists() {
            return legacy;
        }
    }
    if let Some(dir) = user_dir {
        return dir.join(name);
    }
    PathBuf::from(name)
}

/// Full path for the data file `name` (for example `chaos_rpg_save.json`).
///
/// The parent folder is created if needed, so the result can be written to
/// directly. See the module docs for the lookup order.
pub fn data_file(name: &str) -> PathBuf {
    let override_dir = std::env::var_os(DATA_DIR_ENV)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);
    let path = resolve(
        name,
        override_dir.as_deref(),
        exe_dir().as_deref(),
        user_data_dir().as_deref(),
    );
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_wins() {
        let o = Path::new("/override");
        let p = resolve("x.json", Some(o), Some(Path::new("/exe")), Some(Path::new("/user")));
        assert_eq!(p, o.join("x.json"));
    }

    #[test]
    fn existing_file_next_to_exe_is_kept() {
        let exe = std::env::temp_dir().join(format!("chaos-paths-test-{}", std::process::id()));
        std::fs::create_dir_all(&exe).unwrap();
        std::fs::write(exe.join("old_save.json"), "{}").unwrap();
        let user = Path::new("/user");
        assert_eq!(resolve("old_save.json", None, Some(&exe), Some(user)), exe.join("old_save.json"));
        // A file that is not next to the exe goes to the user folder.
        assert_eq!(resolve("new.json", None, Some(&exe), Some(user)), user.join("new.json"));
        let _ = std::fs::remove_dir_all(&exe);
    }

    #[test]
    fn falls_back_to_current_folder() {
        assert_eq!(resolve("a.json", None, None, None), PathBuf::from("a.json"));
    }

    #[test]
    fn user_dir_is_named_after_the_game() {
        if let Some(d) = user_data_dir() {
            assert!(d.to_string_lossy().contains("chaos-rpg"), "{}", d.display());
        }
    }
}
