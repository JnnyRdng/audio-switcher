use crate::audio;
use crate::audio::Device;
use crate::settings::{Settings, Shortcut, Theme, ToastPosition};
use eframe::egui;
use eframe::egui::IconData;
use eframe::egui::style::HandleShape;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

static WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

const WINDOW_TITLE: &str = "Audio Switcher Settings";
const WINDOW_W: f32 = 840.0;
const WINDOW_H: f32 = 340.0;

/// Open the settings window as a subprocess.
/// Does nothing if a window is already open.
pub fn open(settings: Arc<Mutex<Settings>>) {
    if WINDOW_OPEN
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        // Already open — bring it to front.
        #[cfg(target_os = "windows")]
        windows::bring_to_front();
        return;
    }

    std::thread::spawn(move || {
        let exe = std::env::current_exe().expect("failed to get current exe path");
        let mut child = std::process::Command::new(exe)
            .arg("--settings")
            .spawn()
            .expect("failed to spawn settings window");

        // Poll for settings changes while the window is open.
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {}
                Err(_) => break,
            }

            let new_settings = Settings::load();
            crate::tray::apply_current_theme(&new_settings);
            *settings.lock().unwrap() = new_settings;

            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // Final re-read after the window closes.
        let new_settings = Settings::load();
        crate::tray::apply_current_theme(&new_settings);
        *settings.lock().unwrap() = new_settings;

        WINDOW_OPEN.store(false, Ordering::SeqCst);
    });
}

/// Entry point for the settings subprocess (called with `--settings`).
pub fn run() {
    audio::initialise();
    let devices = audio::list_devices();
    let settings = Settings::load();
    let is_dark = settings.is_dark();
    let center_pos = centered_position(WINDOW_W, WINDOW_H);
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_W, WINDOW_H])
            .with_icon(load_icon())
            .with_position(center_pos)
            .with_resizable(false),
        ..Default::default()
    };

    let _ = eframe::run_native(
        WINDOW_TITLE,
        native_options,
        Box::new(move |cc| {
            if is_dark {
                cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            } else {
                cc.egui_ctx.set_visuals(eframe::egui::Visuals::light());
            }
            Ok(Box::new(SettingsApp {
                settings,
                devices,
                capturing_device_id: None,
                #[cfg(target_os = "windows")]
                title_bar_set: false,
            }))
        }),
    );
}

fn centered_position(win_w: f32, win_h: f32) -> [f32; 2] {
    #[cfg(target_os = "windows")]
    return windows::centered_position(win_w, win_h);
    #[cfg(target_os = "linux")]
    return linux::centered_position(win_w, win_h);
}

struct SettingsApp {
    settings: Settings,
    devices: Vec<Device>,
    /// When set, the UI is listening for a key press to assign to this device ID.
    capturing_device_id: Option<String>,
    #[cfg(target_os = "windows")]
    title_bar_set: bool,
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "windows")]
        if !self.title_bar_set {
            self.title_bar_set = true;
            windows::set_title_bar_dark(self.settings.is_dark());
        }

        if self.settings.is_dark() {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        let mut settings_updated = false;

        // Handle key capture if active.
        if let Some(ref device_id) = self.capturing_device_id.clone() {
            for event in &ctx.input(|i| i.events.clone()) {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    // Escape cancels capture.
                    if *key == egui::Key::Escape {
                        self.capturing_device_id = None;
                        break;
                    }

                    // Require at least one modifier.
                    if !modifiers.ctrl && !modifiers.alt && !modifiers.shift {
                        continue;
                    }

                    let shortcut = Shortcut {
                        ctrl: modifiers.ctrl,
                        alt: modifiers.alt,
                        shift: modifiers.shift,
                        win_key: false,
                        key: key.name().to_string(),
                    };
                    self.settings.shortcuts.insert(device_id.clone(), shortcut);
                    self.capturing_device_id = None;
                    self.settings.save();
                    break;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let prev_settings = self.settings.clone();
            let main_width = 500.0;
            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_shortcuts_ui(
                        ui,
                        &mut self.settings,
                        &self.devices,
                        &mut self.capturing_device_id,
                    );
                });
            });

            egui::SidePanel::right("RightPanel")
                .resizable(false)
                .exact_width(WINDOW_W as f32 - main_width)
                .show_inside(ui, |ui| {
                    render_controls_ui(ui, &mut self.settings);
                });

            if self.settings.theme != prev_settings.theme {
                settings_updated = true;

                #[cfg(target_os = "windows")]
                windows::set_title_bar_dark(self.settings.is_dark());
            }
            if prev_settings != self.settings {
                settings_updated = true;
            }

            if settings_updated {
                self.settings.save();
            }
        });
    }
}

