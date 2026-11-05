//! One server: what the panel knows about it, and the two threads that keep that fresh. The poller reads the
//! control API once a second (`.horizon/api.json` gives its address and token); the worker runs what the
//! user asks for (commands, moderation, start and stop, backups, settings) so a slow one never stalls the view.

use crate::scan;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

const CONSOLE_LINES: usize = 3000;
const CHAT_ITEMS: usize = 1000;
const SAMPLES: usize = 600;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Link {
    /// Not running (no control API answers).
    Stopped,
    /// The supervisor answers; its phase says the rest.
    Up,
}

#[derive(Clone, Debug)]
pub struct Sample {
    pub t: f64,
    pub tps: f64,
    pub mspt: f64,
    pub mspt_p95: f64,
    pub players: f64,
    pub ram_mb: f64,
    pub cpu: f64,
}

#[derive(Clone, Debug)]
pub enum ChatKind {
    Chat,
    Join,
    Leave,
    Death,
    Kick,
    Ban,
    Server,
}

#[derive(Clone, Debug)]
pub struct ChatItem {
    /// When it happened, in milliseconds since 1970 (for a network's merged chat).
    pub at: i64,
    pub time: String,
    pub kind: ChatKind,
    pub name: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Notice {
    pub ok: bool,
    pub text: String,
    pub at: Instant,
}

/// From `server.toml`, for a server that isn't running.
#[derive(Clone, Debug, Default)]
pub struct Info {
    pub port: Option<i64>,
    pub motd: String,
    pub world: String,
    pub max_players: Option<i64>,
}

#[derive(Default)]
pub struct State {
    pub link: Option<Link>,
    pub status: Option<Value>,
    pub players: Option<Value>,
    pub samples: VecDeque<Sample>,
    pub console: VecDeque<String>,
    pub chat: VecDeque<ChatItem>,
    pub info: Info,
    pub notices: VecDeque<Notice>,
    pub config: Option<Value>,
    pub backups: Option<Vec<(String, u64)>>,
    pub busy: Option<String>,
    pub ram_mb: Option<f64>,
    pub cpu: Option<f64>,
    events_next: u64,
    metrics_t: f64,
    api_pid: Option<u64>,
}

pub enum Act {
    Command(String),
    Say(String),
    Player(&'static str, Value),
    Start,
    Stop,
    Restart,
    Backup,
    Restore(String),
    LoadBackups,
    LoadConfig,
    SaveConfig(Vec<(String, String)>),
    /// Reload one plugin: its library file and its name.
    ReloadPlugin(PathBuf, String),
}

pub struct Server {
    pub dir: PathBuf,
    pub name: String,
    pub tool: Mutex<Option<PathBuf>>,
    pub state: Mutex<State>,
    acts: Sender<Act>,
    gone: AtomicBool,
}

impl Server {
    pub fn open(dir: PathBuf, tool: Option<PathBuf>, wake: Arc<dyn Fn() + Send + Sync>) -> Arc<Server> {
        let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| dir.display().to_string());
        // A folder called `server` says little: name it by its parent too (`survival/server` -> `survival`).
        let name = if name.eq_ignore_ascii_case("server") {
            dir.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or(name)
        } else {
            name
        };
        let (acts, inbox) = mpsc::channel();
        let server = Arc::new(Server {
            dir,
            name,
            tool: Mutex::new(tool),
            state: Mutex::new(State::default()),
            acts,
            gone: AtomicBool::new(false),
        });
        server.state.lock().unwrap().info = read_info(&server.dir);
        let poller = server.clone();
        let wake_poll = wake.clone();
        thread::Builder::new()
            .name(format!("poll {}", server.name))
            .spawn(move || poll(poller, wake_poll))
            .ok();
        let worker = server.clone();
        thread::Builder::new()
            .name(format!("work {}", server.name))
            .spawn(move || work(worker, inbox, wake))
            .ok();
        server
    }

