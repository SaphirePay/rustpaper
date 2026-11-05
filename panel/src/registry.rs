//! The list of servers the panel shows, kept in `servers.json` in the user's settings folder
//! (Windows: `%APPDATA%\rustpaper`, Linux: `~/.config/rustpaper`, macOS: `~/Library/Application Support/rustpaper`).
//! `setup.py server` and `setup.py start` add to it too; servers found running are added by the panel itself.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    pub dir: PathBuf,
    /// The `horizon-server` program that runs it, when known.
    #[serde(default)]
    pub tool: Option<PathBuf>,
    /// Show the panel when this server is started with `setup.py start` (false: headless).
    #[serde(default = "yes")]
    pub gui: bool,
    /// The network (a folder in the sidebar) it belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

/// A network: servers grouped under one name (a proxy's lobby, survival, minigames...).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Network {
    pub name: String,
    #[serde(default)]
    pub collapsed: bool,
}

fn yes() -> bool {
    true
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Registry {
    #[serde(default)]
    pub servers: Vec<Entry>,
    /// Folders the user took off the list: not added back when found running.
    #[serde(default)]
    pub hidden: Vec<PathBuf>,
    #[serde(default)]
    pub networks: Vec<Network>,
}

pub fn config_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    };
    base.unwrap_or_else(|| PathBuf::from(".")).join("rustpaper")
}

pub fn file() -> PathBuf {
    config_dir().join("servers.json")
}

impl Registry {
    pub fn load() -> Registry {
        let path = file();
        let mut r: Registry = match std::fs::read(&path) {
            Err(_) => Registry::default(),
            // Notepad saves UTF-8 with a byte order mark: it is not part of the list.
            Ok(bytes) => match serde_json::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes)) {
                Ok(r) => r,
                // A list that can't be read is kept aside, never written over: it may hold networks.
                Err(_) => {
                    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                    let _ = std::fs::rename(&path, path.with_extension(format!("json.unreadable-{stamp}")));
                    Registry::default()
                }
            },
        };
        r.tidy();
        r
    }

    pub fn save(&self) {
        let path = file();
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let temp = path.with_extension("json.tmp");
        if let Ok(text) = serde_json::to_vec_pretty(self) {
            if std::fs::write(&temp, text).is_ok() {
                let _ = std::fs::rename(&temp, &path);
            }
        }
    }

    pub fn find(&self, dir: &Path) -> Option<&Entry> {
        let k = key(dir);
        self.servers.iter().find(|e| key(&e.dir) == k)
    }

    pub fn find_mut(&mut self, dir: &Path) -> Option<&mut Entry> {
        let k = key(dir);
        self.servers.iter_mut().find(|e| key(&e.dir) == k)
    }

    pub fn is_hidden(&self, dir: &Path) -> bool {
        let k = key(dir);
        self.hidden.iter().any(|h| key(h) == k)
    }

    /// Adds a server (or fills in its program); true when something changed.
    pub fn add(&mut self, dir: &Path, tool: Option<PathBuf>) -> bool {
        let k = key(dir);
        self.hidden.retain(|h| key(h) != k);
        if let Some(entry) = self.find_mut(dir) {
            if entry.tool.is_none() && tool.is_some() {
                entry.tool = tool;
                return true;
            }
            return false;
        }
        self.servers.push(Entry { dir: clean(dir), tool, gui: true, network: None });
        true
    }

    /// Networks named by servers but missing from the list (setup.py --network) are added.
    pub fn tidy(&mut self) {
        let named: Vec<String> = self.servers.iter().filter_map(|e| e.network.clone()).collect();
        for n in named {
            if !self.networks.iter().any(|g| g.name == n) {
                self.networks.push(Network { name: n, collapsed: false });
            }
        }
    }

    pub fn members(&self, network: &str) -> Vec<PathBuf> {
        self.servers.iter().filter(|e| e.network.as_deref() == Some(network)).map(|e| e.dir.clone()).collect()
    }

    pub fn rename_network(&mut self, old: &str, new: &str) {
        for g in &mut self.networks {
            if g.name == old {
                g.name = new.to_owned();
            }
        }
        for e in &mut self.servers {
            if e.network.as_deref() == Some(old) {
                e.network = Some(new.to_owned());
            }
        }
    }

    pub fn delete_network(&mut self, name: &str) {
        self.networks.retain(|g| g.name != name);
        for e in &mut self.servers {
            if e.network.as_deref() == Some(name) {
                e.network = None;
            }
        }
    }

    pub fn remove(&mut self, dir: &Path) {
        let k = key(dir);
        self.servers.retain(|e| key(&e.dir) != k);
        if !self.is_hidden(dir) {
            self.hidden.push(clean(dir));
        }
    }
}

/// A folder's full path, without Windows' `\\?\` prefix.
pub fn clean(dir: &Path) -> PathBuf {
    let full = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let text = full.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => full.clone(),
    }
}

/// The same folder gives the same key, however it was spelled.
pub fn key(dir: &Path) -> String {
    let mut text = clean(dir).to_string_lossy().into_owned();
    if cfg!(windows) {
        text = text.replace('/', "\\").to_lowercase();
    }
    while text.len() > 1 && (text.ends_with('\\') || text.ends_with('/')) {
        text.pop();
    }
    text
}
