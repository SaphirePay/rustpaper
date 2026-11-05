//! The window: every server in the sidebar, an overview of them all, and one server's tabs (overview,
//! console, chat, players, backups, settings).

use crate::{
    chart::{self, Series},
    files,
    registry::{self, Registry},
    scan,
    server::{Act, ChatKind, Link, Server},
    theme::{self, *},
};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Key, Layout, Margin, Rect, Response, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, pos2, vec2,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Console,
    Chat,
    Players,
    Plugins,
    Backups,
    Settings,
}

impl Tab {
    pub const ALL: [Tab; 7] = [Tab::Overview, Tab::Console, Tab::Chat, Tab::Players, Tab::Plugins, Tab::Backups, Tab::Settings];
    fn label(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Console => "Console",
            Tab::Chat => "Chat",
            Tab::Players => "Players",
            Tab::Plugins => "Plugins",
            Tab::Backups => "Backups",
            Tab::Settings => "Settings",
        }
    }
    pub fn parse(s: &str) -> Option<Tab> {
        Tab::ALL.into_iter().find(|t| t.label().eq_ignore_ascii_case(s))
    }
}

#[derive(Clone, Default, PartialEq)]
struct Settings {
    motd: String,
    port: String,
    max_players: String,
    online_mode: bool,
    whitelist: bool,
    view_distance: String,
    simulation_distance: String,
    difficulty: String,
    game_mode: String,
    hardcore: bool,
}

impl Settings {
    fn from(c: &Value) -> Settings {
        let s = &c["server"];
        let w = &c["world"];
        let num = |v: &Value| v.as_i64().map(|n| n.to_string()).unwrap_or_default();
        Settings {
            motd: s["motd"].as_str().unwrap_or_default().to_owned(),
            port: num(&s["port"]),
            max_players: num(&s["max_players"]),
            online_mode: s["online_mode"].as_bool().unwrap_or(true),
            whitelist: s["whitelist"].as_bool().unwrap_or(false),
            view_distance: num(&s["view_distance"]),
            simulation_distance: num(&s["simulation_distance"]),
            difficulty: w["difficulty"].as_str().unwrap_or("easy").to_owned(),
            game_mode: w["game_mode"].as_str().unwrap_or("survival").to_owned(),
            hardcore: w["hardcore"].as_bool().unwrap_or(false),
        }
    }

    fn changes(&self, old: &Settings) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let mut put = |k: &str, a: String, b: String| {
            if a != b {
                out.push((k.to_owned(), a));
            }
        };
        put("server.motd", self.motd.clone(), old.motd.clone());
        put("server.port", self.port.clone(), old.port.clone());
        put("server.max_players", self.max_players.clone(), old.max_players.clone());
        put("server.online_mode", self.online_mode.to_string(), old.online_mode.to_string());
        put("server.whitelist", self.whitelist.to_string(), old.whitelist.to_string());
        put("server.view_distance", self.view_distance.clone(), old.view_distance.clone());
        put("server.simulation_distance", self.simulation_distance.clone(), old.simulation_distance.clone());
        put("world.difficulty", self.difficulty.clone(), old.difficulty.clone());
        put("world.game_mode", self.game_mode.clone(), old.game_mode.clone());
        put("world.hardcore", self.hardcore.to_string(), old.hardcore.to_string());
        out
    }
}

#[derive(Default)]
struct View {
    tab: Option<Tab>,
    command: String,
    history: Vec<String>,
    history_at: Option<usize>,
    filter: String,
    chat: String,
    name: String,
    settings: Option<(Settings, Settings)>,
    asked_config: bool,
    asked_backups: bool,
    focus_input: bool,
    listing: Option<(Instant, files::Listing)>,
    pick: Option<Pick>,
    editor: Option<Editor>,
}

/// What the Plugins tab shows on the right: a plugin, or a file (and the plugin it belongs to).
#[derive(Clone, PartialEq)]
enum Pick {
    Plugin(PathBuf),
    File(PathBuf, Option<PathBuf>),
}

impl Pick {
    fn plugin(&self) -> Option<&PathBuf> {
        match self {
            Pick::Plugin(p) => Some(p),
            Pick::File(_, owner) => owner.as_ref(),
        }
    }
    fn file(&self) -> Option<&PathBuf> {
        match self {
            Pick::File(f, _) => Some(f),
            _ => None,
        }
    }
}

struct Editor {
    #[allow(dead_code)]
    path: PathBuf,
    text: String,
    saved: String,
    error: Option<String>,
}

enum Dialog {
    Kick { server: usize, name: String, reason: String },
    Ban { server: usize, name: String, reason: String },
    BanIp { server: usize, name: String, reason: String },
    Stop { server: usize },
    Restore { server: usize, name: String },
    Remove { server: usize },
    Add { path: String, error: Option<String> },
    NewNetwork { name: String, server: Option<usize> },
    RenameNetwork { old: String, name: String },
    DeleteNetwork { name: String },
    StopAll { network: String },
    BanNetwork { network: String, name: String, reason: String },
    DeletePlugin { server: usize, plugin: files::Plugin, with_data: bool },
    Discard { server: usize, next: Pick },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetTab {
    Overview,
    Chat,
    Players,
    Settings,
}

impl NetTab {
    const ALL: [NetTab; 4] = [NetTab::Overview, NetTab::Chat, NetTab::Players, NetTab::Settings];
    fn label(self) -> &'static str {
        match self {
            NetTab::Overview => "Overview",
            NetTab::Chat => "Chat",
            NetTab::Players => "Players",
            NetTab::Settings => "Settings",
        }
    }
}

#[derive(Default)]
struct NetView {
    tab: usize,
    say: String,
    command: String,
    ban: String,
    rename: Option<String>,
}

pub struct Shot {
    pub file: PathBuf,
    pub tab: Option<Tab>,
    pub network: Option<String>,
    pub network_tab: Option<String>,
    pub open: Option<PathBuf>,
    pub dialog: Option<String>,
    pub search: Option<String>,
    pub after: Duration,
    pub taken: bool,
}

pub struct App {
    servers: Vec<Arc<Server>>,
    registry: Registry,
    selected: Option<usize>,
    network: Option<String>,
    /// The sidebar's search.
    search: String,
    views: HashMap<String, View>,
    net_views: HashMap<String, NetView>,
    wake: Arc<dyn Fn() + Send + Sync>,
    found: Arc<Mutex<Vec<scan::Found>>>,
    inbox: Arc<Mutex<Vec<PathBuf>>>,
    synced: Instant,
    dialog: Option<Dialog>,
    shot: Option<Shot>,
    opened: Instant,
    want: Option<PathBuf>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext, select: Option<PathBuf>, shot: Option<Shot>, inbox: Arc<Mutex<Vec<PathBuf>>>) -> App {
        theme::install(&cc.egui_ctx);
        let ctx = cc.egui_ctx.clone();
        let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || ctx.request_repaint());
        let found = Arc::new(Mutex::new(Vec::new()));
        let scan_found = found.clone();
        let scan_wake = wake.clone();
        // RUSTPAPER_PANEL_NO_SCAN=1 shows only the servers on the list (for the docs' pictures).
        let scanning = std::env::var_os("RUSTPAPER_PANEL_NO_SCAN").is_none();
        std::thread::Builder::new()
            .name("scan".into())
            .spawn(move || loop {
                let list = scan::scan();
                if scanning {
                    *scan_found.lock().unwrap() = list;
                }
                scan_wake();
                std::thread::sleep(Duration::from_secs(2));
            })
            .ok();
        let mut app = App {
            servers: Vec::new(),
            registry: Registry::load(),
            selected: None,
            network: None,
            search: String::new(),
            views: HashMap::new(),
            net_views: HashMap::new(),
            wake,
            found,
            inbox,
            synced: Instant::now() - Duration::from_secs(10),
            dialog: None,
            shot,
            opened: Instant::now(),
            want: None,
        };
        if let Some(dir) = select {
            app.registry.add(&dir, None);
            app.registry.save();
            app.want = Some(dir);
        }
        app.sync();
        app
    }

    /// The list on disk (setup.py adds to it) and the servers found running, merged.
    fn sync(&mut self) {
        let mut disk = Registry::load();
        for entry in &self.registry.servers {
            if disk.find(&entry.dir).is_none() && !disk.is_hidden(&entry.dir) {
                disk.servers.push(entry.clone());
            }
        }
        let mut changed = false;
        for f in self.found.lock().unwrap().iter() {
            if !disk.is_hidden(&f.dir) {
                changed |= disk.add(&f.dir, Some(f.tool.clone()));
            }
        }
        if changed {
            disk.save();
        }
        self.registry = disk;
        let keys: Vec<String> = self.registry.servers.iter().map(|e| registry::key(&e.dir)).collect();
        // Open servers that are new; close those taken off the list.
        let selected_key = self.selected.and_then(|i| self.servers.get(i)).map(|s| registry::key(&s.dir));
        let mut next = Vec::new();
        for entry in &self.registry.servers {
            let k = registry::key(&entry.dir);
            if let Some(s) = self.servers.iter().find(|s| registry::key(&s.dir) == k) {
                if entry.tool.is_some() {
                    *s.tool.lock().unwrap() = entry.tool.clone();
                }
                next.push(s.clone());
            } else {
                next.push(Server::open(registry::clean(&entry.dir), entry.tool.clone(), self.wake.clone()));
            }
        }
        for s in &self.servers {
            if !keys.contains(&registry::key(&s.dir)) {
                s.close();
            }
        }
        self.servers = next;
        self.selected = selected_key.and_then(|k| self.servers.iter().position(|s| registry::key(&s.dir) == k));
        if let Some(want) = self.want.take() {
            let k = registry::key(&want);
            match self.servers.iter().position(|s| registry::key(&s.dir) == k) {
                Some(i) => self.selected = Some(i),
                None => self.want = Some(want),
            }
        }
        self.synced = Instant::now();
    }

    fn view(&mut self, i: usize) -> &mut View {
        let k = registry::key(&self.servers[i].dir);
        self.views.entry(k).or_default()
    }
}

// ---------- small pieces ----------

#[derive(Clone, Copy)]
enum Kind {
    Primary,
    Accent,
    Normal,
    Danger,
    Quiet,
}

fn button(ui: &mut Ui, text: &str, kind: Kind, enabled: bool) -> Response {
    glass_button(ui, None, text, kind, enabled, 40.0, 14.0)
}

/// A glass button for `ui.put`: the same look as [`button`], as a widget.
fn styled(text: &str, kind: Kind) -> impl egui::Widget + '_ {
    move |ui: &mut Ui| glass_button(ui, None, text, kind, true, ui.available_height().clamp(28.0, 44.0), 13.5)
}

/// Text on one line, cut with an ellipsis to fit `width`.
fn one_line(p: &egui::Painter, text: &str, font: FontId, color: Color32, width: f32) -> Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping { max_width: width.max(10.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
    p.layout_job(job)
}

/// A dot and its words, right-aligned at `right`; gives the width.
fn paint_pill(p: &egui::Painter, right: egui::Pos2, text: &str, color: Color32) -> f32 {
    let galley = p.layout_no_wrap(text.to_owned(), FontId::new(13.0, FontFamily::Proportional), STRONG);
    let w = galley.size().x + 15.0;
    let x = right.x - w;
    p.circle_filled(pos2(x + 3.5, right.y), 3.5, color);
    p.galley(pos2(x + 15.0, right.y - galley.size().y / 2.0), galley, STRONG);
    w
}

fn small_button(ui: &mut Ui, text: &str, kind: Kind) -> Response {
    glass_button(ui, None, text, kind, true, 32.0, 13.5)
}

/// A frosted plate (`rounded-[20px] p-6`).
fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    theme::plate(ui, radius::PANEL, 22.0, add)
}

fn title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).family(theme::bold()).size(16.0).color(INK));
}

fn muted(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).color(MUTED).size(13.5));
}

/// A field (`h-11 rounded-xl bg-white/[0.07] px-4`).
fn field<'a>(text: &'a mut String, hint: &str) -> TextEdit<'a> {
    TextEdit::singleline(text)
        .hint_text(RichText::new(hint).color(PLACEHOLDER))
        .frame(Frame::new().fill(theme::FIELD_FILL).corner_radius(radius::ROW).inner_margin(Margin { left: 14, right: 14, top: 11, bottom: 11 }))
        .text_color(INK)
}

/// A state as the launcher writes one: a dot and its words (`● Running`).
fn pill(ui: &mut Ui, text: &str, color: Color32) {
    let font = FontId::new(13.0, FontFamily::Proportional);
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, STRONG);
    let size = vec2(galley.size().x + 15.0, galley.size().y.max(18.0));
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().circle_filled(pos2(rect.min.x + 3.5, rect.center().y), 3.5, color);
    ui.painter().galley(pos2(rect.min.x + 15.0, rect.center().y - galley.size().y / 2.0), galley, STRONG);
}

fn name_color(name: &str) -> Color32 {
    let palette = [
        Color32::from_rgb(0xa9, 0xa9, 0xff),
        Color32::from_rgb(0x4a, 0xde, 0x80),
        Color32::from_rgb(0xf2, 0xc4, 0x6b),
        Color32::from_rgb(0x7d, 0xd3, 0xfc),
        Color32::from_rgb(0xf9, 0xa8, 0xd4),
        Color32::from_rgb(0xfd, 0xba, 0x74),
        Color32::from_rgb(0x99, 0xf6, 0xe4),
    ];
    let h = name.bytes().fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    palette[h as usize % palette.len()]
}

/// A player's tile: their initial in their colour, on white at 10 %.
fn avatar(ui: &mut Ui, name: &str, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let c = name_color(name);
    ui.painter().rect_filled(rect, (size * 0.28) as u8, theme::SELECTED);
    let initial = name.chars().next().unwrap_or('?').to_ascii_uppercase().to_string();
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, initial, FontId::new(size * 0.45, theme::bold()), c);
}

fn duration(secs: u64) -> String {
    let (d, h, m) = (secs / 86400, secs / 3600 % 24, secs / 60 % 60);
    if d > 0 {
        format!("{d} d {h} h")
    } else if h > 0 {
        format!("{h} h {m} min")
    } else if m > 0 {
        format!("{m} min")
    } else {
        format!("{secs} s")
    }
}

fn bytes(b: u64) -> String {
    if b >= 1 << 30 {
        format!("{:.1} GB", b as f64 / (1u64 << 30) as f64)
    } else if b >= 1 << 20 {
        format!("{:.1} MB", b as f64 / (1u64 << 20) as f64)
    } else {
        format!("{} KB", b / 1024)
    }
}

/// What a server is doing, in a word, and its colour.
fn phase(server: &Server) -> (String, Color32) {
    let state = server.state.lock().unwrap();
    if let Some(b) = &state.busy {
        return (b.clone(), AMBER);
    }
    match state.link {
        None => ("Checking".into(), FAINT),
        Some(Link::Stopped) => ("Stopped".into(), FAINT),
        Some(Link::Up) => {
            let p = state.status.as_ref().and_then(|s| s["phase"].as_str()).unwrap_or("running");
            match p {
                "running" => {
                    let paused = state.status.as_ref().map(|s| s["sample"]["game"]["paused"].as_bool().unwrap_or(false)).unwrap_or(false);
                    if paused { ("Idle".into(), GREEN) } else { ("Running".into(), GREEN) }
                }
                "starting" => ("Starting".into(), AMBER),
                "stopping" => ("Stopping".into(), AMBER),
                "restarting" => ("Restarting".into(), AMBER),
                "crashed" | "backoff" | "failed" => ("Crashed".into(), RED),
                other => {
                    let mut s = other.to_owned();
                    if let Some(f) = s.get_mut(0..1) {
                        f.make_ascii_uppercase();
                    }
                    (s, AMBER)
                }
            }
        }
    }
}

