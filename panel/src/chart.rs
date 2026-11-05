//! A line chart of the last minutes: a soft fill under the line, a few grid lines, and the value under the
//! pointer.

use crate::theme;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2};

pub struct Series<'a> {
    pub label: &'a str,
    pub color: Color32,
    pub points: Vec<[f64; 2]>,
}

pub fn chart(ui: &mut Ui, title: &str, value: &str, series: &[Series], min_top: f64, fixed_top: Option<f64>, unit: &str, height: f32) {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.add(theme::plate_shape(rect, theme::radius::PANEL, false));
    painter.text(rect.min + vec2(22.0, 18.0), Align2::LEFT_TOP, title, FontId::new(13.5, egui::FontFamily::Proportional), theme::MUTED);
    painter.text(rect.min + vec2(22.0, 38.0), Align2::LEFT_TOP, value, FontId::new(24.0, theme::display()), theme::TEXT);
    // The legend, top right.
    let mut x = rect.max.x - 22.0;
    for s in series.iter().rev() {
        let galley = painter.layout_no_wrap(s.label.to_owned(), FontId::new(12.0, egui::FontFamily::Proportional), theme::MUTED);
        x -= galley.size().x;
        painter.galley(pos2(x, rect.min.y + 19.0), galley, theme::MUTED);
        x -= 14.0;
        painter.circle_filled(pos2(x + 5.0, rect.min.y + 26.0), 3.5, s.color);
        x -= 10.0;
    }

    let plot = Rect::from_min_max(pos2(rect.min.x + 22.0, rect.min.y + 80.0), pos2(rect.max.x - 56.0, rect.max.y - 30.0));
    let all = series.iter().flat_map(|s| s.points.iter());
    let (mut t0, mut t1, mut top) = (f64::MAX, f64::MIN, min_top);
    for p in all {
        t0 = t0.min(p[0]);
        t1 = t1.max(p[0]);
        top = top.max(p[1]);
    }
    let top = fixed_top.unwrap_or_else(|| nice(top * 1.15));
    let now = t1;
    if t0 == f64::MAX {
        painter.text(plot.center(), Align2::CENTER_CENTER, "No data yet", FontId::new(13.0, egui::FontFamily::Proportional), theme::FAINT);
        return;
    }
    // Always show at least two minutes, so the first seconds don't stretch across the card.
    let t0 = t0.min(t1 - 120.0);
    let span = (t1 - t0).max(1.0);
    let to_screen = |p: [f64; 2]| -> Pos2 {
        pos2(
            plot.min.x + ((p[0] - t0) / span) as f32 * plot.width(),
            plot.max.y - (p[1] / top).clamp(0.0, 1.05) as f32 * plot.height(),
        )
    };
    let grid = Stroke::new(1.0, theme::LINE);
    for i in 0..=3 {
        let v = top * i as f64 / 3.0;
        let y = plot.max.y - (i as f32 / 3.0) * plot.height();
        painter.line_segment([pos2(plot.min.x, y), pos2(plot.max.x, y)], grid);
        painter.text(pos2(plot.max.x + 8.0, y), Align2::LEFT_CENTER, short(v, unit), FontId::new(11.0, egui::FontFamily::Proportional), theme::FAINT);
    }
    let minutes = span / 60.0;
    painter.text(pos2(plot.min.x, plot.max.y + 8.0), Align2::LEFT_TOP, format!("{:.0} min ago", minutes.ceil()), FontId::new(11.0, egui::FontFamily::Proportional), theme::FAINT);
    painter.text(pos2(plot.max.x, plot.max.y + 8.0), Align2::RIGHT_TOP, "now", FontId::new(11.0, egui::FontFamily::Proportional), theme::FAINT);

    for s in series {
        if s.points.len() < 2 {
            continue;
        }
        let line: Vec<Pos2> = s.points.iter().map(|p| to_screen(*p)).collect();
        // The fill: a strip of quads down to the axis, faded.
        let fill = s.color.linear_multiply(0.16);
        let mut mesh = egui::Mesh::default();
        for (i, p) in line.iter().enumerate() {
            mesh.colored_vertex(*p, fill);
            mesh.colored_vertex(pos2(p.x, plot.max.y), Color32::TRANSPARENT);
            if i > 0 {
                let b = (i as u32) * 2;
                mesh.add_triangle(b - 2, b - 1, b);
                mesh.add_triangle(b - 1, b + 1, b);
            }
        }
        painter.add(Shape::mesh(mesh));
        painter.add(Shape::line(line, Stroke::new(1.8, s.color)));
    }

    if let Some(pointer) = response.hover_pos().filter(|p| plot.expand2(vec2(0.0, 30.0)).contains(*p)) {
        let t = t0 + ((pointer.x - plot.min.x) / plot.width()).clamp(0.0, 1.0) as f64 * span;
        painter.line_segment([pos2(pointer.x, plot.min.y), pos2(pointer.x, plot.max.y)], Stroke::new(1.0, theme::FAINT));
        let mut text = format!("{} s ago", (now - t).round().max(0.0));
        for s in series {
            if let Some(p) = s.points.iter().min_by(|a, b| (a[0] - t).abs().total_cmp(&(b[0] - t).abs())) {
                painter.circle_filled(to_screen(*p), 3.5, s.color);
                text.push_str(&format!("\n{}: {}", s.label, short(p[1], unit)));
            }
        }
        let galley = painter.layout(text, FontId::new(12.0, egui::FontFamily::Proportional), theme::TEXT, 220.0);
        let mut at = pointer + vec2(12.0, -galley.size().y - 12.0);
        if at.x + galley.size().x + 16.0 > rect.max.x {
            at.x = pointer.x - galley.size().x - 24.0;
        }
        let bg = Rect::from_min_size(at, galley.size() + vec2(16.0, 12.0));
        painter.rect(bg, 10.0, Color32::from_rgb(0x1e, 0x1a, 0x33), Stroke::NONE, StrokeKind::Inside);
        painter.galley(at + vec2(8.0, 6.0), galley, theme::TEXT);
    }
}

fn nice(v: f64) -> f64 {
    if v <= 0.0 {
        return 1.0;
    }
    let p = 10f64.powf(v.log10().floor());
    for m in [1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0] {
        if v <= m * p {
            return m * p;
        }
    }
    10.0 * p
}

pub fn short(v: f64, unit: &str) -> String {
    match unit {
        "MB" if v >= 1024.0 => format!("{:.1} GB", v / 1024.0),
        "MB" => format!("{v:.0} MB"),
        "%" => format!("{v:.0}%"),
        "ms" if v < 10.0 => format!("{v:.2} ms"),
        "ms" if v < 100.0 => format!("{v:.1} ms"),
        "ms" => format!("{v:.0} ms"),
        "" => format!("{v:.0}"),
        u => format!("{v:.1} {u}"),
    }
}
