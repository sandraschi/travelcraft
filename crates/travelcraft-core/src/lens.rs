//! LensCraft integration: real photo identification via `lenscraft-core`
//! cross-matched against active TravelCraft cartridges and logged to the
//! Universal Travel Notebook.

use crate::notebook::{NewNotebookEntry, NotebookKind};
use crate::{TravelCraftEngine, TravelCraftError};
use lenscraft_core::{
    identify, prepare_file, status, IdentifyOptions, IdentifyResult, LensError, ModelStatus,
    OllamaHttp, PreparedImage, VisionTransport, DEFAULT_MODEL,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use travelcraft_baedeker::cartridge::Cartridge;
use travelcraft_baedeker::{ArtworkRecord, ChronoWaypoint};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LensIdentifyRequest {
    pub photo_path: String,
    #[serde(default)]
    pub cartridge_id: Option<String>,
    #[serde(default)]
    pub place: Option<String>,
    #[serde(default)]
    pub hint: Option<String>,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub log_trip_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensCartridgeResult {
    pub lens: IdentifyResult,
    pub matched_artwork: Option<ArtworkRecord>,
    pub matched_waypoint: Option<ChronoWaypoint>,
    pub notebook_entry_id: Option<String>,
}

fn map_lens_err(e: LensError) -> TravelCraftError {
    match e {
        LensError::InvalidInput(m) | LensError::Image(m) => TravelCraftError::Invalid(m),
        LensError::Offline(m) | LensError::Model(m) => TravelCraftError::Offline(m),
        LensError::Protocol(m) => TravelCraftError::Protocol(m),
        LensError::Io(m) => TravelCraftError::Store(m),
    }
}

fn match_cartridge_dossier(
    cartridge: &Cartridge,
    result: &IdentifyResult,
) -> (Option<ArtworkRecord>, Option<ChronoWaypoint>) {
    let mut matched_art = None;
    let mut matched_wp = None;

    for cand in &result.candidates {
        let title_lc = cand.title.to_ascii_lowercase();
        let creator_lc = cand.creator.to_ascii_lowercase();
        if matched_art.is_none() {
            for art in &cartridge.artworks {
                let art_title = art.title.to_ascii_lowercase();
                let art_artist = art.artist.to_ascii_lowercase();
                let title_hit = !title_lc.is_empty()
                    && (art_title.contains(&title_lc) || title_lc.contains(&art_title));
                let artist_hit = !creator_lc.is_empty()
                    && (art_artist.contains(&creator_lc) || creator_lc.contains(&art_artist));
                if title_hit || (artist_hit && !title_lc.is_empty()) {
                    matched_art = Some(art.clone());
                    break;
                }
            }
        }
        if matched_wp.is_none() && !title_lc.is_empty() {
            for wp in &cartridge.waypoints {
                let wp_name = wp.name.to_ascii_lowercase();
                if wp_name.contains(&title_lc) || title_lc.contains(&wp_name) {
                    matched_wp = Some(wp.clone());
                    break;
                }
            }
        }
        if matched_art.is_some() && matched_wp.is_some() {
            break;
        }
    }
    (matched_art, matched_wp)
}

impl TravelCraftEngine {
    /// Probe the local LensCraft vision backend (Ollama + vision model).
    pub fn lens_status(&self, model: Option<&str>) -> Result<ModelStatus, TravelCraftError> {
        let http = OllamaHttp::from_env().map_err(map_lens_err)?;
        self.lens_status_with(&http, model)
    }

    pub fn lens_status_with(
        &self,
        transport: &dyn VisionTransport,
        model: Option<&str>,
    ) -> Result<ModelStatus, TravelCraftError> {
        let m = model
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_MODEL);
        status(transport, m).map_err(map_lens_err)
    }

    /// Identify a real photo file using `lenscraft-core` (local Ollama vision),
    /// cross-match against an optional cartridge, and optionally log to the Universal Notebook.
    pub fn lens_identify(
        &self,
        req: &LensIdentifyRequest,
    ) -> Result<LensCartridgeResult, TravelCraftError> {
        let http = OllamaHttp::from_env().map_err(map_lens_err)?;
        let prepared = prepare_file(Path::new(&req.photo_path)).map_err(map_lens_err)?;
        self.lens_identify_with(&http, &prepared, req)
    }

    pub fn lens_identify_with(
        &self,
        transport: &dyn VisionTransport,
        prepared: &PreparedImage,
        req: &LensIdentifyRequest,
    ) -> Result<LensCartridgeResult, TravelCraftError> {
        let cartridge = match req
            .cartridge_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(cid) => Some(self.cartridge_get(cid)?),
            None => None,
        };

        let place = req.place.clone().or_else(|| {
            cartridge
                .as_ref()
                .map(|c| format!("{} ({})", c.title, c.country_code))
        });

        let opts = IdentifyOptions {
            model: req.model.clone().unwrap_or_default(),
            hint: req.hint.clone(),
            place,
            lat: req.lat,
            lon: req.lon,
            language: req.language.clone(),
        };

        let lens = identify(transport, prepared, &opts).map_err(map_lens_err)?;

        let (matched_artwork, matched_waypoint) = match &cartridge {
            Some(c) => match_cartridge_dossier(c, &lens),
            None => (None, None),
        };

        let notebook_entry_id = if let Some(trip_id) = req
            .log_trip_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let title = matched_artwork
                .as_ref()
                .map(|a| format!("{} - {}", a.title, a.artist))
                .or_else(|| matched_waypoint.as_ref().map(|w| w.name.clone()))
                .or_else(|| lens.candidates.first().map(|c| c.title.clone()))
                .unwrap_or_else(|| "LensCraft Capture".to_string());

            let body = if let Some(art) = &matched_artwork {
                format!(
                    "LensCraft verdict: {:?} ({:?}). Museum: {}, {}. Look for: {}",
                    lens.verdict, lens.basis, art.museum, art.room, art.look_for
                )
            } else {
                format!(
                    "LensCraft verdict: {:?} ({:?}). {}",
                    lens.verdict, lens.basis, lens.description
                )
            };

            let entry = self.notebook_add(&NewNotebookEntry {
                trip_id: Some(trip_id.to_string()),
                cartridge_id: cartridge.as_ref().map(|c| c.id.clone()),
                country_code: cartridge.as_ref().map(|c| c.country_code.clone()),
                waypoint_id: matched_waypoint.as_ref().map(|w| w.id.clone()),
                kind: NotebookKind::Photo,
                day_index: None,
                title,
                body,
                sensory_notes: String::new(),
                photo_uris: if req.photo_path.trim().is_empty() {
                    Vec::new()
                } else {
                    vec![req.photo_path.clone()]
                },
                lat: req.lat,
                lon: req.lon,
                tags: vec!["lenscraft".to_string()],
            })?;
            Some(entry.entry_id)
        } else {
            None
        };

        Ok(LensCartridgeResult {
            lens,
            matched_artwork,
            matched_waypoint,
            notebook_entry_id,
        })
    }
}