    pub fn send(&self, act: Act) {
        let _ = self.acts.send(act);
    }

    pub fn close(&self) {
        self.gone.store(true, Ordering::SeqCst);
    }

    pub fn notice(&self, ok: bool, text: impl Into<String>) {
        let mut state = self.state.lock().unwrap();
        state.notices.push_back(Notice { ok, text: text.into(), at: Instant::now() });
        while state.notices.len() > 6 {
            state.notices.pop_front();
        }
    }
}

struct Api {
    base: String,
    token: String,
    agent: ureq::Agent,
}

impl Api {
    fn find(dir: &Path) -> Option<(Api, u64)> {
        let text = std::fs::read(dir.join(".horizon").join("api.json")).ok()?;
        let v: Value = serde_json::from_slice(&text).ok()?;
        let address = v["address"].as_str()?.to_owned();
        let token = v["token"].as_str()?.to_owned();
        let pid = v["pid"].as_u64().unwrap_or(0);
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_millis(800))
            .timeout(Duration::from_secs(20))
            .build();
        Some((Api { base: format!("http://{address}"), token, agent }, pid))
    }

    fn get(&self, path: &str) -> Result<Value, String> {
        let reply = self
            .agent
            .get(&format!("{}{path}", self.base))
            .set("Authorization", &format!("Bearer {}", self.token))
            .call();
        answer(reply)
    }

    fn post(&self, path: &str, body: Value) -> Result<Value, String> {
        let reply = self
            .agent
            .post(&format!("{}{path}", self.base))
            .set("Authorization", &format!("Bearer {}", self.token))
            .send_json(body);
        answer(reply)
    }
}

fn answer(reply: Result<ureq::Response, ureq::Error>) -> Result<Value, String> {
    match reply {
        Ok(r) => r.into_json::<Value>().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(code, r)) => {
            let body: Value = r.into_json().unwrap_or(Value::Null);
            Err(body["error"].as_str().map(str::to_owned).unwrap_or_else(|| format!("error {code}")))
        }
        Err(e) => Err(match e.kind() {
            ureq::ErrorKind::ConnectionFailed | ureq::ErrorKind::Io => "the server isn't answering".to_owned(),
            _ => e.to_string(),
        }),
    }
}

fn read_info(dir: &Path) -> Info {
    let Ok(text) = std::fs::read_to_string(dir.join("server.toml")) else {
        return Info::default();
    };
    let Ok(v) = text.parse::<toml::Table>() else {
        return Info::default();
    };
    let server = v.get("server").and_then(|s| s.as_table());
    let world = v.get("world").and_then(|s| s.as_table());
    Info {
        port: server.and_then(|s| s.get("port")).and_then(|p| p.as_integer()),
        motd: server.and_then(|s| s.get("motd")).and_then(|p| p.as_str()).unwrap_or_default().to_owned(),
        world: world.and_then(|s| s.get("name")).and_then(|p| p.as_str()).unwrap_or("world").to_owned(),
        max_players: server.and_then(|s| s.get("max_players")).and_then(|p| p.as_integer()),
    }
}

fn clock(ms: u64) -> String {
    use chrono::TimeZone;
    chrono::Local
        .timestamp_millis_opt(ms as i64)
        .single()
        .map(|t| t.format("%H:%M:%S").to_string())
        .unwrap_or_default()
}

/// A log line's `[HH:MM:SS]`, today (yesterday when that is still to come).
fn log_time(line: &str) -> Option<i64> {
    let t = line.strip_prefix('[')?.get(..8)?;
    let time = chrono::NaiveTime::parse_from_str(t, "%H:%M:%S").ok()?;
    let now = chrono::Local::now();
    let mut day = now.date_naive();
    if time > now.time() + chrono::Duration::minutes(1) {
        day = day.pred_opt()?;
    }
    use chrono::TimeZone;
    chrono::Local.from_local_datetime(&day.and_time(time)).single().map(|d| d.timestamp_millis())
}

