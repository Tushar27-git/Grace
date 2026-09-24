use egui::{Color32, CornerRadius, Stroke, Visuals};

pub struct Theme;

#[allow(dead_code)]
impl Theme {
    pub const BG_CANVAS: Color32 = Color32::from_rgb(0xF4, 0xF1, 0xEC);
    pub const BG_PANEL: Color32 = Color32::from_rgb(0x15, 0x12, 0x17);
    pub const BG_PANEL_RAISED: Color32 = Color32::from_rgb(0x1E, 0x1A, 0x21);
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(0x6E, 0x4C, 0x9E);
    pub const ACCENT_PINK: Color32 = Color32::from_rgb(0xFF, 0x4F, 0xA3);
    pub const ACCENT_RED: Color32 = Color32::from_rgb(0xE1, 0x4B, 0x4B);
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xED, 0xEA, 0xF0);
    pub const TEXT_ON_CANVAS: Color32 = Color32::from_rgb(0x2A, 0x26, 0x30);
    pub const GRID_LINE: Color32 = Color32::from_rgb(0xDD, 0xD8, 0xCE);

    pub const SIGNAL_LOW: Color32 = Color32::from_rgb(0x4A, 0x45, 0x50);
    pub const SIGNAL_HIGH: Color32 = Self::ACCENT_PINK;
    pub const SIGNAL_X: Color32 = Color32::from_rgb(0xB8, 0xA8, 0xC4);
    // Signal Z is drawn hollow with an outline

    pub fn glass_frame() -> egui::Frame {
        egui::Frame::new()
            .fill(Self::BG_PANEL_RAISED.gamma_multiply(0.88))
            .stroke(Stroke::new(1.0, Self::ACCENT_PURPLE.gamma_multiply(0.5)))
            .corner_radius(8.0)
            .inner_margin(16.0)
    }
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(Theme::TEXT_PRIMARY);
    visuals.panel_fill = Theme::BG_PANEL;
    visuals.window_fill = Theme::BG_PANEL_RAISED;
    visuals.window_stroke = Stroke::new(1.0, Theme::ACCENT_PURPLE);
    visuals.window_corner_radius = CornerRadius::same(8);

    // Non-interactive widgets
    visuals.widgets.noninteractive.bg_fill = Theme::BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke =
        Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.4));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Theme::TEXT_PRIMARY);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

    // Inactive buttons / chips
    visuals.widgets.inactive.bg_fill = Theme::BG_PANEL_RAISED;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.6));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Theme::TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

    // Hovered widgets
    visuals.widgets.hovered.bg_fill = Theme::BG_PANEL_RAISED;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, Theme::ACCENT_PINK);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

    // Active widgets
    visuals.widgets.active.bg_fill = Theme::ACCENT_PURPLE;
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, Theme::ACCENT_PINK);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);

    // Selection
    visuals.selection.bg_fill = Theme::ACCENT_PURPLE.gamma_multiply(0.4);
    visuals.selection.stroke = Stroke::new(1.5, Theme::ACCENT_PINK);

    ctx.set_visuals(visuals);
}
