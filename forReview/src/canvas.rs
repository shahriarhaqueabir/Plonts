use web_sys::CanvasRenderingContext2d;

use garden_core::models::{BedShape, LayoutPlant};

/// Pixels per centimetre on the canvas.
pub const SCALE: f64 = 3.0;

pub fn px(cm: f64) -> f64 {
    cm * SCALE
}

pub fn clear(ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    ctx.clear_rect(0.0, 0.0, width, height);
}

pub fn draw_grid(ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    let step = px(garden_core::layout::GRID_CM);
    ctx.set_stroke_style_str("#1c1917");
    ctx.set_line_width(1.0);
    let mut x = 0.0;
    while x <= width {
        ctx.begin_path();
        ctx.move_to(x, 0.0);
        ctx.line_to(x, height);
        ctx.stroke();
        x += step;
    }
    let mut y = 0.0;
    while y <= height {
        ctx.begin_path();
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
        ctx.stroke();
        y += step;
    }
}

pub fn draw_bed(ctx: &CanvasRenderingContext2d, bed: &BedShape) {
    let x = px(bed.x);
    let y = px(bed.y);
    let w = px(bed.width_cm);
    let h = px(bed.height_cm);
    ctx.set_fill_style_str("#3f3f46");
    ctx.set_global_alpha(0.35);
    ctx.fill_rect(x, y, w, h);
    ctx.set_global_alpha(1.0);
    ctx.set_stroke_style_str("#a1a1aa");
    ctx.set_line_width(2.0);
    ctx.stroke_rect(x, y, w, h);
    ctx.set_fill_style_str("#e4e4e7");
    ctx.set_font("12px sans-serif");
    let label = format!("{} ({}x{}cm)", bed.name, bed.width_cm as i64, bed.height_cm as i64);
    let _ = ctx.fill_text(&label, x + 4.0, y + 14.0);
}

pub fn draw_bed_rect(ctx: &CanvasRenderingContext2d, start_x: f64, start_y: f64, cur_x: f64, cur_y: f64) {
    let x = px(start_x.min(cur_x));
    let y = px(start_y.min(cur_y));
    let w = px((cur_x - start_x).abs());
    let h = px((cur_y - start_y).abs());
    ctx.set_fill_style_str("#6366f1");
    ctx.set_global_alpha(0.25);
    ctx.fill_rect(x, y, w, h);
    ctx.set_global_alpha(1.0);
    ctx.set_stroke_style_str("#818cf8");
    ctx.set_line_width(2.0);
    ctx.stroke_rect(x, y, w, h);
}

pub fn draw_bed_highlight(ctx: &CanvasRenderingContext2d, bed: &BedShape) {
    ctx.set_stroke_style_str("#facc15");
    ctx.set_line_width(2.0);
    ctx.stroke_rect(px(bed.x), px(bed.y), px(bed.width_cm), px(bed.height_cm));
}

pub fn draw_plant(ctx: &CanvasRenderingContext2d, plant: &LayoutPlant, radius_cm: f64) {
    let x = px(plant.x);
    let y = px(plant.y);
    // spacing circle (semi-transparent)
    ctx.begin_path();
    let _ = ctx.arc(x, y, px(radius_cm), 0.0, std::f64::consts::TAU);
    ctx.set_fill_style_str("#22c55e");
    ctx.set_global_alpha(0.12);
    ctx.fill();
    ctx.set_global_alpha(1.0);
    ctx.set_stroke_style_str("#22c55e");
    ctx.stroke();
    // plant dot
    ctx.begin_path();
    let _ = ctx.arc(x, y, 4.0, 0.0, std::f64::consts::TAU);
    ctx.set_fill_style_str("#4ade80");
    ctx.fill();
    // label
    ctx.set_fill_style_str("#e4e4e7");
    ctx.set_font("10px sans-serif");
    let _ = ctx.fill_text(&plant.plant_name, x + 6.0, y - 6.0);
}

pub fn draw_preview_circle(ctx: &CanvasRenderingContext2d, x_cm: f64, y_cm: f64, radius_cm: f64) {
    ctx.begin_path();
    let _ = ctx.arc(px(x_cm), px(y_cm), px(radius_cm), 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style_str("#facc15");
    ctx.set_line_width(1.5);
    ctx.set_global_alpha(0.8);
    ctx.stroke();
    ctx.set_global_alpha(1.0);
}
