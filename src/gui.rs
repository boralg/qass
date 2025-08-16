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
        "My egui App",
        options,
        Box::new(|cc| Ok(Box::<MyApp>::default())),
    )
    .map_err(|e| anyhow!("Failed to run qass GUI: {:?}", e))?;

    Ok(())
}

struct MyApp {
    search_text: String,
    show_suggestions: bool,
    selected_suggestion: Option<usize>,
    suggestions: Vec<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            show_suggestions: false,
            selected_suggestion: None,
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

impl MyApp {
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                if self.show_suggestions {
                    self.show_suggestions = false;
                    self.selected_suggestion = None;
                } else {
                    std::process::exit(0);
                }
            }

            let search_text = self.search_text.clone();
            let search_response = ui.text_edit_singleline(&mut self.search_text);
            let filtered_suggestions = MyApp::filtered_suggestions(&search_text, &self.suggestions);

            if search_response.has_focus() && !filtered_suggestions.is_empty() {
                self.show_suggestions = true;
            } else if !search_response.has_focus() {
                self.show_suggestions = false;
            }

            if search_response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                if let Some(selected) = self.selected_suggestion {
                    self.selected_suggestion =
                        Some((selected + 1).min(filtered_suggestions.len() - 1));
                } else if !filtered_suggestions.is_empty() {
                    self.selected_suggestion = Some(0);
                }
            }

            if search_response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                if let Some(selected) = self.selected_suggestion {
                    if selected > 0 {
                        self.selected_suggestion = Some(selected - 1);
                    }
                } else if !filtered_suggestions.is_empty() {
                    self.selected_suggestion = Some(filtered_suggestions.len() - 1);
                }
            }

            if search_response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(selected_idx) = self.selected_suggestion {
                    if let Some((_, suggestion)) = filtered_suggestions.get(selected_idx) {
                        self.search_text = (*suggestion).clone();
                        self.show_suggestions = false;
                        self.selected_suggestion = None;
                    }
                }
            }

            if search_response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_suggestions = false;
                self.selected_suggestion = None;
            }

            if self.show_suggestions && !filtered_suggestions.is_empty() {
                ui.separator();
                ui.label("Suggestions:");

                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for (list_idx, (_, suggestion)) in filtered_suggestions.iter().enumerate() {
                            let is_selected = self.selected_suggestion == Some(list_idx);

                            let response = ui.selectable_label(is_selected, suggestion.as_str());

                            if response.clicked() {
                                self.search_text = (*suggestion).clone();
                                self.show_suggestions = false;
                                self.selected_suggestion = None;
                            }

                            if response.hovered() {
                                self.selected_suggestion = Some(list_idx);
                            }
                        }
                    });
            }

            if !self.search_text.is_empty() {
                ui.separator();
                ui.label(format!("You selected: {}", self.search_text));
            }
        });
    }
}