struct Numbers {
    up: bool,
    tps: Option<f64>,
    mspt: Option<f64>,
    mspt_p95: Option<f64>,
    players: usize,
    max_players: Option<i64>,
    ram_mb: Option<f64>,
    cpu: Option<f64>,
    uptime: Option<u64>,
    paused: bool,
    version: String,
    port: Option<i64>,
    world: String,
    motd: String,
}

fn numbers(server: &Server) -> Numbers {
    let state = server.state.lock().unwrap();
    let st = state.status.as_ref();
    let game = st.map(|s| &s["sample"]["game"]);
    let up = state.link == Some(Link::Up);
    Numbers {
        up,
        tps: game.and_then(|g| g["tps"].as_f64()),
        mspt: game.and_then(|g| g["mspt"]["average"].as_f64()),
        mspt_p95: game.and_then(|g| g["mspt"]["p95"].as_f64()),
        players: game.and_then(|g| g["players"].as_array().map(|a| a.len())).or(state.players.as_ref().and_then(|p| p["online"].as_array().map(|a| a.len()))).unwrap_or(0),
        max_players: game.and_then(|g| g["max_players"].as_i64()).or(state.info.max_players),
        ram_mb: state.ram_mb,
        cpu: state.cpu,
        uptime: st.and_then(|s| s["engine_uptime_seconds"].as_u64().or(s["uptime_seconds"].as_u64())),
        paused: game.map(|g| g["paused"].as_bool().unwrap_or(false)).unwrap_or(false),
        version: st.and_then(|s| s["version"].as_str()).unwrap_or_default().to_owned(),
        port: state.info.port,
        world: state.info.world.clone(),
        motd: state.info.motd.clone(),
    }
}

fn tps_color(tps: f64) -> Color32 {
    if tps >= 19.0 {
        GREEN
    } else if tps >= 15.0 {
        AMBER
    } else {
        RED
    }
}

// ---------- the window ----------

impl eframe::App for App {
    fn ui(&mut self, root: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = &root.ctx().clone();
        // Another start of the panel asked to show a server.
        let asked: Vec<PathBuf> = std::mem::take(&mut *self.inbox.lock().unwrap());
        for dir in asked {
            if !dir.as_os_str().is_empty() {
                self.registry.add(&dir, None);
                self.registry.save();
                self.want = Some(dir);
                self.synced = Instant::now() - Duration::from_secs(10);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if self.synced.elapsed() > Duration::from_secs(2) {
            self.sync();
        }
        if let Some(q) = self.shot.as_mut().and_then(|shot| shot.search.take()) {
            self.search = q;
        }
        let open = self.shot.as_mut().and_then(|shot| shot.open.take());
        if let Some(d) = self.shot.as_mut().and_then(|shot| shot.dialog.take()) {
            let i = self.selected.unwrap_or(0);
            self.dialog = match d.as_str() {
                "stop" => Some(Dialog::Stop { server: i }),
                "add" => Some(Dialog::Add { path: String::new(), error: None }),
                "network" => Some(Dialog::NewNetwork { name: "Skyline Network 2".into(), server: None }),
                "ban" => Some(Dialog::Ban { server: i, name: "Alex4".into(), reason: String::new() }),
                _ => None,
            };
        }
        if let (Some(path), Some(i)) = (open, self.selected) {
            let lib = path.extension().is_some_and(|e| e == files::LIBRARY);
            let owner = files::list(&self.servers[i].dir).plugins.into_iter().find(|p| p.folder.as_ref().is_some_and(|f| path.starts_with(f))).map(|p| p.path);
            self.open_pick(i, if lib { Pick::Plugin(path) } else { Pick::File(path, owner) });
        }
        if let Some(shot) = &mut self.shot {
            if let (Some(tab), Some(i)) = (shot.tab, self.selected) {
                self.views.entry(registry::key(&self.servers[i].dir)).or_default().tab = Some(tab);
            }
            if let Some(net) = shot.network.clone() {
                self.network = Some(net.clone());
                self.selected = None;
                if let Some(t) = shot.network_tab.as_deref().and_then(|t| NetTab::ALL.iter().position(|x| x.label().eq_ignore_ascii_case(t))) {
                    self.net_views.entry(net).or_default().tab = t;
                }
            }
        }

        theme::keep_scene(ctx);
        let screen = root.max_rect();
        theme::paint_scene(root.painter(), screen);
        // The launcher's frame: a 34 px title line, the sidebar plate (248 wide, 22 from the edges),
        // the top bar beside it, the page under the top bar.
        let title = Rect::from_min_size(screen.min, vec2(screen.width(), 34.0));
        let side = Rect::from_min_max(pos2(screen.min.x + 22.0, screen.min.y + 46.0), pos2(screen.min.x + 270.0, screen.max.y - 22.0));
        let top = Rect::from_min_max(pos2(side.max.x + 32.0, side.min.y), pos2(screen.max.x - 40.0, side.min.y + 44.0));
        let page = Rect::from_min_max(pos2(top.min.x, top.max.y + 28.0), pos2(screen.max.x - 26.0, screen.max.y - 22.0));
        root.painter().add(theme::plate_shape(side, radius::PANEL, false));
        title_bar(root, title);
        root.scope_builder(egui::UiBuilder::new().max_rect(side.shrink2(vec2(10.0, 10.0))), |ui| self.sidebar(ui));
        let which = match (self.network.clone(), self.selected) {
            (Some(net), _) if self.registry.networks.iter().any(|g| g.name == net) => Err(net),
            (_, Some(i)) if i < self.servers.len() => Ok(Some(i)),
            _ => Ok(None),
        };
        // The tab strip sits in the middle of the top bar, over the page.
        root.scope_builder(egui::UiBuilder::new().max_rect(top).layout(Layout::top_down(Align::Center)), |ui| match &which {
            Ok(Some(i)) => {
                let i = *i;
                let view_tab = self.view(i).tab.unwrap_or(Tab::Overview);
                let mut at = Tab::ALL.iter().position(|t| *t == view_tab).unwrap_or(0);
                tab_bar(ui, &Tab::ALL.map(|t| t.label()), &mut at);
                if Tab::ALL[at] != view_tab {
                    let v = self.view(i);
                    v.tab = Some(Tab::ALL[at]);
                    v.focus_input = true;
                }
            }
            Err(net) => {
                let mut tab = self.net_views.entry(net.clone()).or_default().tab;
                tab_bar(ui, &NetTab::ALL.map(|t| t.label()), &mut tab);
                self.net_views.entry(net.clone()).or_default().tab = tab;
            }
            Ok(None) => {
                let mut at = 0;
                tab_bar(ui, &["All servers"], &mut at);
            }
        });
        root.scope_builder(egui::UiBuilder::new().max_rect(page.with_max_x(page.max.x - 14.0)), |ui| match which {
            Err(net) => self.network_page(ui, net),
            Ok(Some(i)) => self.server_page(ui, i),
            Ok(None) => self.all_page(ui),
        });
        resize_edges(root, screen);

        self.dialogs(ctx);
        self.toasts(ctx);
        ctx.request_repaint_after(Duration::from_millis(1000));

        if let Some(shot) = &mut self.shot {
            if !shot.taken && self.opened.elapsed() > shot.after {
                shot.taken = true;
                let file = shot.file.clone();
                let close = ctx.clone();
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::ScreenshotCallback::new(move |image| {
                    let [w, h] = image.size;
                    let mut rgba = Vec::with_capacity(w * h * 4);
                    for p in &image.pixels {
                        rgba.extend_from_slice(&p.to_array());
                    }
                    if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, rgba) {
                        let _ = buf.save(&file);
                    }
                    close.send_viewport_cmd(egui::ViewportCommand::Close);
                    close.request_repaint();
                })));
            }
        }
    }
}

impl App {
    fn sidebar(&mut self, ui: &mut Ui) {
        let running = self.servers.iter().filter(|s| s.state.lock().unwrap().link == Some(Link::Up)).count();
        let home = self.selected.is_none() && self.network.is_none();
        // Search: filters the list by name; networks with a match open while it is typed.
        let mut query = std::mem::take(&mut self.search);
        Frame::new().inner_margin(Margin { left: 2, right: 2, top: 2, bottom: 6 }).show(ui, |ui| {
            let w = ui.available_width();
            let (rect, _) = ui.allocate_exact_size(vec2(w, 36.0), Sense::hover());
            ui.painter().rect_filled(rect, radius::CONTROL, theme::FIELD_FILL);
            theme::paint_icon(ui, "search-01", Rect::from_center_size(pos2(rect.min.x + 18.0, rect.center().y), vec2(15.0, 15.0)), QUIET);
            let edit = Rect::from_min_max(pos2(rect.min.x + 34.0, rect.min.y), pos2(rect.max.x - 8.0, rect.max.y));
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(edit).layout(Layout::left_to_right(Align::Center)));
            child.add(
                TextEdit::singleline(&mut query)
                    .hint_text(RichText::new("Search servers").color(PLACEHOLDER))
                    .frame(Frame::NONE)
                    .desired_width(edit.width())
                    .min_size(vec2(edit.width(), 36.0))
                    .vertical_align(Align::Center)
                    .font(FontId::new(13.5, FontFamily::Proportional))
                    .text_color(INK),
            );
        });
        let q = query.trim().to_lowercase();
        self.search = query;
        let matches = |name: &str| q.is_empty() || name.to_lowercase().contains(&q);

        let bottom = 30.0;
        ScrollArea::vertical().max_height((ui.available_height() - bottom).max(80.0)).auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            if q.is_empty() && nav_row(ui, "dashboard-square-01", "All servers", &format!("{running}/{}", self.servers.len()), home).clicked() {
                self.selected = None;
                self.network = None;
            }

