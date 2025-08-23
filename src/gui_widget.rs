use eframe::egui::{Key, Response, Ui, Widget};
use zeroize::Zeroizing;

pub struct PasswordEdit<'a> {
    password: &'a mut Zeroizing<String>,
}

impl<'a> PasswordEdit<'a> {
    pub fn new(password: &'a mut Zeroizing<String>) -> Self {
        Self { password }
    }
}

impl<'a> Widget for PasswordEdit<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(
            eframe::egui::Vec2::ZERO,
            eframe::egui::Sense::click_and_drag(),
        );

        if response.has_focus() {
            ui.input(|input| {
                for event in &input.events {
                    match event {
                        eframe::egui::Event::Text(text) => {
                            self.password.push_str(text);
                        }
                        eframe::egui::Event::Key {
                            key: Key::Backspace,
                            pressed: true,
                            ..
                        } => {
                            self.password.pop();
                        }
                        eframe::egui::Event::Key {
                            key: Key::Delete,
                            pressed: true,
                            ..
                        } => {
                            self.password.clear();
                        }
                        _ => {}
                    }
                }
            });
        }

        response
    }
}
