use std::collections::HashSet;

use anyhow::anyhow;
use eframe::egui::{self, Color32};

use crate::api::State;

pub fn run() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 150.0])
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
    ) -> Vec<(usize, &'a str)> {
        if search_text.is_empty() {
            return vec![];
        }

        let search_len = search_text.len();
        let mut seen = HashSet::new();

        suggestions
            .iter()
            .enumerate()
            .filter(|(_, item)| item.to_lowercase().starts_with(&search_text.to_lowercase()))
            .filter_map(|(i, item)| {
                let display = if let Some(slash_pos) = item[search_len..].find('/') {
                    &item[..search_len + slash_pos + 1]
                } else {
                    item.as_str()
                };

                if seen.insert(display) {
                    Some((i, display))
                } else {
                    None
                }
            })
            .collect()
    }

    fn suggestions_state(search_text: String) -> QassGui {
        match State::load() {
            Ok(state) => {
                let suggestions = state.list();

                // TODO: do this repeatedly for more efficiency and less privacy?
                let filtered_suggestions =
                    QassGui::filtered_suggestions(search_text.clone(), &suggestions);
                if filtered_suggestions.len() == 1 {
                    return QassGui::Search {
                        search_text: filtered_suggestions[0].1.to_owned(),
                    };
                }

                QassGui::SearchSuggestions {
                    search_text,
                    selected_suggestion: 0,
                    suggestions,
                }
            }
            Err(e) => QassGui::SearchError {
                search_text,
                error_msg: format!("Failed to load credentials: {}", e),
            },
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
                    let search_response = ui.add_sized(
                        ui.available_size() * egui::vec2(1.0, 0.0),
                        egui::TextEdit::singleline(search_text),
                    );
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
                    let search_response = ui.add_sized(
                        ui.available_size() * egui::vec2(1.0, 0.0),
                        egui::TextEdit::singleline(search_text),
                    );
                    search_response.request_focus();

                    ui.separator();

                    let filtered_suggestions =
                        QassGui::filtered_suggestions(search_text.clone(), suggestions);

                    if !filtered_suggestions.is_empty() {
                        *selected_suggestion =
                            std::cmp::min(*selected_suggestion, filtered_suggestions.len() - 1);

                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());

                                for (list_idx, (_, suggestion)) in
                                    filtered_suggestions.iter().enumerate()
                                {
                                    let is_selected = *selected_suggestion == list_idx;
                                    let suggestion: String = (*suggestion).to_owned();

                                    let response = ui.selectable_label(is_selected, &suggestion);

                                    if is_selected {
                                        // TODO: scrolling happens one frame late. when scrolling up, the highlight appears on the item one frame early
                                        response.scroll_to_me_animation(
                                            Some(egui::Align::Max),
                                            egui::style::ScrollAnimation::none(),
                                        );
                                    }

                                    if response.clicked() {
                                        *search_text = suggestion;
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
                }
                QassGui::SearchError {
                    search_text,
                    error_msg,
                } => {
                    let search_response = ui.add_sized(
                        ui.available_size() * egui::vec2(1.0, 0.0),
                        egui::TextEdit::singleline(search_text),
                    );
                    search_response.request_focus();

                    ui.separator();
                    ui.colored_label(Color32::YELLOW, error_msg);

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
