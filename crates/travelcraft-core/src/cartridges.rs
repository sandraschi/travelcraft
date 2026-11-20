//! Durable Country & City Cartridge CRUD engine.
//!
//! Cartridges live in the `cartridges` SQLite table as validated JSON payloads
//! with indexed metadata columns (`cartridge_id`, `kind`, `parent_id`,
//! `country_code`, `title`, `version`, `updated_at`).

use crate::{db_err, TravelCraftEngine, TravelCraftError};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use travelcraft_baedeker::cartridge::{
    italy_country_cartridge, venice_city_cartridge, Cartridge, CartridgeKind, CartridgeSummary,
    CurrentEvent, GeoBbox, Gotcha, LocalSpot, MapViewport, MediaAsset, ResolvedCartridge,
    SurvivalKit, TransitSnapshot, Zone,
};
use travelcraft_baedeker::{ArtworkRecord, ChronoWaypoint, StayItinerary};

const MAX_CARTRIDGE_BYTES: usize = 2 * 1024 * 1024; // 2 MiB per cartridge JSON

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CartridgeFilter {
    #[serde(default)]
    pub kind: Option<CartridgeKind>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CartridgeMetaPatch {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub viewport: Option<MapViewport>,
    #[serde(default)]
    pub bbox: Option<GeoBbox>,
    #[serde(default)]
    pub survival_kit: Option<SurvivalKit>,
    #[serde(default)]
    pub transit: Option<TransitSnapshot>,
    #[serde(default)]
    pub events_refreshed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "item_type", content = "item", rename_all = "snake_case")]
pub enum CartridgeItem {
    Waypoint(ChronoWaypoint),
    Artwork(ArtworkRecord),
    Spot(LocalSpot),
    Itinerary(StayItinerary),
    Gotcha(Gotcha),
    Zone(Zone),
    Event(CurrentEvent),
    Media(MediaAsset),
}

impl TravelCraftEngine {
    fn validate_cartridge(cartridge: &Cartridge) -> Result<String, TravelCraftError> {
        cartridge.validate().map_err(TravelCraftError::Invalid)?;
        let payload = cartridge.to_json_string()?;
        if payload.len() > MAX_CARTRIDGE_BYTES {
            return Err(TravelCraftError::Invalid(format!(
                "cartridge payload exceeds {MAX_CARTRIDGE_BYTES} bytes"
            )));
        }
        Ok(payload)
    }

