use anyhow::anyhow;
use eframe::egui;

pub fn run() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "qass",
        options,
        Box::new(|cc| Ok(Box::<QassGui>::default())),
    )
    .map_err(|e| anyhow!("Failed to run qass GUI: {:?}", e))?;

    Ok(())
}

struct QassGui {
    search_text: String,
    selected_suggestion: usize,
    suggestions: Vec<String>,
}

impl Default for QassGui {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            selected_suggestion: 0,
            suggestions: vec![
                "Apple".to_string(),
                "Banana".to_string(),
                "Blueberry".to_string(),
                "Blackberry".to_string(),
                "Cherry".to_string(),
                "Date".to_string(),
                "Melon".to_string(),
                "Mango".to_string(),
            ],
        }
    }
}

impl QassGui {
    fn filtered_suggestions<'a>(
        search_text: &'a str,
        suggestions: &'a Vec<String>,
    ) -> Vec<(usize, &'a String)> {
        if search_text.is_empty() {
            return vec![];
        }

        suggestions
            .iter()
            .enumerate()
            .filter(|(_, item)| item.to_lowercase().starts_with(&search_text.to_lowercase()))
            .collect()
    }
}

impl eframe::App for QassGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                std::process::exit(0);
            }

            let mut search_text = self.search_text.clone();
            let search_response = ui.text_edit_singleline(&mut search_text);
            search_response.request_focus();

            let filtered_suggestions = QassGui::filtered_suggestions(&search_text, &self.suggestions);
            self.search_text = search_text.clone();

            if filtered_suggestions.is_empty() {
                return;
            }

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.selected_suggestion += 1;
                if self.selected_suggestion >= filtered_suggestions.len() {
                    self.selected_suggestion = 0;
                }
            }

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                if self.selected_suggestion == 0 {
                    self.selected_suggestion = filtered_suggestions.len() - 1;
                } else {
                    self.selected_suggestion -= 1;
                }
            }

            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search_text = filtered_suggestions
                    .get(self.selected_suggestion)
                    .unwrap()
                    .1
                    .to_string();
            }

            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    for (list_idx, (_, suggestion)) in filtered_suggestions.iter().enumerate() {
                        let is_selected = self.selected_suggestion == list_idx;

                        let response = ui.selectable_label(is_selected, suggestion.as_str());

                        if response.clicked() {
                            self.search_text = (*suggestion).clone();
                        }

                        if response.hovered() {
                            self.selected_suggestion = list_idx;
                        }
                    }
                });
        });
    }
}
