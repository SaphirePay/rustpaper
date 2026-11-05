//! rustpaper-panel: one window for every rustpaper (Horizon) server on this computer.
//!
//!     rustpaper-panel                 open the panel (or bring the open one forward)
//!     rustpaper-panel --select DIR    ... showing the server in DIR (added to the list if new)
//!
//! Only one panel runs at a time: a second start hands its `--select` to the first and exits.

#![cfg_attr(windows, windows_subsystem = "windows")]

mod chart;
mod files;
mod registry;
mod scene;
mod scan;
mod server;
mod theme;
mod ui;

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

/// The port the open panel listens on (127.0.0.1 only) for another start's `--select`.
const ONE_PANEL: &str = "127.0.0.1:47131";

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let select = value("--select").map(PathBuf::from).map(|d| registry::clean(&d));
    // For the docs' pictures: the window as it looks after a few seconds, saved as a PNG.
    let shot = value("--screenshot").map(|f| ui::Shot {
        file: PathBuf::from(f),
        tab: value("--tab").and_then(|t| ui::Tab::parse(&t)),
        network: value("--network"),
        network_tab: value("--network-tab"),
        open: value("--open").map(PathBuf::from),
        dialog: value("--dialog"),
        search: value("--search"),
        after: Duration::from_secs_f32(value("--after").and_then(|a| a.parse().ok()).unwrap_or(5.0)),
        taken: false,
    });
    let size = value("--size")
        .and_then(|s| s.split_once('x').and_then(|(w, h)| Some([w.parse::<f32>().ok()?, h.parse::<f32>().ok()?])))
        .unwrap_or([1380.0, 880.0]);

    if let (Some(dir), Some(out)) = (value("--selftest"), args.iter().position(|a| a == "--selftest").and_then(|i| args.get(i + 2))) {
        server::selftest(registry::clean(&PathBuf::from(dir)), &PathBuf::from(out));
        return Ok(());
    }
    let inbox: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    if shot.is_none() {
        match TcpListener::bind(ONE_PANEL) {
            Ok(listener) => {
                let inbox = inbox.clone();
                std::thread::Builder::new()
                    .name("one panel".into())
                    .spawn(move || {
                        for stream in listener.incoming().flatten() {
                            let mut line = String::new();
                            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                            let _ = BufReader::new(&stream).read_line(&mut line);
                            if let Some(dir) = line.trim().strip_prefix("select ") {
                                inbox.lock().unwrap().push(PathBuf::from(dir));
                            } else if line.trim() == "show" {
                                inbox.lock().unwrap().push(PathBuf::new());
                            }
                        }
                    })
                    .ok();
            }
            Err(_) => {
                // A panel is open already: it shows the server instead.
                if let Ok(mut s) = TcpStream::connect(ONE_PANEL) {
                    let message = match &select {
                        Some(d) => format!("select {}\n", d.display()),
                        None => "show\n".to_owned(),
                    };
                    let _ = s.write_all(message.as_bytes());
                    return Ok(());
                }
            }
        }
    }

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("rustpaper")
        .with_app_id("rustpaper-panel")
        .with_inner_size(size)
        .with_min_inner_size([1100.0, 700.0])
        // The launcher draws its own title line on the scene.
        .with_decorations(false)
        .with_icon(icon());
    if shot.is_some() {
        viewport = viewport.with_active(false);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("rustpaper", options, Box::new(move |cc| Ok(Box::new(ui::App::new(cc, select, shot, inbox)))))
}

/// A green square on a dark rounded tile.
fn icon() -> eframe::egui::IconData {
    let n = 64usize;
    let mut rgba = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let inside = |m: f32, r: f32| {
                let cx = fx.clamp(m + r, n as f32 - m - r);
                let cy = fy.clamp(m + r, n as f32 - m - r);
                ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() <= r
            };
            let i = (y * n + x) * 4;
            if inside(2.0, 14.0) {
                rgba[i..i + 4].copy_from_slice(&[0x1f, 0x3a, 0x2f, 255]);
            }
            if inside(18.0, 6.0) {
                rgba[i..i + 4].copy_from_slice(&[0x34, 0xd3, 0x99, 255]);
            }
        }
    }
    eframe::egui::IconData { rgba, width: n as u32, height: n as u32 }
}
