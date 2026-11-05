//! The Horizon launcher's glass look (`horizon_theme::glass` and the launcher's `glass_ui.rs`),
//! for the panel: the pixel sunset behind everything, frosted plates over it, white text at a few
//! opacities, one violet accent, Geist for text and Bricolage Grotesque for names, Hugeicons icons.

use crate::scene;
use eframe::egui::{
    self, Color32, ColorImage, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Painter, Pos2, Rect, Shape,
    Stroke, TextStyle, TextureHandle, TextureOptions, Ui, Vec2, Visuals, epaint::RectShape, pos2, vec2,
};
use std::sync::{Arc, LazyLock, Mutex};

// ---------- tokens (horizon_theme::glass) ----------

/// White at `a` (0 to 255), premultiplied: text and fills in the glass look are white at a few opacities.
pub const fn white(a: u8) -> Color32 {
    Color32::from_rgba_premultiplied(a, a, a, a)
}
const fn pct(p: f32) -> u8 {
    (p * 255.0 + 0.5) as u8
}

pub const ACCENT: Color32 = Color32::from_rgb(0x6b, 0x4b, 0xff);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xa9, 0xa9, 0xff);
pub const LIVE: Color32 = Color32::from_rgb(0x4a, 0xde, 0x80);
pub const WARN: Color32 = Color32::from_rgb(0xf2, 0xc4, 0x6b);
pub const DANGER: Color32 = Color32::from_rgb(0xff, 0x8a, 0x8a);
pub const ON_LIGHT: Color32 = Color32::from_rgb(0x10, 0x0a, 0x1f);
pub const PAGE: Color32 = Color32::from_rgb(0x0a, 0x09, 0x19);
/// The scrim under dialogs: `#07050f` at 50 %.
pub const SCRIM: Color32 = Color32::from_rgba_premultiplied(4, 3, 8, 128);
/// The plate's tint and how much of it covers the blurred scene (`Glass::new` for the default look).
const PLATE_TINT: [f32; 3] = [0x4e as f32 / 255., 0x4c as f32 / 255., 0x5f as f32 / 255.];
const PLATE_ALPHA: f32 = 0.2668;

/// Text opacities (`glass::ink`).
pub const INK: Color32 = white(255);
pub const STRONG: Color32 = white(pct(0.8));
pub const BODY: Color32 = white(pct(0.7));
pub const MUTED: Color32 = white(pct(0.65));
pub const QUIET: Color32 = white(pct(0.55));
pub const FAINT: Color32 = white(pct(0.45));
pub const PLACEHOLDER: Color32 = white(pct(0.4));

/// Fill opacities (`glass::fill`).
pub const HOVER: Color32 = white(pct(0.05));
pub const FIELD_FILL: Color32 = white(pct(0.07));
pub const BUTTON: Color32 = white(pct(0.08));
pub const SELECTED: Color32 = white(pct(0.10));
pub const ACTIVE: Color32 = white(pct(0.12));
pub const BUTTON_HOVER: Color32 = white(pct(0.14));
/// A hairline between rows (the mockup's `white/[0.06]`).
pub const LINE: Color32 = white(pct(0.06));
/// The console's and the editor's well: the scrim colour at 69 %, so text reads over the sun.
pub const WELL: Color32 = Color32::from_rgba_premultiplied(4, 3, 9, 175);

/// Corners (`glass::radius`).
pub mod radius {
    pub const MODAL: u8 = 28;
    pub const PANEL: u8 = 20;
    pub const POPOVER: u8 = 16;
    pub const LARGE: u8 = 14;
    pub const ROW: u8 = 12;
    pub const CONTROL: u8 = 10;
}

// The names the rest of the panel uses for them.
pub const TEXT: Color32 = INK;
pub const GREEN: Color32 = LIVE;
pub const AMBER: Color32 = WARN;
pub const RED: Color32 = DANGER;
pub const VIOLET: Color32 = ACCENT_SOFT;
pub const CYAN: Color32 = Color32::from_rgb(0x7d, 0xd3, 0xfc);
pub const CARD: Color32 = white(pct(0.06));
pub const CARD_HOVER: Color32 = HOVER;
pub const BORDER: Color32 = LINE;
pub const FIELD: Color32 = WELL;

// ---------- type ----------