fn now_clock() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

fn poll(server: Arc<Server>, wake: Arc<dyn Fn() + Send + Sync>) {
    let mut tick = 0u64;
    let mut info_at = Instant::now();
    while !server.gone.load(Ordering::SeqCst) {
        let started = Instant::now();
        let found = Api::find(&server.dir);
        let mut up = false;
        if let Some((api, pid)) = found {
            if let Ok(status) = api.get("/api/v1/status") {
                up = true;
                fresh(&server, &api, pid, status, tick);
            }
        }
        if !up {
            let mut state = server.state.lock().unwrap();
            if state.link != Some(Link::Stopped) {
                state.link = Some(Link::Stopped);
                state.status = None;
                state.players = None;
                state.ram_mb = None;
                state.cpu = None;
            }
        }
        if info_at.elapsed() > Duration::from_secs(5) || tick == 0 {
            let info = read_info(&server.dir);
            server.state.lock().unwrap().info = info;
            info_at = Instant::now();
        }
        wake();
        tick += 1;
        let rest = Duration::from_millis(1000).saturating_sub(started.elapsed());
        thread::sleep(rest.max(Duration::from_millis(200)));
    }
}

fn fresh(server: &Server, api: &Api, pid: u64, status: Value, tick: u64) {
    // A new supervisor numbers its events and samples from the start again.
    let (since, metrics_t) = {
        let mut state = server.state.lock().unwrap();
        if state.api_pid != Some(pid) {
            if state.api_pid.is_some() {
                state.console.push_back(format!("[{}] [Panel] the server started again", now_clock()));
            }
            state.api_pid = Some(pid);
            state.events_next = 0;
            state.metrics_t = 0.0;
            state.samples.clear();
        }
        (state.events_next, state.metrics_t)
    };
    let engine_pid = status["engine_pid"].as_u64();
    let usage = engine_pid.and_then(|p| scan::usage(p as u32));
    let rss = status["sample"]["rss_bytes"].as_f64().map(|b| b / 1048576.0).or(usage.map(|u| u.0 as f64 / 1048576.0));
    let cpu = status["sample"]["cpu_percent"].as_f64().or(usage.map(|u| u.1 as f64));

    let events = api.get(&format!("/api/v1/events?since={since}")).ok();
    let metrics = api.get(&format!("/api/v1/metrics?since={}", metrics_t as u64)).ok();
    let players = if tick % 2 == 0 { api.get("/api/v1/players").ok() } else { None };

    let mut state = server.state.lock().unwrap();
    state.link = Some(Link::Up);
    state.ram_mb = rss;
    state.cpu = cpu;
    state.status = Some(status);
    if let Some(p) = players {
        state.players = Some(p);
    }
    if let Some(m) = metrics {
        for s in m["samples"].as_array().into_iter().flatten() {
            let t = s["t"].as_f64().unwrap_or(0.0);
            if t <= state.metrics_t {
                continue;
            }
            state.metrics_t = t;
            let sample = Sample {
                t,
                tps: s["tps"].as_f64().unwrap_or(0.0),
                mspt: s["mspt"].as_f64().unwrap_or(0.0),
                mspt_p95: s["mspt_p95"].as_f64().unwrap_or(0.0),
                players: s["players"].as_f64().unwrap_or(0.0),
                ram_mb: s["rss_bytes"].as_f64().map(|b| b / 1048576.0).or(rss).unwrap_or(0.0),
                cpu: s["cpu_percent"].as_f64().or(cpu).unwrap_or(0.0),
            };
            state.samples.push_back(sample);
        }
        while state.samples.len() > SAMPLES {
            state.samples.pop_front();
        }
    }
    if let Some(e) = events {
        let next = e["next"].as_u64().unwrap_or(since);
        for event in e["events"].as_array().into_iter().flatten() {
            take_event(&mut state, event);
        }
        state.events_next = next.max(since);
        while state.console.len() > CONSOLE_LINES {
            state.console.pop_front();
        }
        while state.chat.len() > CHAT_ITEMS {
            state.chat.pop_front();
        }
    }
}

