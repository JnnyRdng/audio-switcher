use crate::settings::{Settings, Theme, ToastPosition};
use eframe::egui;
use eframe::egui::IconData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

static WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

const WINDOW_TITLE: &str = "Audio Switcher Settings";
const WINDOW_W: f32 = 360.0;
const WINDOW_H: f32 = 360.0;

/// Open the settings window as a subprocess.
/// Does nothing if a window is already open.
pub fn open(settings: Arc<Mutex<Settings>>) {
    if WINDOW_OPEN
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        // Already open — bring it to front.
        #[cfg(target_os = "windows")]
        bring_to_front();
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
                #[cfg(target_os = "windows")]
                title_bar_set: false,
            }))
        }),
    );
}

#[cfg(target_os = "windows")]
fn centered_position(win_w: f32, win_h: f32) -> [f32; 2] {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };
    unsafe {
        // SPI_GETWORKAREA returns the usable desktop rect (excludes taskbar).
        let mut work = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work as *mut RECT as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );

        let left = work.left as f32;
        let top = work.top as f32;
        let w = (work.right - work.left) as f32;
        let h = (work.bottom - work.top) as f32;

        [(left + w - win_w) / 2.0, (top + h - win_h) / 2.0]
    }
}

struct SettingsApp {
    settings: Settings,
    #[cfg(target_os = "windows")]
    title_bar_set: bool,
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "windows")]
        if !self.title_bar_set {
            self.title_bar_set = true;
            set_title_bar_dark(self.settings.is_dark());
        }

        if self.settings.is_dark() {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        let mut settings_updated = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            let prev_settings = self.settings.clone();

            render_ui(ui, &mut self.settings);

            if self.settings.theme != prev_settings.theme {
                settings_updated = true;

                #[cfg(target_os = "windows")]
                set_title_bar_dark(self.settings.is_dark());
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

fn render_ui(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.heading("Theme");

    add_spacer(ui);

    ui.columns(3, |cols| {
        cols[0].vertical(|col| col.radio_value(&mut settings.theme, Theme::System, "System"));
        cols[1].vertical(|col| col.radio_value(&mut settings.theme, Theme::Dark, "Dark"));
        cols[2].vertical(|col| col.radio_value(&mut settings.theme, Theme::Light, "Light"));
    });

    add_spacer(ui);
    ui.separator();
    add_spacer(ui);

    ui.checkbox(&mut settings.play_sound, "Play sound on switch");
    add_spacer(ui);
    ui.separator();
    add_spacer(ui);
    ui.checkbox(&mut settings.show_toast, "Show toast on switch");

    if settings.show_toast {
        ui.checkbox(&mut settings.toast_fade, "Animate toast fade");
        add_spacer(ui);
        ui.add(
            egui::Slider::new(&mut settings.toast_duration_ms, 500..=5000)
                .text("Toast duration (ms)"),
        );
        add_spacer(ui);
        ui.add(egui::Slider::new(&mut settings.toast_opacity, 0.1..=1.0).text("Toast opacity"));
        add_spacer(ui);
        egui::ComboBox::from_label("Toast position")
            .selected_text(settings.toast_position.to_string())
            .show_ui(ui, |ui| {
                for pos in ToastPosition::iter() {
                    ui.selectable_value(&mut settings.toast_position, pos, pos.to_string());
                }
            });
    }
}

fn add_spacer(ui: &mut egui::Ui) {
    ui.add_space(12.0);
}

#[cfg(target_os = "windows")]
fn bring_to_front() {
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow};
    use windows::core::w;
    unsafe {
        if let Ok(hwnd) = FindWindowW(None, w!("Audio Switcher Settings")) {
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(target_os = "windows")]
fn set_title_bar_dark(dark: bool) {
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
    use windows::core::w;

    unsafe {
        let Ok(hwnd) = FindWindowW(None, w!("Audio Switcher Settings")) else {
            return;
        };
        let value = BOOL::from(dark);
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWINDOWATTRIBUTE(20),
            &value as *const BOOL as *const std::ffi::c_void,
            std::mem::size_of::<BOOL>() as u32,
        );
    }
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