pub fn medium() -> FontFamily {
    FontFamily::Name("medium".into())
}
pub fn bold() -> FontFamily {
    FontFamily::Name("semibold".into())
}
/// Bricolage Grotesque at its display size (page names, 56 px).
pub fn display() -> FontFamily {
    FontFamily::Name("display".into())
}
/// Bricolage Grotesque at its heading size (dialog and card names).
pub fn heading() -> FontFamily {
    FontFamily::Name("heading".into())
}

macro_rules! font {
    ($f:literal) => {
        include_bytes!(concat!("../assets/fonts/", $f)).as_slice()
    };
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut add = |name: &str, bytes: &'static [u8]| {
        fonts.font_data.insert(name.into(), Arc::new(FontData::from_static(bytes)));
    };
    add("geist", font!("Geist-Regular.ttf"));
    add("geist-medium", font!("Geist-Medium.ttf"));
    add("geist-semibold", font!("Geist-SemiBold.ttf"));
    add("geist-mono", font!("GeistMono-Regular.ttf"));
    add("bricolage-display", font!("BricolageGrotesqueDisplay-SemiBold.ttf"));
    add("bricolage-heading", font!("BricolageGrotesqueHeading-SemiBold.ttf"));
    // egui's own fonts stay behind each family for symbols Geist lacks (arrows, dots).
    let fallback: Vec<String> = fonts.families[&FontFamily::Proportional].clone();
    let family = |first: &str| std::iter::once(first.to_owned()).chain(fallback.iter().cloned()).collect::<Vec<_>>();
    fonts.families.insert(FontFamily::Proportional, family("geist"));
    fonts.families.insert(medium(), family("geist-medium"));
    fonts.families.insert(bold(), family("geist-semibold"));
    fonts.families.insert(display(), family("bricolage-display"));
    fonts.families.insert(heading(), family("bricolage-heading"));
    let mono_fallback = fonts.families[&FontFamily::Monospace].clone();
    fonts.families.insert(FontFamily::Monospace, std::iter::once("geist-mono".to_owned()).chain(mono_fallback).collect());
    ctx.set_fonts(fonts);

    let mut v = Visuals::dark();
    v.panel_fill = Color32::TRANSPARENT;
    v.window_fill = Color32::from_rgb(0x1e, 0x1a, 0x33);
    v.window_stroke = Stroke::NONE;
    v.window_corner_radius = CornerRadius::same(radius::POPOVER);
    v.menu_corner_radius = CornerRadius::same(radius::POPOVER);
    v.window_shadow = egui::epaint::Shadow { offset: [0, 12], blur: 40, spread: 0, color: Color32::from_black_alpha(110) };
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 10], blur: 30, spread: 0, color: Color32::from_black_alpha(100) };
    v.extreme_bg_color = FIELD_FILL;
    v.faint_bg_color = HOVER;
    v.code_bg_color = WELL;
    v.override_text_color = Some(INK);
    v.hyperlink_color = ACCENT_SOFT;
    v.selection.bg_fill = ACCENT.linear_multiply(0.55);
    v.selection.stroke = Stroke::new(1.0, ACCENT_SOFT);
    v.text_cursor.stroke = Stroke::new(1.5, ACCENT_SOFT);
    let r = CornerRadius::same(radius::CONTROL);
    for (w, fill) in [
        (&mut v.widgets.noninteractive, Color32::TRANSPARENT),
        (&mut v.widgets.inactive, BUTTON),
        (&mut v.widgets.hovered, BUTTON_HOVER),
        (&mut v.widgets.active, ACTIVE),
        (&mut v.widgets.open, ACTIVE),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::NONE;
        w.fg_stroke = Stroke::new(1.0, INK);
        w.corner_radius = r;
        w.expansion = 0.0;
    }
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, BODY);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, STRONG);
    v.widgets.hovered.bg_stroke = Stroke::NONE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_SOFT.linear_multiply(0.6));
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, v);

    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = vec2(8.0, 8.0);
        s.spacing.button_padding = vec2(14.0, 7.0);
        s.spacing.interact_size.y = 32.0;
        s.spacing.menu_margin = egui::Margin::same(6);
        s.spacing.scroll.bar_width = 6.0;
        s.spacing.scroll.floating = true;
        s.spacing.scroll.foreground_color = false;
        s.text_styles.insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Button, FontId::new(14.0, medium()));
        s.text_styles.insert(TextStyle::Small, FontId::new(12.5, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace));
        s.text_styles.insert(TextStyle::Heading, FontId::new(28.0, heading()));
        s.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, INK);
    });
    egui_extras::install_image_loaders(ctx);
}

