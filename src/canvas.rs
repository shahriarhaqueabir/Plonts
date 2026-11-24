use web_sys::CanvasRenderingContext2d;

use garden_core::models::{BedShape, LayoutPlant};

/// Pixels per centimetre on the canvas.
pub const BASE_SCALE: f64 = 2.0;

pub fn px(cm: f64) -> f64 {
    cm * BASE_SCALE
}

pub fn clear(ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    // Soil background: warm brown (solid — gradient needs CanvasGradient feature).
    ctx.set_fill_style_str("#3a2a1a");
    ctx.fill_rect(0.0, 0.0, width, height);

    // Subtle soil texture: scattered small dots (darker/lighter).
    let mut seed: u64 = 42;
    for _ in 0..300 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let x = (seed >> 32) as f64 % width;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let y = (seed >> 32) as f64 % height;
        let shade = (seed >> 48) as u64 % 3;
        match shade {
            0 => ctx.set_fill_style_str("rgba(0,0,0,0.18)"),
            1 => ctx.set_fill_style_str("rgba(255,255,255,0.04)"),
            _ => ctx.set_fill_style_str("rgba(139,111,71,0.12)"),
        }
        ctx.fill_rect(x, y, 1.5, 1.5);
    }
}

pub fn draw_grid(ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    // Very subtle grid — just enough to guide placement, soil-colored.
    let step = px(garden_core::layout::GRID_CM);
    ctx.set_stroke_style_str("rgba(255,255,255,0.06)");
    ctx.set_line_width(0.5);
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
    // Soil fill — darker brown inside the bed.
    ctx.set_fill_style_str("#5a3e28");
    ctx.fill_rect(x, y, w, h);
    // Raised border — wood-like.
    ctx.set_stroke_style_str("#8b6f47");
    ctx.set_line_width(3.0);
    ctx.stroke_rect(x, y, w, h);
    // Inner shadow for depth.
    ctx.set_stroke_style_str("rgba(0,0,0,0.3)");
    ctx.set_line_width(1.0);
    ctx.stroke_rect(x + 2.0, y + 2.0, w - 4.0, h - 4.0);
    // Label.
    ctx.set_fill_style_str("#e4e4e7");
    ctx.set_font("bold 12px sans-serif");
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

/// Map a plant name to a representative emoji for canvas rendering.
pub fn plant_emoji(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.contains("tomato") { "🍅" }
    else if n.contains("carrot") { "🥕" }
    else if n.contains("lettuce") || n.contains("salad") || n.contains("arugula") { "🥬" }
    else if n.contains("pepper") || n.contains("chili") { "🌶️" }
    else if n.contains("cucumber") { "🥒" }
    else if n.contains("broccoli") { "🥦" }
    else if n.contains("corn") { "🌽" }
    else if n.contains("strawberry") { "🍓" }
    else if n.contains("blueberry") || n.contains("berry") { "🫐" }
    else if n.contains("pumpkin") || n.contains("squash") { "🎃" }
    else if n.contains("potato") { "🥔" }
    else if n.contains("onion") || n.contains("garlic") { "🧅" }
    else if n.contains("pea") || n.contains("bean") { "🫛" }
    else if n.contains("herb") || n.contains("basil") || n.contains("mint") || n.contains("parsley") { "🌿" }
    else if n.contains("flower") || n.contains("rose") || n.contains("sunflower") { "🌻" }
    else if n.contains("tree") || n.contains("apple") || n.contains("peach") || n.contains("apricot") { "🌳" }
    else if n.contains("radish") || n.contains("beet") { "🔴" }
    else if n.contains("cabbage") { "🥬" }
    else { "🌱" }
}

pub fn draw_plant(ctx: &CanvasRenderingContext2d, plant: &LayoutPlant, radius_cm: f64) {
    let x = px(plant.x);
    let y = px(plant.y);
    // spacing circle (semi-transparent green)
    ctx.begin_path();
    let r = radius_cm.min(25.0);
    let _ = ctx.arc(x, y, px(r), 0.0, std::f64::consts::TAU);
    ctx.set_fill_style_str("rgba(34,197,94,0.10)");
    ctx.fill();
    ctx.set_stroke_style_str("rgba(34,197,94,0.4)");
    ctx.set_line_width(1.0);
    ctx.stroke();
    // Plant emoji icon.
    ctx.set_font("18px sans-serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(plant_emoji(&plant.plant_name), x, y);
    // Label.
    ctx.set_fill_style_str("#e4e4e7");
    ctx.set_font("10px sans-serif");
    ctx.set_text_align("left");
    ctx.set_text_baseline("alphabetic");
    let _ = ctx.fill_text(&plant.plant_name, x + 12.0, y - 10.0);
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
