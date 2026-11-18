//! Trip kanban. Drag a card by its title, or use the arrow buttons.
//! The board is the engine's SQLite table, not a list kept only in the window.

use travelcraft_core::cities::TravelCity;
use travelcraft_core::kanban::{self, KanbanCard};
use travelcraft_core::TravelCraftEngine;

const INK: egui::Color32 = egui::Color32::from_rgb(8, 12, 22);
const PAPER: egui::Color32 = egui::Color32::from_rgb(255, 248, 236);
const GOLD: egui::Color32 = egui::Color32::from_rgb(120, 78, 10);
const CARD: egui::Color32 = egui::Color32::from_rgb(255, 252, 245);

pub struct KanbanDraft {
    pub title: String,
    pub note: String,
    pub city: TravelCity,
    pub error: String,
}

impl Default for KanbanDraft {
    fn default() -> Self {
        Self {
            title: String::new(),
            note: String::new(),
            city: TravelCity::Vienna,
            error: String::new(),
        }
    }
}

pub struct MapJump {
    pub lat: f64,
    pub lon: f64,
    pub label: String,
}

enum Act {
    Add,
    Move(String, String),
    Delete(String),
}

pub fn show(
    ui: &mut egui::Ui,
    engine: &TravelCraftEngine,
    draft: &mut KanbanDraft,
) -> Option<MapJump> {
    ui.heading("Trip board");
    ui.label(
        egui::RichText::new(
            "Vienna and Tokyo stops from the catalogue start here. Drag a title between columns. The board is saved in the travelcraft database.",
        )
        .size(14.0),
    );
    if !draft.error.is_empty() {
        ui.colored_label(egui::Color32::from_rgb(180, 40, 40), &draft.error);
    }

    ui.horizontal_wrapped(|ui| {
        ui.label("New card");
        ui.add(
            egui::TextEdit::singleline(&mut draft.title)
                .hint_text("Title")
                .desired_width(220.0),
        );
        egui::ComboBox::from_id_salt("kanban-city")
            .selected_text(draft.city.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut draft.city, TravelCity::Vienna, "Vienna");
                ui.selectable_value(&mut draft.city, TravelCity::Tokyo, "Tokyo");
            });
        ui.add(
            egui::TextEdit::singleline(&mut draft.note)
                .hint_text("Note")
                .desired_width(260.0),
        );
    });

    let cards = match engine.kanban_list() {
        Ok(cards) => cards,
        Err(err) => {
            draft.error = format!("Could not read the board: {err}");
            Vec::new()
        }
    };

    let mut act: Option<Act> = None;
    let mut jump: Option<MapJump> = None;
    if ui.button("Add to Backlog").clicked() {
        act = Some(Act::Add);
    }

    let height = (ui.available_height() - 8.0).max(280.0);
    egui::ScrollArea::horizontal()
        .id_salt("kanban-board")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(height);
                for column in kanban::KANBAN_COLUMNS {
                    let frame = egui::Frame::new()
                        .fill(PAPER)
                        .stroke(egui::Stroke::new(1.0, GOLD))
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin::same(8));
                    let (_zone, dropped) = ui.dnd_drop_zone::<String, _>(frame, |ui| {
                        ui.set_min_width(240.0);
                        ui.set_min_height(height - 12.0);
                        let count = cards.iter().filter(|c| c.column_id == column).count();
                        ui.label(
                            egui::RichText::new(format!(
                                "{}  ({count})",
                                kanban::column_label(column)
                            ))
                            .strong()
                            .size(16.0)
                            .color(INK),
                        );
                        egui::ScrollArea::vertical()
                            .id_salt(("kanban-col", column))
                            .max_height(height - 48.0)
                            .show(ui, |ui| {
                                let mut any = false;
                                for card in cards.iter().filter(|c| c.column_id == column) {
                                    any = true;
                                    draw_card(ui, card, &mut act, &mut jump);
                                }
                                if !any {
                                    ui.label(egui::RichText::new("Drop a card here").color(GOLD));
                                }
                            });
                    });
                    if act.is_none() {
                        if let Some(payload) = dropped {
                            act = Some(Act::Move(payload.as_ref().clone(), column.to_string()));
                        }
                    }
                }
            });
        });

    let acted = act.is_some();
    match act {
        Some(Act::Add) => match engine.kanban_add(&draft.title, draft.city.id(), &draft.note) {
            Ok(_) => {
                draft.title.clear();
                draft.note.clear();
                draft.error.clear();
            }
            Err(err) => draft.error = err.to_string(),
        },
        Some(Act::Move(id, column)) => {
            if let Err(err) = engine.kanban_move(&id, &column) {
                draft.error = err.to_string();
            } else {
                draft.error.clear();
            }
        }
        Some(Act::Delete(id)) => {
            if let Err(err) = engine.kanban_delete(&id) {
                draft.error = err.to_string();
            }
        }
        None => {}
    }
    if acted {
        ui.ctx().request_repaint();
    }
    jump
}

fn draw_card(
    ui: &mut egui::Ui,
    card: &KanbanCard,
    act: &mut Option<Act>,
    jump: &mut Option<MapJump>,
) {
    let drag_id = egui::Id::unique(("travel-kanban-card", card.card_id.as_str()));
    ui.dnd_drag_source(drag_id, card.card_id.clone(), |ui| {
        egui::Frame::new()
            .fill(CARD)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 48, 64)))
            .corner_radius(egui::CornerRadius::same(4))
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.set_width(220.0);
                ui.label(
                    egui::RichText::new(&card.title)
                        .strong()
                        .color(INK)
                        .size(14.0),
                );
                let city = TravelCity::parse(&card.city)
                    .map(TravelCity::label)
                    .unwrap_or("City");
                ui.label(egui::RichText::new(city).size(12.0).color(GOLD));
                if !card.note.is_empty() {
                    ui.label(egui::RichText::new(&card.note).size(12.0).color(INK));
                }
            });
    });
    ui.horizontal(|ui| {
        if ui.button("Map").clicked() {
            let (lng, lat, _zoom) = TravelCity::parse(&card.city)
                .map(TravelCity::home)
                .unwrap_or((16.373, 48.208, 12.0));
            let lat = card.lat.unwrap_or(lat);
            let lon = card.lon.unwrap_or(lng);
            *jump = Some(MapJump {
                lat,
                lon,
                label: card.title.clone(),
            });
        }
        if let Some(prev) = neighbor(&card.column_id, -1) {
            if ui.button("<").clicked() {
                *act = Some(Act::Move(card.card_id.clone(), prev.to_string()));
            }
        }
        if let Some(next) = neighbor(&card.column_id, 1) {
            if ui.button(">").clicked() {
                *act = Some(Act::Move(card.card_id.clone(), next.to_string()));
            }
        }
        if ui.button("Remove").clicked() {
            *act = Some(Act::Delete(card.card_id.clone()));
        }
    });
    ui.add_space(6.0);
}

fn neighbor(column: &str, step: isize) -> Option<&'static str> {
    let index = kanban::KANBAN_COLUMNS.iter().position(|c| *c == column)?;
    let next = index as isize + step;
    if next < 0 {
        return None;
    }
    kanban::KANBAN_COLUMNS.get(next as usize).copied()
}
