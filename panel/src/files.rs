//! A server's plugins and their files: the libraries in `plugins/` (each one's name and version from the
//! `plugin.yml` it carries), their data folders, and the colours the editor gives a config file.

use eframe::egui::{Color32, FontId, text::LayoutJob, text::TextFormat};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::SystemTime,
};

pub const LIBRARY: &str = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
const DISABLED: &str = "disabled";
/// Files above this are not opened in the editor.
const MAX_TEXT: u64 = 4 << 20;

#[derive(Clone, Debug)]
pub struct Plugin {
    pub path: PathBuf,
    pub file: String,
    pub name: String,
    /// The name came from the library's plugin.yml (not its file name).
    pub named: bool,
    pub version: Option<String>,
    pub enabled: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub folder: Option<PathBuf>,
}

#[derive(Clone, Debug, Default)]
pub struct Listing {
    pub plugins: Vec<Plugin>,
    /// Folders in `plugins/` that no plugin here owns.
    pub other: Vec<PathBuf>,
}

/// Name and version per library, by path, size and time, so a big file is read once.
static META: LazyLock<Mutex<HashMap<(PathBuf, u64, Option<SystemTime>), (Option<String>, Option<String>)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn list(server: &Path) -> Listing {
    let dir = server.join("plugins");
    let mut plugins = Vec::new();
    let mut folders = Vec::new();
    for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let file = entry.file_name().to_string_lossy().into_owned();
        if file.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            folders.push(path);
            continue;
        }
        let lower = file.to_lowercase();
        let enabled = lower.ends_with(&format!(".{LIBRARY}"));
        let disabled = lower.ends_with(&format!(".{LIBRARY}.{DISABLED}"));
        if !enabled && !disabled {
            continue;
        }
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta.and_then(|m| m.modified().ok());
        let (name, version) = describe(&path, size, modified);
        let stem = file.split('.').next().unwrap_or(&file).trim_start_matches("lib").to_owned();
        plugins.push(Plugin { path, file, named: name.is_some(), name: name.unwrap_or(stem), version, enabled, size, modified, folder: None });
    }
    plugins.sort_by_key(|p| p.name.to_lowercase());
    // A plugin's data folder is plugins/<its name>, as on Paper.
    let mut other = Vec::new();
    for f in folders {
        let fname = f.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        let owner = plugins.iter_mut().find(|p| p.folder.is_none() && (p.name.to_lowercase() == fname || p.file.to_lowercase().starts_with(&fname)));
        match owner {
            Some(p) => {
                // With no plugin.yml found in the library, its data folder's name is its name (Paper names
                // the folder after the plugin).
                if !p.named {
                    p.name = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| p.name.clone());
                }
                p.folder = Some(f)
            }
            None => other.push(f),
        }
    }
    other.sort();
    Listing { plugins, other }
}

/// The `name:` and `version:` of the plugin.yml a library carries.
fn describe(path: &Path, size: u64, modified: Option<SystemTime>) -> (Option<String>, Option<String>) {
    let key = (path.to_path_buf(), size, modified);
    if let Some(v) = META.lock().unwrap().get(&key) {
        return v.clone();
    }
    let found = std::fs::read(path).ok().map(|bytes| find_manifest(&bytes)).unwrap_or((None, None));
    META.lock().unwrap().insert(key, found.clone());
    found
}

fn find_manifest(bytes: &[u8]) -> (Option<String>, Option<String>) {
    let line_after = |at: usize| -> Option<String> {
        let rest = &bytes[at..bytes.len().min(at + 120)];
        let end = rest.iter().position(|&b| b == b'\n' || b == b'\r' || b == 0).unwrap_or(rest.len());
        let text = std::str::from_utf8(&rest[..end]).ok()?.trim().trim_matches(['"', '\'']).to_owned();
        (!text.is_empty() && text.len() < 64).then_some(text)
    };
    let mut i = 0;
    while let Some(off) = find(&bytes[i..], b"name:") {
        let at = i + off;
        let line_start = at == 0 || bytes[at - 1] == b'\n';
        if line_start {
            let window = &bytes[at.saturating_sub(400)..bytes.len().min(at + 400)];
            if find(window, b"main:").is_some() || find(window, b"version:").is_some() {
                let name = line_after(at + 5);
                let version = find(window, b"\nversion:").map(|v| at.saturating_sub(400) + v + 9).and_then(line_after);
                if name.is_some() {
                    return (name, version);
                }
            }
        }
        i = at + 5;
    }
    (None, None)
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The files under a folder, for the sidebar of the editor: (depth, path, is a folder).
pub fn tree(root: &Path) -> Vec<(usize, PathBuf, bool)> {
    let mut out = Vec::new();
    walk(root, 0, &mut out);
    out
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<(usize, PathBuf, bool)>) {
    if depth > 5 || out.len() > 1500 {
        return;
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| !p.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(true))
        .collect();
    entries.sort_by_key(|p| (!p.is_dir(), p.file_name().map(|n| n.to_string_lossy().to_lowercase())));
    for p in entries {
        let is_dir = p.is_dir();
        out.push((depth, p.clone(), is_dir));
        if is_dir {
            walk(&p, depth + 1, out);
        }
    }
}

pub fn read_text(path: &Path) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > MAX_TEXT {
        return Err(format!("This file is {} MB: too big to edit here.", meta.len() >> 20));
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.iter().take(8192).any(|&b| b == 0) {
        return Err("This isn't a text file.".into());
    }
    String::from_utf8(bytes).map_err(|_| "This file isn't UTF-8 text.".into())
}

/// Saves through a temporary file, so a crash never leaves half a config.
pub fn write_text(path: &Path, text: &str) -> Result<(), String> {
    let temp = path.with_extension(format!("{}.panel-tmp", path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default()));
    std::fs::write(&temp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&temp, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        e.to_string()
    })
}