// ---------- the scene and the glass ----------

struct Glass {
    /// The window size (logical) and scale the textures were made for.
    size: Vec2,
    scale: f32,
    scene: Option<TextureHandle>,
    plate: Option<TextureHandle>,
    float: Option<TextureHandle>,
    /// A size asked for and when, so a window being resized isn't re-baked every frame.
    wanted: Option<(Vec2, f32, std::time::Instant)>,
    baking: bool,
    ready: Option<(Vec2, f32, ColorImage, ColorImage, ColorImage)>,
}

static GLASS: LazyLock<Mutex<Glass>> = LazyLock::new(|| {
    Mutex::new(Glass { size: Vec2::ZERO, scale: 1.0, scene: None, plate: None, float: None, wanted: None, baking: false, ready: None })
});

fn mixed(backdrop: &[u8], w: usize, h: usize, brightness: f32) -> ColorImage {
    let mut px = Vec::with_capacity(w * h);
    for c in backdrop.chunks_exact(4) {
        let m = |i: usize| {
            let v = c[i] as f32 / 255. * brightness;
            ((PLATE_TINT[i] * PLATE_ALPHA + v * (1. - PLATE_ALPHA)) * 255.).round() as u8
        };
        px.push(Color32::from_rgb(m(0), m(1), m(2)));
    }
    ColorImage::new([w, h], px)
}

/// Keeps the scene's textures for the window's size: made again (off the UI thread) once a new
/// size has held for a moment.
pub fn keep_scene(ctx: &egui::Context) {
    let size = ctx.content_rect().size();
    let scale = ctx.pixels_per_point();
    let mut g = GLASS.lock().unwrap();
    if let Some((s, sc, scene, plate, float)) = g.ready.take() {
        let opt = TextureOptions::LINEAR;
        g.scene = Some(ctx.load_texture("scene", scene, TextureOptions::NEAREST));
        g.plate = Some(ctx.load_texture("plate", plate, opt));
        g.float = Some(ctx.load_texture("float", float, opt));
        g.size = s;
        g.scale = sc;
    }
    let same = (g.size - size).length() < 1.0 && (g.scale - scale).abs() < 0.01;
    if same || g.baking {
        if !same {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
        }
        return;
    }
    let settled = match g.wanted {
        Some((s, sc, at)) if (s - size).length() < 1.0 && (sc - scale).abs() < 0.01 => at.elapsed() > std::time::Duration::from_millis(if g.scene.is_none() { 0 } else { 140 }),
        _ => {
            g.wanted = Some((size, scale, std::time::Instant::now()));
            g.scene.is_none()
        }
    };
    if !settled {
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
        return;
    }
    g.baking = true;
    drop(g);
    let ctx = ctx.clone();
    std::thread::Builder::new()
        .name("scene".into())
        .spawn(move || {
            let (w, h) = (size.x as f64, size.y as f64);
            let (pw, ph, pic) = scene::picture(w, h, scale as f64, 0.0);
            let scene_img = ColorImage::from_rgba_unmultiplied([pw, ph], &pic);
            let (bw, bh, glass) = scene::backdrop(w, h, 24.0);
            let plate = mixed(&glass, bw, bh, 1.0);
            let (fw, fh, floaty) = scene::backdrop(w, h, 48.0);
            let float = mixed(&floaty, fw, fh, 0.62);
            let mut g = GLASS.lock().unwrap();
            g.ready = Some((size, scale, scene_img, plate, float));
            g.baking = false;
            drop(g);
            ctx.request_repaint();
        })
        .ok();
}

/// The scene over the whole window.
pub fn paint_scene(p: &Painter, screen: Rect) {
    let g = GLASS.lock().unwrap();
    match &g.scene {
        Some(t) => {
            p.add(Shape::Rect(RectShape::filled(screen, 0, Color32::WHITE).with_texture(t.id(), Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)))));
        }
        None => {
            p.rect_filled(screen, 0, PAGE);
        }
    }
}