fn take_event(state: &mut State, e: &Value) {
    let kind = e["type"].as_str().unwrap_or_default();
    let name = e["name"].as_str().unwrap_or_default().to_owned();
    let at = e["time"].as_i64().unwrap_or_else(|| chrono::Local::now().timestamp_millis());
    let time = clock(at as u64);
    let item = |kind, text: String| ChatItem { at, time: time.clone(), kind, name: name.clone(), text };
    match kind {
        "log" => {
            let line = e["line"].as_str().unwrap_or_default().to_owned();
            // `say` from the console shows in the chat as `[Server] ...`, as players see it; its time is the
            // log line's own `[HH:MM:SS]`.
            if let Some(rest) = line.split("]: [Server] ").nth(1).or_else(|| line.split("]: [Not Secure] [Server] ").nth(1)) {
                let at = log_time(&line).unwrap_or(at);
                state.chat.push_back(ChatItem { at, time: clock(at as u64), kind: ChatKind::Server, name: "Server".into(), text: rest.to_owned() });
            }
            state.console.push_back(line);
        }
        "chat" => state.chat.push_back(item(ChatKind::Chat, e["message"].as_str().unwrap_or_default().to_owned())),
        "join" => state.chat.push_back(item(ChatKind::Join, "joined the game".into())),
        "leave" => state.chat.push_back(item(ChatKind::Leave, "left the game".into())),
        "death" => {
            let text = e["message"].as_str().unwrap_or_default();
            let text = text.strip_prefix(name.as_str()).map(|t| t.trim_start().to_owned()).unwrap_or_else(|| text.to_owned());
            state.chat.push_back(item(ChatKind::Death, text))
        }
        "kick" => {
            let reason = e["reason"].as_str().unwrap_or_default();
            state.chat.push_back(item(ChatKind::Kick, if reason.is_empty() { "was kicked".into() } else { format!("was kicked: {reason}") }))
        }
        "ban" => state.chat.push_back(item(ChatKind::Ban, "was banned".into())),
        _ => {}
    }
}

fn work(server: Arc<Server>, inbox: Receiver<Act>, wake: Arc<dyn Fn() + Send + Sync>) {
    while let Ok(act) = inbox.recv() {
        if server.gone.load(Ordering::SeqCst) {
            return;
        }
        let label = match &act {
            Act::Start => Some("Starting"),
            Act::Stop => Some("Stopping"),
            Act::Restart => Some("Restarting"),
            Act::Backup => Some("Backing up"),
            Act::Restore(_) => Some("Restoring"),
            Act::SaveConfig(_) => Some("Saving settings"),
            _ => None,
        };
        if let Some(l) = label {
            server.state.lock().unwrap().busy = Some(l.to_owned());
            wake();
        }
        let result = run(&server, act);
        server.state.lock().unwrap().busy = None;
        match result {
            Ok(Some(text)) => server.notice(true, text),
            Ok(None) => {}
            Err(text) => server.notice(false, text),
        }
        wake();
    }
}

fn api(server: &Server) -> Result<Api, String> {
    Api::find(&server.dir).map(|(a, _)| a).ok_or_else(|| "the server isn't running".to_owned())
}

fn lines(v: &Value) -> Vec<String> {
    for key in ["output", "lines", "messages"] {
        if let Some(list) = v[key].as_array() {
            return list.iter().filter_map(|l| l.as_str().map(str::to_owned)).collect();
        }
    }
    if let Some(s) = v.as_str() {
        return vec![s.to_owned()];
    }
    Vec::new()
}

