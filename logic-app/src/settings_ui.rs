use crate::theme::{Theme, ThemeMode};
use crate::updater::{
    self, GITHUB_PROFILE_URL, GITHUB_RELEASES_URL, GITHUB_REPO, GITHUB_REPO_URL, GITHUB_USER,
    UpdateStatus, UpdaterState,
};
use eframe::egui::{self, CornerRadius, RichText, Sense, Stroke, Ui, Vec2};

pub struct SettingsUi;

impl SettingsUi {
    pub fn show_panel(
        ui: &mut Ui,
        updater: &UpdaterState,
        theme_mode: &mut ThemeMode,
        show_grid: &mut bool,
        snap_to_grid: &mut bool,
    ) {
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("SETTINGS & UPDATES")
                    .font(Theme::font_bold(13.5))
                    .color(Theme::TEXT_PRIMARY),
            );
        });

        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Card 1: GitHub & Repository Links
            Self::render_github_card(ui);

            ui.add_space(10.0);

            // Card 2: Software Updates & Auto-Update
            Self::render_updates_card(ui, updater);

            ui.add_space(10.0);

            // Card 3: Canvas & Interface Preferences
            Self::render_preferences_card(ui, theme_mode, show_grid, snap_to_grid);

            ui.add_space(16.0);
        });
    }

    fn render_github_card(ui: &mut Ui) {
        let frame = egui::Frame::new()
            .fill(Theme::BG_PANEL_RAISED)
            .stroke(Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.4)))
            .corner_radius(6.0)
            .inner_margin(10.0);

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("GITHUB REPOSITORY")
                        .font(Theme::font_bold(11.0))
                        .color(Theme::ACCENT_PURPLE),
                );
            });

            ui.add_space(6.0);

            // Repository details
            ui.horizontal(|ui| {
                let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(26.0, 18.0), Sense::hover());
                ui.painter().rect_filled(
                    badge_rect,
                    CornerRadius::same(3),
                    Theme::BG_CANVAS_DARK,
                );
                ui.painter().rect_stroke(
                    badge_rect,
                    CornerRadius::same(3),
                    Stroke::new(1.0, Theme::ACCENT_PINK),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "GH",
                    Theme::font_bold(8.5),
                    Theme::ACCENT_PINK,
                );

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("{}/{}", GITHUB_USER, GITHUB_REPO))
                            .font(Theme::font_bold(12.0))
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new("Digital Logic Circuit CAD & Simulator")
                            .font(Theme::font_regular(10.0))
                            .color(Theme::TEXT_MUTED),
                    );
                });
            });

            ui.add_space(8.0);

            // Link 1: Open GitHub Repository
            let repo_btn = egui::Button::new(
                RichText::new("↗  Open GitHub Repository")
                    .font(Theme::font_bold(10.5))
                    .color(Theme::ACCENT_PINK),
            )
            .fill(Theme::ACCENT_PINK.gamma_multiply(0.12))
            .stroke(Stroke::new(1.0, Theme::ACCENT_PINK))
            .corner_radius(CornerRadius::same(4));

            if ui
                .add_sized([ui.available_width(), 26.0], repo_btn)
                .on_hover_text("Open github.com/Tushar27-git/Grace in default browser")
                .clicked()
            {
                updater::open_browser(GITHUB_REPO_URL);
            }

            ui.add_space(5.0);

            // Link 2: Open Developer Profile
            let profile_btn = egui::Button::new(
                RichText::new(format!("↗  Developer Profile ({})", GITHUB_USER))
                    .font(Theme::font_medium(10.5))
                    .color(Theme::TEXT_PRIMARY),
            )
            .fill(Theme::BG_PANEL)
            .stroke(Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.5)))
            .corner_radius(CornerRadius::same(4));

            if ui
                .add_sized([ui.available_width(), 26.0], profile_btn)
                .on_hover_text("Open github.com/Tushar27-git in default browser")
                .clicked()
            {
                updater::open_browser(GITHUB_PROFILE_URL);
            }

            ui.add_space(5.0);

            // Link 3: Releases / Builds Tab
            let releases_btn = egui::Button::new(
                RichText::new("↗  Releases & Builds Tab")
                    .font(Theme::font_medium(10.5))
                    .color(Theme::TEXT_MUTED),
            )
            .fill(Theme::BG_PANEL)
            .stroke(Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.35)))
            .corner_radius(CornerRadius::same(4));

            if ui
                .add_sized([ui.available_width(), 24.0], releases_btn)
                .on_hover_text("View releases and binary builds on GitHub")
                .clicked()
            {
                updater::open_browser(GITHUB_RELEASES_URL);
            }
        });
    }

    fn render_updates_card(ui: &mut Ui, updater: &UpdaterState) {
        let frame = egui::Frame::new()
            .fill(Theme::BG_PANEL_RAISED)
            .stroke(Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.4)))
            .corner_radius(6.0)
            .inner_margin(10.0);

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("UPDATES & BUILDS")
                        .font(Theme::font_bold(11.0))
                        .color(Theme::ACCENT_PURPLE),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let version_text = format!("v{}", env!("CARGO_PKG_VERSION"));
                    ui.label(
                        RichText::new(version_text)
                            .font(Theme::font_bold(10.0))
                            .color(Theme::ACCENT_PINK),
                    );
                });
            });

            ui.add_space(6.0);

            let status = updater.status.lock().unwrap().clone();

            match status {
                UpdateStatus::Idle => {
                    ui.label(
                        RichText::new("Checks GitHub releases for new builds and provides clean 1-click auto update.")
                            .font(Theme::font_regular(10.5))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(8.0);
                    let check_btn = egui::Button::new(
                        RichText::new("Check for Updates")
                            .font(Theme::font_bold(11.0))
                            .color(Theme::ACCENT_PINK),
                    )
                    .fill(Theme::ACCENT_PINK.gamma_multiply(0.18))
                    .stroke(Stroke::new(1.0, Theme::ACCENT_PINK))
                    .corner_radius(CornerRadius::same(4));

                    if ui.add_sized([ui.available_width(), 28.0], check_btn).clicked() {
                        updater.check_for_updates();
                    }
                }
                UpdateStatus::Checking => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new("Checking GitHub for updates...")
                                .font(Theme::font_medium(11.0))
                                .color(Theme::ACCENT_PINK),
                        );
                    });
                }
                UpdateStatus::UpToDate {
                    current_version,
                    checked_time,
                } => {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("✓ Up to Date")
                                .font(Theme::font_bold(11.5))
                                .color(Theme::ACCENT_PINK),
                        );
                    });
                    ui.label(
                        RichText::new(format!("Version: {} ({})", current_version, checked_time))
                            .font(Theme::font_regular(10.0))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(6.0);

                    if ui
                        .button(RichText::new("Check Again").font(Theme::font_regular(10.0)))
                        .clicked()
                    {
                        updater.check_for_updates();
                    }
                }
                UpdateStatus::UpdateAvailable {
                    current_version: _,
                    latest_version,
                    release_url,
                    release_name,
                    release_notes,
                    download_url,
                    asset_name,
                } => {
                    let alert_frame = egui::Frame::new()
                        .fill(Theme::ACCENT_PINK.gamma_multiply(0.12))
                        .stroke(Stroke::new(1.0, Theme::ACCENT_PINK))
                        .corner_radius(4.0)
                        .inner_margin(8.0);

                    alert_frame.show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("★ Update Available: {}", latest_version))
                                .font(Theme::font_bold(12.0))
                                .color(Theme::ACCENT_PINK),
                        );
                        if !release_name.is_empty() {
                            ui.label(
                                RichText::new(release_name)
                                    .font(Theme::font_medium(10.5))
                                    .color(Theme::TEXT_PRIMARY),
                            );
                        }
                        if !release_notes.is_empty() {
                            let preview: String = release_notes.chars().take(120).collect();
                            ui.label(
                                RichText::new(preview)
                                    .font(Theme::font_regular(9.5))
                                    .color(Theme::TEXT_MUTED),
                            );
                        }
                    });

                    ui.add_space(8.0);

                    if let Some(dl_url) = download_url {
                        let btn_text = if let Some(ref a_name) = asset_name {
                            format!("⤓  Auto-Update Now ({})", a_name)
                        } else {
                            "⤓  Auto-Update Now".to_string()
                        };

                        let update_btn = egui::Button::new(
                            RichText::new(btn_text)
                                .font(Theme::font_bold(11.0))
                                .color(egui::Color32::WHITE),
                        )
                        .fill(Theme::ACCENT_PINK)
                        .corner_radius(CornerRadius::same(4));

                        if ui
                            .add_sized([ui.available_width(), 30.0], update_btn)
                            .on_hover_text("Downloads new executable build from GitHub and updates cleanly without wizard")
                            .clicked()
                        {
                            updater.start_auto_update(dl_url, latest_version.clone());
                        }
                    } else {
                        ui.label(
                            RichText::new("New release found on GitHub. Direct binary asset not attached.")
                                .font(Theme::font_regular(10.0))
                                .color(Theme::TEXT_MUTED),
                        );
                    }

                    ui.add_space(4.0);

                    let view_btn = egui::Button::new(
                        RichText::new("↗  View on GitHub Releases")
                            .font(Theme::font_regular(10.0))
                            .color(Theme::TEXT_PRIMARY),
                    )
                    .fill(Theme::BG_PANEL)
                    .corner_radius(CornerRadius::same(4));

                    if ui.add_sized([ui.available_width(), 24.0], view_btn).clicked() {
                        updater::open_browser(&release_url);
                    }
                }
                UpdateStatus::Downloading { progress_msg } => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new(&progress_msg)
                                .font(Theme::font_medium(11.0))
                                .color(Theme::ACCENT_PINK),
                        );
                    });
                    ui.label(
                        RichText::new("Downloading build artifact and replacing executable...")
                            .font(Theme::font_regular(10.0))
                            .color(Theme::TEXT_MUTED),
                    );
                }
                UpdateStatus::UpdatedRestartRequired { version, message } => {
                    let success_frame = egui::Frame::new()
                        .fill(Theme::ACCENT_PINK.gamma_multiply(0.18))
                        .stroke(Stroke::new(1.5, Theme::ACCENT_PINK))
                        .corner_radius(4.0)
                        .inner_margin(8.0);

                    success_frame.show(ui, |ui| {
                        ui.label(
                            RichText::new("✓ Update Complete!")
                                .font(Theme::font_bold(12.5))
                                .color(Theme::ACCENT_PINK),
                        );
                        ui.label(
                            RichText::new(format!("{}: {}", version, message))
                                .font(Theme::font_regular(10.5))
                                .color(Theme::TEXT_PRIMARY),
                        );
                    });

                    ui.add_space(8.0);

                    let restart_btn = egui::Button::new(
                        RichText::new("⟳  Restart Application Now")
                            .font(Theme::font_bold(11.5))
                            .color(egui::Color32::WHITE),
                    )
                    .fill(Theme::ACCENT_PINK)
                    .corner_radius(CornerRadius::same(4));

                    if ui
                        .add_sized([ui.available_width(), 32.0], restart_btn)
                        .on_hover_text("Relaunches Logic Lab with the updated version")
                        .clicked()
                    {
                        updater::restart_application();
                    }
                }
                UpdateStatus::Error(err_msg) => {
                    ui.label(
                        RichText::new(format!("Notice: {}", err_msg))
                            .font(Theme::font_regular(10.0))
                            .color(Theme::ACCENT_RED),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new("Check Again").font(Theme::font_regular(10.0)))
                            .clicked()
                        {
                            updater.check_for_updates();
                        }
                        if ui
                            .button(
                                RichText::new("Open GitHub Releases").font(Theme::font_regular(10.0)),
                            )
                            .clicked()
                        {
                            updater::open_browser(GITHUB_RELEASES_URL);
                        }
                    });
                }
            }
        });
    }

    fn render_preferences_card(
        ui: &mut Ui,
        theme_mode: &mut ThemeMode,
        show_grid: &mut bool,
        snap_to_grid: &mut bool,
    ) {
        let frame = egui::Frame::new()
            .fill(Theme::BG_PANEL_RAISED)
            .stroke(Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.4)))
            .corner_radius(6.0)
            .inner_margin(10.0);

        frame.show(ui, |ui| {
            ui.label(
                RichText::new("INTERFACE PREFERENCES")
                    .font(Theme::font_bold(11.0))
                    .color(Theme::ACCENT_PURPLE),
            );

            ui.add_space(8.0);

            // Theme toggle
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Color Theme:")
                        .font(Theme::font_medium(11.0))
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_text = if theme_mode.is_dark() {
                        "Dark Mode"
                    } else {
                        "Light Mode"
                    };
                    if ui.button(RichText::new(theme_text).font(Theme::font_bold(10.5))).clicked() {
                        theme_mode.toggle();
                    }
                });
            });

            ui.add_space(6.0);

            // Grid toggle
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Canvas Grid:")
                        .font(Theme::font_medium(11.0))
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let grid_text = if *show_grid { "Visible" } else { "Hidden" };
                    if ui.button(RichText::new(grid_text).font(Theme::font_bold(10.5))).clicked() {
                        *show_grid = !*show_grid;
                    }
                });
            });

            ui.add_space(6.0);

            // Snap toggle
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Grid Snapping:")
                        .font(Theme::font_medium(11.0))
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let snap_text = if *snap_to_grid { "Enabled" } else { "Disabled" };
                    if ui.button(RichText::new(snap_text).font(Theme::font_bold(10.5))).clicked() {
                        *snap_to_grid = !*snap_to_grid;
                    }
                });
            });
        });
    }
}