            let networks = self.registry.networks.clone();
            if let Some(r) = side_heading(ui, "Networks", Some("New network")) {
                if r.clicked() {
                    self.dialog = Some(Dialog::NewNetwork { name: String::new(), server: None });
                }
            }
            if networks.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("None yet").color(FAINT).size(12.5));
                });
            }
            for g in &networks {
                let members = self.members(&g.name);
                let shown: Vec<usize> = members.iter().copied().filter(|&i| matches(&self.servers[i].name)).collect();
                if !q.is_empty() && shown.is_empty() && !matches(&g.name) {
                    continue;
                }
                let up = members.iter().filter(|&&i| self.servers[i].state.lock().unwrap().link == Some(Link::Up)).count();
                let open = !g.collapsed || !q.is_empty();
                let (r, arrow) = network_row(ui, &g.name, up, members.len(), self.network.as_deref() == Some(g.name.as_str()), open);
                if arrow.clicked() {
                    if let Some(n) = self.registry.networks.iter_mut().find(|n| n.name == g.name) {
                        n.collapsed = !n.collapsed;
                    }
                    self.registry.save();
                } else if r.clicked() {
                    self.network = Some(g.name.clone());
                    self.selected = None;
                }
                let name = g.name.clone();
                let mut ask = None;
                r.context_menu(|ui| {
                    if ui.button("Rename…").clicked() {
                        ask = Some(Dialog::RenameNetwork { old: name.clone(), name: name.clone() });
                        ui.close();
                    }
                    if ui.button("Delete network").clicked() {
                        ask = Some(Dialog::DeleteNetwork { name: name.clone() });
                        ui.close();
                    }
                });
                if ask.is_some() {
                    self.dialog = ask;
                }
                if open {
                    let list = if q.is_empty() { members.clone() } else { shown };
                    let top = ui.cursor().min.y;
                    for &i in &list {
                        self.server_row(ui, i, 22.0);
                    }
                    if !list.is_empty() {
                        // A guide line down the network's servers.
                        let x = ui.max_rect().min.x + 15.0;
                        ui.painter().line_segment([pos2(x, top), pos2(x, ui.cursor().min.y - 4.0)], Stroke::new(1.0, LINE));
                    } else if q.is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(32.0);
                            ui.label(RichText::new("Empty: right-click a server to move it here").color(FAINT).size(12.0));
                        });
                    }
                }
            }

            let loose: Vec<usize> = (0..self.servers.len()).filter(|&i| self.network_of(i).is_none() && matches(&self.servers[i].name)).collect();
            if let Some(r) = side_heading(ui, "Servers", Some("Add a server folder")) {
                if r.clicked() {
                    self.dialog = Some(Dialog::Add { path: String::new(), error: None });
                }
            }
            for i in loose.iter().copied() {
                self.server_row(ui, i, 0.0);
            }
            if loose.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    let text = if !q.is_empty() { "No server matches" } else if self.servers.is_empty() { "None yet: + adds a server folder" } else { "All in networks" };
                    ui.label(RichText::new(text).color(FAINT).size(12.5));
                });
            }
        });
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("Closing keeps servers running").color(FAINT).size(11.5));
            });
        });
    }

    fn server_row(&mut self, ui: &mut Ui, i: usize, indent: f32) {
        let s = self.servers[i].clone();
        let (word, color) = phase(&s);
        let n = numbers(&s);
        let busy = s.state.lock().unwrap().busy.clone();
        let (meta, meta_color) = match (&busy, n.up) {
            (Some(b), _) => (format!("{b}…"), WARN),
            (None, true) => (format!("{}/{}", n.players, n.max_players.unwrap_or(0)), MUTED),
            (None, false) => ("Off".to_owned(), FAINT),
        };
        let r = compact_row(ui, color, &s.name, &meta, meta_color, self.selected == Some(i), indent, !n.up);
        let r = r.on_hover_text(format!("{word} · localhost:{}", n.port.map(|p| p.to_string()).unwrap_or("?".into())));
        if r.clicked() {
            self.selected = Some(i);
            self.network = None;
        }
        let networks: Vec<String> = self.registry.networks.iter().map(|g| g.name.clone()).collect();
        let current = self.network_of(i);
        let mut moved: Option<Option<String>> = None;
        let mut ask = None;
        let up = n.up;
        r.context_menu(|ui| {
            if up {
                if ui.button("Stop").clicked() {
                    ask = Some(Dialog::Stop { server: i });
                    ui.close();
                }
            } else if ui.button("Start").clicked() {
                s.send(Act::Start);
                ui.close();
            }
            ui.separator();
            ui.label(RichText::new("Move to network").color(QUIET).size(12.5));
            for g in &networks {
                let here = current.as_deref() == Some(g.as_str());
                if ui.add_enabled(!here, egui::Button::new(g.as_str())).clicked() {
                    moved = Some(Some(g.clone()));
                    ui.close();
                }
            }
            if current.is_some() && ui.button("No network").clicked() {
                moved = Some(None);
                ui.close();
            }
            if ui.button("New network…").clicked() {
                ask = Some(Dialog::NewNetwork { name: String::new(), server: Some(i) });
                ui.close();
            }
        });
        if let Some(to) = moved {
            self.move_to(i, to);
        }
        if ask.is_some() {
            self.dialog = ask;
        }
    }

    fn network_of(&self, i: usize) -> Option<String> {
        let n = self.registry.find(&self.servers[i].dir)?.network.clone()?;
        self.registry.networks.iter().any(|g| g.name == n).then_some(n)
    }

    fn members(&self, network: &str) -> Vec<usize> {
        (0..self.servers.len()).filter(|&i| self.network_of(i).as_deref() == Some(network)).collect()
    }

    fn move_to(&mut self, i: usize, network: Option<String>) {
        let dir = self.servers[i].dir.clone();
        if let Some(e) = self.registry.find_mut(&dir) {
            e.network = network;
        }
        self.registry.save();
    }

    fn all_page(&mut self, ui: &mut Ui) {
        let all: Vec<usize> = (0..self.servers.len()).collect();
        let (running, players) = totals(&self.servers, &all);
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let line = format!(
                "Every server on this computer, in one window. {running} of {} running · {players} {} online.",
                self.servers.len(),
                if players == 1 { "player" } else { "players" }
            );
            let mut add = false;
            let mut net = false;
            page_head(ui, None, "Servers", &line, |ui| {
                add = button(ui, "Add a server folder", Kind::Primary, true).clicked();
                net = button(ui, "New network", Kind::Normal, true).clicked();
            });
            if add {
                self.dialog = Some(Dialog::Add { path: String::new(), error: None });
            }
            if net {
                self.dialog = Some(Dialog::NewNetwork { name: String::new(), server: None });
            }
            if self.servers.is_empty() {
                card(ui, |ui| {
                    title(ui, "No servers yet");
                    muted(ui, "Make one with  python setup.py server myserver  and start it with  python setup.py start myserver.");
                    muted(ui, "Servers already running on this computer show up here by themselves.");
                });
                return;
            }
            summary(ui, &self.servers, &all);
            ui.add_space(16.0);
            self.table(ui, &all, true);
        });
    }

    /// The servers as a table on one plate: a row each, with its tile, state and numbers, and the
    /// one action that fits now.
    fn table(&mut self, ui: &mut Ui, list: &[usize], show_network: bool) {
        const HEAD: f32 = 44.0;
        const ROW: f32 = 64.0;
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(width, HEAD + ROW * list.len().max(1) as f32 + 10.0), Sense::hover());
        let p = ui.painter().clone();
        p.add(theme::plate_shape(rect, radius::PANEL, false));
        let inner_x = rect.min.x + 20.0;
        let inner_w = rect.width() - 40.0;
        let cols = [0.0, 0.33, 0.48, 0.58, 0.68, 0.77, 0.84];
        let x = |c: usize| inner_x + cols[c] * inner_w;
        let small = FontId::new(13.0, FontFamily::Proportional);
        for (c, label) in ["Server", "State", "TPS", "Players", "Memory", "CPU", "Uptime"].iter().enumerate() {
            let lx = if c == 0 { x(0) + 52.0 } else { x(c) };
            p.text(pos2(lx, rect.min.y + HEAD / 2.0 + 4.0), Align2::LEFT_CENTER, *label, small.clone(), QUIET);
        }
        if list.is_empty() {
            p.text(pos2(inner_x, rect.min.y + HEAD + ROW / 2.0), Align2::LEFT_CENTER, "No servers in this network yet: right-click a server in the sidebar to move it here.", FontId::new(14.0, FontFamily::Proportional), MUTED);
            return;
        }
        let mut open = None;
        for (k, &i) in list.iter().enumerate() {
            let s = self.servers[i].clone();
            let (word, color) = phase(&s);
            let n = numbers(&s);
            let busy = s.state.lock().unwrap().busy.is_some();
            let top = rect.min.y + HEAD + k as f32 * ROW;
            let row_rect = Rect::from_min_size(pos2(rect.min.x + 8.0, top), vec2(rect.width() - 16.0, ROW - 4.0));
            let r = ui.interact(row_rect, ui.id().with(("table row", i)), Sense::click());
            if r.hovered() {
                p.rect_filled(row_rect, radius::ROW, theme::HOVER);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let mid = top + (ROW - 4.0) / 2.0;
            let tile = Rect::from_min_size(pos2(x(0), mid - 20.0), vec2(40.0, 40.0));
            p.rect_filled(tile, radius::CONTROL, theme::BUTTON);
            theme::paint_icon(ui, "computer-terminal-01", Rect::from_center_size(tile.center(), vec2(20.0, 20.0)), STRONG);
            let tx = tile.max.x + 12.0;
            let name = one_line(&p, &s.name, FontId::new(14.5, theme::bold()), INK, x(1) - tx - 12.0);
            p.galley(pos2(tx, mid - 19.0), name, INK);
            let mut about = format!("localhost:{}", n.port.map(|v| v.to_string()).unwrap_or("?".into()));
            if show_network {
                if let Some(net) = self.network_of(i) {
                    about.push_str(&format!(" · {net}"));
                }
            }
            let about = one_line(&p, &about, FontId::new(12.5, FontFamily::Proportional), MUTED, x(1) - tx - 12.0);
            p.galley(pos2(tx, mid + 2.0), about, MUTED);
            let pw = pill_size(&p, &word);
            paint_pill(&p, pos2(x(1) + pw, mid), &word, color);
            let value = FontId::new(14.5, FontFamily::Proportional);
            let dim = if n.up { INK } else { FAINT };
            let dash = |v: Option<String>| v.filter(|_| n.up).unwrap_or_else(|| "–".into());
            let tps = n.tps.filter(|_| n.up);
            p.text(pos2(x(2), mid), Align2::LEFT_CENTER, dash(tps.map(|t| format!("{t:.1}"))), FontId::new(14.5, theme::bold()), tps.map(tps_color).unwrap_or(FAINT));
            p.text(pos2(x(3), mid), Align2::LEFT_CENTER, if n.up { format!("{} / {}", n.players, n.max_players.unwrap_or(0)) } else { "–".into() }, value.clone(), dim);
            p.text(pos2(x(4), mid), Align2::LEFT_CENTER, dash(n.ram_mb.map(|m| chart::short(m, "MB"))), value.clone(), dim);
            p.text(pos2(x(5), mid), Align2::LEFT_CENTER, dash(n.cpu.map(|c| format!("{c:.0}%"))), value.clone(), dim);
            p.text(pos2(x(6), mid), Align2::LEFT_CENTER, dash(n.uptime.map(duration)), value, dim);
            let b = Rect::from_min_size(pos2(rect.max.x - 20.0 - 84.0, mid - 18.0), vec2(84.0, 36.0));
            let clicked = if n.up {
                ui.add_enabled_ui(!busy, |ui| ui.put(b, styled("Stop", Kind::Normal))).inner.clicked()
            } else {
                ui.add_enabled_ui(!busy, |ui| ui.put(b, styled(if busy { "Starting" } else { "Start" }, Kind::Accent))).inner.clicked()
            };
            if clicked {
                if n.up {
                    self.dialog = Some(Dialog::Stop { server: i });
                } else {
                    s.send(Act::Start);
                }
            } else if r.clicked() {
                open = Some(i);
            }
        }
        if let Some(i) = open {
            self.selected = Some(i);
            self.network = None;
        }
    }

    fn network_page(&mut self, ui: &mut Ui, name: String) {
        let members = self.members(&name);
        let (running, players) = totals(&self.servers, &members);
        let stopped = members.len() - running;
        let tab = self.net_views.entry(name.clone()).or_default().tab;
        let state = format!("Network · {running} of {} running", members.len());
        let slots: i64 = members.iter().filter_map(|&i| numbers(&self.servers[i]).max_players).sum();
        let line = format!("{} {} · {players} {} online · {slots} slots", members.len(), if members.len() == 1 { "server" } else { "servers" }, if players == 1 { "player" } else { "players" });
        let color = if running == members.len() && running > 0 { LIVE } else if running > 0 { WARN } else { FAINT };
        let (mut stop_all, mut restart_all, mut start_all) = (false, false, false);
        page_head(ui, Some((color, state)), &name, &line, |ui| {
            if stopped > 0 {
                start_all = button(ui, "Start all", Kind::Accent, true).clicked();
            }
            if running > 0 {
                stop_all = button(ui, "Stop all", Kind::Primary, true).clicked();
                restart_all = button(ui, "Restart all", Kind::Normal, true).clicked();
            }
        });
        if stop_all {
            self.dialog = Some(Dialog::StopAll { network: name.clone() });
        }
        for &i in &members {
            let up = self.servers[i].state.lock().unwrap().link == Some(Link::Up);
            if restart_all && up {
                self.servers[i].send(Act::Restart);
            }
            if start_all && !up {
                self.servers[i].send(Act::Start);
            }
        }
        match NetTab::ALL[tab] {
            NetTab::Overview => {
                ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    summary(ui, &self.servers, &members);
                    ui.add_space(16.0);
                    self.table(ui, &members, false);
                    ui.add_space(16.0);
                    let v = self.net_views.entry(name.clone()).or_default();
                    let mut run = None;
                    card(ui, |ui| {
                        title(ui, "Run on every server");
                        muted(ui, "A console command for each running server in this network, like  whitelist add Steve  or  save-all.");
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let r = ui.add(field(&mut v.command, "Console command").desired_width(ui.available_width() - 90.0).font(egui::TextStyle::Monospace));
                            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                            if (enter || button(ui, "  Run  ", Kind::Primary, !v.command.trim().is_empty()).clicked()) && !v.command.trim().is_empty() {
                                run = Some(std::mem::take(&mut v.command));
                            }
                        });
                    });
                    if let Some(line) = run {
                        for &i in &members {
                            if self.servers[i].state.lock().unwrap().link == Some(Link::Up) {
                                self.servers[i].send(Act::Command(line.clone()));
                            }
                        }
                    }
                });
            }
            NetTab::Chat => self.network_chat(ui, &name, &members),
            NetTab::Players => self.network_players(ui, &name, &members),
            NetTab::Settings => self.network_settings(ui, &name),
        }
    }

    fn network_chat(&mut self, ui: &mut Ui, name: &str, members: &[usize]) {
        let mut items: Vec<(String, crate::server::ChatItem)> = members
            .iter()
            .flat_map(|&i| {
                let s = &self.servers[i];
                s.state.lock().unwrap().chat.iter().map(|c| (s.name.clone(), c.clone())).collect::<Vec<_>>()
            })
            .collect();
        items.sort_by_key(|(_, c)| c.at);
        let any_up = members.iter().any(|&i| self.servers[i].state.lock().unwrap().link == Some(Link::Up));
        surface(ui, true, radius::PANEL, Margin::same(18), |ui| {
            ui.set_height((ui.available_height() - 70.0).max(120.0));
            ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                if items.is_empty() {
                    muted(ui, "The chat of every server in this network shows here, in one place.");
                }
                for (server, c) in &items {
                    chat_line_on(ui, c, Some(server));
                }
            });
        });
        ui.add_space(10.0);
        let v = self.net_views.entry(name.to_owned()).or_default();
        let mut send = None;
        surface(ui, true, radius::LARGE, Margin::same(6), |ui| ui.horizontal(|ui| {
            let r = ui.add_enabled(any_up, field(&mut v.say, "Message everyone on every server as [Server]").desired_width(ui.available_width() - 100.0));
            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if (enter || button(ui, "  Send all  ", Kind::Primary, any_up && !v.say.trim().is_empty()).clicked()) && !v.say.trim().is_empty() {
                send = Some(std::mem::take(&mut v.say));
                r.request_focus();
            }
        }));
        if let Some(text) = send {
            for &i in members {
                if self.servers[i].state.lock().unwrap().link == Some(Link::Up) {
                    self.servers[i].send(Act::Say(text.clone()));
                }
            }
        }
    }

    fn network_players(&mut self, ui: &mut Ui, name: &str, members: &[usize]) {
        let mut online: Vec<(usize, Value)> = Vec::new();
        for &i in members {
            if let Some(p) = &self.servers[i].state.lock().unwrap().players {
                for pl in p["online"].as_array().into_iter().flatten() {
                    online.push((i, pl.clone()));
                }
            }
        }
        online.sort_by_key(|(_, p)| p["name"].as_str().unwrap_or_default().to_lowercase());
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            card(ui, |ui| {
                title(ui, &format!("Online across the network  {}", online.len()));
                ui.add_space(4.0);
                if online.is_empty() {
                    muted(ui, "Nobody is online on this network's servers.");
                }
                for (k, (i, pl)) in online.iter().enumerate() {
                    let player = pl["name"].as_str().unwrap_or_default().to_owned();
                    if k > 0 {
                        ui.separator();
                    }
                    ui.horizontal(|ui| {
                        avatar(ui, &player, 30.0);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&player).family(theme::bold()));
                                if pl["op_level"].as_i64().unwrap_or(0) > 0 {
                                    pill(ui, "Operator", VIOLET);
                                }
                            });
                            ui.label(RichText::new(format!("on {} · {} ms", self.servers[*i].name, pl["ping_ms"].as_i64().unwrap_or(0))).color(MUTED).size(12.0));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if small_button(ui, "Ban everywhere", Kind::Danger).clicked() {
                                self.dialog = Some(Dialog::BanNetwork { network: name.to_owned(), name: player.clone(), reason: String::new() });
                            }
                            if small_button(ui, "Kick", Kind::Normal).clicked() {
                                self.dialog = Some(Dialog::Kick { server: *i, name: player.clone(), reason: String::new() });
                            }
                            if small_button(ui, "Open server", Kind::Normal).clicked() {
                                self.selected = Some(*i);
                                self.network = None;
                                let v = self.views.entry(registry::key(&self.servers[*i].dir)).or_default();
                                v.tab = Some(Tab::Players);
                            }
                        });
                    });
                }
            });
            ui.add_space(14.0);
            let v = self.net_views.entry(name.to_owned()).or_default();
            let mut who = std::mem::take(&mut v.ban);
            let mut act: Option<(&'static str, String)> = None;
            card(ui, |ui| {
                title(ui, "A player on every server");
                muted(ui, "Applies to each running server in this network.");
                ui.horizontal(|ui| {
                    ui.add(field(&mut who, "Player name").desired_width(220.0));
                    let ok = !who.trim().is_empty();
                    ui.add_enabled_ui(ok, |ui| {
                        if small_button(ui, "Whitelist", Kind::Normal).clicked() {
                            act = Some(("whitelist/add", who.trim().to_owned()));
                        }
                        if small_button(ui, "Make op", Kind::Primary).clicked() {
                            act = Some(("players/op", who.trim().to_owned()));
                        }
                        if small_button(ui, "Unban", Kind::Normal).clicked() {
                            act = Some(("players/pardon", who.trim().to_owned()));
                        }
                        if small_button(ui, "Ban", Kind::Danger).clicked() {
                            self.dialog = Some(Dialog::BanNetwork { network: name.to_owned(), name: who.trim().to_owned(), reason: String::new() });
                        }
                    });
                });
            });
            self.net_views.entry(name.to_owned()).or_default().ban = who;
            if let Some((what, player)) = act {
                for &i in members {
                    if self.servers[i].state.lock().unwrap().link == Some(Link::Up) {
                        self.servers[i].send(Act::Player(what, json!({ "name": player })));
                    }
                }
            }
        });
    }

    fn network_settings(&mut self, ui: &mut Ui, name: &str) {
        let mut rename = self.net_views.entry(name.to_owned()).or_default().rename.take().unwrap_or_else(|| name.to_owned());
        let mut moves: Vec<(usize, Option<String>)> = Vec::new();
        let mut renamed = None;
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            card(ui, |ui| {
                title(ui, "Network");
                ui.horizontal(|ui| {
                    ui.add(field(&mut rename, "Network name").desired_width(280.0));
                    let ok = !rename.trim().is_empty() && rename.trim() != name && !self.registry.networks.iter().any(|g| g.name == rename.trim());
                    if button(ui, "Rename", Kind::Primary, ok).clicked() {
                        renamed = Some(rename.trim().to_owned());
                    }
                });
            });
            ui.add_space(14.0);
            card(ui, |ui| {
                title(ui, "Servers in this network");
                muted(ui, "Tick the servers that belong here, for example the lobby, survival and minigames behind one proxy.");
                ui.add_space(4.0);
                for i in 0..self.servers.len() {
                    let current = self.network_of(i);
                    let mut inside = current.as_deref() == Some(name);
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut inside, RichText::new(&self.servers[i].name)).changed() {
                            moves.push((i, if inside { Some(name.to_owned()) } else { None }));
                        }
                        if let Some(other) = current.filter(|c| c != name) {
                            ui.label(RichText::new(format!("in {other}")).color(FAINT).size(12.0));
                        }
                    });
                }
            });
            ui.add_space(14.0);
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        title(ui, "Delete this network");
                        muted(ui, "Its servers stay as they are and move back to the main list.");
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button(ui, "Delete network", Kind::Danger, true).clicked() {
                            self.dialog = Some(Dialog::DeleteNetwork { name: name.to_owned() });
                        }
                    });
                });
            });
        });
        for (i, to) in moves {
            self.move_to(i, to);
        }
        if let Some(new) = renamed {
            self.registry.rename_network(name, &new);
            self.registry.save();
            self.net_views.remove(name);
            self.network = Some(new);
        } else {
            self.net_views.entry(name.to_owned()).or_default().rename = Some(rename);
        }
    }

    fn server_page(&mut self, ui: &mut Ui, i: usize) {
        let s = self.servers[i].clone();
        let (word, color) = phase(&s);
        let n = numbers(&s);
        let busy = s.state.lock().unwrap().busy.is_some();

        let tab = self.view(i).tab.unwrap_or(Tab::Overview);
        let mut state = word.clone();
        if let Some(u) = n.uptime.filter(|_| n.up) {
            state.push_str(&format!(" · up {}", duration(u)));
        }
        state.push_str(&format!(" · localhost:{}", n.port.map(|p| p.to_string()).unwrap_or("?".into())));
        let line = if n.up {
            let mut l = format!("{} of {} players", n.players, n.max_players.unwrap_or(0));
            if let Some(t) = n.tps {
                l.push_str(&format!(" · {t:.1} TPS"));
            }
            if let Some(m) = n.mspt {
                l.push_str(&format!(" · {} a tick", chart::short(m, "ms")));
            }
            l.push_str(&format!(" · world \"{}\"", n.world));
            l
        } else {
            format!("Stopped · world \"{}\"{}", n.world, if n.version.is_empty() { String::new() } else { format!(" · {}", n.version) })
        };
        let (mut stop, mut restart, mut start) = (false, false, false);
        page_head(ui, Some((color, state)), &s.name, &line, |ui| {
            if n.up {
                stop = button(ui, "Stop", Kind::Primary, !busy).clicked();
                restart = button(ui, "Restart", Kind::Normal, !busy).clicked();
            } else {
                start = button(ui, if busy { "Starting…" } else { "Start server" }, Kind::Accent, !busy).clicked();
            }
        });
        if stop {
            self.dialog = Some(Dialog::Stop { server: i });
        }
        if restart {
            s.send(Act::Restart);
        }
        if start {
            s.send(Act::Start);
        }

        match tab {
            Tab::Overview => self.overview(ui, i),
            Tab::Console => self.console(ui, i),
            Tab::Chat => self.chat(ui, i),
            Tab::Players => self.players(ui, i),
            Tab::Plugins => self.plugins(ui, i),
            Tab::Backups => self.backups(ui, i),
            Tab::Settings => self.settings(ui, i),
        }
    }

    fn overview(&mut self, ui: &mut Ui, i: usize) {
        let s = self.servers[i].clone();
        let n = numbers(&s);
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if !n.up {
                card(ui, |ui| {
                    title(ui, "This server is stopped");
                    muted(ui, "Start it to see its TPS, players, memory and CPU here. Its console, chat and players come back when it runs.");
                });
                ui.add_space(14.0);
            }
            ui.columns(5, |c| {
                let tps = n.tps.filter(|_| n.up);
                stat(&mut c[0], "TPS", &tps.map(|t| format!("{t:.1}")).unwrap_or("–".into()), if n.paused { "idle, nobody online" } else { "target 20.0" }, tps.map(tps_color).unwrap_or(TEXT));
                stat(&mut c[1], "Tick time", &n.mspt.filter(|_| n.up).map(|m| chart::short(m, "ms")).unwrap_or("–".into()), &n.mspt_p95.filter(|_| n.up).map(|m| format!("95th percentile {}", chart::short(m, "ms"))).unwrap_or("of 50 ms per tick".into()), TEXT);
                stat(&mut c[2], "Players", &if n.up { n.players.to_string() } else { "–".into() }, &format!("of {} slots", n.max_players.unwrap_or(0)), TEXT);
                stat(&mut c[3], "Memory", &n.ram_mb.map(|m| chart::short(m, "MB")).unwrap_or("–".into()), "used by the game", TEXT);
                stat(&mut c[4], "CPU", &n.cpu.map(|c| format!("{c:.1}%")).unwrap_or("–".into()), "of this computer", TEXT);
            });
            ui.add_space(14.0);
            let samples: Vec<_> = s.state.lock().unwrap().samples.iter().cloned().collect();
            let pts = |f: &dyn Fn(&crate::server::Sample) -> f64| samples.iter().map(|x| [x.t, f(x)]).collect::<Vec<_>>();
            let last = samples.last();
            ui.columns(2, |c| {
                chart::chart(&mut c[0], "Ticks per second", &last.map(|l| format!("{:.1} TPS", l.tps)).unwrap_or("–".into()),
                    &[Series { label: "TPS", color: GREEN, points: pts(&|x| x.tps) }], 20.0, Some(21.0), "", 220.0);
                chart::chart(&mut c[1], "Tick time", &last.map(|l| chart::short(l.mspt, "ms")).unwrap_or("–".into()),
                    &[Series { label: "average", color: ACCENT, points: pts(&|x| x.mspt) }, Series { label: "95th pct", color: VIOLET, points: pts(&|x| x.mspt_p95) }], 1.0, None, "ms", 220.0);
            });
            ui.add_space(14.0);
            ui.columns(2, |c| {
                chart::chart(&mut c[0], "Players online", &last.map(|l| format!("{:.0}", l.players)).unwrap_or("–".into()),
                    &[Series { label: "players", color: CYAN, points: pts(&|x| x.players) }], 4.0, None, "", 200.0);
                chart::chart(&mut c[1], "Memory", &last.map(|l| chart::short(l.ram_mb, "MB")).unwrap_or("–".into()),
                    &[Series { label: "RAM", color: AMBER, points: pts(&|x| x.ram_mb) }], 256.0, None, "MB", 200.0);
            });
            ui.add_space(14.0);
            let chat: Vec<_> = s.state.lock().unwrap().chat.iter().rev().take(6).cloned().collect();
            card(ui, |ui| {
                title(ui, "Recent activity");
                if chat.is_empty() {
                    muted(ui, "Joins, leaves, chat and deaths show here.");
                }
                for c in chat.iter().rev() {
                    chat_line(ui, c);
                }
            });
        });
    }

    fn console(&mut self, ui: &mut Ui, i: usize) {
        let s = self.servers[i].clone();
        let up = s.state.lock().unwrap().link == Some(Link::Up);
        let filter = self.view(i).filter.to_lowercase();
        let lines: Vec<String> = s.state.lock().unwrap().console.iter().filter(|l| filter.is_empty() || l.to_lowercase().contains(&filter)).cloned().collect();
        ui.horizontal(|ui| {
            muted(ui, &format!("{} lines", lines.len()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(field(&mut self.view(i).filter, "Filter the log").desired_width(240.0));
            });
        });
        ui.add_space(4.0);
        let input_h = 46.0;
        surface(ui, true, radius::POPOVER, Margin::same(16), |ui| {
            ui.set_height((ui.available_height() - input_h - 24.0).max(120.0));
            ScrollArea::both().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                if lines.is_empty() {
                    ui.label(RichText::new(if up { "Nothing logged yet." } else { "The server is stopped: its log shows here when it runs." }).color(FAINT).monospace());
                }
                for l in &lines {
                    let color = if l.starts_with("> ") {
                        ACCENT
                    } else if l.contains("/ERROR]") || l.contains("ERROR") && l.contains("]:") {
                        RED
                    } else if l.contains("/WARN]") {
                        AMBER
                    } else if l.contains("[Supervisor/") || l.contains("[Panel]") {
                        CYAN.linear_multiply(0.85)
                    } else {
                        Color32::from_rgb(0xc9, 0xce, 0xd6)
                    };
                    ui.add(egui::Label::new(RichText::new(l).monospace().color(color)).extend());
                }
            });
        });
        ui.add_space(10.0);
        let v = self.view(i);
        let mut submit = None;
        surface(ui, true, radius::LARGE, Margin::same(6), |ui| ui.horizontal(|ui| {
            let w = ui.available_width() - 90.0;
            let r = ui.add_enabled(up, field(&mut v.command, if up { "Type a console command, like  time set day  or  give Steve diamond 5" } else { "Start the server to run commands" }).desired_width(w).font(egui::TextStyle::Monospace));
            if v.focus_input && up {
                r.request_focus();
                v.focus_input = false;
            }
            if r.has_focus() {
                let (upk, down) = ui.input(|i| (i.key_pressed(Key::ArrowUp), i.key_pressed(Key::ArrowDown)));
                if upk && !v.history.is_empty() {
                    let at = v.history_at.map(|a| a.saturating_sub(1)).unwrap_or(v.history.len() - 1);
                    v.history_at = Some(at);
                    v.command = v.history[at].clone();
                }
                if down {
                    if let Some(a) = v.history_at {
                        if a + 1 < v.history.len() {
                            v.history_at = Some(a + 1);
                            v.command = v.history[a + 1].clone();
                        } else {
                            v.history_at = None;
                            v.command.clear();
                        }
                    }
                }
            }
            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if (enter || button(ui, "  Run  ", Kind::Primary, up && !v.command.trim().is_empty()).clicked()) && !v.command.trim().is_empty() {
                submit = Some(std::mem::take(&mut v.command));
                v.history_at = None;
                r.request_focus();
            }
        }));
        if let Some(line) = submit {
            let v = self.view(i);
            if v.history.last() != Some(&line) {
                v.history.push(line.clone());
            }
            s.send(Act::Command(line));
        }
    }

    fn chat(&mut self, ui: &mut Ui, i: usize) {
        let s = self.servers[i].clone();
        let up = s.state.lock().unwrap().link == Some(Link::Up);
        let items: Vec<_> = s.state.lock().unwrap().chat.iter().cloned().collect();
        let input_h = 46.0;
        surface(ui, true, radius::PANEL, Margin::same(18), |ui| {
            ui.set_height((ui.available_height() - input_h - 24.0).max(120.0));
            ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                if items.is_empty() {
                    muted(ui, if up { "No messages yet. What players say, and who joins and leaves, shows here." } else { "The server is stopped." });
                }
                for c in &items {
                    chat_line(ui, c);
                }
            });
        });
        ui.add_space(10.0);
        let v = self.view(i);
        let mut send = None;
        surface(ui, true, radius::LARGE, Margin::same(6), |ui| ui.horizontal(|ui| {
            let w = ui.available_width() - 90.0;
            let r = ui.add_enabled(up, field(&mut v.chat, if up { "Message everyone as [Server]" } else { "Start the server to chat" }).desired_width(w));
            if v.focus_input && up {
                r.request_focus();
                v.focus_input = false;
            }
            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if (enter || button(ui, "  Send  ", Kind::Primary, up && !v.chat.trim().is_empty()).clicked()) && !v.chat.trim().is_empty() {
                send = Some(std::mem::take(&mut v.chat));
                r.request_focus();
            }
        }));
        if let Some(text) = send {
            s.send(Act::Say(text));
        }
    }

    fn players(&mut self, ui: &mut Ui, i: usize) {
        let s = self.servers[i].clone();
        let up = s.state.lock().unwrap().link == Some(Link::Up);
        let p = s.state.lock().unwrap().players.clone().unwrap_or(Value::Null);
        if !up {
            card(ui, |ui| {
                title(ui, "This server is stopped");
                muted(ui, "Start it to see who is online and to kick, ban or op players.");
            });
            return;
        }
        let act = |what: &'static str, body: Value| Act::Player(what, body);
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let online = p["online"].as_array().cloned().unwrap_or_default();
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    title(ui, &format!("Online now  {} / {}", online.len(), p["max_players"].as_i64().unwrap_or(0)));
                });
                ui.add_space(4.0);
                if online.is_empty() {
                    muted(ui, "Nobody is online.");
                }
                for (n, pl) in online.iter().enumerate() {
                    let name = pl["name"].as_str().unwrap_or_default().to_owned();
                    let op = pl["op_level"].as_i64().unwrap_or(0) > 0;
                    if n > 0 {
                        ui.separator();
                    }
                    ui.horizontal(|ui| {
                        avatar(ui, &name, 32.0);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&name).family(theme::bold()));
                                if op {
                                    pill(ui, "Operator", VIOLET);
                                }
                            });
                            ui.label(RichText::new(format!("{} ms · {}", pl["ping_ms"].as_i64().unwrap_or(0), pl["address"].as_str().unwrap_or(""))).color(MUTED).size(12.0));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if small_button(ui, "Ban", Kind::Danger).clicked() {
                                self.dialog = Some(Dialog::Ban { server: i, name: name.clone(), reason: String::new() });
                            }
                            if small_button(ui, "Kick", Kind::Normal).clicked() {
                                self.dialog = Some(Dialog::Kick { server: i, name: name.clone(), reason: String::new() });
                            }
                            // One width for both words, so every row's buttons line up.
                            let (label, path) = if op { ("Remove op", "players/deop") } else { ("Make op", "players/op") };
                            if ui.add_sized(vec2(96.0, 32.0), styled(label, Kind::Normal)).clicked() {
                                s.send(act(path, json!({ "name": name })));
                            }
                            if small_button(ui, "Ban IP", Kind::Normal).on_hover_text("Ban this player's address").clicked() {
                                self.dialog = Some(Dialog::BanIp { server: i, name: name.clone(), reason: String::new() });
                            }
                        });
                    });
                }
            });
            ui.add_space(14.0);

            // A player by name, online or not.
            let mut who = std::mem::take(&mut self.view(i).name);
            card(ui, |ui| {
                title(ui, "A player by name");
                ui.horizontal(|ui| {
                    ui.add(field(&mut who, "Player name").desired_width(220.0));
                    let ok = !who.trim().is_empty();
                    let name = who.trim().to_owned();
                    ui.add_enabled_ui(ok, |ui| {
                        if small_button(ui, "Make op", Kind::Primary).clicked() {
                            s.send(act("players/op", json!({ "name": name })));
                        }
                        if small_button(ui, "Whitelist", Kind::Normal).clicked() {
                            s.send(act("whitelist/add", json!({ "name": name })));
                        }
                        if small_button(ui, "Ban", Kind::Danger).clicked() {
                            self.dialog = Some(Dialog::Ban { server: i, name: name.clone(), reason: String::new() });
                        }
                    });
                });
            });
            self.view(i).name = who;
            ui.add_space(14.0);

            ui.columns(2, |c| {
                let ops = p["ops"].as_array().cloned().unwrap_or_default();
                list_card(&mut c[0], "Operators", "No operators.", &ops, |e| (e["name"].as_str().unwrap_or("").to_owned(), format!("level {}", e["level"].as_i64().unwrap_or(4))), "Remove", |name| {
                    s.send(act("players/deop", json!({ "name": name })))
                });
                c[0].add_space(14.0);
                let bans = p["bans"].as_array().cloned().unwrap_or_default();
                list_card(&mut c[0], "Banned players", "Nobody is banned.", &bans, |e| (e["name"].as_str().unwrap_or("").to_owned(), e["reason"].as_str().unwrap_or("").to_owned()), "Unban", |name| {
                    s.send(act("players/pardon", json!({ "name": name })))
                });
                let wl = &p["whitelist"];
                let enabled = wl["enabled"].as_bool().unwrap_or(false);
                let players = wl["players"].as_array().cloned().unwrap_or_default();
                card(&mut c[1], |ui| {
                    ui.horizontal(|ui| {
                        title(ui, "Whitelist");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let mut on = enabled;
                            if ui.add(toggle(&mut on)).changed() {
                                s.send(act("whitelist", json!({ "enabled": on })));
                            }
                            ui.label(RichText::new(if enabled { "On" } else { "Off" }).color(MUTED));
                        });
                    });
                    muted(ui, if enabled { "Only these players and operators can join." } else { "Off: anyone can join." });
                    for e in &players {
                        let name = e["name"].as_str().unwrap_or("").to_owned();
                        ui.horizontal(|ui| {
                            avatar(ui, &name, 22.0);
                            ui.label(&name);
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if small_button(ui, "Remove", Kind::Normal).clicked() {
                                    s.send(act("whitelist/remove", json!({ "name": name })));
                                }
                            });
                        });
                    }
                });
                c[1].add_space(14.0);
                let ip_bans = p["ip_bans"].as_array().cloned().unwrap_or_default();
                list_card(&mut c[1], "Banned addresses", "No addresses are banned.", &ip_bans, |e| (e["ip"].as_str().or(e["name"].as_str()).unwrap_or("").to_owned(), e["reason"].as_str().unwrap_or("").to_owned()), "Unban", |ip| {
                    s.send(act("players/pardon-ip", json!({ "ip": ip })))
                });
            });
        });
    }

    fn plugins(&mut self, ui: &mut Ui, i: usize) {
        let s = self.servers[i].clone();
        let up = s.state.lock().unwrap().link == Some(Link::Up);
        // The folder is read again every two seconds (a copied-in plugin shows up by itself).
        let stale = self.view(i).listing.as_ref().is_none_or(|(at, _)| at.elapsed() > Duration::from_secs(2));
        if stale {
            let l = files::list(&s.dir);
            self.view(i).listing = Some((Instant::now(), l));
        }
        let listing = self.view(i).listing.as_ref().map(|(_, l)| l.clone()).unwrap_or_default();
        let active = listing.plugins.iter().filter(|p| p.enabled).count();

        ui.horizontal(|ui| {
            muted(ui, &format!("{} {} · {active} on · changes load while the server runs", listing.plugins.len(), if listing.plugins.len() == 1 { "plugin" } else { "plugins" }));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if button(ui, "Upload plugin…", Kind::Primary, true).clicked() {
                    if let Some(picked) = rfd::FileDialog::new().add_filter("Plugin", &[files::LIBRARY]).set_title("Upload plugins").pick_files() {
                        match files::upload(&s.dir, &picked) {
                            Ok(names) => s.notice(true, format!("Uploaded {}: {}", names.join(", "), if up { "it loads in a few seconds" } else { "it loads when the server starts" })),
                            Err(e) => s.notice(false, e),
                        }
                        self.view(i).listing = None;
                    }
                }
                if button(ui, "Open folder", Kind::Normal, true).clicked() {
                    let _ = std::fs::create_dir_all(s.dir.join("plugins"));
                    open_folder(&s.dir.join("plugins"));
                }
            });
        });
        ui.add_space(8.0);

        let height = ui.available_height();
        let left_w = 300.0;
        let mut pick: Option<Pick> = None;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            // Left: the plugins, each with its files when open; then the server's own files.
            let current = self.view(i).pick.clone();
            ui.allocate_ui_with_layout(vec2(left_w, height), Layout::top_down(Align::Min), |ui| {
                surface(ui, false, radius::PANEL, Margin::same(10), |ui| {
                    ui.set_width(left_w - 18.0);
                    ui.set_height(height - 18.0);
                    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        list_heading(ui, "PLUGINS");
                        if listing.plugins.is_empty() {
                            ui.label(RichText::new("  No plugins yet: upload one.").color(MUTED).size(12.5));
                        }
                        for p in &listing.plugins {
                            let selected = current.as_ref().is_some_and(|c| c.plugin() == Some(&p.path));
                            let sub = format!("{}{}", p.version.as_deref().map(|v| format!("v{v} · ")).unwrap_or_default(), if p.enabled { "on" } else { "off" });
                            if file_row(ui, &p.name, Some(&sub), 0, if p.enabled { Some(GREEN) } else { Some(FAINT) }, matches!(current, Some(Pick::Plugin(ref x)) if *x == p.path), true).clicked() {
                                pick = Some(Pick::Plugin(p.path.clone()));
                            }
                            if selected {
                                if let Some(folder) = &p.folder {
                                    for (depth, path, is_dir) in files::tree(folder) {
                                        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                        let on = matches!(current, Some(Pick::File(ref x, _)) if *x == path);
                                        if file_row(ui, &name, None, depth + 1, None, on, is_dir).clicked() && !is_dir {
                                            pick = Some(Pick::File(path.clone(), Some(p.path.clone())));
                                        }
                                    }
                                }
                            }
                        }
                        if !listing.other.is_empty() {
                            list_heading(ui, "OTHER FOLDERS");
                            for f in &listing.other {
                                let name = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                let open = current.as_ref().and_then(|c| c.file()).is_some_and(|x| x.starts_with(f));
                                if file_row(ui, &name, None, 0, None, false, true).clicked() {
                                    if let Some((_, first, _)) = files::tree(f).into_iter().find(|(_, _, d)| !d) {
                                        pick = Some(Pick::File(first, None));
                                    }
                                }
                                if open {
                                    for (depth, path, is_dir) in files::tree(f) {
                                        let n = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                        let on = matches!(current, Some(Pick::File(ref x, _)) if *x == path);
                                        if file_row(ui, &n, None, depth + 1, None, on, is_dir).clicked() && !is_dir {
                                            pick = Some(Pick::File(path.clone(), None));
                                        }
                                    }
                                }
                            }
                        }
                        list_heading(ui, "SERVER FILES");
                        for name in ["server.toml", "ops.json", "whitelist.json", "banned-players.json", "banned-ips.json"] {
                            let path = s.dir.join(name);
                            if path.is_file() {
                                let on = matches!(current, Some(Pick::File(ref x, _)) if *x == path);
                                if file_row(ui, name, None, 0, None, on, false).clicked() {
                                    pick = Some(Pick::File(path, None));
                                }
                            }
                        }
                    });
                });
            });

            // Right: the plugin, or the file in the editor.
            let right_w = ui.available_width();
            ui.allocate_ui_with_layout(vec2(right_w, height), Layout::top_down(Align::Min), |ui| {
                match current.clone() {
                    None => {
                        card(ui, |ui| {
                            title(ui, "Plugins");
                            muted(ui, "Pick a plugin on the left to reload, turn off, replace or delete it, and to edit its config files.");
                            muted(ui, &format!("Upload adds .{} files to the server's plugins folder: a running server loads them in a few seconds, and a newer build of a plugin replaces the old one the same way.", files::LIBRARY));
                        });
                    }
                    Some(Pick::Plugin(path)) => {
                        if let Some(p) = listing.plugins.iter().find(|p| p.path == path) {
                            self.plugin_details(ui, i, p, up, &mut pick);
                        } else {
                            self.view(i).pick = None;
                        }
                    }
                    Some(Pick::File(path, owner)) => {
                        let owner = owner.and_then(|o| listing.plugins.iter().find(|p| p.path == o).cloned());
                        self.editor(ui, i, &path, owner.as_ref(), up);
                    }
                }
            });
        });

        if let Some(next) = pick {
            let dirty = self.view(i).editor.as_ref().is_some_and(|e| e.text != e.saved);
            if dirty {
                self.dialog = Some(Dialog::Discard { server: i, next });
            } else {
                self.open_pick(i, next);
            }
        }
    }

    fn open_pick(&mut self, i: usize, next: Pick) {
        let v = self.view(i);
        v.editor = match &next {
            Pick::File(path, _) => Some(match files::read_text(path) {
                Ok(text) => Editor { path: path.clone(), saved: text.clone(), text, error: None },
                Err(e) => Editor { path: path.clone(), text: String::new(), saved: String::new(), error: Some(e) },
            }),
            Pick::Plugin(_) => None,
        };
        v.pick = Some(next);
    }

    fn plugin_details(&mut self, ui: &mut Ui, i: usize, p: &files::Plugin, up: bool, pick: &mut Option<Pick>) {
        let s = self.servers[i].clone();
        card(ui, |ui| {
            ui.horizontal(|ui| {
                avatar(ui, &p.name, 40.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&p.name).family(theme::heading()).size(24.0));
                        if p.enabled {
                            pill(ui, "On", GREEN);
                        } else {
                            pill(ui, "Off", FAINT);
                        }
                    });
                    let mut about = Vec::new();
                    if let Some(v) = &p.version {
                        about.push(format!("version {v}"));
                    }
                    about.push(p.file.clone());
                    about.push(bytes(p.size));
                    about.push(format!("changed {}", files::ago(p.modified)));
                    muted(ui, &about.join("  ·  "));
                });
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if button(ui, "Reload now", Kind::Primary, up && p.enabled).on_hover_text("Turn it off and on again (/hotswap), changed or not").clicked() {
                    s.send(Act::ReloadPlugin(p.path.clone(), p.name.clone()));
                }
                if button(ui, "Replace with a new build…", Kind::Normal, true).clicked() {
                    if let Some(f) = rfd::FileDialog::new().add_filter("Plugin", &[files::LIBRARY]).set_title(format!("A new build of {}", p.name)).pick_file() {
                        let to = p.path.with_file_name(p.file.trim_end_matches(".disabled"));
                        match std::fs::copy(&f, &to) {
                            Ok(_) => {
                                if !p.enabled && to != p.path {
                                    let _ = std::fs::remove_file(&p.path);
                                }
                                s.notice(true, format!("{} replaced: {}", p.name, if up { "it reloads in a few seconds" } else { "it loads at the next start" }));
                                *pick = Some(Pick::Plugin(to));
                            }
                            Err(e) => s.notice(false, e.to_string()),
                        }
                        self.view(i).listing = None;
                    }
                }
                let (label, on) = if p.enabled { ("Turn off", false) } else { ("Turn on", true) };
                if button(ui, label, Kind::Normal, true).on_hover_text(if on { "Load it again" } else { "Unload it and keep the file (renamed .disabled)" }).clicked() {
                    match files::set_enabled(p, on) {
                        Ok(to) => {
                            s.notice(true, format!("{} turned {}", p.name, if on { "on" } else { "off" }));
                            *pick = Some(Pick::Plugin(to));
                        }
                        Err(e) => s.notice(false, e),
                    }
                    self.view(i).listing = None;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if button(ui, "Delete…", Kind::Danger, true).clicked() {
                        self.dialog = Some(Dialog::DeletePlugin { server: i, plugin: p.clone(), with_data: false });
                    }
                });
            });
        });
        ui.add_space(14.0);
        card(ui, |ui| {
            title(ui, "Config files");
            match &p.folder {
                None => muted(ui, &format!("{} has no data folder yet (plugins/{}): most plugins make one the first time they run.", p.name, p.name)),
                Some(folder) => {
                    let list: Vec<_> = files::tree(folder).into_iter().filter(|(_, _, d)| !d).collect();
                    if list.is_empty() {
                        muted(ui, "Its folder is empty.");
                    }
                    for (_, path, _) in list {
                        let rel = path.strip_prefix(folder).unwrap_or(&path).display().to_string();
                        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&rel).monospace());
                            ui.label(RichText::new(bytes(size)).color(FAINT).size(12.0));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if small_button(ui, "Edit", Kind::Normal).clicked() {
                                    *pick = Some(Pick::File(path.clone(), Some(p.path.clone())));
                                }
                            });
                        });
                    }
                }
            }
        });
    }

    fn editor(&mut self, ui: &mut Ui, i: usize, path: &std::path::Path, owner: Option<&files::Plugin>, up: bool) {
        let s = self.servers[i].clone();
        let rel = path.strip_prefix(&s.dir).unwrap_or(path).display().to_string();
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let height = ui.available_height();
        let v = self.view(i);
        let Some(ed) = v.editor.as_mut() else {
            return;
        };
        let dirty = ed.text != ed.saved;
        let mut save = ui.input_mut(|inp| inp.consume_key(egui::Modifiers::COMMAND, Key::S)) && dirty;
        let mut save_reload = false;
        let mut revert = false;
        surface(ui, false, radius::PANEL, Margin::same(8), |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(height - 2.0);
            // The bar: the file, and what can be done with it.
            Frame::new().inner_margin(Margin { left: 10, right: 4, top: 4, bottom: 8 }).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&rel).monospace().color(TEXT));
                    if dirty {
                        ui.label(RichText::new("●  unsaved").color(AMBER).size(12.0));
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(p) = owner {
                            if button(ui, "Save and reload", Kind::Primary, ed.error.is_none() && (dirty || up) && up && p.enabled).on_hover_text(format!("Save, then reload {} so it reads the change", p.name)).clicked() {
                                save_reload = true;
                            }
                        }
                        if button(ui, "Save", if owner.is_some() { Kind::Normal } else { Kind::Primary }, dirty && ed.error.is_none()).on_hover_text("Ctrl+S").clicked() {
                            save = true;
                        }
                        if dirty && button(ui, "Revert", Kind::Quiet, true).clicked() {
                            revert = true;
                        }
                    });
                });
            });
            let r = ui.cursor();
            ui.painter().line_segment([pos2(r.min.x, r.min.y), pos2(r.min.x + ui.available_width(), r.min.y)], Stroke::new(1.0, BORDER));
            if let Some(e) = &ed.error {
                Frame::new().inner_margin(Margin::same(16)).show(ui, |ui| {
                    ui.label(RichText::new(e).color(MUTED));
                });
                return;
            }
            // Line numbers beside the text, in the same scroll.
            let font = FontId::new(13.0, FontFamily::Monospace);
            let lines = ed.text.lines().count().max(1) + usize::from(ed.text.ends_with('\n'));
            surface(ui, true, radius::LARGE, Margin { left: 0, right: 8, top: 10, bottom: 8 }, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                ScrollArea::both().auto_shrink([false, false]).id_salt(("editor", path)).show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        let numbers: String = (1..=lines).map(|n| format!("{n:>4}  \n")).collect();
                        ui.add(egui::Label::new(RichText::new(numbers).font(font.clone()).color(FAINT)).selectable(false));
                        let ext2 = ext.clone();
                        let font2 = font.clone();
                        let mut layouter = move |ui: &Ui, text: &dyn egui::TextBuffer, _wrap: f32| {
                            let job = files::highlight(text.as_str(), &ext2, font2.clone());
                            ui.fonts_mut(|f| f.layout_job(job))
                        };
                        ui.add(
                            TextEdit::multiline(&mut ed.text)
                                .font(font.clone())
                                .code_editor()
                                .frame(Frame::NONE)
                                .desired_width(f32::INFINITY)
                                .desired_rows(lines.max(30))
                                .lock_focus(true)
                                .layouter(&mut layouter),
                        );
                    });
                });
            });
        });
        if revert {
            ed.text = ed.saved.clone();
        }
        if save || save_reload {
            match files::write_text(path, &ed.text) {
                Ok(()) => {
                    ed.saved = ed.text.clone();
                    s.notice(true, format!("Saved {}", path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()));
                    if save_reload {
                        if let Some(p) = owner {
                            s.send(Act::ReloadPlugin(p.path.clone(), p.name.clone()));
                        }
                    } else if rel.eq_ignore_ascii_case("server.toml") && up {
                        s.notice(true, "server.toml is read at the next start (Restart applies it)");
                    }
                }
                Err(e) => s.notice(false, format!("Couldn't save: {e}")),
            }
        }
    }

    fn backups(&mut self, ui: &mut Ui, i: usize) {
        let s = self.servers[i].clone();
        if !self.view(i).asked_backups {
            self.view(i).asked_backups = true;
            s.send(Act::LoadBackups);
        }
        let list = s.state.lock().unwrap().backups.clone();
        let busy = s.state.lock().unwrap().busy.is_some();
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        title(ui, "Backups");
                        muted(ui, "A backup zips the world and the settings into the server's backups folder. A running server saves first and stays up.");
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button(ui, "Back up now", Kind::Primary, !busy).clicked() {
                            s.send(Act::Backup);
                        }
                    });
                });
                ui.add_space(6.0);
                match &list {
                    None => muted(ui, "Loading…"),
                    Some(l) if l.is_empty() => muted(ui, "No backups yet."),
                    Some(l) => {
                        for (n, (name, size)) in l.iter().rev().enumerate() {
                            if n > 0 {
                                ui.separator();
                            }
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(name).monospace());
                                ui.label(RichText::new(bytes(*size)).color(MUTED));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if small_button(ui, "Restore", Kind::Normal).clicked() {
                                        self.dialog = Some(Dialog::Restore { server: i, name: name.clone() });
                                    }
                                });
                            });
                        }
                    }
                }
            });
        });
    }

    fn settings(&mut self, ui: &mut Ui, i: usize) {
        let s = self.servers[i].clone();
        if !self.view(i).asked_config {
            self.view(i).asked_config = true;
            s.send(Act::LoadConfig);
        }
        let loaded = s.state.lock().unwrap().config.clone();
        if let Some(c) = &loaded {
            let fresh = Settings::from(c);
            let v = self.view(i);
            match &mut v.settings {
                Some((edit, old)) if *old != fresh => {
                    *edit = fresh.clone();
                    *old = fresh;
                }
                None => v.settings = Some((fresh.clone(), fresh)),
                _ => {}
            }
        }
        let busy = s.state.lock().unwrap().busy.is_some();
        let gui = self.registry.find(&s.dir).map(|e| e.gui).unwrap_or(true);
        let mut new_gui = gui;
        let mut remove = false;
        let dir = s.dir.clone();
        let net_names: Vec<String> = self.registry.networks.iter().map(|g| g.name.clone()).collect();
        let net_before = self.network_of(i);
        let mut net_choice = net_before.clone();
        let v = self.view(i);
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            card(ui, |ui| {
                title(ui, "Server");
                match &mut v.settings {
                    None => muted(ui, "Loading…"),
                    Some((edit, old)) => {
                        let changes = edit.changes(old);
                        setting_row(ui, "Message of the day", "under the name in the server list", |ui| {
                            ui.add(field(&mut edit.motd, "A Minecraft Server").desired_width(360.0));
                        });
                        setting_row(ui, "Port", "players connect to localhost:port", |ui| {
                            ui.add(field(&mut edit.port, "25565").desired_width(120.0));
                        });
                        setting_row(ui, "Player slots", "how many can be online at once", |ui| {
                            ui.add(field(&mut edit.max_players, "20").desired_width(120.0));
                        });
                        setting_row(ui, "Difficulty", "", |ui| {
                            combo(ui, "difficulty", &mut edit.difficulty, &["peaceful", "easy", "normal", "hard"]);
                        });
                        setting_row(ui, "Game mode", "for new players", |ui| {
                            combo(ui, "gamemode", &mut edit.game_mode, &["survival", "creative", "adventure", "spectator"]);
                        });
                        setting_row(ui, "View distance", "chunks sent around each player", |ui| {
                            ui.add(field(&mut edit.view_distance, "10").desired_width(120.0));
                        });
                        setting_row(ui, "Simulation distance", "chunks that tick around each player", |ui| {
                            ui.add(field(&mut edit.simulation_distance, "10").desired_width(120.0));
                        });
                        setting_row(ui, "Online mode", "check players' Minecraft accounts", |ui| {
                            ui.add(toggle(&mut edit.online_mode));
                        });
                        setting_row(ui, "Whitelist", "only listed players can join", |ui| {
                            ui.add(toggle(&mut edit.whitelist));
                        });
                        setting_row(ui, "Hardcore", "one life", |ui| {
                            ui.add(toggle(&mut edit.hardcore));
                        });
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if button(ui, "Save changes", Kind::Primary, !changes.is_empty() && !busy).clicked() {
                                s.send(Act::SaveConfig(changes.clone()));
                            }
                            if !changes.is_empty() && button(ui, "Discard", Kind::Quiet, true).clicked() {
                                *edit = old.clone();
                            }
                            if !changes.is_empty() {
                                muted(ui, &format!("{} unsaved {}", changes.len(), if changes.len() == 1 { "change" } else { "changes" }));
                            }
                        });
                    }
                }
            });
            ui.add_space(14.0);
            card(ui, |ui| {
                title(ui, "Panel");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Network");
                        muted(ui, "Group it with other servers, for example behind the same proxy.");
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let shown = net_choice.clone().unwrap_or_else(|| "None".into());
                        egui::ComboBox::from_id_salt("network").selected_text(shown).width(200.0).show_ui(ui, |ui| {
                            ui.selectable_value(&mut net_choice, None, "None");
                            for g in &net_names {
                                ui.selectable_value(&mut net_choice, Some(g.clone()), g.as_str());
                            }
                        });
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Open this panel when the server starts");
                        muted(ui, "Off runs it headless: python setup.py start opens no window (the same as --nogui).");
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(toggle(&mut new_gui));
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Folder");
                        ui.label(RichText::new(dir.display().to_string()).monospace().color(MUTED));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if small_button(ui, "Remove from list", Kind::Danger).clicked() {
                            remove = true;
                        }
                        if small_button(ui, "Open folder", Kind::Normal).clicked() {
                            open_folder(&dir);
                        }
                    });
                });
            });
        });
        if new_gui != gui {
            if let Some(e) = self.registry.find_mut(&s.dir) {
                e.gui = new_gui;
            }
            self.registry.save();
        }
        if remove {
            self.dialog = Some(Dialog::Remove { server: i });
        }
        if net_choice != net_before {
            self.move_to(i, net_choice);
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        let mut keep = true;
        let modal = egui::Modal::new(egui::Id::new("dialog")).backdrop_color(theme::SCRIM).frame(
            Frame::new().fill(Color32::from_rgb(0x1f, 0x1b, 0x34)).corner_radius(radius::MODAL).inner_margin(Margin::same(28)).shadow(egui::epaint::Shadow { offset: [0, 18], blur: 48, spread: 0, color: Color32::from_black_alpha(120) }),
        );
        let response = modal.show(ctx, |ui| {
            ui.set_width(420.0);
            match &mut dialog {
                Dialog::Kick { server, name, reason } => {
                    let s = self.servers.get(*server).cloned();
                    keep = reason_dialog(ui, &format!("Kick {name}?"), "They're disconnected and can join again.", reason, "Kick", |reason| {
                        if let Some(s) = &s {
                            s.send(Act::Player("players/kick", reason_body(json!({ "name": name.clone() }), reason)));
                        }
                    });
                }
                Dialog::Ban { server, name, reason } => {
                    let s = self.servers.get(*server).cloned();
                    keep = reason_dialog(ui, &format!("Ban {name}?"), "They're disconnected if online and can't join until unbanned (Players tab).", reason, "Ban", |reason| {
                        if let Some(s) = &s {
                            s.send(Act::Player("players/ban", reason_body(json!({ "name": name.clone() }), reason)));
                        }
                    });
                }
                Dialog::BanIp { server, name, reason } => {
                    let s = self.servers.get(*server).cloned();
                    keep = reason_dialog(ui, &format!("Ban {name}'s address?"), "Everyone on that address is disconnected and can't join until it's unbanned.", reason, "Ban address", |reason| {
                        if let Some(s) = &s {
                            s.send(Act::Player("players/ban-ip", reason_body(json!({ "name": name.clone() }), reason)));
                        }
                    });
                }
                Dialog::Stop { server } => {
                    let s = self.servers.get(*server).cloned();
                    ui.label(RichText::new(format!("Stop {}?", s.as_ref().map(|s| s.name.as_str()).unwrap_or("this server"))).family(theme::heading()).size(24.0));
                    muted(ui, "The world is saved and players are disconnected. You can start it again from here.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "Stop server", Kind::Danger, true).clicked() {
                            if let Some(s) = &s {
                                s.send(Act::Stop);
                            }
                            keep = false;
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::Restore { server, name } => {
                    ui.label(RichText::new("Restore this backup?").family(theme::heading()).size(24.0));
                    muted(ui, &format!("{name} replaces the current world. The current world is kept in backups first. A running server restarts with it."));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "Restore", Kind::Danger, true).clicked() {
                            if let Some(s) = self.servers.get(*server) {
                                s.send(Act::Restore(name.clone()));
                            }
                            keep = false;
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::Remove { server } => {
                    ui.label(RichText::new("Remove from the list?").family(theme::heading()).size(24.0));
                    muted(ui, "The panel stops showing this server. Nothing is deleted: the folder and the world stay as they are, and a running server keeps running. Add the folder again any time.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "Remove", Kind::Danger, true).clicked() {
                            if let Some(s) = self.servers.get(*server) {
                                self.registry.remove(&s.dir);
                                self.registry.save();
                                self.selected = None;
                                self.synced = Instant::now() - Duration::from_secs(10);
                            }
                            keep = false;
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::NewNetwork { name, server } => {
                    ui.label(RichText::new("New network").family(theme::heading()).size(24.0));
                    muted(ui, "A folder for servers that belong together, like the lobby, survival and minigames behind one proxy.");
                    ui.add_space(8.0);
                    let r = ui.add(field(name, "My Network").desired_width(f32::INFINITY));
                    r.request_focus();
                    let taken = self.registry.networks.iter().any(|g| g.name == name.trim());
                    if taken {
                        ui.label(RichText::new("There's a network with that name already.").color(RED));
                    }
                    let ok = !name.trim().is_empty() && !taken;
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if (button(ui, "Create", Kind::Primary, ok).clicked() || enter) && ok {
                            let n = name.trim().to_owned();
                            self.registry.networks.push(registry::Network { name: n.clone(), collapsed: false });
                            if let Some(i) = *server {
                                let dir = self.servers[i].dir.clone();
                                if let Some(e) = self.registry.find_mut(&dir) {
                                    e.network = Some(n.clone());
                                }
                            }
                            self.registry.save();
                            self.network = Some(n);
                            self.selected = None;
                            keep = false;
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::RenameNetwork { old, name } => {
                    ui.label(RichText::new("Rename network").family(theme::heading()).size(24.0));
                    ui.add_space(8.0);
                    let r = ui.add(field(name, "Network name").desired_width(f32::INFINITY));
                    r.request_focus();
                    let ok = !name.trim().is_empty() && !self.registry.networks.iter().any(|g| g.name == name.trim());
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if (button(ui, "Rename", Kind::Primary, ok).clicked() || enter) && ok {
                            self.registry.rename_network(old, name.trim());
                            self.registry.save();
                            if self.network.as_deref() == Some(old.as_str()) {
                                self.network = Some(name.trim().to_owned());
                            }
                            keep = false;
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::DeleteNetwork { name } => {
                    ui.label(RichText::new(format!("Delete the network {name}?")).family(theme::heading()).size(24.0));
                    muted(ui, "Only the folder goes: its servers keep running and move back to the main list.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "Delete network", Kind::Danger, true).clicked() {
                            self.registry.delete_network(name);
                            self.registry.save();
                            if self.network.as_deref() == Some(name.as_str()) {
                                self.network = None;
                            }
                            keep = false;
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::StopAll { network } => {
                    ui.label(RichText::new(format!("Stop every server in {network}?")).family(theme::heading()).size(24.0));
                    muted(ui, "Each world is saved and its players are disconnected. Start them again with Start all.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "Stop all", Kind::Danger, true).clicked() {
                            for i in self.members(network) {
                                if self.servers[i].state.lock().unwrap().link == Some(Link::Up) {
                                    self.servers[i].send(Act::Stop);
                                }
                            }
                            keep = false;
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::BanNetwork { network, name, reason } => {
                    let members = self.members(network);
                    let servers = self.servers.clone();
                    keep = reason_dialog(ui, &format!("Ban {name} on every server?"), &format!("Banned on all running servers in {network} and kicked wherever they are online."), reason, "Ban everywhere", |reason| {
                        for &i in &members {
                            if servers[i].state.lock().unwrap().link == Some(Link::Up) {
                                servers[i].send(Act::Player("players/ban", reason_body(json!({ "name": name.clone() }), reason)));
                            }
                        }
                    });
                }
                Dialog::DeletePlugin { server, plugin, with_data } => {
                    ui.label(RichText::new(format!("Delete {}?", plugin.name)).family(theme::heading()).size(24.0));
                    muted(ui, &format!("{} is deleted from the plugins folder and a running server unloads it. This can't be undone.", plugin.file));
                    if let Some(folder) = &plugin.folder {
                        ui.add_space(6.0);
                        ui.checkbox(with_data, format!("Also delete its data folder (plugins/{})", folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()));
                    }
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "Delete", Kind::Danger, true).clicked() {
                            if let Some(s) = self.servers.get(*server).cloned() {
                                match std::fs::remove_file(&plugin.path) {
                                    Ok(()) => {
                                        if *with_data {
                                            if let Some(folder) = &plugin.folder {
                                                let _ = std::fs::remove_dir_all(folder);
                                            }
                                        }
                                        s.notice(true, format!("Deleted {}", plugin.name));
                                    }
                                    Err(e) => s.notice(false, format!("Couldn't delete it: {e}")),
                                }
                                let v = self.view(*server);
                                v.pick = None;
                                v.editor = None;
                                v.listing = None;
                            }
                            keep = false;
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::Discard { server, next } => {
                    ui.label(RichText::new("Discard your changes?").family(theme::heading()).size(24.0));
                    muted(ui, "This file has edits that aren't saved.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "Discard", Kind::Danger, true).clicked() {
                            let (server, next) = (*server, next.clone());
                            self.open_pick(server, next);
                            keep = false;
                        }
                        if button(ui, "Keep editing", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::Add { path, error } => {
                    ui.label(RichText::new("Add a server folder").family(theme::heading()).size(24.0));
                    muted(ui, "A folder made by  python setup.py server  (it has a server.toml).");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add(field(path, "D:\\servers\\survival").desired_width(300.0));
                        if small_button(ui, "Browse…", Kind::Normal).clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                *path = p.display().to_string();
                            }
                        }
                    });
                    if let Some(e) = error {
                        ui.label(RichText::new(e.as_str()).color(RED));
                    }
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "Add", Kind::Primary, !path.trim().is_empty()).clicked() {
                            let dir = PathBuf::from(path.trim());
                            if dir.join("server.toml").is_file() {
                                self.registry.add(&dir, None);
                                self.registry.save();
                                self.want = Some(dir);
                                self.synced = Instant::now() - Duration::from_secs(10);
                                keep = false;
                            } else {
                                *error = Some("There's no server.toml in that folder.".into());
                            }
                        }
                        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
                            keep = false;
                        }
                    });
                }
            }
        });
        if response.should_close() {
            keep = false;
        }
        if keep {
            self.dialog = Some(dialog);
        }
    }

    fn toasts(&mut self, ctx: &egui::Context) {
        let list: Vec<(String, crate::server::Notice)> = self
            .servers
            .iter()
            .flat_map(|s| {
                let state = s.state.lock().unwrap();
                state.notices.iter().filter(|n| n.at.elapsed() < Duration::from_secs(5)).map(|n| (s.name.clone(), n.clone())).collect::<Vec<_>>()
            })
            .collect();
        if list.is_empty() {
            return;
        }
        egui::Area::new(egui::Id::new("toasts")).anchor(Align2::RIGHT_BOTTOM, vec2(-20.0, -20.0)).order(egui::Order::Foreground).show(ctx, |ui| {
            for (name, n) in list.iter().rev().take(4) {
                Frame::new().fill(Color32::from_rgb(0x1b, 0x20, 0x28)).stroke(Stroke::new(1.0, if n.ok { BORDER } else { RED.linear_multiply(0.5) })).corner_radius(10).inner_margin(Margin::symmetric(14, 10)).show(ui, |ui| {
                    ui.set_max_width(380.0);
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                        ui.painter().circle_filled(rect.center(), 4.0, if n.ok { GREEN } else { RED });
                        ui.label(RichText::new(name).family(theme::bold()).size(13.0));
                        ui.label(RichText::new(&n.text).size(13.0));
                    });
                });
                ui.add_space(8.0);
            }
        });
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

fn reason_body(mut body: Value, reason: &str) -> Value {
    if !reason.trim().is_empty() {
        body["reason"] = reason.trim().into();
    }
    body
}

/// A yes-or-cancel question with an optional reason; false once answered.
fn reason_dialog(ui: &mut Ui, heading: &str, text: &str, reason: &mut String, yes: &str, on_yes: impl FnOnce(&str)) -> bool {
    ui.label(RichText::new(heading).family(theme::heading()).size(24.0));
    muted(ui, text);
    ui.add_space(8.0);
    ui.label(RichText::new("Reason (optional, shown to them)").color(MUTED).size(12.5));
    let r = ui.add(field(reason, "Breaking the rules").desired_width(f32::INFINITY));
    r.request_focus();
    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    ui.add_space(12.0);
    let mut open = true;
    ui.horizontal(|ui| {
        if button(ui, yes, Kind::Danger, true).clicked() || enter {
            on_yes(reason);
            open = false;
        }
        if button(ui, "Cancel", Kind::Quiet, true).clicked() {
            open = false;
        }
    });
    open
}

// ---------- widgets ----------

/// A sidebar row (`glass_ui::row`): a 40 px tile, the name (14 px semibold) and its state after a dot
/// (12.5 px at 65 %); white at 10 % when it is the one shown, 5 % under the pointer.
fn row(ui: &mut Ui, dot: Option<Color32>, name: &str, sub: &str, selected: bool, _sub_color: Color32, indent: f32) -> Response {
    sidebar_row(ui, if dot.is_some() { "computer-terminal-01" } else { "dashboard-square-01" }, name, dot.map(|d| (d, sub)).or(Some((Color32::TRANSPARENT, sub))), selected, indent, dot.is_none())
}

fn stat(ui: &mut Ui, label: &str, value: &str, hint: &str, color: Color32) {
    card(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.label(RichText::new(label).color(MUTED).size(13.0));
        ui.label(RichText::new(value).family(theme::display()).size(30.0).color(color));
        ui.label(RichText::new(hint).color(FAINT).size(12.0));
    });
}

fn stat_small(ui: &mut Ui, label: &str, value: &str, color: Color32) {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.label(RichText::new(label).color(FAINT).size(12.0));
    ui.label(RichText::new(value).family(theme::bold()).size(17.0).color(color));
}

fn label_cell(ui: &mut Ui, label: &str, hint: &str) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.label(RichText::new(label).family(theme::medium()).size(14.0).color(INK));
        if !hint.is_empty() {
            ui.label(RichText::new(hint).color(QUIET).size(12.5));
        }
    });
}

fn combo(ui: &mut Ui, id: &str, value: &mut String, options: &[&str]) {
    let cap = |s: &str| {
        let mut s = s.to_owned();
        if let Some(f) = s.get_mut(0..1) {
            f.make_ascii_uppercase();
        }
        s
    };
    egui::ComboBox::from_id_salt(id).selected_text(RichText::new(cap(value)).color(INK)).width(200.0).height(260.0).show_ui(ui, |ui| {
        for o in options {
            ui.selectable_value(value, o.to_string(), cap(o));
        }
    });
}

/// The launcher's switch: a 40 × 24 track, white at 14 % off, the accent on; the knob slides.
fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut Ui| {
        let size = vec2(40.0, 24.0);
        let (rect, mut r) = ui.allocate_exact_size(size, Sense::click());
        if r.clicked() {
            *on = !*on;
            r.mark_changed();
        }
        let t = ui.ctx().animate_bool_with_time(r.id, *on, 0.2);
        let off = theme::BUTTON_HOVER;
        let bg = lerp_color(off, ACCENT, t);
        ui.painter().rect_filled(rect, CornerRadius::same(255), bg);
        let x = egui::lerp((rect.min.x + 12.0)..=(rect.max.x - 12.0), t);
        ui.painter().circle_filled(pos2(x, rect.center().y), 9.0, Color32::WHITE);
        if r.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        r
    }
}

fn list_card(ui: &mut Ui, heading: &str, empty: &str, items: &[Value], show: impl Fn(&Value) -> (String, String), action: &str, mut on: impl FnMut(String)) {
    card(ui, |ui| {
        title(ui, heading);
        ui.add_space(2.0);
        if items.is_empty() {
            muted(ui, empty);
        }
        for e in items {
            let (name, detail) = show(e);
            ui.horizontal(|ui| {
                avatar(ui, &name, 28.0);
                ui.label(RichText::new(&name).family(theme::medium()));
                if !detail.is_empty() {
                    ui.label(RichText::new(&detail).color(QUIET).size(12.5));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if small_button(ui, action, Kind::Normal).clicked() {
                        on(name.clone());
                    }
                });
            });
        }
    });
}

fn chat_line(ui: &mut Ui, c: &crate::server::ChatItem) {
    chat_line_on(ui, c, None);
}

/// One chat line: the time, (on a network) the server, who, and what.
fn chat_line_on(ui: &mut Ui, c: &crate::server::ChatItem, server: Option<&str>) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.label(RichText::new(&c.time).color(FAINT).size(12.5).monospace());
        if let Some(server) = server {
            ui.label(RichText::new(server).color(MUTED).size(12.5).family(theme::medium()));
            ui.label(RichText::new("·").color(FAINT));
        }
        match c.kind {
            ChatKind::Chat => {
                ui.label(RichText::new(&c.name).family(theme::bold()).color(name_color(&c.name)));
                ui.label(RichText::new(&c.text).color(INK));
            }
            ChatKind::Server => {
                ui.label(RichText::new("[Server]").family(theme::bold()).color(ACCENT_SOFT));
                ui.label(RichText::new(&c.text).color(INK));
            }
            ChatKind::Join | ChatKind::Leave => {
                let color = if matches!(c.kind, ChatKind::Join) { LIVE } else { QUIET };
                ui.label(RichText::new(format!("{} {}", c.name, c.text)).color(color));
            }
            ChatKind::Death => {
                ui.label(RichText::new(format!("{} {}", c.name, c.text)).color(WARN));
            }
            ChatKind::Kick | ChatKind::Ban => {
                ui.label(RichText::new(format!("{} {}", c.name, c.text)).color(DANGER));
            }
        }
    });
}