fn render_controls_ui(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.heading("Appearance");
    add_spacer(ui);
    add_spacer(ui);
    ui.horizontal(|ui| {
        for theme in Theme::iter() {
            ui.selectable_value(&mut settings.theme, theme, theme.to_string());
        }
    });
    add_spacer(ui);

    ui.checkbox(&mut settings.play_sound, "Play sound on switch");
    ui.checkbox(&mut settings.show_toast, "Show toast on switch");

    if settings.show_toast && cfg!(target_os = "windows") {
        ui.checkbox(&mut settings.toast_fade, "Animate toast fade");
        add_spacer(ui);
        egui::ComboBox::from_label("Toast position")
            .selected_text(settings.toast_position.to_string())
            .show_ui(ui, |ui| {
                for pos in ToastPosition::iter() {
                    ui.selectable_value(&mut settings.toast_position, pos, pos.to_string());
                }
            });
        add_spacer(ui);
        ui.add(
            egui::Slider::new(&mut settings.toast_duration_ms, 500..=5000)
                .text("Toast duration (ms)")
                .handle_shape(HandleShape::Circle)
                .trailing_fill(true),
        );
        add_spacer(ui);
        ui.add(
            egui::Slider::new(&mut settings.toast_opacity, 0.1..=1.0)
                .text("Toast opacity")
                .handle_shape(HandleShape::Circle)
                .trailing_fill(true),
        );
    }
}

fn render_shortcuts_ui(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    devices: &[Device],
    capturing_device_id: &mut Option<String>,
) {
    ui.heading("Keyboard Shortcuts");
    add_spacer(ui);
    add_spacer(ui);

    if devices.is_empty() {
        ui.label("No audio devices found.");
        return;
    }

    // Check for duplicate shortcuts.
    let mut seen = std::collections::HashMap::new();
    let mut duplicates = std::collections::HashSet::new();
    for (device_id, shortcut) in &settings.shortcuts {
        let key = shortcut.to_string();
        if let Some(prev_id) = seen.insert(key.clone(), device_id.clone()) {
            duplicates.insert(prev_id);
            duplicates.insert(device_id.clone());
        }
    }

    egui::Grid::new("shortcuts_grid")
        .num_columns(3)
        .spacing([12.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Device");
            ui.strong("Shortcut");
            ui.strong("");
            ui.end_row();

            for device in devices {
                ui.label(&device.name);

                let is_capturing = capturing_device_id.as_ref() == Some(&device.id);
                let is_duplicate = duplicates.contains(&device.id);

                if is_capturing {
                    let colour = if settings.is_dark() {
                        egui::Color32::ORANGE
                    } else {
                        egui::Color32::BROWN
                    };
                    ui.colored_label(colour, "Press shortcut...");
                } else if let Some(shortcut) = settings.shortcuts.get(&device.id) {
                    let text = shortcut.to_string();
                    if is_duplicate {
                        ui.colored_label(egui::Color32::RED, format!("{text} (duplicate)"));
                    } else {
                        ui.label(text);
                    }
                } else {
                    ui.colored_label(egui::Color32::GRAY, "None");
                }

                ui.horizontal(|ui| {
                    if is_capturing {
                        if ui.button("Cancel").clicked() {
                            *capturing_device_id = None;
                        }
                    } else {
                        if ui.button("Assign").clicked() {
                            *capturing_device_id = Some(device.id.clone());
                        }
                        if settings.shortcuts.contains_key(&device.id) {
                            if ui.button("Clear").clicked() {
                                settings.shortcuts.remove(&device.id);
                            }
                        }
                    }
                });

                ui.end_row();
            }
        });

    if !duplicates.is_empty() {
        add_spacer(ui);
        ui.colored_label(
            egui::Color32::RED,
            "Warning: duplicate shortcuts will only work for one device.",
        );
    }
}

fn add_spacer(ui: &mut egui::Ui) {
    ui.add_space(12.0);
}

fn load_icon() -> IconData {
    let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.ico"));
    let image = image::load_from_memory(bytes).unwrap().into_rgba8();
    let (width, height) = image.dimensions();
    IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