fn run(server: &Server, act: Act) -> Result<Option<String>, String> {
    match act {
        Act::Command(line) => {
            let line = line.trim().trim_start_matches('/').to_owned();
            server.state.lock().unwrap().console.push_back(format!("> {line}"));
            let reply = api(server)?.post("/api/v1/command", json!({ "line": line }))?;
            let out = lines(&reply);
            let mut state = server.state.lock().unwrap();
            // What the command printed shows in the log stream too; only an answer that would be lost is kept.
            let recent: Vec<&String> = state.console.iter().rev().take(40).collect();
            let missing: Vec<String> = out.into_iter().filter(|l| !recent.iter().any(|r| r.ends_with(l.as_str()))).collect();
            for l in missing {
                state.console.push_back(l);
            }
            Ok(None)
        }
        Act::Say(text) => {
            api(server)?.post("/api/v1/command", json!({ "line": format!("say {text}") }))?;
            Ok(None)
        }
        Act::Player(what, body) => {
            let reply = api(server)?.post(&format!("/api/v1/{what}"), body)?;
            let out = lines(&reply).join(" ");
            let changed = reply["changed"].as_bool().unwrap_or(true);
            if let Ok(p) = api(server)?.get("/api/v1/players") {
                server.state.lock().unwrap().players = Some(p);
            }
            if changed { Ok(Some(if out.is_empty() { "Done".into() } else { out })) } else { Err(if out.is_empty() { "Nothing changed".into() } else { out }) }
        }
        Act::Start => {
            if Api::find(&server.dir).and_then(|(a, _)| a.get("/api/v1/status").ok()).is_some() {
                return Ok(Some("Already running".into()));
            }
            // `start` leaves the supervisor running in the background: its output goes to logs/supervisor.log,
            // never to a pipe the panel would wait on.
            let mut child = command(server, &["start"])?
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| format!("couldn't start it: {e}"))?;
            for _ in 0..80 {
                if let Ok(Some(code)) = child.try_wait() {
                    if !code.success() {
                        return Err("it didn't start: see logs/supervisor.log in its folder".into());
                    }
                }
                if Api::find(&server.dir).and_then(|(a, _)| a.get("/api/v1/status").ok()).is_some() {
                    return Ok(Some("Started".into()));
                }
                thread::sleep(Duration::from_millis(250));
            }
            Ok(Some("Starting".into()))
        }
        Act::Stop => {
            api(server)?.post("/api/v1/stop", json!({}))?;
            for _ in 0..120 {
                if Api::find(&server.dir).and_then(|(a, _)| a.get("/api/v1/status").ok()).is_none() {
                    return Ok(Some("Stopped".into()));
                }
                thread::sleep(Duration::from_millis(250));
            }
            Ok(Some("Stopping".into()))
        }
        Act::ReloadPlugin(path, name) => {
            if let Ok(reply) = api(server)?.post("/api/v1/command", json!({ "line": format!("hotswap {name}") })) {
                let out = lines(&reply).join(" ");
                if !out.contains("Unknown or incomplete command") && !out.contains("Unknown command") {
                    return Ok(Some(if out.is_empty() { format!("Reloaded {name}") } else { out }));
                }
            }
            // An engine without /hotswap: the file goes and comes back, which unloads it and loads it again.
            let off = PathBuf::from(format!("{}.disabled", path.display()));
            std::fs::rename(&path, &off).map_err(|e| format!("couldn't reload {name}: {e}"))?;
            thread::sleep(Duration::from_secs(3));
            std::fs::rename(&off, &path).map_err(|e| format!("couldn't turn {name} back on: {e}"))?;
            Ok(Some(format!("Reloading {name}")))
        }
        Act::Restart => {
            api(server)?.post("/api/v1/restart", json!({}))?;
            Ok(Some("Restarting the game".into()))
        }
        Act::Backup => {
            let out = cli(server, &["backup"])?;
            list_backups(server)?;
            Ok(Some(last_line(&out, "Backed up")))
        }
        Act::Restore(name) => {
            let out = cli(server, &["restore", &name, "--yes"])?;
            Ok(Some(last_line(&out, "Restored")))
        }
        Act::LoadBackups => {
            list_backups(server)?;
            Ok(None)
        }
        Act::LoadConfig => {
            let out = cli(server, &["config", "--json"])?;
            let v: Value = serde_json::from_str(&out).map_err(|_| "couldn't read the settings".to_owned())?;
            let v = if v.get("config").is_some() { v["config"].clone() } else { v };
            server.state.lock().unwrap().config = Some(v);
            Ok(None)
        }
        Act::SaveConfig(changes) => {
            let mut args = vec!["config".to_owned(), "set".to_owned()];
            args.extend(changes.iter().map(|(k, v)| format!("{k}={v}")));
            args.push("--json".to_owned());
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            let out = cli(server, &refs)?;
            let reply: Value = serde_json::from_str(&out).unwrap_or(Value::Null);
            let restart = reply["restart"].as_array().map(|a| !a.is_empty()).unwrap_or(false);
            let running = reply["running"].as_bool().unwrap_or(false);
            let _ = run(server, Act::LoadConfig);
            server.state.lock().unwrap().info = read_info(&server.dir);
            Ok(Some(if restart && running { "Saved. Some changes take effect at the next restart".into() } else { "Saved".into() }))
        }
    }
}