fn list_heading(ui: &mut Ui, text: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let mut t = text.to_lowercase();
        if let Some(f) = t.get_mut(0..1) {
            f.make_ascii_uppercase();
        }
        ui.label(RichText::new(t).color(QUIET).size(13.0).family(theme::medium()));
    });
    ui.add_space(2.0);
}

/// A line of the Plugins tab's list: a plugin (a tile, its name and state) or a file in its folder.
fn file_row(ui: &mut Ui, name: &str, sub: Option<&str>, depth: usize, dot: Option<Color32>, selected: bool, folder: bool) -> Response {
    if let Some(sub) = sub {
        return sidebar_row(ui, "puzzle", name, Some((dot.unwrap_or(FAINT), sub)), selected, 0.0, false);
    }
    let w = ui.available_width();
    let (rect, r) = ui.allocate_exact_size(vec2(w, 32.0), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, radius::CONTROL, theme::SELECTED);
    } else if r.hovered() {
        p.rect_filled(rect, radius::CONTROL, theme::HOVER);
    }
    let x = rect.min.x + 14.0 + depth as f32 * 16.0;
    let icon = if folder { "folder-01" } else { "pencil-edit-02" };
    theme::paint_icon(ui, icon, Rect::from_center_size(pos2(x + 8.0, rect.center().y), vec2(15.0, 15.0)), if selected { INK } else { FAINT });
    let color = if selected { INK } else if folder { MUTED } else { BODY };
    let g = one_line(p, name, FontId::new(13.5, if selected { theme::medium() } else { FontFamily::Proportional }), color, rect.max.x - x - 30.0);
    p.galley(pos2(x + 24.0, rect.center().y - g.size().y / 2.0), g, color);
    if r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    r
}

