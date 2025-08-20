use anyhow::anyhow;
use eframe::egui::{self, Color32};

use crate::api::State;

pub fn run() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_decorations(false),
        ..Default::default()
    };

    let state = QassGui::Search {
        search_text: String::new(),
    };

    eframe::run_native("qass", options, Box::new(|cc| Ok(Box::new(state))))
        .map_err(|e| anyhow!("Failed to run qass GUI: {:?}", e))?;

    Ok(())
}

#[derive(Clone)]
enum QassGui {
    Search {
        search_text: String,
    },
    SearchSuggestions {
        search_text: String,
        selected_suggestion: usize,
        suggestions: Vec<String>,
    },
    SearchError {
        search_text: String,
        error_msg: String,
    },
}

impl QassGui {
    fn filtered_suggestions<'a>(
        search_text: String,
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

    fn suggestions_state(search_text: String) -> QassGui {
        if let Ok(state) = State::load() {
            QassGui::SearchSuggestions {
                search_text,
                selected_suggestion: 0,
                suggestions: state.list(),
            }
        } else {
            QassGui::SearchError {
                search_text,
                error_msg: "Failed to load credentials.".to_string(),
            }
        }
    }
}

impl eframe::App for QassGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                std::process::exit(0);
            }

            let mut next_state = None;

            match self {
                QassGui::Search { search_text } => {
                    let search_response = ui.text_edit_singleline(search_text);
                    search_response.request_focus();

                    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
                        next_state = Some(QassGui::suggestions_state(search_text.clone()));
                    }
                }
                QassGui::SearchSuggestions {
                    search_text,
                    selected_suggestion,
                    suggestions,
                } => {
                    let search_response = ui.text_edit_singleline(search_text);
                    search_response.request_focus();

                    ui.separator();

                    let filtered_suggestions =
                        QassGui::filtered_suggestions(search_text.clone(), suggestions);

                    // TODO: this will be a bug someday
                    if filtered_suggestions.is_empty() {
                        return;
                    }

                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for (list_idx, (_, suggestion)) in
                                filtered_suggestions.iter().enumerate()
                            {
                                let is_selected = *selected_suggestion == list_idx;

                                let response =
                                    ui.selectable_label(is_selected, suggestion.as_str());

                                if response.clicked() {
                                    *search_text = (*suggestion).clone();
                                    next_state = Some(QassGui::Search {
                                        search_text: search_text.to_string(),
                                    });
                                }

                                if response.hovered() {
                                    *selected_suggestion = list_idx;
                                }
                            }
                        });

                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        *selected_suggestion += 1;
                        if *selected_suggestion >= filtered_suggestions.len() {
                            *selected_suggestion = 0;
                        }
                    }

                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        if *selected_suggestion == 0 {
                            *selected_suggestion = filtered_suggestions.len() - 1;
                        } else {
                            *selected_suggestion -= 1;
                        }
                    }

                    if ctx.input_mut(|i| {
                        i.key_pressed(egui::Key::Enter)
                            || i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                    }) {
                        *search_text = filtered_suggestions
                            .get(*selected_suggestion)
                            .unwrap()
                            .1
                            .to_string();

                        next_state = Some(QassGui::Search {
                            search_text: search_text.to_string(),
                        });
                    }
                }
                QassGui::SearchError {
                    search_text,
                    error_msg,
                } => {
                    let search_response = ui.text_edit_singleline(search_text);
                    search_response.request_focus();

                    ui.separator();
                    ui.colored_label(Color32::RED, error_msg);

                    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
                        next_state = Some(QassGui::suggestions_state(search_text.clone()));
                    }
                }
            };

            if let Some(new_state) = next_state {
                *self = new_state;
            }
        });
    }
}