/// A frosted plate at `rect`: the blurred scene behind it, tinted as the launcher's plates are.
/// `float` is the darker glass of menus and dialogs.
pub fn plate_shape(rect: Rect, corner: u8, float: bool) -> Shape {
    let g = GLASS.lock().unwrap();
    let tex = if float { &g.float } else { &g.plate };
    match tex {
        Some(t) if g.size.x > 0.0 => {
            let uv = Rect::from_min_max(pos2(rect.min.x / g.size.x, rect.min.y / g.size.y), pos2(rect.max.x / g.size.x, rect.max.y / g.size.y));
            Shape::Rect(RectShape::filled(rect, corner, Color32::WHITE).with_texture(t.id(), uv))
        }
        _ => Shape::Rect(RectShape::filled(rect, corner, if float { Color32::from_rgb(0x1c, 0x18, 0x30) } else { Color32::from_rgb(0x2a, 0x25, 0x40) })),
    }
}

/// A plate as tall as its content: what `add` puts in it, `pad` from its edges.
pub fn plate<R>(ui: &mut Ui, corner: u8, pad: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let at = ui.painter().add(Shape::Noop);
    let r = egui::Frame::new().inner_margin(egui::Margin::same(pad as i8)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui)
    });
    ui.painter().set(at, plate_shape(r.response.rect, corner, false));
    r.inner
}

// ---------- icons (Hugeicons, as the launcher ships them) ----------

macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        fn svg(name: &str) -> Option<&'static str> {
            match name {
                $($name => Some(include_str!(concat!("../assets/icons/", $name, ".svg"))),)*
                _ => None,
            }
        }
    };
}
icons!(
    "play", "stop", "refresh", "plus-sign", "folder-01", "search-01", "computer-terminal-01", "dashboard-square-01",
    "user-multiple-02", "puzzle", "upload-04", "delete-02", "pencil-edit-02", "more-horizontal", "arrow-down-01",
    "arrow-right-01", "cancel-01", "tick-02", "sliders-horizontal", "package-01", "ai-chat-02", "globe-02",
    "information-circle", "copy-01", "minus-sign", "computer", "view",
);

/// An icon as an image egui can draw: drawn white (its `currentColor`), so a tint sets its colour.
pub fn icon(name: &str) -> egui::ImageSource<'static> {
    static CACHE: LazyLock<Mutex<std::collections::HashMap<String, Arc<[u8]>>>> = LazyLock::new(|| Mutex::new(Default::default()));
    let bytes = CACHE
        .lock()
        .unwrap()
        .entry(name.to_owned())
        .or_insert_with(|| Arc::from(svg(name).unwrap_or("<svg xmlns='http://www.w3.org/2000/svg'/>").replace("currentColor", "#ffffff").into_bytes()))
        .clone();
    egui::ImageSource::Bytes { uri: format!("bytes://icon/{name}.svg").into(), bytes: egui::load::Bytes::Shared(bytes) }
}

/// An icon `size` px square in `color`.
pub fn icon_image(name: &str, size: f32, color: Color32) -> egui::Image<'static> {
    egui::Image::new(icon(name)).fit_to_exact_size(vec2(size, size)).tint(color)
}

pub fn paint_icon(ui: &Ui, name: &str, rect: Rect, color: Color32) {
    icon_image(name, rect.width(), color).paint_at(ui, rect);
}

/// Horizon's mark: a sun half set below the horizon, three bars over a line (`docs/design-system.md`).
pub fn paint_mark(p: &Painter, at: Pos2, size: f32, color: Color32, line: Color32) {
    let u = size / 16.0;
    let bar = |x: f32, y: f32, w: f32| Rect::from_min_size(pos2(at.x + x * u, at.y + y * u), vec2(w * u, 1.6 * u));
    // Bars of 4, 8 and 10 units, centred, over a 12 unit line.
    p.rect_filled(bar(6.0, 4.0, 4.0), 0, color);
    p.rect_filled(bar(4.0, 6.6, 8.0), 0, color);
    p.rect_filled(bar(3.0, 9.2, 10.0), 0, color);
    p.rect_filled(bar(2.0, 11.8, 12.0), 0, line);
}