fn section(ui: &mut Ui, text: &str) {
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(RichText::new(text).color(QUIET).size(13.0).family(theme::medium()));
    });
    ui.add_space(2.0);
}

/// A network in the sidebar: a row with a folder tile, and an arrow at its right that opens and
/// closes it.
fn folder_row(ui: &mut Ui, name: &str, sub: &str, selected: bool, collapsed: bool) -> (Response, Response) {
    let r = sidebar_row(ui, "folder-01", name, Some((Color32::TRANSPARENT, sub)), selected, 0.0, true);
    let arrow_rect = Rect::from_center_size(pos2(r.rect.max.x - 22.0, r.rect.center().y), vec2(28.0, 28.0));
    let arrow = ui.interact(arrow_rect, r.id.with("arrow"), Sense::click());
    if arrow.hovered() {
        ui.painter().rect_filled(arrow_rect, radius::CONTROL - 1, theme::BUTTON);
    }
    theme::paint_icon(ui, if collapsed { "arrow-right-01" } else { "arrow-down-01" }, Rect::from_center_size(arrow_rect.center(), vec2(16.0, 16.0)), if arrow.hovered() { INK } else { MUTED });
    (r, arrow)
}

/// How many of these servers run, and their players.
fn totals(servers: &[Arc<Server>], list: &[usize]) -> (usize, usize) {
    let running = list.iter().filter(|&&i| servers[i].state.lock().unwrap().link == Some(Link::Up)).count();
    let players = list.iter().map(|&i| numbers(&servers[i]).players).sum();
    (running, players)
}

