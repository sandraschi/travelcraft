//! Trip board. Cards live in SQLite, same file as the journal.
//! Columns are fixed: backlog, this_trip, booked, done.

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::cities::{places_in_city, TravelCity};
use crate::{db_err, TravelCraftEngine, TravelCraftError};

pub const KANBAN_COLUMNS: [&str; 4] = ["backlog", "this_trip", "booked", "done"];

pub fn column_label(column_id: &str) -> &'static str {
    match column_id {
        "backlog" => "Backlog",
        "this_trip" => "This trip",
        "booked" => "Booked",
        "done" => "Done",
        _ => "Backlog",
    }
}

pub fn is_column(column_id: &str) -> bool {
    KANBAN_COLUMNS.contains(&column_id)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanbanCard {
    pub card_id: String,
    pub column_id: String,
    pub title: String,
    pub city: String,
    pub note: String,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub sort_key: i64,
}

impl TravelCraftEngine {
    pub fn kanban_list(&self) -> Result<Vec<KanbanCard>, TravelCraftError> {
        let conn = self.lock();
        let mut stmt = conn
            .prepare(
                "SELECT card_id, column_id, title, city, note, lat, lon, sort_key
                 FROM kanban_cards
                 ORDER BY sort_key ASC, title ASC",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(KanbanCard {
                    card_id: row.get(0)?,
                    column_id: row.get(1)?,
                    title: row.get(2)?,
                    city: row.get(3)?,
                    note: row.get(4)?,
                    lat: row.get(5)?,
                    lon: row.get(6)?,
                    sort_key: row.get(7)?,
                })
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(db_err)?);
        }
        Ok(out)
    }

    /// Insert catalogue stops for Vienna and Tokyo when the board is empty.
    /// Returns how many cards were inserted. A second call inserts nothing.
    pub fn kanban_seed_cities(&self) -> Result<usize, TravelCraftError> {
        {
            let conn = self.lock();
            let existing: i64 = conn
                .query_row("SELECT COUNT(*) FROM kanban_cards", [], |row| row.get(0))
                .map_err(db_err)?;
            if existing > 0 {
                return Ok(0);
            }
        }
        let mut rows = Vec::new();
        for city in [TravelCity::Vienna, TravelCity::Tokyo] {
            for (index, place) in places_in_city(city).into_iter().enumerate() {
                let column = if index == 0 { "this_trip" } else { "backlog" };
                let title: String = place.name.chars().take(200).collect();
                let note: String = place.note.chars().take(2_000).collect();
                Self::check_text("title", &title, 200, true)?;
                Self::check_text("note", &note, 2_000, false)?;
                rows.push((
                    self.new_entry_id("CARD"),
                    column,
                    title,
                    city.id().to_string(),
                    note,
                    place.lat,
                    place.lon,
                    (index as i64) * 10,
                ));
            }
        }
        let mut conn = self.lock();
        let tx = conn.transaction().map_err(db_err)?;
        for (card_id, column, title, city, note, lat, lon, sort_key) in &rows {
            tx.execute(
                "INSERT INTO kanban_cards
                    (card_id, column_id, title, city, note, lat, lon, sort_key)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![card_id, column, title, city, note, lat, lon, sort_key],
            )
            .map_err(db_err)?;
        }
        tx.commit().map_err(db_err)?;
        Ok(rows.len())
    }

    pub fn kanban_add(
        &self,
        title: &str,
        city: &str,
        note: &str,
    ) -> Result<KanbanCard, TravelCraftError> {
        let title = title.trim();
        let note = note.trim();
        Self::check_text("title", title, 200, true)?;
        Self::check_text("note", note, 2_000, false)?;
        let city = TravelCity::parse(city)
            .ok_or_else(|| TravelCraftError::Invalid("city must be vienna or tokyo".to_string()))?;
        let card_id = self.new_entry_id("CARD");
        let conn = self.lock();
        let sort_key: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_key), 0) + 10 FROM kanban_cards WHERE column_id = 'backlog'",
                [],
                |row| row.get(0),
            )
            .map_err(db_err)?;
        conn.execute(
            "INSERT INTO kanban_cards
                (card_id, column_id, title, city, note, lat, lon, sort_key)
             VALUES (?1, 'backlog', ?2, ?3, ?4, NULL, NULL, ?5)",
            params![card_id, title, city.id(), note, sort_key],
        )
        .map_err(db_err)?;
        Ok(KanbanCard {
            card_id,
            column_id: "backlog".to_string(),
            title: title.to_string(),
            city: city.id().to_string(),
            note: note.to_string(),
            lat: None,
            lon: None,
            sort_key,
        })
    }

    pub fn kanban_move(&self, card_id: &str, column_id: &str) -> Result<(), TravelCraftError> {
        Self::check_text("card_id", card_id, 200, true)?;
        if !is_column(column_id) {
            return Err(TravelCraftError::Invalid(
                "column must be backlog, this_trip, booked, or done".to_string(),
            ));
        }
        let conn = self.lock();
        let found: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM kanban_cards WHERE card_id = ?1",
                params![card_id],
                |row| row.get(0),
            )
            .map_err(db_err)?;
        if found == 0 {
            return Err(TravelCraftError::NotFound(card_id.to_string()));
        }
        let sort_key: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_key), 0) + 10 FROM kanban_cards WHERE column_id = ?1",
                params![column_id],
                |row| row.get(0),
            )
            .map_err(db_err)?;
        conn.execute(
            "UPDATE kanban_cards SET column_id = ?1, sort_key = ?2 WHERE card_id = ?3",
            params![column_id, sort_key, card_id],
        )
        .map_err(db_err)?;
        Ok(())
    }

    pub fn kanban_delete(&self, card_id: &str) -> Result<(), TravelCraftError> {
        Self::check_text("card_id", card_id, 200, true)?;
        let conn = self.lock();
        let n = conn
            .execute(
                "DELETE FROM kanban_cards WHERE card_id = ?1",
                params![card_id],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(TravelCraftError::NotFound(card_id.to_string()));
        }
        Ok(())
    }
}
