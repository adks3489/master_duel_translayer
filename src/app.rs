use eframe::{
    egui::{self, FontDefinitions, FontFamily, TextStyle},
    epi,
};
use master_duel_translayer::{attach_process, capture, recognize_card, Mode, Process};
pub struct App {
    status: String,
    process: Option<Process>,
    current_card: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            status: "".to_owned(),
            process: None,
            current_card: None,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Master Duel Translayer"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        let mut fonts = FontDefinitions::default();
        fonts
            .family_and_size
            .insert(TextStyle::Body, (FontFamily::Proportional, 24.0));
        fonts
            .family_and_size
            .insert(TextStyle::Button, (FontFamily::Monospace, 32.0));
        ctx.set_fonts(fonts);
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.status);
            if self.process.is_some() {
                if ui.button("Stop").clicked() {
                    self.process = None;
                }
                if ui.button("Capture").clicked() {
                    self.recognize_card();
                }
            } else {
                if ui.button("Start").clicked() {
                    let p = attach_process().unwrap();
                    println!("pid: {}", p.pid);
                    self.process = Some(p);
                }
            }
            if let Some(card) = &self.current_card {
                ui.label(card);
            }
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

impl App {
    fn recognize_card(&mut self) {
        let p = self.process.as_ref().unwrap();
        if let Ok(buf) = capture(p) {
            self.current_card = recognize_card(&buf, Mode::DeckEdit).ok();
        }
    }
}