/// Five numbers for a set of servers, on one plate.
fn summary(ui: &mut Ui, servers: &[Arc<Server>], list: &[usize]) {
    let (running, players) = totals(servers, list);
    let nums: Vec<Numbers> = list.iter().map(|&i| numbers(&servers[i])).collect();
    let ram: f64 = nums.iter().filter(|n| n.up).filter_map(|n| n.ram_mb).sum();
    let slots: i64 = nums.iter().filter(|n| n.up).filter_map(|n| n.max_players).sum();
    let low = nums.iter().filter(|n| n.up).filter_map(|n| n.tps).fold(None, |m: Option<f64>, t| Some(m.map_or(t, |m| m.min(t))));
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 96.0), Sense::hover());
    let p = ui.painter();
    p.add(theme::plate_shape(rect, radius::PANEL, false));
    let cells = [
        ("Servers running", format!("{running} of {}", list.len()), if running == list.len() && running > 0 { LIVE } else { INK }),
        ("Players online", format!("{players}"), INK),
        ("Player slots", format!("{slots}"), INK),
        ("Memory in use", if ram > 0.0 { chart::short(ram, "MB") } else { "–".into() }, INK),
        ("Lowest TPS", low.map(|t| format!("{t:.1}")).unwrap_or("–".into()), low.map(tps_color).unwrap_or(INK)),
    ];
    let w = rect.width() / cells.len() as f32;
    for (k, (label, value, color)) in cells.iter().enumerate() {
        let x = rect.min.x + k as f32 * w;
        if k > 0 {
            p.line_segment([pos2(x, rect.min.y + 22.0), pos2(x, rect.max.y - 22.0)], Stroke::new(1.0, LINE));
        }
        p.text(pos2(x + 24.0, rect.min.y + 20.0), Align2::LEFT_TOP, *label, FontId::new(13.0, FontFamily::Proportional), MUTED);
        p.text(pos2(x + 24.0, rect.min.y + 42.0), Align2::LEFT_TOP, value, FontId::new(28.0, theme::display()), *color);
    }
}