pub fn set_enabled(p: &Plugin, on: bool) -> Result<PathBuf, String> {
    let to = if on {
        let s = p.path.to_string_lossy();
        PathBuf::from(s.strip_suffix(&format!(".{DISABLED}")).unwrap_or(&s).to_owned())
    } else {
        PathBuf::from(format!("{}.{DISABLED}", p.path.to_string_lossy()))
    };
    std::fs::rename(&p.path, &to).map_err(|e| e.to_string())?;
    Ok(to)
}

/// Copies plugin libraries into `plugins/` (a newer build of one replaces it); gives the file names.
pub fn upload(server: &Path, files: &[PathBuf]) -> Result<Vec<String>, String> {
    let dir = server.join("plugins");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut names = Vec::new();
    for f in files {
        let name = f.file_name().ok_or("no file name")?.to_string_lossy().into_owned();
        let to = dir.join(&name);
        let temp = dir.join(format!(".{name}.upload"));
        std::fs::copy(f, &temp).map_err(|e| format!("{name}: {e}"))?;
        std::fs::rename(&temp, &to).map_err(|e| format!("{name}: {e}"))?;
        names.push(name);
    }
    Ok(names)
}

pub fn ago(t: Option<SystemTime>) -> String {
    let Some(t) = t else { return String::new() };
    let secs = SystemTime::now().duration_since(t).map(|d| d.as_secs()).unwrap_or(0);
    match secs {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", secs / 60),
        3600..=86399 => format!("{} h ago", secs / 3600),
        _ => format!("{} d ago", secs / 86400),
    }
}

/// Colours for YAML, TOML, JSON, properties and plain text: comments, keys, strings, numbers.
pub fn highlight(text: &str, ext: &str, font: FontId) -> LayoutJob {
    let plain = Color32::from_rgb(0xd5, 0xd9, 0xe0);
    let comment = Color32::from_rgb(0x6b, 0x73, 0x80);
    let key = Color32::from_rgb(0x8a, 0xb4, 0xff);
    let string = Color32::from_rgb(0x9f, 0xd9, 0x8b);
    let number = Color32::from_rgb(0xf5, 0xb0, 0x41);
    let section = Color32::from_rgb(0xc4, 0xa7, 0xff);
    let fmt = |c: Color32| TextFormat { font_id: font.clone(), color: c, ..Default::default() };
    let mut job = LayoutJob::default();
    let code = matches!(ext, "yml" | "yaml" | "toml" | "json" | "properties" | "conf" | "cfg" | "ini" | "hocon");
    for line in text.split_inclusive('\n') {
        if !code {
            job.append(line, 0.0, fmt(plain));
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.starts_with("//") || (trimmed.starts_with(';') && ext == "ini") {
            job.append(line, 0.0, fmt(comment));
            continue;
        }
        if trimmed.starts_with('[') && matches!(ext, "toml" | "ini" | "cfg") {
            job.append(line, 0.0, fmt(section));
            continue;
        }
        // key: value  /  key = value  /  "key": value
        let sep = if ext == "json" {
            line.find("\":").map(|i| i + 1)
        } else {
            let colon = line.find(": ").or_else(|| line.trim_end().ends_with(':').then(|| line.trim_end().len() - 1));
            let eq = line.find('=');
            match (colon, eq) {
                (Some(c), Some(e)) => Some(c.min(e)),
                (c, e) => c.or(e),
            }
        };
        let (head, rest) = match sep {
            Some(i) if !trimmed.starts_with('-') || ext == "json" => line.split_at(i),
            _ => ("", line),
        };
        if !head.is_empty() {
            job.append(head, 0.0, fmt(key));
        }
        // The value: a string, a number or true/false, then a trailing comment.
        let (value, tail) = match rest.find(" #") {
            Some(i) if !matches!(ext, "json") => rest.split_at(i),
            _ => (rest, ""),
        };
        let v = value.trim_start_matches([':', '=', ' ', '-']).trim();
        let color = if v.starts_with('"') || v.starts_with('\'') {
            string
        } else if v.trim_end_matches(',').parse::<f64>().is_ok() || matches!(v.trim_end_matches(','), "true" | "false" | "null" | "yes" | "no") {
            number
        } else if head.is_empty() {
            plain
        } else {
            string
        };
        job.append(value, 0.0, fmt(color));
        if !tail.is_empty() {
            job.append(tail, 0.0, fmt(comment));
        }
    }
    job
}