fn last_line(out: &str, fallback: &str) -> String {
    out.trim().lines().last().map(str::to_owned).filter(|l| !l.is_empty()).unwrap_or_else(|| fallback.to_owned())
}

fn list_backups(server: &Server) -> Result<(), String> {
    let out = cli(server, &["backups", "--json"])?;
    let v: Value = serde_json::from_str(&out).map_err(|_| "couldn't read the backups".to_owned())?;
    let list = v["backups"].as_array().or(v.as_array()).cloned().unwrap_or_default();
    let backups = list
        .iter()
        .map(|b| (b["name"].as_str().unwrap_or_default().to_owned(), b["bytes"].as_u64().unwrap_or(0)))
        .collect();
    server.state.lock().unwrap().backups = Some(backups);
    Ok(())
}

/// Runs `horizon-server --dir DIR ARGS...` and gives what it printed.
pub fn cli(server: &Server, args: &[&str]) -> Result<String, String> {
    let out = command(server, args)?.output().map_err(|e| format!("couldn't run horizon-server: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        Ok(text)
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        let msg = err.trim().lines().last().or(text.trim().lines().last()).unwrap_or("it failed").trim_start_matches("error: ").to_owned();
        Err(msg)
    }
}

fn command(server: &Server, args: &[&str]) -> Result<std::process::Command, String> {
    let tool = server.tool.lock().unwrap().clone().or_else(scan::find_tool).ok_or_else(|| {
        "horizon-server wasn't found: run `python setup.py runtime`, or start this server once from a terminal".to_owned()
    })?;
    let mut command = std::process::Command::new(&tool);
    command.arg("--dir").arg(&server.dir).args(args);
    command.stdin(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    Ok(command)
}

/// `rustpaper-panel --selftest DIR OUT`: every action the panel's buttons take, run against the server in
/// DIR (a test server: it is stopped, started and restarted), with what happened written to OUT.
pub fn selftest(dir: PathBuf, out: &Path) {
    let s = Server::open(dir.clone(), None, Arc::new(|| {}));
    let mut log = String::new();
    let mut fails = 0;
    let mut note = |ok: bool, what: &str, detail: String| {
        if !ok {
            fails += 1;
        }
        log.push_str(&format!("{} {what}: {detail}\n", if ok { "PASS" } else { "FAIL" }));
        let _ = std::fs::write(out, &log);
    };
    let up = |s: &Server| Api::find(&s.dir).and_then(|(a, _)| a.get("/api/v1/status").ok()).is_some_and(|v| v["phase"] == "running");
    let wait = |s: &Server, want: bool, secs: u64| {
        let t = Instant::now();
        while t.elapsed() < Duration::from_secs(secs) {
            if up(s) == want {
                return true;
            }
            thread::sleep(Duration::from_millis(500));
        }
        false
    };
    let players = |s: &Server| api(s).and_then(|a| a.get("/api/v1/players")).unwrap_or(Value::Null);
    let has = |v: &Value, list: &str, name: &str| v[list].as_array().is_some_and(|a| a.iter().any(|e| e["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(name))));

    if !up(&s) {
        let r = run(&s, Act::Start);
        note(wait(&s, true, 90), "start (stopped at first)", format!("{r:?}"));
    }
    note(up(&s), "running", String::new());
    thread::sleep(Duration::from_secs(3));

    let r = run(&s, Act::Command("time set night".into()));
    let c = s.state.lock().unwrap().console.iter().rev().take(10).cloned().collect::<Vec<_>>().join(" | ");
    note(r.is_ok() && c.contains("> time set night"), "console command", format!("{r:?} :: {c}"));

    let r = run(&s, Act::Say("Selftest message from the panel".into()));
    thread::sleep(Duration::from_secs(3));
    let chat = s.state.lock().unwrap().chat.iter().any(|c| matches!(c.kind, ChatKind::Server) && c.text.contains("Selftest message"));
    note(r.is_ok() && chat, "chat as [Server]", format!("{r:?}, shown in chat: {chat}"));

    for (what, path, list, name, undo) in [
        ("op", "players/op", "ops", "PanelOpTest", "players/deop"),
        ("ban", "players/ban", "bans", "PanelBanTest", "players/pardon"),
    ] {
        let r = run(&s, Act::Player(path, json!({ "name": name, "reason": "panel selftest" })));
        let p = players(&s);
        note(r.is_ok() && has(&p, list, name), what, format!("{r:?}"));
        let r = run(&s, Act::Player(undo, json!({ "name": name })));
        let p = players(&s);
        note(r.is_ok() && !has(&p, list, name), &format!("undo {what}"), format!("{r:?}"));
    }
    let r = run(&s, Act::Player("whitelist/add", json!({ "name": "PanelWlTest" })));
    let p = players(&s);
    let inside = p["whitelist"]["players"].as_array().is_some_and(|a| a.iter().any(|e| e["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case("PanelWlTest"))));
    note(r.is_ok() && inside, "whitelist add", format!("{r:?}"));
    let r = run(&s, Act::Player("whitelist/remove", json!({ "name": "PanelWlTest" })));
    note(r.is_ok(), "whitelist remove", format!("{r:?}"));

    let r = run(&s, Act::LoadConfig);
    let motd = s.state.lock().unwrap().config.as_ref().map(|c| c["server"]["motd"].as_str().unwrap_or("").to_owned()).unwrap_or_default();
    note(r.is_ok() && !motd.is_empty(), "read settings", format!("{r:?} motd={motd}"));
    let r = run(&s, Act::SaveConfig(vec![("server.motd".into(), "Panel selftest motd".into())]));
    let now = s.state.lock().unwrap().config.as_ref().map(|c| c["server"]["motd"].as_str().unwrap_or("").to_owned()).unwrap_or_default();
    note(r.is_ok() && now == "Panel selftest motd", "save settings", format!("{r:?} motd={now}"));
    let r = run(&s, Act::SaveConfig(vec![("server.motd".into(), motd.clone())]));
    note(r.is_ok(), "settings put back", format!("{r:?}"));

    let r = run(&s, Act::Backup);
    let n = s.state.lock().unwrap().backups.as_ref().map(|b| b.len()).unwrap_or(0);
    note(r.is_ok() && n > 0, "backup now", format!("{r:?}, {n} backups"));

    // Plugin files: off and on again, and a config saved through the editor's write.
    let listing = crate::files::list(&dir);
    if let Some(p) = listing.plugins.iter().find(|p| p.enabled) {
        let before = s.state.lock().unwrap().console.len();
        let off = crate::files::set_enabled(p, false);
        thread::sleep(Duration::from_secs(6));
        let c: Vec<String> = s.state.lock().unwrap().console.iter().skip(before).cloned().collect();
        note(c.iter().any(|l| l.to_lowercase().contains("disabl") || l.contains("unload") || l.contains("Goodbye") || l.contains("shutdown")), "the server unloads a plugin turned off", c.join(" | ").chars().take(400).collect());
        let after = crate::files::list(&dir);
        let is_off = after.plugins.iter().any(|q| q.name == p.name && !q.enabled);
        note(off.is_ok() && is_off, &format!("turn off {}", p.name), format!("{off:?}"));
        if let Ok(to) = off {
            let q = after.plugins.iter().find(|q| q.path == to).cloned().unwrap();
            let before = s.state.lock().unwrap().console.len();
            let on = crate::files::set_enabled(&q, true);
            thread::sleep(Duration::from_secs(8));
            let c: Vec<String> = s.state.lock().unwrap().console.iter().skip(before).cloned().collect();
            note(c.iter().any(|l| l.contains("Loading") || l.contains("Enabling") || l.contains("enabled")), "the server loads a plugin turned on", c.join(" | ").chars().take(400).collect());
            note(on.is_ok() && crate::files::list(&dir).plugins.iter().any(|q| q.name == p.name && q.enabled), &format!("turn on {}", p.name), format!("{on:?}"));
        }
        if let Some(folder) = &p.folder {
            let f = folder.join("panel-selftest.yml");
            let w = crate::files::write_text(&f, "greeting: 'hello'\ncount: 3\n");
            let back = crate::files::read_text(&f);
            note(w.is_ok() && back.as_deref() == Ok("greeting: 'hello'\ncount: 3\n"), "editor save + read", format!("{w:?}"));
            let _ = std::fs::remove_file(&f);
        }
        let before = s.state.lock().unwrap().console.len();
        let r = run(&s, Act::ReloadPlugin(p.path.clone(), p.name.clone()));
        thread::sleep(Duration::from_secs(6));
        let c: Vec<String> = s.state.lock().unwrap().console.iter().skip(before).cloned().collect();
        let loaded = c.iter().any(|l| l.contains("Loading") || l.contains("Enabling") || l.contains("enabled") || l.contains("Reloaded") || l.contains("hotswap"));
        note(r.is_ok() && loaded, &format!("reload {} (and the server loads it again)", p.name), format!("{r:?} :: {}", c.join(" | ").chars().take(500).collect::<String>()));
    }

    let pid0 = api(&s).and_then(|a| a.get("/api/v1/status")).ok().and_then(|v| v["engine_pid"].as_u64());
    let r = run(&s, Act::Restart);
    let t = Instant::now();
    let mut restarted = false;
    while t.elapsed() < Duration::from_secs(120) {
        let v = api(&s).and_then(|a| a.get("/api/v1/status")).ok();
        if v.as_ref().is_some_and(|v| v["phase"] == "running" && v["engine_pid"].as_u64() != pid0) {
            restarted = true;
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }
    note(r.is_ok() && restarted, "restart", format!("{r:?} in {:.0} s", t.elapsed().as_secs_f32()));

    let r = run(&s, Act::Stop);
    note(r.is_ok() && wait(&s, false, 60), "stop", format!("{r:?}"));
    let t = Instant::now();
    let r = run(&s, Act::Start);
    note(wait(&s, true, 120), "start", format!("{r:?} in {:.0} s", t.elapsed().as_secs_f32()));
    log.push_str(&format!("\n{} failed\n", fails));
    let _ = std::fs::write(out, &log);
    s.close();
}