fn page_title(ui: &mut Ui, text: &str, pill_text: Option<(&str, Color32)>) {
    if let Some((t, c)) = pill_text {
        pill(ui, t, c);
    }
    ui.label(RichText::new(text).family(theme::display()).size(52.0).color(INK));
}

/// The launcher's tab strip (`Tabs`, size md) on its own plate: 36 px tabs, the open one white on
/// 12 %, medium.
fn tab_bar(ui: &mut Ui, labels: &[&str], at: &mut usize) {
    let font = FontId::new(15.0, FontFamily::Proportional);
    let widths: Vec<f32> = labels.iter().map(|l| ui.painter().layout_no_wrap(l.to_string(), font.clone(), INK).size().x + 28.0).collect();
    let total: f32 = widths.iter().sum::<f32>() + 8.0 + 2.0 * (labels.len() as f32 - 1.0);
    let (rect, _) = ui.allocate_exact_size(vec2(total, 44.0), Sense::hover());
    ui.painter().add(theme::plate_shape(rect, radius::LARGE, false));
    let mut x = rect.min.x + 4.0;
    for (k, label) in labels.iter().enumerate() {
        let tab = Rect::from_min_size(pos2(x, rect.min.y + 4.0), vec2(widths[k], 36.0));
        let r = ui.interact(tab, ui.id().with(("tab", k, *label)), Sense::click());
        let open = k == *at;
        if open {
            ui.painter().rect_filled(tab, radius::CONTROL, ACTIVE);
        } else if r.hovered() {
            ui.painter().rect_filled(tab, radius::CONTROL, theme::HOVER);
        }
        let f = FontId::new(15.0, if open { theme::medium() } else { FontFamily::Proportional });
        ui.painter().text(tab.center(), Align2::CENTER_CENTER, *label, f, if open || r.hovered() { INK } else { BODY });
        if r.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if r.clicked() {
            *at = k;
        }
        x += widths[k] + 2.0;
    }
}

fn pill_size(p: &egui::Painter, text: &str) -> f32 {
    p.layout_no_wrap(text.to_owned(), FontId::new(13.0, FontFamily::Proportional), TEXT).size().x + 15.0
}

fn open_folder(dir: &std::path::Path) {
    let program = if cfg!(windows) { "explorer" } else if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    let _ = std::process::Command::new(program).arg(dir).spawn();
}

// ---------- glass widgets (the launcher's glass_ui.rs) ----------

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()), l(a.a(), b.a()))
}

/// A button as the launcher draws them: `Primary` is the white one (`solid`, dark text), `Accent`
/// the violet one, `Normal` the quiet one (white at 8 %, 14 % under the pointer), `Danger` quiet with
/// red words, `Quiet` words alone. `h` tall, `size` px text, medium; at 40 % when off.
fn glass_button(ui: &mut Ui, icon: Option<&str>, text: &str, kind: Kind, enabled: bool, h: f32, size: f32) -> Response {
    let font = FontId::new(size, theme::medium());
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, INK);
    let icon_w = if icon.is_some() { 16.0 + 8.0 } else { 0.0 };
    let pad = if h >= 40.0 { 18.0 } else { 14.0 };
    let w = (galley.size().x + icon_w + pad * 2.0).max(h);
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, r) = ui.allocate_exact_size(vec2(w, h), sense);
    let hovered = enabled && r.hovered();
    let (fill, fg) = match kind {
        Kind::Primary => (if hovered { Color32::from_rgb(0xe8, 0xe6, 0xf0) } else { Color32::WHITE }, theme::ON_LIGHT),
        Kind::Accent => (if hovered { Color32::from_rgb(0x7c, 0x60, 0xff) } else { ACCENT }, Color32::WHITE),
        Kind::Normal => (if hovered { theme::BUTTON_HOVER } else { theme::BUTTON }, INK),
        Kind::Danger => (if hovered { Color32::from_rgba_unmultiplied(0xff, 0x6b, 0x6b, 31) } else { theme::BUTTON }, DANGER),
        Kind::Quiet => (if hovered { theme::HOVER } else { Color32::TRANSPARENT }, if hovered { INK } else { BODY }),
    };
    let alpha = if enabled { 1.0 } else { 0.4 };
    ui.painter().rect_filled(rect, radius::CONTROL, fill.gamma_multiply(alpha));
    let fg = fg.gamma_multiply(alpha);
    let mut x = rect.center().x - (galley.size().x + icon_w) / 2.0;
    if let Some(name) = icon {
        theme::paint_icon(ui, name, Rect::from_min_size(pos2(x, rect.center().y - 8.0), vec2(16.0, 16.0)), fg);
        x += icon_w;
    }
    ui.painter().galley_with_override_text_color(pos2(x, rect.center().y - galley.size().y / 2.0), galley, fg);
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    r
}

/// A square button with one icon (`icon_button`): white at 65 %, white with an 8 % fill under the pointer.
fn icon_button(ui: &mut Ui, name: &str, size: f32, tip: &str) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    if r.hovered() {
        ui.painter().rect_filled(rect, radius::CONTROL - 1, theme::BUTTON);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let s = (size * 0.55).round();
    theme::paint_icon(ui, name, Rect::from_center_size(rect.center(), vec2(s, s)), if r.hovered() { INK } else { MUTED });
    r.on_hover_text(tip)
}

