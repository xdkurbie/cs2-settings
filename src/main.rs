//! CS2 Settings — live-game chrome recreation (centered column + GAME sub-tabs).
//! No Valve assets. Fonts: Barlow Condensed (OFL).
mod export;
mod presets;
mod settings;
mod theme;

use eframe::egui::{self, Align, Color32, FontFamily, FontId, Layout, RichText, Sense, Stroke, Vec2};
use export::{default_export_dir, export_all};
use settings::*;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1000.0, 640.0])
            .with_title("SETTINGS"),
        ..Default::default()
    };
    eframe::run_native(
        "CS2 Settings",
        options,
        Box::new(|cc| {
            theme::install_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(Cs2SettingsApp::default()))
        }),
    )
}

thread_local! { static SEARCH: std::cell::RefCell<String> = std::cell::RefCell::new(String::new()); }
fn search_hides(label: &str) -> bool {
    SEARCH.with(|q| { let q = q.borrow(); !q.trim().is_empty() && !label.to_lowercase().contains(&q.trim().to_lowercase()) })
}

struct Cs2SettingsApp {
    tab: Tab,
    game_sub: GameSubTab,
    video_sub: VideoSubTab,
    settings: Settings,
    baseline: Settings,
    status: String,
    last_export: Option<PathBuf>,
    search: String,
    video_advanced: bool,
    preview_bg: usize,
    dynamic_preview: bool,
    fonts_ready: bool,
    bg_tex: Option<egui::TextureHandle>,
}

impl Default for Cs2SettingsApp {
    fn default() -> Self {
        let settings = Settings::default();
        Self {
            tab: Tab::Game,
            game_sub: GameSubTab::Crosshair,
            video_sub: VideoSubTab::AdvancedVideo,
            baseline: settings.clone(),
            settings,
            status: String::new(),
            last_export: None,
            search: String::new(),
            video_advanced: true,
            preview_bg: 0,
            dynamic_preview: true,
            fonts_ready: false,
            bg_tex: None,
        }
    }
}

impl eframe::App for Cs2SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.fonts_ready {
            theme::install_fonts(ctx);
            self.fonts_ready = true;
        }
        theme::apply(ctx);
        SEARCH.with(|q| *q.borrow_mut() = self.search.clone());

        // Full-window dark / faux-blur backdrop
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::BLACK))
            .show(ctx, |ui| {
                // Decorative side vignette bars (blurred-map feel without Valve art)
                let full = ui.max_rect();
                // Blurred lobby backdrop (user-provided image, pre-blurred/darkened, embedded)
                let tex = self.bg_tex.get_or_insert_with(|| {
                    let img = image::load_from_memory(include_bytes!("../assets/lobby_bg.png"))
                        .expect("bg").to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    ctx.load_texture("lobby_bg",
                        egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()),
                        egui::TextureOptions::LINEAR)
                }).clone();
                // cover-fit
                let ts = tex.size_vec2();
                let sc = (full.width() / ts.x).max(full.height() / ts.y);
                let ds = ts * sc;
                let r = egui::Rect::from_center_size(full.center(), ds);
                ui.painter().image(tex.id(), r,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                // Centered settings column (~54% width)
                let col_w = (full.width() * 0.54).clamp(640.0, 900.0);
                let col_x = full.center().x - col_w * 0.5;
                let col = egui::Rect::from_min_size(
                    egui::pos2(col_x, full.top() + 10.0),
                    Vec2::new(col_w, full.height() - 20.0),
                );

                ui.painter().rect_filled(col.expand(6.0), 2.0, Color32::from_rgba_unmultiplied(10, 11, 13, 170));
                ui.scope_builder(egui::UiBuilder::new().max_rect(col), |ui| {
                    ui.set_clip_rect(col);
                    self.ui_chrome(ui, ctx);
                });
            });
    }
}