    fn verify_parent_exists(
        conn: &rusqlite::Connection,
        cartridge: &Cartridge,
    ) -> Result<(), TravelCraftError> {
        if let Some(parent_id) = &cartridge.parent_id {
            let parent_kind: Option<String> = conn
                .query_row(
                    "SELECT kind FROM cartridges WHERE cartridge_id = ?1",
                    params![parent_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db_err)?;
            match parent_kind.as_deref() {
                Some("country") => {}
                Some(other) => {
                    return Err(TravelCraftError::Invalid(format!(
                        "parent cartridge '{parent_id}' has kind '{other}', expected 'country'"
                    )));
                }
                None => {
                    return Err(TravelCraftError::NotFound(format!(
                        "parent cartridge '{parent_id}'"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Create a new cartridge. Fails with `Invalid` if the id already exists.
    pub fn cartridge_create(
        &self,
        cartridge: &Cartridge,
    ) -> Result<CartridgeSummary, TravelCraftError> {
        let payload = Self::validate_cartridge(cartridge)?;
        let now = chrono::Utc::now().timestamp();
        let conn = self.lock();
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM cartridges WHERE cartridge_id = ?1",
                params![&cartridge.id],
                |_| Ok(true),
            )
            .optional()
            .map_err(db_err)?
            .unwrap_or(false);
        if exists {
            return Err(TravelCraftError::Invalid(format!(
                "cartridge '{}' already exists",
                cartridge.id
            )));
        }
        Self::verify_parent_exists(&conn, cartridge)?;
        conn.execute(
            "INSERT INTO cartridges (cartridge_id, kind, parent_id, country_code, title, version, updated_at, payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &cartridge.id,
                cartridge.kind.as_str(),
                &cartridge.parent_id,
                &cartridge.country_code,
                &cartridge.title,
                &cartridge.version,
                now,
                &payload,
            ],
        )
        .map_err(db_err)?;
        Ok(summary_of(cartridge, now))
    }

    /// Insert or replace a cartridge (used by import and seeding).
    pub fn cartridge_upsert(
        &self,
        cartridge: &Cartridge,
    ) -> Result<CartridgeSummary, TravelCraftError> {
        let payload = Self::validate_cartridge(cartridge)?;
        let now = chrono::Utc::now().timestamp();
        let conn = self.lock();
        Self::verify_parent_exists(&conn, cartridge)?;
        conn.execute(
            "INSERT INTO cartridges (cartridge_id, kind, parent_id, country_code, title, version, updated_at, payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(cartridge_id) DO UPDATE SET
                 kind = excluded.kind,
                 parent_id = excluded.parent_id,
                 country_code = excluded.country_code,
                 title = excluded.title,
                 version = excluded.version,
                 updated_at = excluded.updated_at,
                 payload = excluded.payload",
            params![
                &cartridge.id,
                cartridge.kind.as_str(),
                &cartridge.parent_id,
                &cartridge.country_code,
                &cartridge.title,
                &cartridge.version,
                now,
                &payload,
            ],
        )
        .map_err(db_err)?;
        Ok(summary_of(cartridge, now))
    }

    /// Read a raw cartridge by id.
    pub fn cartridge_get(&self, id: &str) -> Result<Cartridge, TravelCraftError> {
        let conn = self.lock();
        let payload: Option<String> = conn
            .query_row(
                "SELECT payload FROM cartridges WHERE cartridge_id = ?1",
                params![id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_err)?;
        let Some(raw) = payload else {
            return Err(TravelCraftError::NotFound(format!("cartridge '{id}'")));
        };
        Ok(Cartridge::from_json_str(&raw)?)
    }

    /// Read a cartridge and merge its parent country cartridge (if any).
    pub fn cartridge_resolve(&self, id: &str) -> Result<ResolvedCartridge, TravelCraftError> {
        let cartridge = self.cartridge_get(id)?;
        let parent = match &cartridge.parent_id {
            Some(pid) => Some(self.cartridge_get(pid)?),
            None => None,
        };
        Ok(cartridge.resolve_with_parent(parent.as_ref()))
    }

    /// List cartridges with optional filters.
    pub fn cartridge_list(
        &self,
        filter: &CartridgeFilter,
    ) -> Result<Vec<CartridgeSummary>, TravelCraftError> {
        let conn = self.lock();
        let mut stmt = conn
            .prepare(
                "SELECT updated_at, payload FROM cartridges ORDER BY kind ASC, country_code ASC, title ASC",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            let (updated_at, payload) = row.map_err(db_err)?;
            let c = Cartridge::from_json_str(&payload)?;
            if let Some(kind) = filter.kind {
                if c.kind != kind {
                    continue;
                }
            }
            if let Some(cc) = &filter.country_code {
                if !c.country_code.eq_ignore_ascii_case(cc.trim()) {
                    continue;
                }
            }
            if let Some(pid) = &filter.parent_id {
                if c.parent_id.as_deref() != Some(pid.as_str()) {
                    continue;
                }
            }
            out.push(summary_of(&c, updated_at));
        }
        Ok(out)
    }

    /// Update metadata, viewport, bbox, survival kit, or transit snapshot on an existing cartridge.
    pub fn cartridge_update_meta(
        &self,
        id: &str,
        patch: &CartridgeMetaPatch,
    ) -> Result<CartridgeSummary, TravelCraftError> {
        let mut c = self.cartridge_get(id)?;
        if let Some(v) = &patch.title {
            c.title = v.clone();
        }
        if let Some(v) = &patch.subtitle {
            c.subtitle = v.clone();
        }
        if let Some(v) = &patch.description {
            c.description = v.clone();
        }
        if let Some(v) = &patch.version {
            c.version = v.clone();
        }
        if let Some(v) = &patch.currency {
            c.currency = v.clone();
        }
        if let Some(v) = &patch.languages {
            c.languages = v.clone();
        }
        if let Some(v) = &patch.timezone {
            c.timezone = v.clone();
        }
        if let Some(v) = &patch.viewport {
            c.viewport = Some(v.clone());
        }
        if let Some(v) = &patch.bbox {
            c.bbox = Some(v.clone());
        }
        if let Some(v) = &patch.survival_kit {
            c.survival_kit = v.clone();
        }
        if let Some(v) = &patch.transit {
            c.transit = v.clone();
        }
        if let Some(v) = &patch.events_refreshed_at {
            c.events_refreshed_at = if v.trim().is_empty() {
                None
            } else {
                Some(v.trim().to_string())
            };
        }
        self.cartridge_upsert(&c)
    }

    /// Delete a cartridge. Blocks deleting a country with child cities unless `cascade` is true.
    pub fn cartridge_delete(&self, id: &str, cascade: bool) -> Result<usize, TravelCraftError> {
        let conn = self.lock();
        let child_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cartridges WHERE parent_id = ?1",
                params![id],
                |r| r.get(0),
            )
            .map_err(db_err)?;
        if child_count > 0 && !cascade {
            return Err(TravelCraftError::Invalid(format!(
                "cartridge '{id}' has {child_count} child cartridge(s); pass cascade=true to delete all"
            )));
        }
        let mut deleted = 0usize;
        if child_count > 0 {
            deleted += conn
                .execute("DELETE FROM cartridges WHERE parent_id = ?1", params![id])
                .map_err(db_err)?;
        }
        let n = conn
            .execute(
                "DELETE FROM cartridges WHERE cartridge_id = ?1",
                params![id],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(TravelCraftError::NotFound(format!("cartridge '{id}'")));
        }
        deleted += n;
        Ok(deleted)
    }

    /// Import a `.tcpack.json` string (validates and upserts).
    pub fn cartridge_import_json(
        &self,
        raw_json: &str,
    ) -> Result<CartridgeSummary, TravelCraftError> {
        if raw_json.len() > MAX_CARTRIDGE_BYTES {
            return Err(TravelCraftError::Invalid(format!(
                "import JSON exceeds {MAX_CARTRIDGE_BYTES} bytes"
            )));
        }
        let cartridge = Cartridge::from_json_str(raw_json)?;
        self.cartridge_upsert(&cartridge)
    }

    /// Export a cartridge as pretty-printed `.tcpack.json`.
    pub fn cartridge_export_json(&self, id: &str) -> Result<String, TravelCraftError> {
        let c = self.cartridge_get(id)?;
        Ok(c.to_json_string()?)
    }

    /// Upsert a single item inside a cartridge without rewriting the full JSON manually.
    pub fn cartridge_item_upsert(
        &self,
        cartridge_id: &str,
        item: CartridgeItem,
    ) -> Result<CartridgeSummary, TravelCraftError> {
        let mut c = self.cartridge_get(cartridge_id)?;
        match item {
            CartridgeItem::Waypoint(wp) => {
                if wp.id.trim().is_empty() || wp.name.trim().is_empty() {
                    return Err(TravelCraftError::Invalid(
                        "waypoint id and name required".into(),
                    ));
                }
                if let Some(slot) = c.waypoints.iter_mut().find(|x| x.id == wp.id) {
                    *slot = wp;
                } else {
                    c.waypoints.push(wp);
                }
            }
            CartridgeItem::Artwork(art) => {
                if art.id.trim().is_empty() || art.title.trim().is_empty() {
                    return Err(TravelCraftError::Invalid(
                        "artwork id and title required".into(),
                    ));
                }
                if let Some(slot) = c.artworks.iter_mut().find(|x| x.id == art.id) {
                    *slot = art;
                } else {
                    c.artworks.push(art);
                }
            }
            CartridgeItem::Spot(spot) => {
                if spot.id.trim().is_empty() || spot.name.trim().is_empty() {
                    return Err(TravelCraftError::Invalid(
                        "spot id and name required".into(),
                    ));
                }
                if let Some(slot) = c.spots.iter_mut().find(|x| x.id == spot.id) {
                    *slot = spot;
                } else {
                    c.spots.push(spot);
                }
            }
            CartridgeItem::Itinerary(itin) => {
                if itin.total_days == 0 {
                    return Err(TravelCraftError::Invalid(
                        "itinerary total_days must be >= 1".into(),
                    ));
                }
                if let Some(slot) = c
                    .itineraries
                    .iter_mut()
                    .find(|x| x.total_days == itin.total_days)
                {
                    *slot = itin;
                } else {
                    c.itineraries.push(itin);
                }
            }
            CartridgeItem::Gotcha(g) => {
                if g.id.trim().is_empty() || g.title.trim().is_empty() {
                    return Err(TravelCraftError::Invalid(
                        "gotcha id and title required".into(),
                    ));
                }
                if let Some(slot) = c.gotchas.iter_mut().find(|x| x.id == g.id) {
                    *slot = g;
                } else {
                    c.gotchas.push(g);
                }
            }
            CartridgeItem::Zone(z) => {
                if z.id.trim().is_empty() || z.name.trim().is_empty() {
                    return Err(TravelCraftError::Invalid(
                        "zone id and name required".into(),
                    ));
                }
                if let Some(slot) = c.zones.iter_mut().find(|x| x.id == z.id) {
                    *slot = z;
                } else {
                    c.zones.push(z);
                }
            }
            CartridgeItem::Event(ev) => {
                if ev.id.trim().is_empty() || ev.title.trim().is_empty() {
                    return Err(TravelCraftError::Invalid(
                        "event id and title required".into(),
                    ));
                }
                if let Some(slot) = c.events.iter_mut().find(|x| x.id == ev.id) {
                    *slot = ev;
                } else {
                    c.events.push(ev);
                }
            }
            CartridgeItem::Media(m) => {
                if m.id.trim().is_empty() {
                    return Err(TravelCraftError::Invalid("media id required".into()));
                }
                if let Some(slot) = c.media.iter_mut().find(|x| x.id == m.id) {
                    *slot = m;
                } else {
                    c.media.push(m);
                }
            }
        }
        self.cartridge_upsert(&c)
    }

    /// Remove a single item by type and key (`id`, or `total_days` string for `itinerary`).
    pub fn cartridge_item_remove(
        &self,
        cartridge_id: &str,
        item_type: &str,
        item_key: &str,
    ) -> Result<CartridgeSummary, TravelCraftError> {
        let mut c = self.cartridge_get(cartridge_id)?;
        let (before, after) = match item_type.trim().to_ascii_lowercase().as_str() {
            "waypoint" => {
                let b = c.waypoints.len();
                c.waypoints.retain(|x| x.id != item_key);
                (b, c.waypoints.len())
            }
            "artwork" => {
                let b = c.artworks.len();
                c.artworks.retain(|x| x.id != item_key);
                (b, c.artworks.len())
            }
            "spot" => {
                let b = c.spots.len();
                c.spots.retain(|x| x.id != item_key);
                (b, c.spots.len())
            }
            "itinerary" => {
                let days: u32 = item_key.trim().parse().map_err(|_| {
                    TravelCraftError::Invalid(format!("invalid itinerary days '{item_key}'"))
                })?;
                let b = c.itineraries.len();
                c.itineraries.retain(|x| x.total_days != days);
                (b, c.itineraries.len())
            }
            "gotcha" => {
                let b = c.gotchas.len();
                c.gotchas.retain(|x| x.id != item_key);
                (b, c.gotchas.len())
            }
            "zone" => {
                let b = c.zones.len();
                c.zones.retain(|x| x.id != item_key);
                (b, c.zones.len())
            }
            "event" => {
                let b = c.events.len();
                c.events.retain(|x| x.id != item_key);
                (b, c.events.len())
            }
            "media" => {
                let b = c.media.len();
                c.media.retain(|x| x.id != item_key);
                (b, c.media.len())
            }
            other => {
                return Err(TravelCraftError::Invalid(format!(
                    "unknown item_type '{other}'; expected waypoint|artwork|spot|itinerary|gotcha|zone|event|media"
                )));
            }
        };
        if before == after {
            return Err(TravelCraftError::NotFound(format!(
                "{item_type} '{item_key}' in cartridge '{cartridge_id}'"
            )));
        }
        self.cartridge_upsert(&c)
    }

    /// Seed the reference Italy (`it`) + Venice (`it-venice`) cartridge pair (idempotent).
    pub fn cartridge_seed_venice(&self) -> Result<Vec<CartridgeSummary>, TravelCraftError> {
        let it = self.cartridge_upsert(&italy_country_cartridge())?;
        let ve = self.cartridge_upsert(&venice_city_cartridge())?;
        Ok(vec![it, ve])
    }

    /// Seed the model Japan (`jp`) + Kyoto (`jp-kyoto`) cartridge pair (idempotent).
    pub fn cartridge_seed_japan(&self) -> Result<Vec<CartridgeSummary>, TravelCraftError> {
        use travelcraft_baedeker::packs_japan::{japan_country_cartridge, kyoto_city_cartridge};
        let jp = self.cartridge_upsert(&japan_country_cartridge())?;
        let ky = self.cartridge_upsert(&kyoto_city_cartridge())?;
        Ok(vec![jp, ky])
    }

    /// Seed Austria/Salzburg, UK/London, and France/Paris cartridge pairs (idempotent).
    pub fn cartridge_seed_europe(&self) -> Result<Vec<CartridgeSummary>, TravelCraftError> {
        use travelcraft_baedeker::packs_europe::{
            austria_country_cartridge, france_country_cartridge, london_city_cartridge,
            paris_city_cartridge, salzburg_city_cartridge, uk_country_cartridge,
        };
        let at = self.cartridge_upsert(&austria_country_cartridge())?;
        let szg = self.cartridge_upsert(&salzburg_city_cartridge())?;
        let gb = self.cartridge_upsert(&uk_country_cartridge())?;
        let lon = self.cartridge_upsert(&london_city_cartridge())?;
        let fr = self.cartridge_upsert(&france_country_cartridge())?;
        let par = self.cartridge_upsert(&paris_city_cartridge())?;
        Ok(vec![at, szg, gb, lon, fr, par])
    }

    /// Seed all 5 country cartridges (`it`, `jp`, `at`, `gb`, `fr`) and 5 city cartridges
    /// (`it-venice`, `jp-kyoto`, `at-salzburg`, `gb-london`, `fr-paris`) (idempotent).
    pub fn cartridge_seed_all(&self) -> Result<Vec<CartridgeSummary>, TravelCraftError> {
        let mut all = self.cartridge_seed_venice()?;
        all.extend(self.cartridge_seed_japan()?);
        all.extend(self.cartridge_seed_europe()?);
        Ok(all)
    }

    /// Compute an offline slippy-tile prefetch plan for a cartridge's `bbox` via `giscraft-core`.
    pub fn cartridge_tile_plan(
        &self,
        id: &str,
        min_zoom: u8,
        max_zoom: u8,
    ) -> Result<CartridgeTilePlan, TravelCraftError> {
        use giscraft_core::tiles::lnglat_to_tile_float;

        if min_zoom > max_zoom || max_zoom > 18 {
            return Err(TravelCraftError::Invalid(format!(
                "invalid zoom range {min_zoom}..={max_zoom} (expected 0 <= min <= max <= 18)"
            )));
        }
        let c = self.cartridge_get(id)?;
        let bbox = c.bbox.ok_or_else(|| {
            TravelCraftError::Invalid(format!("cartridge '{id}' has no bbox defined"))
        })?;

        let mut per_zoom = Vec::new();
        let mut total_tiles = 0usize;
        for z in min_zoom..=max_zoom {
            let (x0, y0) = lnglat_to_tile_float(bbox.min_lon, bbox.max_lat, z);
            let (x1, y1) = lnglat_to_tile_float(bbox.max_lon, bbox.min_lat, z);
            let nx = ((x1.floor() - x0.floor()).max(0.0) as usize).saturating_add(1);
            let ny = ((y1.floor() - y0.floor()).max(0.0) as usize).saturating_add(1);
            let count = nx.saturating_mul(ny);
            total_tiles = total_tiles.saturating_add(count);
            per_zoom.push(ZoomTileCount {
                zoom: z,
                tiles: count,
            });
        }
        // Typical OSM PNG tile ~18 KiB average in urban areas.
        let estimated_mb = (total_tiles as f64 * 18.0) / 1024.0;
        Ok(CartridgeTilePlan {
            cartridge_id: c.id,
            title: c.title,
            min_zoom,
            max_zoom,
            total_tiles,
            estimated_mb: (estimated_mb * 100.0).round() / 100.0,
            per_zoom,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomTileCount {
    pub zoom: u8,
    pub tiles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartridgeTilePlan {
    pub cartridge_id: String,
    pub title: String,
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub total_tiles: usize,
    pub estimated_mb: f64,
    pub per_zoom: Vec<ZoomTileCount>,
}

fn summary_of(c: &Cartridge, updated_at: i64) -> CartridgeSummary {
    CartridgeSummary {
        id: c.id.clone(),
        kind: c.kind,
        parent_id: c.parent_id.clone(),
        country_code: c.country_code.clone(),
        title: c.title.clone(),
        version: c.version.clone(),
        updated_at,
        waypoint_count: c.waypoints.len(),
        artwork_count: c.artworks.len(),
        spot_count: c.spots.len(),
        itinerary_count: c.itineraries.len(),
        event_count: c.events.len(),
        media_count: c.media.len(),
    }
}