/// A sidebar row: `p-2`, a 40 px tile with an icon (white at 8 %), the name and its state.
fn sidebar_row(ui: &mut Ui, icon: &str, name: &str, state: Option<(Color32, &str)>, selected: bool, indent: f32, accent_tile: bool) -> Response {
    let w = ui.available_width();
    let (full, r) = ui.allocate_exact_size(vec2(w, 56.0), Sense::click());
    let rect = Rect::from_min_max(pos2(full.min.x + indent, full.min.y), full.max);
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, radius::ROW, theme::SELECTED);
    } else if r.hovered() {
        p.rect_filled(rect, radius::ROW, theme::HOVER);
    }
    let tile = Rect::from_min_size(pos2(rect.min.x + 8.0, rect.min.y + 8.0), vec2(40.0, 40.0));
    p.rect_filled(tile, radius::CONTROL, if accent_tile { ACCENT.linear_multiply(0.30) } else { theme::BUTTON });
    theme::paint_icon(ui, icon, Rect::from_center_size(tile.center(), vec2(20.0, 20.0)), if accent_tile { ACCENT_SOFT } else { STRONG });
    let tx = tile.max.x + 12.0;
    let max = rect.max.x - tx - 34.0;
    let g = one_line(p, name, FontId::new(14.0, theme::bold()), INK, max);
    p.galley(pos2(tx, rect.min.y + 8.0), g, INK);
    if let Some((dot, sub)) = state {
        let mut sx = tx;
        if dot != Color32::TRANSPARENT {
            p.circle_filled(pos2(sx + 3.5, rect.min.y + 38.0), 3.5, dot);
            sx += 13.0;
        }
        let g = one_line(p, sub, FontId::new(12.5, FontFamily::Proportional), MUTED, rect.max.x - sx - 14.0);
        p.galley(pos2(sx, rect.min.y + 29.0), g, MUTED);
    }
    if r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    r
}

/// A sidebar section's name, with a + at its right when `plus` is set.
fn sidebar_section(ui: &mut Ui, text: &str, plus: Option<&str>) -> Option<Response> {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, 34.0), Sense::hover());
    ui.painter().text(pos2(rect.min.x + 8.0, rect.center().y), Align2::LEFT_CENTER, text, FontId::new(13.0, theme::medium()), QUIET);
    plus.map(|tip| {
        let b = Rect::from_center_size(pos2(rect.max.x - 16.0, rect.center().y), vec2(28.0, 28.0));
        let r = ui.interact(b, ui.id().with(("plus", text)), Sense::click());
        if r.hovered() {
            ui.painter().rect_filled(b, radius::CONTROL - 1, theme::BUTTON);
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        theme::paint_icon(ui, "plus-sign", Rect::from_center_size(b.center(), vec2(16.0, 16.0)), if r.hovered() { INK } else { MUTED });
        r.on_hover_text(tip)
    })
}

/// A quiet row with an icon (the launcher's "Create a server", "Add a server folder…").
fn action_row(ui: &mut Ui, icon: &str, text: &str) -> Response {
    let w = ui.available_width();
    let (rect, r) = ui.allocate_exact_size(vec2(w, 40.0), Sense::click());
    if r.hovered() {
        ui.painter().rect_filled(rect, radius::ROW, theme::HOVER);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    theme::paint_icon(ui, icon, Rect::from_center_size(pos2(rect.min.x + 20.0, rect.center().y), vec2(17.0, 17.0)), if r.hovered() { INK } else { STRONG });
    ui.painter().text(pos2(rect.min.x + 40.0, rect.center().y), Align2::LEFT_CENTER, text, FontId::new(14.0, FontFamily::Proportional), if r.hovered() { INK } else { STRONG });
    r
}

/// The window's own top line (the launcher draws its title bar on the scene): the mark and the
/// name, the window's buttons at the right; drag it to move the window.
fn title_bar(ui: &mut Ui, rect: Rect) {
    let ctx = ui.ctx().clone();
    let drag = ui.interact(rect, ui.id().with("title bar"), Sense::click_and_drag());
    if drag.drag_started() {
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    if drag.double_clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
    }
    let p = ui.painter();
    egui::Image::new(egui::include_image!("../assets/horizon-64.png")).paint_at(ui, Rect::from_min_size(pos2(rect.min.x + 14.0, rect.center().y - 8.0), vec2(16.0, 16.0)));
    p.text(pos2(rect.min.x + 38.0, rect.center().y), Align2::LEFT_CENTER, "Horizon Servers", FontId::new(13.0, theme::medium()), MUTED);
    // Minimise, maximise, close: 46 px each, as Windows draws them.
    let mut x = rect.max.x;
    for k in 0..3 {
        x -= 46.0;
        let b = Rect::from_min_size(pos2(x, rect.min.y), vec2(46.0, rect.height()));
        let r = ui.interact(b, ui.id().with(("window button", k)), Sense::click());
        if r.hovered() {
            p.rect_filled(b, 0, if k == 0 { Color32::from_rgb(0xc4, 0x2b, 0x1c) } else { theme::BUTTON });
        }
        let c = b.center();
        let s = Stroke::new(1.0, if r.hovered() { INK } else { MUTED });
        match k {
            0 => {
                p.line_segment([c + vec2(-5.0, -5.0), c + vec2(5.0, 5.0)], s);
                p.line_segment([c + vec2(5.0, -5.0), c + vec2(-5.0, 5.0)], s);
                if r.clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            1 => {
                p.rect_stroke(Rect::from_center_size(c, vec2(10.0, 10.0)), 1, s, StrokeKind::Inside);
                if r.clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                }
            }
            _ => {
                p.line_segment([c + vec2(-5.0, 0.0), c + vec2(5.0, 0.0)], s);
                if r.clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
            }
        }
    }
}

/// The window has no system frame: its edges resize it.
fn resize_edges(ui: &mut Ui, screen: Rect) {
    use egui::{CursorIcon as C, ResizeDirection as D};
    let ctx = ui.ctx().clone();
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    let Some(at) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    let e = 6.0;
    let (l, r, t, b) = (at.x - screen.min.x < e, screen.max.x - at.x < e, at.y - screen.min.y < e, screen.max.y - at.y < e);
    let dir = match (l, r, t, b) {
        (true, _, true, _) => Some((D::NorthWest, C::ResizeNorthWest)),
        (_, true, true, _) => Some((D::NorthEast, C::ResizeNorthEast)),
        (true, _, _, true) => Some((D::SouthWest, C::ResizeSouthWest)),
        (_, true, _, true) => Some((D::SouthEast, C::ResizeSouthEast)),
        (true, ..) => Some((D::West, C::ResizeWest)),
        (_, true, ..) => Some((D::East, C::ResizeEast)),
        (_, _, true, _) => Some((D::North, C::ResizeNorth)),
        (_, _, _, true) => Some((D::South, C::ResizeSouth)),
        _ => None,
    };
    if let Some((d, c)) = dir {
        ctx.set_cursor_icon(c);
        if ctx.input(|i| i.pointer.primary_pressed()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(d));
        }
    }
}

/// A page's head, as the launcher's pages open: a state line, the name in Bricolage Display, a
/// line under it, and the page's buttons at the right, level with the name.
fn page_head(ui: &mut Ui, state: Option<(Color32, String)>, name: &str, line: &str, buttons: impl FnOnce(&mut Ui)) {
    let h = if state.is_some() { 114.0 } else { 92.0 };
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
    let p = ui.painter().clone();
    let mut y = rect.min.y;
    if let Some((c, s)) = &state {
        paint_pill(&p, pos2(rect.min.x + pill_size(&p, s), y + 10.0), s, *c);
        y += 22.0;
    }
    let title = one_line(&p, name, FontId::new(52.0, theme::display()), INK, rect.width() - 380.0);
    let title_h = title.size().y;
    p.galley(pos2(rect.min.x - 2.0, y - 2.0), title, INK);
    if !line.is_empty() {
        let g = one_line(&p, line, FontId::new(16.0, FontFamily::Proportional), BODY, rect.width() - 380.0);
        p.galley(pos2(rect.min.x, y + title_h - 4.0), g, BODY);
    }
    // The buttons, right-aligned on the name's line.
    let row = Rect::from_min_max(pos2(rect.max.x - 380.0, y + 8.0), pos2(rect.max.x, y + 48.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(row).layout(Layout::right_to_left(Align::Center)));
    child.spacing_mut().item_spacing.x = 10.0;
    buttons(&mut child);
    ui.add_space(14.0);
}

/// A setting as the launcher lays one out (`setting_row`): its name and a note on the left, its
/// control on the right, both centred on the row, a hairline under it.
fn setting_row(ui: &mut Ui, label: &str, hint: &str, control: impl FnOnce(&mut Ui)) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, 60.0), Sense::hover());
    let p = ui.painter().clone();
    let name = p.layout_no_wrap(label.to_owned(), FontId::new(14.0, theme::medium()), INK);
    let note = (!hint.is_empty()).then(|| p.layout_no_wrap(hint.to_owned(), FontId::new(12.5, FontFamily::Proportional), QUIET));
    let total = name.size().y + note.as_ref().map(|n| n.size().y + 2.0).unwrap_or(0.0);
    let y = rect.center().y - total / 2.0;
    let nh = name.size().y;
    p.galley(pos2(rect.min.x, y), name, INK);
    if let Some(n) = note {
        p.galley(pos2(rect.min.x, y + nh + 2.0), n, QUIET);
    }
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.with_min_x(rect.min.x + w * 0.4)).layout(Layout::right_to_left(Align::Center)));
    control(&mut child);
    p.line_segment([pos2(rect.min.x, rect.max.y), pos2(rect.max.x, rect.max.y)], Stroke::new(1.0, LINE));
}

/// A glass surface as tall as its content: a plate, or (`well`) the console's and editor's well, the
/// more blurred, dimmed glass of the launcher's menus under the scrim colour at 69 %, so text reads
/// over the brightest part of the scene.
fn surface<R>(ui: &mut Ui, well: bool, corner: u8, margin: Margin, add: impl FnOnce(&mut Ui) -> R) -> R {
    let at = ui.painter().add(egui::Shape::Noop);
    let r = Frame::new().inner_margin(margin).show(ui, add);
    let rect = r.response.rect;
    let shape = if well {
        egui::Shape::Vec(vec![theme::plate_shape(rect, corner, true), egui::Shape::rect_filled(rect, corner, theme::WELL)])
    } else {
        theme::plate_shape(rect, corner, false)
    };
    ui.painter().set(at, shape);
    r.inner
}

// ---------- the sidebar's compact rows ----------

/// A section's name in the sidebar (12.5 px medium at 55 %), with a + at its right.
fn side_heading(ui: &mut Ui, text: &str, plus: Option<&str>) -> Option<Response> {
    ui.add_space(8.0);
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, 28.0), Sense::hover());
    ui.painter().text(pos2(rect.min.x + 10.0, rect.center().y), Align2::LEFT_CENTER, text, FontId::new(12.5, theme::medium()), QUIET);
    plus.map(|tip| {
        let b = Rect::from_center_size(pos2(rect.max.x - 14.0, rect.center().y), vec2(24.0, 24.0));
        let r = ui.interact(b, ui.id().with(("side plus", text)), Sense::click());
        if r.hovered() {
            ui.painter().rect_filled(b, 7, theme::BUTTON);
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        theme::paint_icon(ui, "plus-sign", Rect::from_center_size(b.center(), vec2(14.0, 14.0)), if r.hovered() { INK } else { QUIET });
        r.on_hover_text(tip)
    })
}

/// The All servers row: an icon, its name, a count at the right.
fn nav_row(ui: &mut Ui, icon: &str, name: &str, meta: &str, selected: bool) -> Response {
    let w = ui.available_width();
    let (rect, r) = ui.allocate_exact_size(vec2(w, 38.0), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, radius::CONTROL, theme::SELECTED);
    } else if r.hovered() {
        p.rect_filled(rect, radius::CONTROL, theme::HOVER);
    }
    theme::paint_icon(ui, icon, Rect::from_center_size(pos2(rect.min.x + 18.0, rect.center().y), vec2(16.0, 16.0)), if selected { INK } else { STRONG });
    p.text(pos2(rect.min.x + 36.0, rect.center().y), Align2::LEFT_CENTER, name, FontId::new(13.5, theme::medium()), INK);
    p.text(pos2(rect.max.x - 10.0, rect.center().y), Align2::RIGHT_CENTER, meta, FontId::new(12.0, FontFamily::Proportional), MUTED);
    if r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    r
}

/// A server in the sidebar: its state's dot, its name, and at the right its players (or Off).
#[allow(clippy::too_many_arguments)]
fn compact_row(ui: &mut Ui, dot: Color32, name: &str, meta: &str, meta_color: Color32, selected: bool, indent: f32, off: bool) -> Response {
    let w = ui.available_width();
    let (full, r) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
    let rect = Rect::from_min_max(pos2(full.min.x + indent, full.min.y), full.max);
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, radius::CONTROL, theme::SELECTED);
    } else if r.hovered() {
        p.rect_filled(rect, radius::CONTROL, theme::HOVER);
    }
    p.circle_filled(pos2(rect.min.x + 14.0, rect.center().y), 3.5, dot);
    let meta_g = p.layout_no_wrap(meta.to_owned(), FontId::new(12.0, FontFamily::Proportional), meta_color);
    let name_color = if off && !selected { MUTED } else { INK };
    let g = one_line(p, name, FontId::new(13.5, if selected { theme::bold() } else { theme::medium() }), name_color, rect.width() - 26.0 - meta_g.size().x - 18.0);
    p.galley(pos2(rect.min.x + 26.0, rect.center().y - g.size().y / 2.0), g, name_color);
    p.galley(pos2(rect.max.x - 10.0 - meta_g.size().x, rect.center().y - meta_g.size().y / 2.0), meta_g, meta_color);
    if r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    r
}

/// A network in the sidebar: an arrow that opens and closes it, a folder, its name, and how many
/// of its servers run. Gives the row and the arrow.
fn network_row(ui: &mut Ui, name: &str, up: usize, total: usize, selected: bool, open: bool) -> (Response, Response) {
    let w = ui.available_width();
    let (rect, r) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());
    let arrow_rect = Rect::from_min_size(rect.min, vec2(28.0, rect.height()));
    let arrow = ui.interact(arrow_rect, r.id.with("arrow"), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, radius::CONTROL, theme::SELECTED);
    } else if r.hovered() || arrow.hovered() {
        p.rect_filled(rect, radius::CONTROL, theme::HOVER);
    }
    theme::paint_icon(ui, if open { "arrow-down-01" } else { "arrow-right-01" }, Rect::from_center_size(pos2(rect.min.x + 15.0, rect.center().y), vec2(13.0, 13.0)), if arrow.hovered() { INK } else { QUIET });
    theme::paint_icon(ui, "folder-01", Rect::from_center_size(pos2(rect.min.x + 37.0, rect.center().y), vec2(16.0, 16.0)), ACCENT_SOFT);
    let meta = format!("{up}/{total}");
    let meta_color = if total > 0 && up == total { LIVE } else if up > 0 { WARN } else { FAINT };
    let meta_g = p.layout_no_wrap(meta, FontId::new(12.0, theme::medium()), meta_color);
    let g = one_line(p, name, FontId::new(13.5, theme::bold()), INK, rect.width() - 54.0 - meta_g.size().x - 16.0);
    p.galley(pos2(rect.min.x + 52.0, rect.center().y - g.size().y / 2.0), g, INK);
    p.galley(pos2(rect.max.x - 10.0 - meta_g.size().x, rect.center().y - meta_g.size().y / 2.0), meta_g, meta_color);
    if r.hovered() || arrow.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    (r, arrow)
}