impl Cs2SettingsApp {
    fn ui_chrome(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Header row: SETTINGS title + search + close
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("SETTINGS")
                    .font(theme::title_font())
                    .color(Color32::WHITE),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("CLOSE")
                                .font(theme::button_font())
                                .color(theme::TEXT_DIM),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE),
                    )
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.add_space(10.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Search…")
                        .desired_width(160.0),
                );
            });
        });
        ui.add_space(6.0);

        // Top-level tabs (NEW | VIDEO | AUDIO | GAME | …) with TEAL selection
        self.draw_top_tabs(ui);
        ui.add_space(4.0);

        // Category sub-tabs
        if self.tab == Tab::Game {
            self.draw_game_subtabs(ui);
            ui.add_space(6.0);
        } else if self.tab == Tab::Video {
            self.draw_video_subtabs(ui);
            ui.add_space(6.0);
        }

        // Content + footer in remaining space
        let avail = ui.available_height();
        let footer_h = 52.0;
        let content_h = (avail - footer_h).max(120.0);

        egui::ScrollArea::vertical()
            .max_height(content_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                match self.tab {
                    Tab::New => self.ui_new(ui),
                    Tab::Video => match self.video_sub {
                        VideoSubTab::Video => self.ui_video_basic(ui),
                        VideoSubTab::AdvancedVideo => self.ui_video_advanced(ui),
                        VideoSubTab::HudEdgePositions => self.ui_hud_edge(ui),
                    },
                    Tab::Audio => self.ui_audio(ui),
                    Tab::Game => match self.game_sub {
                        GameSubTab::Game => self.ui_game_general(ui),
                        GameSubTab::Hud => self.ui_hud(ui),
                        GameSubTab::Team => self.ui_team(ui),
                        GameSubTab::Communication => self.ui_communication(ui),
                        GameSubTab::Spectator => self.ui_spectator(ui),
                        GameSubTab::Item => self.ui_item(ui),
                        GameSubTab::Radar => self.ui_radar(ui),
                        GameSubTab::Crosshair => self.ui_crosshair(ui),
                        GameSubTab::Telemetry => self.ui_telemetry(ui),
                    },
                    Tab::KeyboardMouse => self.ui_mouse(ui),
                    Tab::Controller => self.ui_controller(ui),
                    Tab::CustomCommands => self.ui_custom(ui),
                }
            });

        ui.add_space(6.0);
        // Footer: APPLY / EXPORT left-ish, RESET right (live CS2 has RESET bottom-right)
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("* Recommended setting for your system configuration")
                    .size(11.0)
                    .color(theme::TEXT_DIM),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if gold_btn(ui, "APPLY CHANGES").clicked() {
                    self.baseline = self.settings.clone();
                    self.status = "Applied (export to write cfg).".into();
                }
                ui.add_space(6.0);
                if dim_btn(ui, "EXPORT CONFIG").clicked() {
                    self.do_export(ctx);
                }
                ui.add_space(6.0);
                let dirty = self.settings != self.baseline;
                let discard = ui.add_enabled(
                    dirty,
                    egui::Button::new(RichText::new("DISCARD CHANGES").font(theme::button_font()).color(theme::TEXT_DIM))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE),
                );
                if discard.clicked() {
                    self.settings = self.baseline.clone();
                    self.status = "Discarded changes.".into();
                }
                ui.add_space(6.0);
                if dim_btn(ui, "RESET").clicked() {
                    match self.tab {
                        Tab::Game if self.game_sub == GameSubTab::Crosshair => {
                            self.settings.crosshair = Crosshair::default();
                            self.status = "Crosshair reset.".into();
                        }
                        Tab::Video => {
                            let keeps = self.settings.custom_commands.clone();
                            let binds = self.settings.binds.clone();
                            let xh = self.settings.crosshair.clone();
                            self.settings = Settings::default();
                            self.settings.custom_commands = keeps;
                            self.settings.binds = binds;
                            self.settings.crosshair = xh;
                            self.status = "Video defaults restored.".into();
                        }
                        _ => {
                            self.settings = Settings::default();
                            self.status = "All settings reset.".into();
                        }
                    }
                }
            });
        });
        if !self.status.is_empty() {
            ui.label(RichText::new(&self.status).size(12.0).color(theme::TEAL));
        }
    }

    fn draw_top_tabs(&mut self, ui: &mut egui::Ui) {
        let h = 34.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            // search icon placeholder
            ui.label(RichText::new("🔍").size(12.0).color(theme::TEXT_DIM));
            ui.add_space(6.0);
            for tab in Tab::all() {
                let selected = self.tab == *tab;
                let label = tab.label();
                let galley = ui.painter().layout_no_wrap(
                    label.to_owned(),
                    theme::tab_font(),
                    if selected { Color32::WHITE } else { theme::TEXT },
                );
                let w = (galley.size().x + 20.0).max(56.0);
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
                if selected {
                    ui.painter().rect_filled(rect, 0.0, theme::TEAL_DIM);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(rect.left(), rect.bottom() - 2.0),
                            rect.right_bottom(),
                        ),
                        0.0,
                        theme::TEAL,
                    );
                } else if resp.hovered() {
                    ui.painter().rect_filled(rect, 0.0, theme::ROW_HOVER);
                }
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    theme::tab_font(),
                    if selected { Color32::WHITE } else { theme::TEXT },
                );
                if resp.clicked() {
                    self.tab = *tab;
                    if *tab == Tab::Game {
                        // keep current sub
                    }
                }
            }
        });
        ui.painter().hline(
            ui.max_rect().x_range(),
            ui.cursor().top(),
            Stroke::new(1.0_f32, theme::BORDER_SOFT),
        );
    }

    fn draw_game_subtabs(&mut self, ui: &mut egui::Ui) {
        let h = 28.0;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for sub in GameSubTab::all() {
                let selected = self.game_sub == *sub;
                let label = sub.label();
                let galley = ui.painter().layout_no_wrap(
                    label.to_owned(),
                    FontId::new(12.0, FontFamily::Name("barlow_semibold".into())),
                    Color32::WHITE,
                );
                let w = (galley.size().x + 16.0).max(48.0);
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
                if selected {
                    ui.painter().rect_filled(rect, 0.0, theme::TEAL_DIM);
                } else if resp.hovered() {
                    ui.painter().rect_filled(rect, 0.0, theme::ROW_HOVER);
                }
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    FontId::new(12.0, FontFamily::Name("barlow_semibold".into())),
                    if selected { Color32::WHITE } else { theme::TEXT_DIM },
                );
                if resp.clicked() {
                    self.game_sub = *sub;
                }
            }
        });
    }

    fn draw_video_subtabs(&mut self, ui: &mut egui::Ui) {
        let h = 28.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for sub in VideoSubTab::all() {
                let selected = self.video_sub == *sub;
                let label = sub.label();
                let galley = ui.painter().layout_no_wrap(
                    label.to_owned(),
                    FontId::new(12.0, FontFamily::Name("barlow_semibold".into())),
                    Color32::WHITE,
                );
                let w = (galley.size().x + 16.0).max(48.0);
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
                if selected {
                    ui.painter().rect_filled(rect, 0.0, theme::TEAL_DIM);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(rect.left() + 4.0, rect.bottom() - 2.0),
                            egui::pos2(rect.right() - 4.0, rect.bottom()),
                        ),
                        0.0,
                        theme::TEAL,
                    );
                } else if resp.hovered() {
                    ui.painter().rect_filled(rect, 0.0, theme::ROW_HOVER);
                }
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    FontId::new(12.0, FontFamily::Name("barlow_semibold".into())),
                    if selected { Color32::WHITE } else { theme::TEXT_DIM },
                );
                if resp.clicked() {
                    self.video_sub = *sub;
                }
            }
        });
    }


    fn do_export(&mut self, _ctx: &egui::Context) {
        let dir = rfd::FileDialog::new()
            .set_title("Export CS2 config folder")
            .set_directory(default_export_dir())
            .pick_folder()
            .unwrap_or_else(default_export_dir);
        match export_all(&self.settings, &dir) {
            Ok(files) => {
                self.last_export = Some(dir.clone());
                self.status = format!("Exported {} files → {}", files.len(), dir.display());
                let _ = std::process::Command::new("open").arg(&dir).spawn();
            }
            Err(e) => self.status = format!("Export failed: {e}"),
        }
    }

    fn section_header(&self, ui: &mut egui::Ui, title: &str) {
        ui.add_space(8.0);
        ui.label(
            RichText::new(title)
                .font(theme::section_font())
                .italics()
                .color(Color32::WHITE),
        );
        ui.add_space(4.0);
    }

    fn setting_row<R>(
        ui: &mut egui::Ui,
        label: &str,
        badge: Option<&str>,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        if search_hides(label) {
            // filtered by search: lay out invisibly in a zero-height clip so state is untouched
            let mut out = None;
            let r = egui::Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 0.0));
            ui.scope_builder(egui::UiBuilder::new().max_rect(r).invisible(), |ui| {
                ui.set_clip_rect(egui::Rect::NOTHING);
                out = Some(add_contents(ui));
            });
            ui.add_space(-ui.spacing().item_spacing.y);
            return out.unwrap();
        }
        let mut result = None;
        egui::Frame::none()
            .fill(Color32::from_rgba_unmultiplied(32, 32, 32, 140))
            .inner_margin(egui::Margin::symmetric(12.0, 7.0))
            .show(ui, |ui| {
                let inner_w = ui.available_width();
                ui.set_min_width(inner_w);
                ui.horizontal(|ui| {
                    ui.set_min_height(26.0);
                    let label_w = (inner_w * 0.45).clamp(120.0, 360.0);
                    let control_w = (inner_w - label_w - 8.0).max(100.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(label_w, 26.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_max_width(label_w);
                            ui.add(egui::Label::new(
                                RichText::new(label)
                                    .font(theme::row_font())
                                    .color(theme::TEXT),
                            ).wrap()).on_hover_text(label);
                            if let Some(b) = badge {
                                ui.add_space(6.0);
                                egui::Frame::none()
                                    .fill(theme::GOLD)
                                    .inner_margin(egui::Margin::symmetric(5.0, 1.0))
                                    .rounding(2.0)
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(b)
                                                .size(10.0)
                                                .strong()
                                                .color(Color32::BLACK),
                                        );
                                    });
                            }
                        },
                    );
                    ui.allocate_ui_with_layout(
                        Vec2::new(control_w, 26.0),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            ui.set_max_width(control_w);
                            result = Some(add_contents(ui));
                        },
                    );
                });
            });
        // hairline
        ui.painter().hline(
            ui.max_rect().x_range(),
            ui.cursor().top(),
            Stroke::new(1.0_f32, theme::BORDER_SOFT),
        );
        result.expect("setting_row")
    }

    fn combo_quality(ui: &mut egui::Ui, id: &str, q: &mut Quality) {
        egui::ComboBox::from_id_salt(id)
            .selected_text(q.label())
            .width(160.0)
            .show_ui(ui, |ui| {
                for v in [Quality::Low, Quality::Medium, Quality::High, Quality::VeryHigh] {
                    ui.selectable_value(q, v, v.label());
                }
            });
    }

    fn toggle(ui: &mut egui::Ui, v: &mut bool) {
        let label = if *v { "YES" } else { "NO" };
        if ui
            .add(
                egui::Button::new(RichText::new(label).color(if *v {
                    theme::TEAL
                } else {
                    theme::TEXT_DIM
                }))
                .fill(theme::PANEL_ALT)
                .stroke(Stroke::new(1.0_f32, theme::BORDER))
                .min_size(Vec2::new(88.0, 24.0)),
            )
            .clicked()
        {
            *v = !*v;
        }
    }

    fn yes_no_combo(ui: &mut egui::Ui, id: &str, v: &mut bool) {
        let text = if *v { "YES" } else { "NO" };
        egui::ComboBox::from_id_salt(id)
            .selected_text(text)
            .width(120.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(v, true, "YES");
                ui.selectable_value(v, false, "NO");
            });
    }

    fn telemetry(ui: &mut egui::Ui, id: &str, m: &mut TelemetryMode) {
        egui::ComboBox::from_id_salt(id)
            .selected_text(m.label())
            .width(160.0)
            .show_ui(ui, |ui| {
                for v in [TelemetryMode::Never, TelemetryMode::MatchOnly, TelemetryMode::Always] {
                    ui.selectable_value(m, v, v.label());
                }
            });
    }

    fn ui_new(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "What's New");
        ui.label(
            RichText::new("Rush Hour (Sep 2026): crosshair length/thickness/gap are pixels at cl_crosshair_screen_height and scale to your screen. Share codes are now \"CS\" + 44 chars; old CSGO-xxxxx codes use a different scale.\nTest 2–3 candidates ~10 min DM each: bright walls, dark corners, smokes, far heads. Change one setting at a time.")
                .size(14.0)
                .color(theme::TEXT),
        );
        ui.add_space(12.0);
        self.section_header(ui, "Preset Packs");
        for p in presets::all() {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("APPLY").size(13.0).color(theme::TEAL)).clicked() {
                    (p.apply)(&mut self.settings);
                    self.status = format!("Applied preset: {} (Export to write cfg).", p.name);
                }
                ui.vertical(|ui| {
                    ui.label(RichText::new(p.name).size(16.0).color(theme::TEXT));
                    ui.label(RichText::new(p.summary).size(12.0).color(theme::TEXT));
                    ui.label(RichText::new(p.source).size(11.0).weak());
                });
            });
            ui.add_space(8.0);
        }
        ui.horizontal(|ui| {
            if ui.button(RichText::new("ROLL").size(13.0).color(theme::TEAL)).clicked() {
                presets::randomize_crosshair(&mut self.settings.crosshair);
                self.status = "Random crosshair rolled (Rush Hour cvars).".into();
            }
            ui.vertical(|ui| {
                ui.label(RichText::new("Random Crosshair (legacy RNG)").size(16.0).color(theme::TEXT));
                ui.label(RichText::new("Mark Tella's RNG roulette, using new length/thickness/gap names — lose a round, roll again.").size(12.0).color(theme::TEXT));
                ui.label(RichText::new("Mark Tella — steamcommunity.com/sharedfiles/filedetails/?id=3829117969").size(11.0).weak());
            });
        });
    }


    fn ui_video_basic(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Video");
        Self::setting_row(ui, "Brightness", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.brightness, 1.0..=2.6));
        });
        Self::setting_row(ui, "Color Mode", None, |ui| {
            egui::ComboBox::from_id_salt("colormode")
                .selected_text(if self.settings.color_mode == 0 { "Computer Monitor" } else { "Television" })
                .width(160.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.color_mode, 0, "Computer Monitor");
                    ui.selectable_value(&mut self.settings.color_mode, 1, "Television");
                });
        });
        Self::setting_row(ui, "Resolution", None, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.settings.resolution_w).range(640..=7680));
                ui.label("×");
                ui.add(egui::DragValue::new(&mut self.settings.resolution_h).range(480..=4320));
            });
        });
        Self::setting_row(ui, "Refresh Rate", None, |ui| {
            ui.add(egui::DragValue::new(&mut self.settings.refresh_hz).range(30..=540).suffix(" Hz"));
        });
        Self::setting_row(ui, "Display Mode", None, |ui| {
            egui::ComboBox::from_id_salt("disp").selected_text(self.settings.display_mode.label()).width(160.0).show_ui(ui, |ui| {
                for m in [DisplayMode::Fullscreen, DisplayMode::FullscreenWindowed, DisplayMode::Windowed] {
                    ui.selectable_value(&mut self.settings.display_mode, m, m.label());
                }
            });
        });
        Self::setting_row(ui, "Aspect Ratio Mode", None, |ui| {
            egui::ComboBox::from_id_salt("aspect")
                .selected_text(match self.settings.aspect_mode { 0 => "Normal / Native", 1 => "16:9", 2 => "4:3", _ => "Custom" })
                .width(160.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.aspect_mode, 0, "Normal / Native");
                    ui.selectable_value(&mut self.settings.aspect_mode, 1, "16:9");
                    ui.selectable_value(&mut self.settings.aspect_mode, 2, "4:3");
                });
        });
        if self.settings.aspect_mode == 2 && !search_hides("Aspect Ratio Mode") {
            ui.label(RichText::new("4:3 stretched: set Display Mode to Fullscreen and GPU scaling to \"Full-screen\" (NVIDIA) / \"Full panel\" (AMD), e.g. 1280×960 or 1440×1080.").size(12.0).color(theme::TEXT_DIM));
        }
        Self::setting_row(ui, "Laptop Power Savings", None, |ui| {
            Self::yes_no_combo(ui, "lps", &mut self.settings.laptop_power_savings);
        });
    }

    fn ui_hud_edge(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "HUD Edge Positions");
        ui.label(
            RichText::new("HUD element edge offsets are edited in-game with the interactive placer.\nThis tab keeps the entry point; values export via hud_scaling / related cvars.")
                .size(13.0)
                .color(theme::TEXT_DIM),
        );
        Self::setting_row(ui, "HUD Scale", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.hud_scaling, 0.5..=1.1));
        });
        Self::setting_row(ui, "HUD Edge Horizontal (safezonex)", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.safezonex, 0.85..=1.0));
        });
        Self::setting_row(ui, "HUD Edge Vertical (safezoney)", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.safezoney, 0.85..=1.0));
        });
    }

    /// Advanced Video: left live preview + magnified inset, right setting rows (nocries / live CS2).
    fn ui_video_advanced(&mut self, ui: &mut egui::Ui) {
        let full_w = ui.available_width();
        let left_w = (full_w * 0.46).clamp(260.0, 420.0);
        let right_w = (full_w - left_w - 16.0).max(280.0);

        ui.horizontal_top(|ui| {
            // LEFT preview column
            ui.vertical(|ui| {
                ui.set_width(left_w);
                let main_h = (left_w * 0.62).clamp(160.0, 240.0);
                egui::Frame::none()
                    .fill(theme::PREVIEW_DARK)
                    .stroke(Stroke::new(1.0_f32, theme::BORDER))
                    .show(ui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(left_w, main_h), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0, theme::PREVIEW_BG);
                        // faux map grid
                        let mut x = rect.left();
                        while x < rect.right() {
                            ui.painter().line_segment(
                                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                Stroke::new(1.0_f32, theme::PREVIEW_GRID),
                            );
                            x += 28.0;
                        }
                        // center cyan crosshair mark
                        let c = rect.center();
                        ui.painter().line_segment(
                            [egui::pos2(c.x - 10.0, c.y), egui::pos2(c.x + 10.0, c.y)],
                            Stroke::new(2.0_f32, theme::TEAL),
                        );
                        ui.painter().line_segment(
                            [egui::pos2(c.x, c.y - 10.0), egui::pos2(c.x, c.y + 10.0)],
                            Stroke::new(2.0_f32, theme::TEAL),
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 8.0, rect.bottom() - 6.0),
                            egui::Align2::LEFT_BOTTOM,
                            "PREVIEW",
                            FontId::proportional(11.0),
                            theme::TEXT_DIM,
                        );
                    });
                ui.add_space(6.0);
                // Magnified inset
                let mag_h = 72.0;
                egui::Frame::none()
                    .fill(Color32::from_rgb(0x28, 0x30, 0x28))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER))
                    .show(ui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(left_w, mag_h), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(0x32, 0x3C, 0x32));
                        ui.painter().text(
                            rect.left_top() + Vec2::new(8.0, 6.0),
                            egui::Align2::LEFT_TOP,
                            "MAGNIFIED",
                            FontId::proportional(11.0),
                            theme::TEXT_DIM,
                        );
                        // zoomed texture blocks
                        for i in 0..8 {
                            let x = rect.left() + 10.0 + i as f32 * 28.0;
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(egui::pos2(x, rect.center().y - 10.0), Vec2::splat(22.0)),
                                0.0,
                                Color32::from_rgb(0x4A + i * 8, 0x55, 0x40),
                            );
                        }
                    });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Magnification Mode").size(12.0).color(theme::TEXT));
                    egui::ComboBox::from_id_salt("magmode")
                        .selected_text(if self.settings.magnification_mode == 0 { "ZOOM" } else { "PAN" })
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.settings.magnification_mode, 0, "ZOOM");
                            ui.selectable_value(&mut self.settings.magnification_mode, 1, "PAN");
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Magnification").size(12.0).color(theme::TEXT));
                    ui.add(egui::Slider::new(&mut self.settings.magnification, 1.0..=4.0));
                });
            });

            ui.add_space(12.0);

            // RIGHT settings column
            ui.vertical(|ui| {
                ui.set_width(right_w);
                egui::ScrollArea::vertical().max_height(ui.available_height().max(360.0)).show(ui, |ui| {
                    self.section_header(ui, "Advanced Video");
                    Self::setting_row(ui, "Boost Player Contrast", None, |ui| {
                        Self::yes_no_combo(ui, "bpc", &mut self.settings.boost_player_contrast);
                    });

                    self.section_header(ui, "Frame Pacing");
                    Self::setting_row(ui, "V-Sync", None, |ui| {
                        Self::yes_no_combo(ui, "vsync", &mut self.settings.vsync);
                    });
                    Self::setting_row(ui, "NVIDIA G-Sync", None, |ui| {
                        if self.settings.nvidia_gsync {
                            ui.label(RichText::new("ENABLED").color(theme::TEAL));
                        } else {
                            ui.label(RichText::new("DISABLED").color(theme::TEXT_DIM));
                        }
                        ui.add_space(8.0);
                        if ui.small_button("VISIT NVIDIA").clicked() {
                            self.status = "Open NVIDIA control panel on Windows / driver UI.".into();
                        }
                    });
                    Self::setting_row(ui, "Maximum FPS In Game", None, |ui| {
                        ui.add(egui::Slider::new(&mut self.settings.fps_max, 0.0..=1000.0));
                    });
                    Self::setting_row(ui, "Maximum FPS In Menus", None, |ui| {
                        ui.add(egui::Slider::new(&mut self.settings.fps_max_ui, 0.0..=1000.0));
                    });
                    Self::setting_row(ui, "NVIDIA Reflex Low Latency", None, |ui| {
                        egui::ComboBox::from_id_salt("reflex").selected_text(self.settings.nvidia_reflex.label()).width(150.0).show_ui(ui, |ui| {
                            for m in [LowLatency::Disabled, LowLatency::Enabled, LowLatency::EnabledBoost] {
                                ui.selectable_value(&mut self.settings.nvidia_reflex, m, m.label());
                            }
                        });
                    });

                    self.section_header(ui, "Presets");
                    Self::setting_row(ui, "Current Video Values Preset", None, |ui| {
                        egui::ComboBox::from_id_salt("vpreset")
                            .selected_text(match self.settings.video_preset {
                                0 => "CUSTOM",
                                1 => "LOW",
                                2 => "MEDIUM",
                                3 => "HIGH",
                                _ => "VERY HIGH",
                            })
                            .width(150.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.settings.video_preset, 0, "CUSTOM");
                                ui.selectable_value(&mut self.settings.video_preset, 1, "LOW");
                                ui.selectable_value(&mut self.settings.video_preset, 2, "MEDIUM");
                                ui.selectable_value(&mut self.settings.video_preset, 3, "HIGH");
                                ui.selectable_value(&mut self.settings.video_preset, 4, "VERY HIGH");
                            });
                    });
                    Self::setting_row(ui, "Multisampling Anti-Aliasing Mode", None, |ui| {
                        egui::ComboBox::from_id_salt("msaa").selected_text(self.settings.msaa.label()).width(150.0).show_ui(ui, |ui| {
                            for m in [Msaa::None, Msaa::X2, Msaa::X4, Msaa::X8] {
                                ui.selectable_value(&mut self.settings.msaa, m, m.label());
                            }
                        });
                    });
                    Self::setting_row(ui, "Global Shadow Quality", None, |ui| {
                        Self::combo_quality(ui, "shadow", &mut self.settings.shadow_quality);
                    });
                    Self::setting_row(ui, "Dynamic Shadows", None, |ui| {
                        egui::ComboBox::from_id_salt("dsh").width(120.0)
                            .selected_text(if self.settings.dynamic_shadows { "ALL" } else { "SUN ONLY" })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.settings.dynamic_shadows, false, "SUN ONLY");
                                ui.selectable_value(&mut self.settings.dynamic_shadows, true, "ALL");
                            });
                    });
                    Self::setting_row(ui, "Model / Texture Detail", None, |ui| {
                        Self::combo_quality(ui, "tex", &mut self.settings.model_texture);
                    });
                    Self::setting_row(ui, "Texture Filtering Mode", None, |ui| {
                        egui::ComboBox::from_id_salt("texfilt").selected_text(self.settings.texture_filter.label()).width(150.0).show_ui(ui, |ui| {
                            for m in [TextureFilter::Bilinear, TextureFilter::Trilinear, TextureFilter::Aniso2x, TextureFilter::Aniso4x, TextureFilter::Aniso8x, TextureFilter::Aniso16x] {
                                ui.selectable_value(&mut self.settings.texture_filter, m, m.label());
                            }
                        });
                    });
                    Self::setting_row(ui, "Shader Detail", None, |ui| {
                        Self::combo_quality(ui, "shader", &mut self.settings.shader_detail);
                    });
                    Self::setting_row(ui, "Particle Detail", None, |ui| {
                        Self::combo_quality(ui, "part", &mut self.settings.particle_detail);
                    });
                    Self::setting_row(ui, "Ambient Occlusion", None, |ui| {
                        Self::yes_no_combo(ui, "ao", &mut self.settings.ambient_occlusion);
                    });
                    Self::setting_row(ui, "High Dynamic Range", None, |ui| {
                        egui::ComboBox::from_id_salt("hdr")
                            .selected_text(match self.settings.hdr_detail { -1 => "PERFORMANCE", 0 => "QUALITY", _ => "CUSTOM" })
                            .width(150.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.settings.hdr_detail, -1, "PERFORMANCE");
                                ui.selectable_value(&mut self.settings.hdr_detail, 0, "QUALITY");
                                ui.selectable_value(&mut self.settings.hdr_detail, 1, "CUSTOM");
                            });
                    });
                    Self::setting_row(ui, "FidelityFX Super Resolution", None, |ui| {
                        egui::ComboBox::from_id_salt("fsr")
                            .selected_text(match self.settings.fsr_quality {
                                0 => "DISABLED (HIGHEST QUALITY)",
                                1 => "ULTRA QUALITY",
                                2 => "QUALITY",
                                3 => "BALANCED",
                                _ => "PERFORMANCE",
                            })
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.settings.fsr_quality, 0, "DISABLED (HIGHEST QUALITY)");
                                ui.selectable_value(&mut self.settings.fsr_quality, 1, "ULTRA QUALITY");
                                ui.selectable_value(&mut self.settings.fsr_quality, 2, "QUALITY");
                                ui.selectable_value(&mut self.settings.fsr_quality, 3, "BALANCED");
                                ui.selectable_value(&mut self.settings.fsr_quality, 4, "PERFORMANCE");
                            });
                    });
                    Self::setting_row(ui, "FXAA / CMAA Anti-Aliasing", None, |ui| {
                        Self::yes_no_combo(ui, "cmaa", &mut self.settings.fxaa_cmaa);
                    });
                    Self::setting_row(ui, "Multicore Rendering", None, |ui| {
                        Self::yes_no_combo(ui, "mcr", &mut self.settings.multicore);
                    });
                    Self::setting_row(ui, "Motion Blur", None, |ui| {
                        Self::yes_no_combo(ui, "mblur", &mut self.settings.motion_blur);
                    });
                });
            });
        });
    }


    fn ui_audio(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Audio");
        Self::setting_row(ui, "Master Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Audio Device", None, |ui| {
            ui.add(egui::TextEdit::singleline(&mut self.settings.audio_device).desired_width(180.0));
        });
        Self::setting_row(ui, "EQ Profile", None, |ui| {
            egui::ComboBox::from_id_salt("eq").selected_text(self.settings.eq_profile.label()).width(160.0).show_ui(ui, |ui| {
                for m in [EqProfile::Headphones, EqProfile::PowerEQ, EqProfile::Natural] {
                    ui.selectable_value(&mut self.settings.eq_profile, m, m.label());
                }
            });
        });
        Self::setting_row(ui, "L/R Isolation", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.lr_isolation, 0.0..=1.0)); });
        Self::setting_row(ui, "Perspective Correction", None, |ui| { Self::toggle(ui, &mut self.settings.perspective_correction); });
        Self::setting_row(ui, "Headphones / HRTF Preferred", None, |ui| { Self::toggle(ui, &mut self.settings.headphone); });
        Self::setting_row(ui, "Play Audio In Background", None, |ui| {
            let mut play = !self.settings.mute_losefocus;
            Self::toggle(ui, &mut play);
            self.settings.mute_losefocus = !play;
        });
        self.section_header(ui, "Voice");
        Self::setting_row(ui, "Enable Voice", None, |ui| { Self::toggle(ui, &mut self.settings.enable_voice); });
        Self::setting_row(ui, "Voice Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.voip_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Mic Threshold", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.voice_threshold, 0.0..=1.0)); });
        Self::setting_row(ui, "Voice Loopback", None, |ui| { Self::toggle(ui, &mut self.settings.voice_loopback); });
        self.section_header(ui, "Music");
        for (label, field) in [
            ("Music Volume", true),
        ] {
            let _ = label;
            let _ = field;
        }
        Self::setting_row(ui, "Music Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.music_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Menu Music Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.menumusic_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Main Menu Ambience", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.menumap_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Round Start Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.roundstart_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Round Action Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.roundaction_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Round End Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.roundend_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "MVP Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.mvp_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Bomb / Hostage Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.mapobjective_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "10 Second Warning Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.tensecond_warning, 0.0..=1.0)); });
        Self::setting_row(ui, "Death Camera Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.deathcamera_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Mute MVP Music (Live Players)", None, |ui| { Self::toggle(ui, &mut self.settings.mute_mvp_music_live_players); });
    }

    fn ui_game_general(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Game");
        Self::setting_row(ui, "Enable Developer Console (~)", None, |ui| { Self::toggle(ui, &mut self.settings.enable_console); });
        Self::setting_row(ui, "Max Acceptable Matchmaking Ping", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.mm_dedicated_search_maxping, 25..=350).suffix(" ms"));
        });
        Self::setting_row(ui, "Max Acceptable Game Traffic Bandwidth", None, |ui| {
            ui.add(egui::DragValue::new(&mut self.settings.rate).range(98304..=1000000));
        });
        Self::setting_row(ui, "Buffering for Packet Loss / Jitter", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.cl_net_buffer_ticks, 0..=2));
        });
        Self::setting_row(ui, "Switch Weapon on Pickup", None, |ui| { Self::yes_no_combo(ui, "awp", &mut self.settings.cl_autowepswitch); });
        Self::setting_row(ui, "Enable Game Instructor Messages", None, |ui| { Self::yes_no_combo(ui, "gi", &mut self.settings.game_instructor); });
        Self::setting_row(ui, "Right Hand Weapon Model", None, |ui| { Self::yes_no_combo(ui, "rh", &mut self.settings.cl_righthand); });
        Self::setting_row(ui, "Viewmodel FOV", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.viewmodel_fov, 54.0..=68.0)); });
        Self::setting_row(ui, "Viewmodel Offset X", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.viewmodel_offset_x, -2.0..=2.5)); });
        Self::setting_row(ui, "Viewmodel Offset Y", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.viewmodel_offset_y, -2.0..=2.0)); });
        Self::setting_row(ui, "Viewmodel Offset Z", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.viewmodel_offset_z, -2.0..=2.0)); });
        Self::setting_row(ui, "cl_interp_ratio", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.cl_interp_ratio, 1..=2)); });
    }

    fn ui_hud(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "HUD");
        Self::setting_row(ui, "HUD Scale", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.hud_scaling, 0.5..=1.1)); });
        Self::setting_row(ui, "HUD Color", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.cl_hud_color, 0..=12)); });
        Self::setting_row(ui, "Close Buy Menu After Purchase", None, |ui| { Self::yes_no_combo(ui, "cbuy", &mut self.settings.close_buymenu_after_purchase); });
        Self::setting_row(ui, "Always Show Inventory", None, |ui| { Self::yes_no_combo(ui, "ainv", &mut self.settings.always_show_inventory); });
        Self::setting_row(ui, "Network Graph", None, |ui| { Self::yes_no_combo(ui, "ng", &mut self.settings.net_graph); });
    }

    fn ui_team(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Team");
        Self::setting_row(ui, "Friends Lobby / Join Advertise", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.cl_join_advertise, 0..=2));
        });
        ui.label(RichText::new("More team options map to Steam / lobby UI; export covers join advertise.").size(12.0).color(theme::TEXT_DIM));
    }

    fn ui_communication(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Communication");
        Self::setting_row(ui, "Enable Voice", None, |ui| { Self::yes_no_combo(ui, "cvoice", &mut self.settings.enable_voice); });
        Self::setting_row(ui, "Voice Volume", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.voip_volume, 0.0..=1.0)); });
        Self::setting_row(ui, "Mic Threshold", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.voice_threshold, 0.0..=1.0)); });
        ui.label(RichText::new("Text chat / mute filters are primarily in-game; voice cvars export to autoexec.").size(12.0).color(theme::TEXT_DIM));
    }

    fn ui_spectator(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Spectator / Scoreboard");
        Self::setting_row(ui, "Show Player Crosshairs", None, |ui| {
            egui::ComboBox::from_id_salt("obscross")
                .selected_text(match self.settings.crosshair.show_observer_crosshair { 0 => "No", 1 => "Friends and Party", _ => "Everyone" })
                .width(160.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.crosshair.show_observer_crosshair, 0, "No");
                    ui.selectable_value(&mut self.settings.crosshair.show_observer_crosshair, 1, "Friends and Party");
                    ui.selectable_value(&mut self.settings.crosshair.show_observer_crosshair, 2, "Everyone");
                });
        });
        Self::setting_row(ui, "Show My Crosshair Spectating Bots", None, |ui| {
            egui::ComboBox::from_id_salt("botcross")
                .selected_text(match self.settings.crosshair.observed_bot_crosshair { 0 => "Never", 1 => "When I can take over", _ => "Always" })
                .width(160.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.crosshair.observed_bot_crosshair, 0, "Never");
                    ui.selectable_value(&mut self.settings.crosshair.observed_bot_crosshair, 1, "When I can take over");
                    ui.selectable_value(&mut self.settings.crosshair.observed_bot_crosshair, 2, "Always");
                });
        });
    }

    fn ui_item(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Item");
        Self::setting_row(ui, "Always Show Inventory", None, |ui| { Self::yes_no_combo(ui, "item_inv", &mut self.settings.always_show_inventory); });
        ui.label(RichText::new("Loadout / item rarity filters live in CS2 inventory UI.").size(12.0).color(theme::TEXT_DIM));
    }

    fn ui_radar(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Radar");
        Self::setting_row(ui, "Radar Always Centered", None, |ui| { Self::yes_no_combo(ui, "rc", &mut self.settings.radar_always_centered); });
        Self::setting_row(ui, "Radar Rotate With Player", None, |ui| { Self::yes_no_combo(ui, "rr", &mut self.settings.radar_rotate); });
        Self::setting_row(ui, "Radar Map Zoom / Scale", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.radar_scale, 0.25..=1.0)); });
        Self::setting_row(ui, "Radar HUD Size", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.radar_hud_size, 0.5..=1.3)); });
        Self::setting_row(ui, "Radar Icon Scale", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.radar_icon_scale, 0.4..=1.0)); });
        Self::setting_row(ui, "Square Radar With Scoreboard", None, |ui| { Self::yes_no_combo(ui, "rsq", &mut self.settings.radar_square_with_scoreboard); });
    }

    fn ui_telemetry(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Telemetry");
        Self::setting_row(ui, "Show Frame Time / FPS", None, |ui| { Self::telemetry(ui, "tel_fps", &mut self.settings.telemetry_fps); });
        Self::setting_row(ui, "Show Ping", None, |ui| { Self::telemetry(ui, "tel_ping", &mut self.settings.telemetry_ping); });
        Self::setting_row(ui, "Show Packet Loss / Misdelivery", None, |ui| { Self::telemetry(ui, "tel_loss", &mut self.settings.telemetry_packet_loss); });
        Self::setting_row(ui, "Show Network Quality Graph", None, |ui| { Self::telemetry(ui, "tel_nq", &mut self.settings.telemetry_net_quality); });
        Self::setting_row(ui, "Hide Player Avatars", None, |ui| { Self::toggle(ui, &mut self.settings.hide_avatar_images); });
        Self::setting_row(ui, "Team Counter: Player Count Instead of Avatars", None, |ui| { Self::toggle(ui, &mut self.settings.teamcounter_playercount); });
        Self::setting_row(ui, "Mute Enemy Team Voice", None, |ui| { Self::toggle(ui, &mut self.settings.mute_enemy_team); });
    }

    fn ui_crosshair(&mut self, ui: &mut egui::Ui) {
        // Wide short preview (real CS2 ~21:9-ish)
        let avail_w = ui.available_width();
        let preview_h = (avail_w / 21.0 * 9.0).clamp(120.0, 200.0);

        egui::Frame::none()
            .fill(theme::PREVIEW_DARK)
            .stroke(Stroke::new(1.0_f32, theme::BORDER))
            .show(ui, |ui| {
                ui.set_width(avail_w);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, preview_h), Sense::hover());
                let (bg, grid) = match self.preview_bg % 4 {
                    0 => (theme::PREVIEW_BG, theme::PREVIEW_GRID),
                    1 => (Color32::from_rgb(0x4A, 0x3A, 0x2E), Color32::from_rgb(0x5A, 0x48, 0x38)),
                    2 => (Color32::from_rgb(0x2E, 0x3A, 0x4A), Color32::from_rgb(0x38, 0x48, 0x5A)),
                    _ => (Color32::from_rgb(0x3A, 0x3A, 0x3A), Color32::from_rgb(0x4A, 0x4A, 0x4A)),
                };
                ui.painter().rect_filled(rect, 0.0, bg);
                let mut x = rect.left();
                while x < rect.right() {
                    ui.painter().line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], Stroke::new(1.0_f32, grid));
                    x += 40.0;
                }
                let left = egui::Rect::from_center_size(egui::pos2(rect.left() + 24.0, rect.center().y), Vec2::splat(32.0));
                let right = egui::Rect::from_center_size(egui::pos2(rect.right() - 24.0, rect.center().y), Vec2::splat(32.0));
                if ui.interact(left, ui.id().with("pl"), Sense::click()).clicked() { self.preview_bg = self.preview_bg.wrapping_sub(1); }
                if ui.interact(right, ui.id().with("pr"), Sense::click()).clicked() { self.preview_bg = self.preview_bg.wrapping_add(1); }
                ui.painter().text(left.center(), egui::Align2::CENTER_CENTER, "‹", FontId::proportional(26.0), Color32::WHITE);
                ui.painter().text(right.center(), egui::Align2::CENTER_CENTER, "›", FontId::proportional(26.0), Color32::WHITE);
                paint_crosshair(ui, rect.center(), &self.settings.crosshair, theme::CROSSHAIR_PREVIEW_ZOOM);
                // Share / Undo / Reset bottom-right of preview (live CS2)
                let links = ["Share or Import", "Undo Changes", "Reset"];
                let mut lx = rect.right() - 12.0;
                for (i, t) in links.iter().enumerate().rev() {
                    let g = ui.painter().layout_no_wrap((*t).to_owned(), FontId::proportional(12.0), theme::TEXT_DIM);
                    lx -= g.size().x;
                    let r = egui::Rect::from_min_size(egui::pos2(lx, rect.bottom() - 22.0), g.size());
                    let resp = ui.interact(r, ui.id().with(("xhlink", i)), Sense::click());
                    ui.painter().text(r.left_center(), egui::Align2::LEFT_CENTER, *t, FontId::proportional(12.0), if resp.hovered() { Color32::WHITE } else { theme::TEXT_DIM });
                    if resp.clicked() {
                        match i {
                            2 => { self.settings.crosshair = Crosshair::default(); self.status = "Crosshair reset.".into(); }
                            1 => { self.settings.crosshair = self.baseline.crosshair.clone(); self.status = "Undo crosshair.".into(); }
                            _ => {}
                        }
                    }
                    lx -= 14.0;
                }
            });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Share code").size(12.0).color(theme::TEXT_DIM));
            ui.add(egui::TextEdit::singleline(&mut self.settings.crosshair.share_code).hint_text("CS…").desired_width(ui.available_width().min(420.0)));
        });

        Self::setting_row(ui, "Style", Some("Updated"), |ui| {
            egui::ComboBox::from_id_salt("xhstyle")
                .selected_text(format!("{} — {}", self.settings.crosshair.style.as_u8(), self.settings.crosshair.style.label()))
                .width(200.0)
                .show_ui(ui, |ui| {
                    for s in CrosshairStyle::all() {
                        ui.selectable_value(&mut self.settings.crosshair.style, *s, format!("{} — {}", s.as_u8(), s.label()));
                    }
                });
        });
        Self::setting_row(ui, "Friendly Fire Reticle Warning", None, |ui| {
            Self::yes_no_combo(ui, "ffw", &mut self.settings.crosshair.friendly_warning);
        });
        Self::setting_row(ui, "Follow Recoil", None, |ui| { Self::yes_no_combo(ui, "fr", &mut self.settings.crosshair.follow_recoil); });
        Self::setting_row(ui, "Center Dot", None, |ui| { Self::yes_no_combo(ui, "dot", &mut self.settings.crosshair.dot); });
        Self::setting_row(ui, "Length (px)", Some("New"), |ui| { ui.add(egui::Slider::new(&mut self.settings.crosshair.length, 0.0..=32.0)); });
        Self::setting_row(ui, "Thickness (px)", Some("New"), |ui| { ui.add(egui::Slider::new(&mut self.settings.crosshair.thickness, 0.0..=32.0)); });
        Self::setting_row(ui, "Gap (px)", Some("New"), |ui| { ui.add(egui::Slider::new(&mut self.settings.crosshair.gap, -10.0..=128.0)); });
        Self::setting_row(ui, "Screen Height Base", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.crosshair.screen_height, 240.0..=4320.0));
        });
        Self::setting_row(ui, "Outline", Some("Updated"), |ui| {
            egui::ComboBox::from_id_salt("xhout").selected_text(self.settings.crosshair.outline.label()).width(140.0).show_ui(ui, |ui| {
                for m in [OutlineMode::Off, OutlineMode::Full, OutlineMode::Half] {
                    ui.selectable_value(&mut self.settings.crosshair.outline, m, m.label());
                }
            });
        });
        Self::setting_row(ui, "Color and Transparency", Some("Updated"), |ui| {
            let c = &mut self.settings.crosshair;
            let mut rgba = [c.color_r as f32 / 255.0, c.color_g as f32 / 255.0, c.color_b as f32 / 255.0, c.color_a as f32 / 255.0];
            if ui.color_edit_button_rgba_unmultiplied(&mut rgba).changed() {
                c.color_r = (rgba[0] * 255.0).round() as u8;
                c.color_g = (rgba[1] * 255.0).round() as u8;
                c.color_b = (rgba[2] * 255.0).round() as u8;
                c.color_a = (rgba[3] * 255.0).round() as u8;
            }
        });
        Self::setting_row(ui, "Red", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.crosshair.color_r, 0..=255)); });
        Self::setting_row(ui, "Green", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.crosshair.color_g, 0..=255)); });
        Self::setting_row(ui, "Blue", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.crosshair.color_b, 0..=255)); });
        Self::setting_row(ui, "Alpha", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.crosshair.color_a, 0..=255)); });
        Self::setting_row(ui, "Outline Color", Some("New"), |ui| {
            let c = &mut self.settings.crosshair;
            let mut rgba = [c.outline_r as f32 / 255.0, c.outline_g as f32 / 255.0, c.outline_b as f32 / 255.0, c.outline_a as f32 / 255.0];
            if ui.color_edit_button_rgba_unmultiplied(&mut rgba).changed() {
                c.outline_r = (rgba[0] * 255.0).round() as u8;
                c.outline_g = (rgba[1] * 255.0).round() as u8;
                c.outline_b = (rgba[2] * 255.0).round() as u8;
                c.outline_a = (rgba[3] * 255.0).round() as u8;
            }
        });
        Self::setting_row(ui, "T Style", None, |ui| { Self::yes_no_combo(ui, "tstyle", &mut self.settings.crosshair.t_style); });
        Self::setting_row(ui, "Dynamic Spread Limit", Some("New"), |ui| {
            ui.add(egui::Slider::new(&mut self.settings.crosshair.dynamic_spread_limit, 0.0..=255.0));
        });
        Self::setting_row(ui, "Split Distance", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.crosshair.dynamic_splitdist, 0.0..=127.0));
        });
        Self::setting_row(ui, "Split Alpha Inner", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.crosshair.dynamic_splitalpha_innermod, 0.0..=1.0));
        });
        Self::setting_row(ui, "Split Alpha Outer", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.crosshair.dynamic_splitalpha_outermod, 0.3..=1.0));
        });
        Self::setting_row(ui, "Max Dist Split Ratio", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.crosshair.dynamic_maxdist_splitratio, 0.0..=1.0));
        });
        Self::setting_row(ui, "Sniper Crosshair Width", None, |ui| {
            ui.add(egui::Slider::new(&mut self.settings.crosshair.sniper_width, 0.0..=20.0));
        });
        Self::setting_row(ui, "Sniper Show Movement Inaccuracy", None, |ui| {
            Self::yes_no_combo(ui, "snipacc", &mut self.settings.crosshair.sniper_show_normal_inaccuracy);
        });
        Self::setting_row(ui, "Ironsight Dot Scale", Some("New"), |ui| {
            ui.add(egui::Slider::new(&mut self.settings.crosshair.ironsight_dot_scale, 0.1..=2.0));
        });
        Self::setting_row(ui, "Ironsight Use Crosshair Color", Some("New"), |ui| {
            Self::yes_no_combo(ui, "ironcol", &mut self.settings.crosshair.ironsight_use_crosshair_color);
        });
    }

    fn ui_mouse(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Keyboard / Mouse");
        Self::setting_row(ui, "Mouse Sensitivity", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.sensitivity, 0.0001..=8.0)); });
        Self::setting_row(ui, "Zoom Sensitivity Ratio", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.zoom_sensitivity_ratio, 0.01..=3.0)); });
        Self::setting_row(ui, "Raw Input", None, |ui| { Self::yes_no_combo(ui, "raw", &mut self.settings.m_rawinput); });
        Self::setting_row(ui, "Mouse Acceleration", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.m_customaccel, 0..=3)); });
        Self::setting_row(ui, "Reverse Mouse", None, |ui| { Self::yes_no_combo(ui, "invy", &mut self.settings.mouse_inverty); });
        Self::setting_row(ui, "Duck Mode (Toggle)", None, |ui| { Self::yes_no_combo(ui, "duck", &mut self.settings.option_duck_method); });
        Self::setting_row(ui, "Walk Mode (Toggle)", None, |ui| { Self::yes_no_combo(ui, "walk", &mut self.settings.option_speed_method); });
        Self::setting_row(ui, "Zoom Button Hold", None, |ui| { Self::yes_no_combo(ui, "zdeb", &mut self.settings.cl_debounce_zoom); });

        self.section_header(ui, "Binds");
        let mut remove = None;
        for (i, bind) in self.settings.binds.iter_mut().enumerate() {
            egui::Frame::none().fill(theme::ROW).inner_margin(egui::Margin::symmetric(8.0, 5.0)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut bind.key).desired_width(90.0));
                    ui.label(RichText::new("→").color(theme::TEAL));
                    ui.add(egui::TextEdit::singleline(&mut bind.command).desired_width(ui.available_width().min(320.0).max(140.0)));
                    if ui.small_button("✕").clicked() { remove = Some(i); }
                });
            });
        }
        if let Some(i) = remove { self.settings.binds.remove(i); }
        if dim_btn(ui, "+ ADD BIND").clicked() {
            self.settings.binds.push(Bind { key: String::new(), command: String::new() });
        }
    }

    fn ui_controller(&mut self, ui: &mut egui::Ui) {
        self.section_header(ui, "Controller");
        Self::setting_row(ui, "Enable Controller", None, |ui| { Self::yes_no_combo(ui, "joy", &mut self.settings.joy_enabled); });
        Self::setting_row(ui, "Look Sensitivity", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.joy_sensitivity, 0.1..=5.0)); });
        Self::setting_row(ui, "Pitch Sensitivity", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.joy_pitchsensitivity, 0.1..=5.0)); });
        Self::setting_row(ui, "Yaw Sensitivity", None, |ui| { ui.add(egui::Slider::new(&mut self.settings.joy_yawsensitivity, 0.1..=5.0)); });
        Self::setting_row(ui, "Invert Y Axis", None, |ui| { Self::yes_no_combo(ui, "jinv", &mut self.settings.joy_invertedy); });
        Self::setting_row(ui, "Movement Stick", None, |ui| {
            egui::ComboBox::from_id_salt("jstick")
                .selected_text(if self.settings.joy_movement_stick == 0 { "Left" } else { "Right" })
                .width(120.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.joy_movement_stick, 0, "Left");
                    ui.selectable_value(&mut self.settings.joy_movement_stick, 1, "Right");
                });
        });
        Self::setting_row(ui, "Aim Assist", None, |ui| { Self::yes_no_combo(ui, "jaa", &mut self.settings.joy_aim_assist); });
    }

    fn ui_custom(&mut self, ui: &mut egui::Ui) {
        Self::setting_row(ui, "Steam Launch Options", None, |ui| {
            ui.add(egui::TextEdit::singleline(&mut self.settings.launch_options).hint_text("-novid -fullscreen -console").desired_width(260.0));
        });
        self.section_header(ui, "Custom Commands");
        ui.label(RichText::new("Raw console lines → autoexec.cfg (disabled lines become comments).").size(12.0).color(theme::TEXT_DIM));
        ui.add_space(6.0);
        let mut remove = None;
        for (i, cmd) in self.settings.custom_commands.iter_mut().enumerate() {
            egui::Frame::none().fill(theme::ROW).inner_margin(egui::Margin::symmetric(8.0, 6.0)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut cmd.enabled, "");
                    ui.add(egui::TextEdit::singleline(&mut cmd.line).desired_width(ui.available_width().min(640.0).max(180.0)).hint_text("cl_showfps 1"));
                    if ui.small_button("✕").clicked() { remove = Some(i); }
                });
            });
        }
        if let Some(i) = remove { self.settings.custom_commands.remove(i); }
        if dim_btn(ui, "+ ADD COMMAND").clicked() {
            self.settings.custom_commands.push(CustomCommand { line: String::new(), enabled: true });
        }
    }
}

