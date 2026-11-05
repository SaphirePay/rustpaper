//! Finds the servers running on this computer (every `horizon-server` process and its `--dir`), and measures
//! the game processes' RAM and CPU, which the supervisor doesn't report on every system.

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

pub struct Found {
    pub dir: PathBuf,
    pub tool: PathBuf,
}

struct Scanner {
    system: System,
    usage: HashMap<u32, (u64, f32)>,
    tool: Option<PathBuf>,
}

static SCANNER: LazyLock<Mutex<Scanner>> =
    LazyLock::new(|| Mutex::new(Scanner { system: System::new(), usage: HashMap::new(), tool: None }));

/// The running servers, and fresh RAM and CPU numbers for every process.
pub fn scan() -> Vec<Found> {
    let mut s = SCANNER.lock().unwrap();
    let kind = ProcessRefreshKind::new()
        .with_cpu()
        .with_memory()
        .with_cmd(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet)
        .with_cwd(UpdateKind::OnlyIfNotSet);
    s.system.refresh_processes_specifics(ProcessesToUpdate::All, kind);
    let cores = s.system.cpus().len().max(num_cores()) as f32;
    let mut found = Vec::new();
    let mut usage = HashMap::new();
    let mut tool = None;
    for (pid, p) in s.system.processes() {
        usage.insert(pid.as_u32(), (p.memory(), p.cpu_usage() / cores.max(1.0)));
        let name = p.name().to_string_lossy().to_lowercase();
        if name != "horizon-server" && name != "horizon-server.exe" {
            continue;
        }
        let cmd: Vec<String> = p.cmd().iter().map(|a| a.to_string_lossy().into_owned()).collect();
        // `run` and the background supervisor both say `--dir DIR`; with no `--dir`, the folder it runs in.
        let dir = cmd
            .iter()
            .position(|a| a == "--dir")
            .and_then(|i| cmd.get(i + 1))
            .map(PathBuf::from)
            .or_else(|| cmd.iter().find_map(|a| a.strip_prefix("--dir=").map(PathBuf::from)))
            .map(|d| match (d.is_relative(), p.cwd()) {
                (true, Some(cwd)) => cwd.join(d),
                _ => d,
            })
            .or_else(|| p.cwd().map(PathBuf::from));
        let exe = p.exe().map(PathBuf::from);
        if let Some(e) = &exe {
            tool.get_or_insert(e.clone());
        }
        if let (Some(dir), Some(exe)) = (dir, exe) {
            if dir.join("server.toml").exists() {
                found.push(Found { dir, tool: exe });
            }
        }
    }
    s.usage = usage;
    if tool.is_some() {
        s.tool = tool;
    }
    found
}

fn num_cores() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

/// A process's RAM (bytes) and CPU (percent of the whole computer), from the last scan.
pub fn usage(pid: u32) -> Option<(u64, f32)> {
    let s = SCANNER.lock().unwrap();
    s.usage.get(&pid).copied().filter(|_| s.system.process(Pid::from_u32(pid)).is_some())
}

/// A `horizon-server` to start servers with: one running now, the one beside this panel (`runtime/`), or the
/// Horizon launcher's.
pub fn find_tool() -> Option<PathBuf> {
    if let Some(t) = SCANNER.lock().unwrap().tool.clone() {
        return Some(t);
    }
    let exe = if cfg!(windows) { "horizon-server.exe" } else { "horizon-server" };
    let mut places = Vec::new();
    if let Ok(me) = std::env::current_exe() {
        for up in me.ancestors().skip(1).take(4) {
            places.push(up.join("runtime").join(exe));
            places.push(up.join(exe));
        }
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        places.push(PathBuf::from(appdata).join("horizon-launcher/server-runtime/current").join(exe));
    }
    if let Some(home) = std::env::var_os("HOME") {
        places.push(PathBuf::from(&home).join(".local/share/horizon-launcher/server-runtime/current").join(exe));
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            places.push(dir.join(exe));
        }
    }
    places.into_iter().find(|p| p.is_file())
}
