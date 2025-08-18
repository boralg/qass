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
            show_suggestions: true,
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
                std::process::exit(0);
            }

            let mut search_text = self.search_text.clone();
            let search_response = ui.text_edit_singleline(&mut search_text);
            search_response.request_focus();
            
            let filtered_suggestions = MyApp::filtered_suggestions(&search_text, &self.suggestions);
            self.search_text = search_text.clone();

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

            if self.show_suggestions && !filtered_suggestions.is_empty() {
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for (list_idx, (_, suggestion)) in filtered_suggestions.iter().enumerate() {
                            let is_selected = self.selected_suggestion == Some(list_idx);

                            let response = ui.selectable_label(is_selected, suggestion.as_str());

                            if response.clicked() {
                                self.search_text = (*suggestion).clone();
                            }

                            if response.hovered() {
                                self.selected_suggestion = Some(list_idx);
                            }
                        }
                    });
            }
        });
    }
}