fn gold_btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).font(theme::button_font()).color(Color32::BLACK))
            .fill(theme::GOLD)
            .rounding(0.0)
            .min_size(Vec2::new(120.0, 32.0)),
    )
}

fn dim_btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).font(theme::button_font()).color(theme::TEXT))
            .fill(theme::PANEL_ALT)
            .stroke(Stroke::new(1.0_f32, theme::BORDER))
            .rounding(0.0)
            .min_size(Vec2::new(120.0, 32.0)),
    )
}

fn paint_crosshair(ui: &mut egui::Ui, center: egui::Pos2, x: &Crosshair, zoom: f32) {
    let len = (x.length * zoom).max(0.0);
    let gap = x.gap * zoom;
    let thick = (x.thickness * zoom).max(1.0);
    let color = Color32::from_rgba_unmultiplied(x.color_r, x.color_g, x.color_b, x.color_a);
    let ocolor = Color32::from_rgba_unmultiplied(x.outline_r, x.outline_g, x.outline_b, x.outline_a);
    let painter = ui.painter();
    let draw_bar = |from: egui::Pos2, to: egui::Pos2| {
        if x.outline != OutlineMode::Off {
            let expand = if x.outline == OutlineMode::Half { 0.5 } else { 1.0 };
            painter.line_segment([from, to], Stroke::new(thick + expand * 2.0 * zoom / 3.5, ocolor));
        }
        painter.line_segment([from, to], Stroke::new(thick, color));
    };
    match x.style {
        CrosshairStyle::DotOnly | CrosshairStyle::StaticCircle | CrosshairStyle::DynamicCircle => {
            let r = (len + thick).max(2.0 * zoom / 3.5);
            if x.outline != OutlineMode::Off {
                painter.circle_stroke(center, r + zoom * 0.3, Stroke::new(1.5_f32, ocolor));
            }
            painter.circle_stroke(center, r, Stroke::new(thick.max(1.5), color));
            if x.dot || matches!(x.style, CrosshairStyle::DotOnly) {
                painter.circle_filled(center, thick.max(1.5), color);
            }
        }
        CrosshairStyle::StaticSquare | CrosshairStyle::StaticQuad | CrosshairStyle::DynamicQuad => {
            let half = (gap.abs() + len).max(4.0);
            let rect = egui::Rect::from_center_size(center, Vec2::splat(half * 2.0));
            if x.outline != OutlineMode::Off {
                painter.rect_stroke(rect.expand(1.0), 0.0, Stroke::new(1.0_f32, ocolor));
            }
            painter.rect_stroke(rect, 0.0, Stroke::new(thick.max(1.0), color));
            if x.dot { painter.circle_filled(center, thick.max(1.0), color); }
        }
        _ => {
            let g = gap.max(0.0);
            draw_bar(egui::pos2(center.x + g, center.y), egui::pos2(center.x + g + len, center.y));
            draw_bar(egui::pos2(center.x - g, center.y), egui::pos2(center.x - g - len, center.y));
            draw_bar(egui::pos2(center.x, center.y + g), egui::pos2(center.x, center.y + g + len));
            if !x.t_style {
                draw_bar(egui::pos2(center.x, center.y - g), egui::pos2(center.x, center.y - g - len));
            }
            if x.dot { painter.circle_filled(center, (thick * 0.6).max(1.0), color); }
        }
    }
}
