//! Layered Country & City Cartridge schema and reference builders.
//!
//! A `Country` cartridge holds national survival rules, rail basics, phrasebook,
//! and macro knowledge. A `City` cartridge declares `parent_id` pointing to its
//! country cartridge and adds local zones, chrono-waypoints, artworks, local
//! food/drink spots, transit snapshot, stay itineraries, and gotchas.

use crate::{
    ArtworkRecord, ChronoWaypoint, CuratedRoute, InfoBranch, InfoTopicItem, KnowledgeTree,
    SensoryAtmosphere, StayItinerary,
};
use serde::{Deserialize, Serialize};

pub const CARTRIDGE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CartridgeKind {
    Country,
    City,
}

impl CartridgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Country => "country",
            Self::City => "city",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "country" => Some(Self::Country),
            "city" => Some(Self::City),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapViewport {
    pub lon: f64,
    pub lat: f64,
    pub zoom: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoBbox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl GeoBbox {
    pub fn contains(&self, lat: f64, lon: f64) -> bool {
        lat >= self.min_lat && lat <= self.max_lat && lon >= self.min_lon && lon <= self.max_lon
    }

    pub fn is_valid(&self) -> bool {
        self.min_lat.is_finite()
            && self.max_lat.is_finite()
            && self.min_lon.is_finite()
            && self.max_lon.is_finite()
            && (-90.0..=90.0).contains(&self.min_lat)
            && (-90.0..=90.0).contains(&self.max_lat)
            && (-180.0..=180.0).contains(&self.min_lon)
            && (-180.0..=180.0).contains(&self.max_lon)
            && self.min_lat <= self.max_lat
            && self.min_lon <= self.max_lon
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurvivalKit {
    #[serde(default)]
    pub emergency_number: String,
    #[serde(default)]
    pub power_plugs: String,
    #[serde(default)]
    pub tap_water: String,
    #[serde(default)]
    pub tipping_and_cover: String,
    #[serde(default)]
    pub transit_basics: String,
    #[serde(default)]
    pub medical_notes: String,
    #[serde(default)]
    pub general_notes: String,
}

impl SurvivalKit {
    /// Overlay `self` (child) onto `parent`, keeping parent values wherever child is empty.
    pub fn overlay_on(&self, parent: &Self) -> Self {
        fn pick(child: &str, base: &str) -> String {
            if child.trim().is_empty() {
                base.to_string()
            } else {
                child.to_string()
            }
        }
        Self {
            emergency_number: pick(&self.emergency_number, &parent.emergency_number),
            power_plugs: pick(&self.power_plugs, &parent.power_plugs),
            tap_water: pick(&self.tap_water, &parent.tap_water),
            tipping_and_cover: pick(&self.tipping_and_cover, &parent.tipping_and_cover),
            transit_basics: pick(&self.transit_basics, &parent.transit_basics),
            medical_notes: pick(&self.medical_notes, &parent.medical_notes),
            general_notes: pick(&self.general_notes, &parent.general_notes),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhraseEntry {
    pub phrase: String,
    pub local: String,
    #[serde(default)]
    pub pronunciation: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriceTier {
    Budget,
    Midrange,
    Splurge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpotCategory {
    Bacaro,
    Restaurant,
    Cafe,
    Pasticceria,
    Market,
    Bookshop,
    Viewpoint,
    Stay,
    Shop,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalSpot {
    pub id: String,
    pub name: String,
    pub category: SpotCategory,
    #[serde(default)]
    pub tier: Option<PriceTier>,
    #[serde(default)]
    pub zone_id: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
    #[serde(default)]
    pub price_band: String,
    #[serde(default)]
    pub booking_note: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub photo_uris: Vec<String>,
    #[serde(default)]
    pub source_urls: Vec<String>,
    #[serde(default)]
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventCategory {
    Exhibition,
    Concert,
    Festival,
    Closure,
    Alert,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentEvent {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub venue: String,
    pub category: EventCategory,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub booking_url: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaAsset {
    pub id: String,
    #[serde(default)]
    pub subject_id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub local_path: String,
    #[serde(default)]
    pub caption: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub attribution: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitSnapshot {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub lines: Vec<String>,
    #[serde(default)]
    pub airport_links: Vec<String>,
    #[serde(default)]
    pub passes: Vec<String>,
    #[serde(default)]
    pub tips: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gotcha {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub season: String,
    pub advice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cartridge {
    pub id: String,
    pub kind: CartridgeKind,
    #[serde(default)]
    pub parent_id: Option<String>,
    pub country_code: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub timezone: String,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub viewport: Option<MapViewport>,
    #[serde(default)]
    pub bbox: Option<GeoBbox>,
    #[serde(default)]
    pub zones: Vec<Zone>,
    #[serde(default)]
    pub survival_kit: SurvivalKit,
    #[serde(default)]
    pub phrases: Vec<PhraseEntry>,
    #[serde(default)]
    pub waypoints: Vec<ChronoWaypoint>,
    #[serde(default)]
    pub artworks: Vec<ArtworkRecord>,
    #[serde(default)]
    pub spots: Vec<LocalSpot>,
    #[serde(default)]
    pub transit: TransitSnapshot,
    #[serde(default)]
    pub itineraries: Vec<StayItinerary>,
    #[serde(default)]
    pub routes: Vec<CuratedRoute>,
    #[serde(default)]
    pub gotchas: Vec<Gotcha>,
    #[serde(default)]
    pub events: Vec<CurrentEvent>,
    #[serde(default)]
    pub events_refreshed_at: Option<String>,
    #[serde(default)]
    pub media: Vec<MediaAsset>,
    #[serde(default)]
    pub knowledge_tree: Option<KnowledgeTree>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_schema_version() -> u32 {
    CARTRIDGE_SCHEMA_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartridgeSummary {
    pub id: String,
    pub kind: CartridgeKind,
    pub parent_id: Option<String>,
    pub country_code: String,
    pub title: String,
    pub version: String,
    pub updated_at: i64,
    pub waypoint_count: usize,
    pub artwork_count: usize,
    pub spot_count: usize,
    pub itinerary_count: usize,
    pub event_count: usize,
    pub media_count: usize,
}

/// Effective cartridge after inheriting parent country data (if any).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCartridge {
    pub cartridge: Cartridge,
    pub parent_id: Option<String>,
    pub parent_title: Option<String>,
    pub effective_currency: String,
    pub effective_languages: Vec<String>,
    pub effective_timezone: String,
    pub effective_survival_kit: SurvivalKit,
    pub effective_phrases: Vec<PhraseEntry>,
    pub effective_gotchas: Vec<Gotcha>,
    pub effective_knowledge_branches: Vec<InfoBranch>,
}

pub fn is_valid_slug(id: &str) -> bool {
    let len = id.len();
    if !(2..=64).contains(&len) {
        return false;
    }
    id.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

impl Cartridge {
    pub fn from_json_str(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(raw)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_slug(&self.id) {
            return Err(format!(
                "invalid cartridge id '{}': must be 2..=64 lowercase ASCII [a-z0-9-]",
                self.id
            ));
        }
        if self.title.trim().is_empty() {
            return Err("cartridge title must not be empty".to_string());
        }
        let cc = self.country_code.trim();
        if cc.len() != 2 || !cc.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(format!(
                "country_code '{}' must be 2 uppercase ASCII letters (ISO 3166-1 alpha-2)",
                self.country_code
            ));
        }
        if self.schema_version == 0 || self.schema_version > CARTRIDGE_SCHEMA_VERSION {
            return Err(format!(
                "unsupported schema_version {} (supported: 1..={})",
                self.schema_version, CARTRIDGE_SCHEMA_VERSION
            ));
        }
        if let Some(parent) = &self.parent_id {
            if self.kind == CartridgeKind::Country {
                return Err("country cartridge must not have a parent_id".to_string());
            }
            if !is_valid_slug(parent) {
                return Err(format!("invalid parent_id '{parent}'"));
            }
            if parent == &self.id {
                return Err("cartridge cannot be its own parent".to_string());
            }
        }
        if let Some(vp) = &self.viewport {
            if !vp.lat.is_finite()
                || !vp.lon.is_finite()
                || !(-90.0..=90.0).contains(&vp.lat)
                || !(-180.0..=180.0).contains(&vp.lon)
                || !vp.zoom.is_finite()
                || !(1.0..=22.0).contains(&vp.zoom)
            {
                return Err("viewport coordinates or zoom out of range".to_string());
            }
        }
        if let Some(bb) = &self.bbox {
            if !bb.is_valid() {
                return Err("bbox coordinates are invalid".to_string());
            }
        }
        for wp in &self.waypoints {
            if !wp.lat.is_finite()
                || !wp.lon.is_finite()
                || !(-90.0..=90.0).contains(&wp.lat)
                || !(-180.0..=180.0).contains(&wp.lon)
            {
                return Err(format!("waypoint '{}' has invalid coordinates", wp.id));
            }
        }
        Ok(())
    }

    pub fn resolve_with_parent(&self, parent: Option<&Self>) -> ResolvedCartridge {
        let effective_currency = if !self.currency.trim().is_empty() {
            self.currency.clone()
        } else {
            parent.map(|p| p.currency.clone()).unwrap_or_default()
        };
        let effective_languages = if !self.languages.is_empty() {
            self.languages.clone()
        } else {
            parent.map(|p| p.languages.clone()).unwrap_or_default()
        };
        let effective_timezone = if !self.timezone.trim().is_empty() {
            self.timezone.clone()
        } else {
            parent.map(|p| p.timezone.clone()).unwrap_or_default()
        };
        let effective_survival_kit = match parent {
            Some(p) => self.survival_kit.overlay_on(&p.survival_kit),
            None => self.survival_kit.clone(),
        };

        let mut effective_phrases = parent.map(|p| p.phrases.clone()).unwrap_or_default();
        for ph in &self.phrases {
            if !effective_phrases
                .iter()
                .any(|x| x.phrase.eq_ignore_ascii_case(&ph.phrase))
            {
                effective_phrases.push(ph.clone());
            }
        }

        let mut effective_gotchas = parent.map(|p| p.gotchas.clone()).unwrap_or_default();
        for g in &self.gotchas {
            if let Some(existing) = effective_gotchas.iter_mut().find(|x| x.id == g.id) {
                *existing = g.clone();
            } else {
                effective_gotchas.push(g.clone());
            }
        }

        let mut effective_knowledge_branches = parent
            .and_then(|p| p.knowledge_tree.as_ref())
            .map(|kt| kt.branches.clone())
            .unwrap_or_default();
        if let Some(kt) = &self.knowledge_tree {
            for b in &kt.branches {
                if let Some(existing) = effective_knowledge_branches
                    .iter_mut()
                    .find(|x| x.id == b.id)
                {
                    *existing = b.clone();
                } else {
                    effective_knowledge_branches.push(b.clone());
                }
            }
        }

        ResolvedCartridge {
            cartridge: self.clone(),
            parent_id: parent.map(|p| p.id.clone()),
            parent_title: parent.map(|p| p.title.clone()),
            effective_currency,
            effective_languages,
            effective_timezone,
            effective_survival_kit,
            effective_phrases,
            effective_gotchas,
            effective_knowledge_branches,
        }
    }
}

/// Reference Country Cartridge: Italy (`it`).
pub fn italy_country_cartridge() -> Cartridge {
    Cartridge {
        id: "it".to_string(),
        kind: CartridgeKind::Country,
        parent_id: None,
        country_code: "IT".to_string(),
        title: "Italy".to_string(),
        subtitle: "Baedeker Country Cartridge - Survival, Rail & Culture".to_string(),
        description: "National practical kit, rail rules, coffee/coperto etiquette, and phrasebook inherited by Italian city cartridges.".to_string(),
        version: "1.0.0".to_string(),
        schema_version: CARTRIDGE_SCHEMA_VERSION,
        currency: "EUR".to_string(),
        languages: vec!["it".to_string()],
        timezone: "Europe/Rome".to_string(),
        sources: vec!["travelcraft-reference-italy".to_string()],
        viewport: Some(MapViewport {
            lon: 12.5674,
            lat: 41.8719,
            zoom: 5.5,
        }),
        bbox: Some(GeoBbox {
            min_lat: 36.6,
            max_lat: 47.1,
            min_lon: 6.6,
            max_lon: 18.6,
        }),
        zones: Vec::new(),
        survival_kit: SurvivalKit {
            emergency_number: "112 (EU emergency), 118 (medical), 113 (police)".to_string(),
            power_plugs: "Type C, F, L (230V / 50Hz); Type L 3-pin grounded outlets are common in older hotels".to_string(),
            tap_water: "Safe to drink; public fountains (nasoni/fontanelle) are potable unless marked 'acqua non potabile'".to_string(),
            tipping_and_cover: "Coperto (EUR 1.50-4 per person) covers bread/table; tipping is optional (round up or leave 5-10% for exceptional service)".to_string(),
            transit_basics: "Trenitalia and Italo run high-speed rail (book Frecce/Italo early). Paper regional tickets MUST be stamped in the green/yellow platform validator before boarding; digital regional tickets require check-in in the app.".to_string(),
            medical_notes: "Green glowing cross marks pharmacies (Farmacia); rotating 24h duty pharmacy (farmacia di turno) posted on every pharmacy door.".to_string(),
            general_notes: "Standing at the coffee bar (al banco) is often half the price of sitting at a table (al tavolo). Many museums close Mondays.".to_string(),
        },
        phrases: vec![
            PhraseEntry {
                phrase: "Good morning / Good evening".to_string(),
                local: "Buongiorno / Buonasera".to_string(),
                pronunciation: "bwon-JOR-no / bwo-na-SEH-ra".to_string(),
                context: "Always greet when entering small shops or bacari".to_string(),
            },
            PhraseEntry {
                phrase: "The bill, please".to_string(),
                local: "Il conto, per favore".to_string(),
                pronunciation: "eel KON-to per fa-VO-reh".to_string(),
                context: "Waiters rarely bring the bill until asked".to_string(),
            },
            PhraseEntry {
                phrase: "A glass of house wine".to_string(),
                local: "Un'ombra di vino della casa".to_string(),
                pronunciation: "oon OM-bra dee VEE-no DEL-la KA-za".to_string(),
                context: "Venetian idiom for a small glass at a bacaro".to_string(),
            },
        ],
        waypoints: vec![
            ChronoWaypoint {
                id: "it-rome-forum".to_string(),
                name: "Rome - Forum & Colosseum".to_string(),
                region: "Lazio".to_string(),
                lat: 41.8902,
                lon: 12.4922,
                year_ad: 80,
                historic_narrative: "Political and ceremonial core of Imperial Rome; pair early Forum entry with Capitoline Museums and Trastevere.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Seagulls over travertine arches, distant Roman traffic".to_string(),
                    architectural_era: "Imperial Roman (1st c. AD)".to_string(),
                    olfactory_notes: "Warm umbrella pines, dry travertine dust".to_string(),
                    light_condition: "Amber late-afternoon light across Palatine Hill".to_string(),
                },
                culinary_specialty: vec!["Cacio e pepe".to_string(), "Carciofi alla giudia".to_string()],
                visitor_info: "Book Colosseum + Forum combo 30 days ahead on ticketing.colosseo.it.".to_string(),
            },
            ChronoWaypoint {
                id: "it-florence-duomo".to_string(),
                name: "Florence - Santa Maria del Fiore & Uffizi".to_string(),
                region: "Tuscany".to_string(),
                lat: 43.7731,
                lon: 11.2560,
                year_ad: 1436,
                historic_narrative: "Brunelleschi's double-shell terracotta dome crowned the Renaissance; Uffizi and Oltrarno workshops sit 10 minutes south.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Cathedral bells, footsteps on pietra serena stone".to_string(),
                    architectural_era: "Early Renaissance (1296-1436)".to_string(),
                    olfactory_notes: "Leather workshops, espresso, roasted chestnuts in autumn".to_string(),
                    light_condition: "Crisp Tuscan light on green-and-white marble".to_string(),
                },
                culinary_specialty: vec!["Ribollita".to_string(), "Bistecca alla fiorentina".to_string()],
                visitor_info: "Frecciarossa 2h05m from Venice Santa Lucia; book Uffizi and Dome climb weeks ahead.".to_string(),
            },
            ChronoWaypoint {
                id: "it-bologna-portici".to_string(),
                name: "Bologna - Piazza Maggiore & Porticoes".to_string(),
                region: "Emilia-Romagna".to_string(),
                lat: 44.4937,
                lon: 11.3430,
                year_ad: 1088,
                historic_narrative: "Europe's oldest university city, sheltered by 62 km of UNESCO porticoes and anchored by the Quadrilatero food alleys.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Student chatter echoing under vaulted brick arcades".to_string(),
                    architectural_era: "Medieval brick & Renaissance porticoes".to_string(),
                    olfactory_notes: "Aged Parmigiano, mortadella, ragu simmering".to_string(),
                    light_condition: "Warm ochre brick glowing even in November rain".to_string(),
                },
                culinary_specialty: vec!["Tagliatelle al ragu".to_string(), "Tortellini in brodo".to_string()],
                visitor_info: "90 min high-speed train from Venice; ideal rain-proof food stop.".to_string(),
            },
            ChronoWaypoint {
                id: "it-venice-anchor".to_string(),
                name: "Venice - Grand Canal & San Marco".to_string(),
                region: "Veneto".to_string(),
                lat: 45.4371,
                lon: 12.3326,
                year_ad: 1094,
                historic_narrative: "Maritime republic built on oak pilings across 118 lagoon islands; open the `it-venice` city cartridge for the full 7-day guide.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Vaporetto wake slapping Istrian stone, church bells".to_string(),
                    architectural_era: "Veneto-Byzantine & Venetian Gothic".to_string(),
                    olfactory_notes: "Lagoon brine, espresso, wet stone".to_string(),
                    light_condition: "Reflected water light across marble facades".to_string(),
                },
                culinary_specialty: vec!["Cicchetti & ombra".to_string(), "Baccala mantecato".to_string()],
                visitor_info: "Direct OeBB Railjet / Nightjet from Wien Hbf to Venezia Santa Lucia.".to_string(),
            },
        ],
        artworks: Vec::new(),
        spots: Vec::new(),
        transit: TransitSnapshot {
            summary: "Direct OeBB Railjet / Nightjet from Wien Hbf to Venezia Santa Lucia (~7.5h day / overnight sleeper).".to_string(),
            lines: vec![
                "OeBB Railjet / Nightjet: Wien Hbf -> Villach -> Udine -> Venezia Santa Lucia".to_string(),
                "Frecciarossa / Italo: Venezia Santa Lucia -> Padova (25 min) -> Verona (70 min) -> Bologna (90 min)".to_string(),
            ],
            airport_links: Vec::new(),
            passes: Vec::new(),
            tips: vec![
                "Watch for 'sciopero' (scheduled transit strikes); minimum guaranteed rail windows usually apply 06:00-09:00 and 18:00-21:00.".to_string(),
            ],
        },
        itineraries: Vec::new(),
        routes: Vec::new(),
        gotchas: vec![
            Gotcha {
                id: "it-regional-validation".to_string(),
                title: "Regional train ticket validation".to_string(),
                season: "year-round".to_string(),
                advice: "Validate paper regional tickets before boarding or check in on the Trenitalia app; conductors fine unvalidated tickets on the spot.".to_string(),
            },
            Gotcha {
                id: "it-monday-closures".to_string(),
                title: "Monday museum closures".to_string(),
                season: "year-round".to_string(),
                advice: "Many state and civic museums close Mondays (some close Tuesdays). Check each palazzo before planning Day 1.".to_string(),
            },
        ],
        events: vec![
            CurrentEvent {
                id: "ev-it-autumn-truffle-2026".to_string(),
                title: "Italy Autumn White Truffle & New Olive Oil Season".to_string(),
                venue: "Piedmont (Alba), Emilia-Romagna, Tuscany & Veneto".to_string(),
                category: EventCategory::Festival,
                start_date: Some("2026-10-15".to_string()),
                end_date: Some("2026-12-08".to_string()),
                booking_url: "https://www.fieradeltartufo.org/en/".to_string(),
                note: "Peak season for olio nuovo, white truffles, radicchio tardivo di Treviso, and autumn sagre across Northern Italy.".to_string(),
                source_url: "https://www.fieradeltartufo.org/en/".to_string(),
                verified_at: Some("2026-11-20".to_string()),
            },
        ],
        events_refreshed_at: Some("2026-11-20".to_string()),
        media: Vec::new(),
        knowledge_tree: Some(KnowledgeTree {
            country_code: "IT".to_string(),
            title: "Italy Primer".to_string(),
            branches: vec![InfoBranch {
                id: "it-etiquette".to_string(),
                title: "Coffee & Dining Rules".to_string(),
                icon: "coffee".to_string(),
                items: vec![InfoTopicItem {
                    id: "it-bar-rule".to_string(),
                    title: "Al Banco vs Al Tavolo".to_string(),
                    subtitle: "Standing espresso costs EUR 1.20-1.50; table service can triple it in tourist squares.".to_string(),
                    tags: vec!["coffee".to_string(), "budget".to_string()],
                    body: "Pay at the cassa first in classic bars, then hand the receipt (scontrino) to the barista. Cappuccino is culturally a morning drink.".to_string(),
                    practical_tips: vec![
                        "Keep your scontrino until you leave the shop.".to_string(),
                    ],
                    lat: None,
                    lon: None,
                }],
            }],
        }),
    }
}

/// Reference City Cartridge: "A Week in Venice" (`it-venice`, parent `it`).
pub fn venice_city_cartridge() -> Cartridge {
    Cartridge {
        id: "it-venice".to_string(),
        kind: CartridgeKind::City,
        parent_id: Some("it".to_string()),
        country_code: "IT".to_string(),
        title: "Venice".to_string(),
        subtitle: "A Week in Venice - Six Sestieri & the Lagoon".to_string(),
        description: "Offline Baedeker city cartridge for a 7-day stay in Venice: sestieri zones, chrono-waypoints, Tintoretto/Titian/Carpaccio artworks, bacari/cicchetti spots, ACTV vaporetto transit, and acqua alta gotchas.".to_string(),
        version: "1.0.0".to_string(),
        schema_version: CARTRIDGE_SCHEMA_VERSION,
        currency: String::new(), // inherited from `it`
        languages: Vec::new(),   // inherited from `it`
        timezone: String::new(), // inherited from `it`
        sources: vec!["travelcraft-reference-venice".to_string()],
        viewport: Some(MapViewport {
            lon: 12.3326,
            lat: 45.4371,
            zoom: 13.5,
        }),
        bbox: Some(GeoBbox {
            min_lat: 45.35,
            max_lat: 45.52,
            min_lon: 12.22,
            max_lon: 12.46,
        }),
        zones: vec![
            Zone {
                id: "san-marco".to_string(),
                name: "San Marco".to_string(),
                summary: "Political and ceremonial heart; visit before 08:30 or after 19:00 to escape day-trippers.".to_string(),
                lat: Some(45.4342),
                lon: Some(12.3388),
            },
            Zone {
                id: "dorsoduro".to_string(),
                name: "Dorsoduro".to_string(),
                summary: "University and museum quarter: Accademia, Peggy Guggenheim, Zattere promenade.".to_string(),
                lat: Some(45.4325),
                lon: Some(12.3266),
            },
            Zone {
                id: "san-polo".to_string(),
                name: "San Polo".to_string(),
                summary: "Rialto market, Frari basilica, Scuola Grande di San Rocco, dense bacari alleys.".to_string(),
                lat: Some(45.4376),
                lon: Some(12.3292),
            },
            Zone {
                id: "cannaregio".to_string(),
                name: "Cannaregio".to_string(),
                summary: "Venetian Ghetto, Fondamenta della Misericordia evening canals, Madonna dell'Orto.".to_string(),
                lat: Some(45.4449),
                lon: Some(12.3264),
            },
            Zone {
                id: "castello".to_string(),
                name: "Castello".to_string(),
                summary: "Arsenale, Biennale Giardini, Via Garibaldi, quiet residential eastern edge.".to_string(),
                lat: Some(45.4346),
                lon: Some(12.3526),
            },
            Zone {
                id: "santa-croce".to_string(),
                name: "Santa Croce".to_string(),
                summary: "Piazzale Roma gateway, San Giacomo dell'Orio, quieter west canals.".to_string(),
                lat: Some(45.4408),
                lon: Some(12.3220),
            },
            Zone {
                id: "lagoon-north".to_string(),
                name: "Murano, Burano & Torcello".to_string(),
                summary: "Northern lagoon islands reachable on ACTV Line 12 from Fondamente Nove.".to_string(),
                lat: Some(45.4854),
                lon: Some(12.4167),
            },
        ],
        survival_kit: SurvivalKit {
            transit_basics: "No cars inside historic Venice. Walk or take ACTV vaporetti; use EUR 2 standing traghetti to cross the Grand Canal without detouring to a bridge.".to_string(),
            general_notes: "Avoid restaurants with laminated photo menus or touts near San Marco and Rialto. Two bridges away in San Polo, Cannaregio, or Castello the food is local and half the price.".to_string(),
            ..SurvivalKit::default()
        },
        phrases: vec![
            PhraseEntry {
                phrase: "Small Venetian bar snacks".to_string(),
                local: "Cicchetti".to_string(),
                pronunciation: "chee-KET-tee".to_string(),
                context: "Point at the glass counter: baccala mantecato, sarde in saor, polpette".to_string(),
            },
        ],
        waypoints: vec![
            ChronoWaypoint {
                id: "ve-torcello".to_string(),
                name: "Basilica di Santa Maria Assunta (Torcello)".to_string(),
                region: "lagoon-north".to_string(),
                lat: 45.4981,
                lon: 12.4192,
                year_ad: 639,
                historic_narrative: "Oldest surviving monument in the Venetian lagoon. The 11th-century Byzantine Last Judgment mosaic and lonely golden Virgin in the apse predate the rise of San Marco.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Marsh reeds, distant vaporetto horn, wind across empty brick".to_string(),
                    architectural_era: "Veneto-Byzantine (639-1008)".to_string(),
                    olfactory_notes: "Brackish lagoon mud, wild fennel, cool stone".to_string(),
                    light_condition: "Slanted golden light catching apse tesserae".to_string(),
                },
                culinary_specialty: vec!["Risotto di go".to_string(), "Carciofi violetti di Sant'Erasmo".to_string()],
                visitor_info: "Line 12 to Burano then Line 9 shuttle (5 min). Go early morning before tour boats.".to_string(),
            },
            ChronoWaypoint {
                id: "ve-san-marco".to_string(),
                name: "Basilica di San Marco".to_string(),
                region: "san-marco".to_string(),
                lat: 45.4345,
                lon: 12.3397,
                year_ad: 1094,
                historic_narrative: "Chapel of the Doge clad in 8,000 square meters of gold-ground mosaics and spoils of the Fourth Crusade. Pay the extra ticket for the Pala d'Oro and upper loggia horses.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Hushed footsteps over undulating marble wave-floor, bells on the hour".to_string(),
                    architectural_era: "Italo-Byzantine / Gothic (1094-15th c.)".to_string(),
                    olfactory_notes: "Beeswax, damp Istrian stone, sea air".to_string(),
                    light_condition: "Dim nave; mosaic illumination turns on around 11:30-12:30".to_string(),
                },
                culinary_specialty: vec!["Espresso at Caffe Florian (al banco)".to_string()],
                visitor_info: "Book timed skip-the-line ticket online; no backpacks (free luggage drop in Calle San Basso).".to_string(),
            },
            ChronoWaypoint {
                id: "ve-frari".to_string(),
                name: "Basilica di Santa Maria Gloriosa dei Frari".to_string(),
                region: "san-polo".to_string(),
                lat: 45.4367,
                lon: 12.3266,
                year_ad: 1338,
                historic_narrative: "Vast brick Franciscan Gothic church housing Titian's towering Assumption of the Virgin, his Pesaro Madonna, Bellini's Frari Triptych, and Canova's pyramid tomb.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Deep brick resonance, quiet wooden choir stalls".to_string(),
                    architectural_era: "Venetian Brick Gothic (1338-1443)".to_string(),
                    olfactory_notes: "Old walnut choir wood, cool brick".to_string(),
                    light_condition: "Clear morning sun illuminating the high altar".to_string(),
                },
                culinary_specialty: vec!["Tramezzini near Campo dei Frari".to_string()],
                visitor_info: "Included in Chorus Pass or single ticket (~EUR 5). Pair directly with San Rocco behind it.".to_string(),
            },
            ChronoWaypoint {
                id: "ve-ghetto".to_string(),
                name: "Campo del Ghetto Nuovo".to_string(),
                region: "cannaregio".to_string(),
                lat: 45.4453,
                lon: 12.3266,
                year_ad: 1516,
                historic_narrative: "Established by decree of the Venetian Senate in 1516 on the old foundry (geto) island, giving the word 'ghetto' to the world. Tall vertical tenements rise around a quiet paved campo.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Quiet footsteps, muffled canal lap, children playing in the campo".to_string(),
                    architectural_era: "16th-century Venetian vertical tenements".to_string(),
                    olfactory_notes: "Fresh kosher bakery pastries, canal breeze".to_string(),
                    light_condition: "Crisp afternoon shadow across tall ochre facades".to_string(),
                },
                culinary_specialty: vec!["Sarde in saor".to_string(), "Almond impade pastries".to_string()],
                visitor_info: "Book guided synagogue tour through Museo Ebraico di Venezia.".to_string(),
            },
            ChronoWaypoint {
                id: "ve-san-rocco".to_string(),
                name: "Scuola Grande di San Rocco".to_string(),
                region: "san-polo".to_string(),
                lat: 45.4365,
                lon: 12.3252,
                year_ad: 1564,
                historic_narrative: "The 'Sistine Chapel of Venice': Jacopo Tintoretto spent 23 years painting over 60 dramatic canvases across ceilings and walls for this charitable confraternity.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Low murmur, creak of mirror-holders tilted toward the ceiling".to_string(),
                    architectural_era: "Venetian Renaissance (1517-1560)".to_string(),
                    olfactory_notes: "Varnished timber, cool marble stairs".to_string(),
                    light_condition: "Dramatic side light; use the handheld mirrors provided in the Sala Superiore".to_string(),
                },
                culinary_specialty: vec!["Cicchetti at nearby Campo Santa Margherita".to_string()],
                visitor_info: "Open daily ~09:30-17:30; allow 90 minutes. Rarely crowded compared to Doge's Palace.".to_string(),
            },
            ChronoWaypoint {
                id: "ve-accademia".to_string(),
                name: "Gallerie dell'Accademia".to_string(),
                region: "dorsoduro".to_string(),
                lat: 45.4314,
                lon: 12.3281,
                year_ad: 1750,
                historic_narrative: "Definitive collection of Venetian painting from 14th-century gold-ground polyptychs through Bellini, Giorgione's La Tempesta, Carpaccio, Veronese, Titian, and Tintoretto.".to_string(),
                sensory_profile: SensoryAtmosphere {
                    acoustic_profile: "Calm museum galleries, wooden Ponte dell'Accademia footsteps outside".to_string(),
                    architectural_era: "Former Scuola della Carita + Palladian wing".to_string(),
                    olfactory_notes: "Waxed floors, old plaster".to_string(),
                    light_condition: "Controlled skylight galleries".to_string(),
                },
                culinary_specialty: vec!["Gelato along Fondamenta delle Zattere".to_string()],
                visitor_info: "Closed Mondays (verify seasonally). Book online in peak months.".to_string(),
            },
        ],
        artworks: vec![
            ArtworkRecord {
                id: "art-tintoretto-crucifixion".to_string(),
                title: "The Crucifixion".to_string(),
                artist: "Jacopo Tintoretto".to_string(),
                year: "1565".to_string(),
                museum: "Scuola Grande di San Rocco".to_string(),
                room: "Sala dell'Albergo".to_string(),
                lat: 45.4365,
                lon: 12.3252,
                story: "A 12-meter-wide cinematic panorama completed in a single year after Tintoretto won the Scuola commission by secretly installing his ceiling painting ahead of the competition.".to_string(),
                look_for: "The circular swirl of labor around the rising crosses, the eerie phosphorescent halo, and the quiet knot of mourners at the base.".to_string(),
            },
            ArtworkRecord {
                id: "art-titian-assumption".to_string(),
                title: "Assumption of the Virgin (Assunta)".to_string(),
                artist: "Titian".to_string(),
                year: "1516-1518".to_string(),
                museum: "Basilica dei Frari".to_string(),
                room: "High Altar".to_string(),
                lat: 45.4367,
                lon: 12.3266,
                story: "Almost seven meters tall, painted on 21 poplar planks; its blazing vermilion pyramid shocked Venetian patrons accustomed to Bellini's stillness.".to_string(),
                look_for: "How the triangular red robes of the apostles and Virgin pull the eye straight up through the stone choir screen from the nave.".to_string(),
            },
            ArtworkRecord {
                id: "art-giorgione-tempesta".to_string(),
                title: "The Tempest (La Tempesta)".to_string(),
                artist: "Giorgione".to_string(),
                year: "c. 1508".to_string(),
                museum: "Gallerie dell'Accademia".to_string(),
                room: "Room V".to_string(),
                lat: 45.4314,
                lon: 12.3281,
                story: "Small, enigmatic landscape with a lightning bolt, a halberdier, and a nursing woman - five centuries of art historians still debate its subject.".to_string(),
                look_for: "The atmospheric storm light over the lagoon-like town and X-ray pentimento where a bathing woman once stood under the soldier.".to_string(),
            },
        ],
        spots: vec![
            LocalSpot {
                id: "spot-cantina-do-mori".to_string(),
                name: "Cantina Do Mori".to_string(),
                category: SpotCategory::Bacaro,
                tier: Some(PriceTier::Budget),
                zone_id: "san-polo".to_string(),
                address: "Calle Do Mori 429, San Polo".to_string(),
                lat: Some(45.4383),
                lon: Some(12.3340),
                price_band: "EUR".to_string(),
                booking_note: "Walk-in only, standing counter; best 11:00-12:30 before lunch rush.".to_string(),
                note: "Operating since 1462 near Rialto market. Standing only, copper pots overhead; order francobolli tramezzini and an ombra of local white.".to_string(),
                tags: vec!["bacaro".to_string(), "cicchetti".to_string(), "historic".to_string()],
                photo_uris: Vec::new(),
                source_urls: Vec::new(),
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-al-timon".to_string(),
                name: "Osteria Al Timon".to_string(),
                category: SpotCategory::Bacaro,
                tier: Some(PriceTier::Midrange),
                zone_id: "cannaregio".to_string(),
                address: "Fondamenta dei Ormesini 2754, Cannaregio".to_string(),
                lat: Some(45.4458),
                lon: Some(12.3274),
                price_band: "EUR-EUR".to_string(),
                booking_note: "Walk-in for canal cicchetti; book 3-5 days ahead for seated dinner.".to_string(),
                note: "Lively canal-side bacaro with a moored wooden bragozzo boat for evening drinks and cicchetti.".to_string(),
                tags: vec!["bacaro".to_string(), "canal".to_string(), "evening".to_string()],
                photo_uris: Vec::new(),
                source_urls: Vec::new(),
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-acqua-alta-books".to_string(),
                name: "Libreria Acqua Alta".to_string(),
                category: SpotCategory::Bookshop,
                tier: Some(PriceTier::Budget),
                zone_id: "castello".to_string(),
                address: "Calle Lunga Santa Maria Formosa 5176B, Castello".to_string(),
                lat: Some(45.4378),
                lon: Some(12.3422),
                price_band: "EUR".to_string(),
                booking_note: "No booking; go right at 09:00 opening to avoid queues.".to_string(),
                note: "Secondhand books stacked in bathtubs and a full gondola to survive high tides. Go early right at opening.".to_string(),
                tags: vec!["bookshop".to_string(), "quirky".to_string()],
                photo_uris: Vec::new(),
                source_urls: Vec::new(),
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-zattere-nico".to_string(),
                name: "Gelateria Nico (Zattere)".to_string(),
                category: SpotCategory::Pasticceria,
                tier: Some(PriceTier::Budget),
                zone_id: "dorsoduro".to_string(),
                address: "Fondamenta Zattere ai Gesuati 922, Dorsoduro".to_string(),
                lat: Some(45.4293),
                lon: Some(12.3257),
                price_band: "EUR".to_string(),
                booking_note: "Walk-in counter; table service on the floating raft costs more.".to_string(),
                note: "Order the gianduiotto (dense hazelnut-chocolate bar drowned in whipped cream) and walk the sunny Giudecca canal promenade.".to_string(),
                tags: vec!["gelato".to_string(), "promenade".to_string()],
                photo_uris: Vec::new(),
                source_urls: Vec::new(),
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-antiche-carampane".to_string(),
                name: "Antiche Carampane".to_string(),
                category: SpotCategory::Restaurant,
                tier: Some(PriceTier::Midrange),
                zone_id: "san-polo".to_string(),
                address: "Rio Tera delle Carampane 1911, San Polo".to_string(),
                lat: Some(45.4379),
                lon: Some(12.3316),
                price_band: "EUR-EUR".to_string(),
                booking_note: "Reserve 1-2 weeks ahead; closed Sun-Mon. Famously 'no tourist menu, no pizza, no lasagna'.".to_string(),
                note: "Historic family-run Venetian trattoria off Rialto; order spider-crab tagliolini (granseola) and baccala mantecato.".to_string(),
                tags: vec!["trattoria".to_string(), "seafood".to_string(), "historic".to_string()],
                photo_uris: Vec::new(),
                source_urls: vec!["https://www.antichecarampane.com".to_string()],
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-alle-testiere".to_string(),
                name: "Osteria alle Testiere".to_string(),
                category: SpotCategory::Restaurant,
                tier: Some(PriceTier::Splurge),
                zone_id: "castello".to_string(),
                address: "Calle del Mondo Novo 5801, Castello".to_string(),
                lat: Some(45.4373),
                lon: Some(12.3404),
                price_band: "EUR-EUR-EUR".to_string(),
                booking_note: "Tiny ~10-table seafood room; book 4-8 weeks ahead. Two dinner seatings.".to_string(),
                note: "Daily Rialto fish-market catch cooked with surgical restraint; strong natural Veneto/Friuli wine list.".to_string(),
                tags: vec!["seafood".to_string(), "splurge".to_string(), "wine".to_string()],
                photo_uris: Vec::new(),
                source_urls: vec!["https://www.osterialletestiere.it".to_string()],
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-local-venice".to_string(),
                name: "Ristorante Local".to_string(),
                category: SpotCategory::Restaurant,
                tier: Some(PriceTier::Splurge),
                zone_id: "castello".to_string(),
                address: "Salizzada dei Greci 3303, Castello".to_string(),
                lat: Some(45.4352),
                lon: Some(12.3452),
                price_band: "EUR-EUR-EUR".to_string(),
                booking_note: "1 Michelin star tasting menu; book 3-6 weeks ahead. Casual sibling Trattoria del Local nearby.".to_string(),
                note: "Contemporary lagoon-to-table tasting menu with open kitchen and Murano-glass interior details.".to_string(),
                tags: vec!["michelin".to_string(), "contemporary".to_string(), "splurge".to_string()],
                photo_uris: Vec::new(),
                source_urls: vec!["https://www.ristorantelocal.com".to_string()],
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-hotel-antico-capon".to_string(),
                name: "Hotel Antico Capon".to_string(),
                category: SpotCategory::Stay,
                tier: Some(PriceTier::Budget),
                zone_id: "dorsoduro".to_string(),
                address: "Campo Santa Margherita 3004B, Dorsoduro".to_string(),
                lat: Some(45.4339),
                lon: Some(12.3237),
                price_band: "EUR".to_string(),
                booking_note: "Book early for spring/autumn; simple Venetian rooms directly on lively Campo Santa Margherita.".to_string(),
                note: "Reliable budget base in Dorsoduro, ~10 min walk from Piazzale Roma with few bridges.".to_string(),
                tags: vec!["hotel".to_string(), "budget".to_string(), "dorsoduro".to_string()],
                photo_uris: Vec::new(),
                source_urls: vec!["https://www.booking.com".to_string()],
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-pensione-accademia".to_string(),
                name: "Pensione Accademia - Villa Maravege".to_string(),
                category: SpotCategory::Stay,
                tier: Some(PriceTier::Midrange),
                zone_id: "dorsoduro".to_string(),
                address: "Fondamenta Maravege 1058, Dorsoduro".to_string(),
                lat: Some(45.4317),
                lon: Some(12.3270),
                price_band: "EUR-EUR".to_string(),
                booking_note: "Reserve 2-3 months ahead; November/February rates drop sharply compared to peak spring.".to_string(),
                note: "17th-century villa with rare private canal-side gardens steps from Accademia vaporetto stop.".to_string(),
                tags: vec!["hotel".to_string(), "garden".to_string(), "historic".to_string()],
                photo_uris: Vec::new(),
                source_urls: vec!["https://www.pensioneaccademia.it".to_string()],
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-ca-maria-adele".to_string(),
                name: "Ca' Maria Adele".to_string(),
                category: SpotCategory::Stay,
                tier: Some(PriceTier::Splurge),
                zone_id: "dorsoduro".to_string(),
                address: "Dorsoduro 111, near Basilica della Salute".to_string(),
                lat: Some(45.4307),
                lon: Some(12.3338),
                price_band: "EUR-EUR-EUR".to_string(),
                booking_note: "Small 16th-century boutique palazzo; book months ahead. Salute vaporetto stop right beside it.".to_string(),
                note: "Velvet-dark Venetian boutique palazzo overlooking the Salute canal with rooftop terrace breakfast.".to_string(),
                tags: vec!["hotel".to_string(), "palazzo".to_string(), "splurge".to_string()],
                photo_uris: Vec::new(),
                source_urls: vec!["https://www.camariaadele.it".to_string()],
                verified_at: Some("2026-11-20".to_string()),
            },
            LocalSpot {
                id: "spot-aman-venice".to_string(),
                name: "Aman Venice".to_string(),
                category: SpotCategory::Stay,
                tier: Some(PriceTier::Splurge),
                zone_id: "san-polo".to_string(),
                address: "Palazzo Papadopoli, Calle Tiepolo 1364, San Polo".to_string(),
                lat: Some(45.4366),
                lon: Some(12.3324),
                price_band: "EUR-EUR-EUR".to_string(),
                booking_note: "Grand Canal 16th-century palazzo with private garden; ultra-splurge tier.".to_string(),
                note: "Tiepolo frescoes, private Grand Canal water-taxi landing, and one of the largest private gardens in central Venice.".to_string(),
                tags: vec!["hotel".to_string(), "grand_canal".to_string(), "splurge".to_string()],
                photo_uris: Vec::new(),
                source_urls: vec!["https://www.aman.com/hotels/aman-venice".to_string()],
                verified_at: Some("2026-11-20".to_string()),
            },
        ],
        transit: TransitSnapshot {
            summary: "ACTV vaporetti cover the Grand Canal and lagoon islands. Single rides are expensive (EUR 9.50 / 75 min), so multi-day passes pay off quickly.".to_string(),
            lines: vec![
                "Line 1: Slow all-stops Grand Canal cruise (Piazzale Roma -> Ferrovia -> Rialto -> San Marco -> Lido)".to_string(),
                "Line 2: Faster Grand Canal + Giudecca loop".to_string(),
                "Lines 4.1/4.2 & 5.1/5.2: Outer perimeter ring around Venice + Murano".to_string(),
                "Line 12: Fondamente Nove -> Murano Faro -> Mazzorbo -> Burano -> Torcello (via Line 9)".to_string(),
            ],
            airport_links: vec![
                "ATVO Express Bus: Marco Polo Airport (VCE) -> Piazzale Roma (~20 min, EUR 10)".to_string(),
                "Alilaguna Blue/Orange water bus: VCE Airport -> San Marco / Zattere / Fondamente Nove (~60-75 min)".to_string(),
            ],
            passes: vec![
                "ACTV 24h / 48h / 72h / 7-day travelcards (7-day pass ~EUR 65; breaks even at 7 single rides)".to_string(),
                "Traghetto gondola ferries across Grand Canal (Santa Sofia, San Toma, Dogana): EUR 2 cash, stand like a local".to_string(),
            ],
            tips: vec![
                "Validate your ACTV card at the white reader on the pontoon BEFORE boarding every single time.".to_string(),
                "Walk between sestieri whenever possible - Venice is compact (~3 km end to end) and alleys beat crowded Line 1.".to_string(),
            ],
        },
        itineraries: vec![
            StayItinerary {
                country_code: "IT".to_string(),
                total_days: 7,
                pace_label: "A Week in Venice (unhurried, sestiere by sestiere)".to_string(),
                days: vec![
                    crate::ItineraryDay {
                        day: 1,
                        title: "San Marco Early & Castello Backstreets".to_string(),
                        waypoint_ids: vec!["ve-san-marco".to_string()],
                        note: "Arrive before 08:30 for Basilica di San Marco + Doge's Palace; escape east to San Zaccaria, Via Garibaldi, and Libreria Acqua Alta.".to_string(),
                    },
                    crate::ItineraryDay {
                        day: 2,
                        title: "Dorsoduro Museums & Zattere Sunset".to_string(),
                        waypoint_ids: vec!["ve-accademia".to_string()],
                        note: "Morning at Gallerie dell'Accademia, afternoon Peggy Guggenheim Collection, gianduiotto on the Zattere.".to_string(),
                    },
                    crate::ItineraryDay {
                        day: 3,
                        title: "San Polo: Rialto, Frari & Tintoretto".to_string(),
                        waypoint_ids: vec!["ve-frari".to_string(), "ve-san-rocco".to_string()],
                        note: "08:00 Rialto fish market, mid-morning Frari, afternoon Scuola Grande di San Rocco, evening cicchetti crawl.".to_string(),
                    },
                    crate::ItineraryDay {
                        day: 4,
                        title: "Cannaregio, the Ghetto & Fondamenta della Misericordia".to_string(),
                        waypoint_ids: vec!["ve-ghetto".to_string()],
                        note: "Jewish Ghetto synagogues, Madonna dell'Orto (Tintoretto's parish church), sunset aperitivo along Misericordia.".to_string(),
                    },
                    crate::ItineraryDay {
                        day: 5,
                        title: "Northern Lagoon: Torcello, Burano & Murano".to_string(),
                        waypoint_ids: vec!["ve-torcello".to_string()],
                        note: "Take an early Line 12 from Fondamente Nove straight to Torcello first, work back through Burano and Murano.".to_string(),
                    },
                    crate::ItineraryDay {
                        day: 6,
                        title: "Arsenale, San Giorgio Maggiore & Giudecca".to_string(),
                        waypoint_ids: Vec::new(),
                        note: "Bell tower view from San Giorgio Maggiore (better and calmer than San Marco campanile), Palladian Redentore, Arsenale walk.".to_string(),
                    },
                    crate::ItineraryDay {
                        day: 7,
                        title: "Day Trip to Padua (Scrovegni Chapel) or Slow Revisit".to_string(),
                        waypoint_ids: Vec::new(),
                        note: "25-minute regional train from Santa Lucia to Padua for Giotto's Scrovegni Chapel (book ahead), or revisit favorite quiet campi.".to_string(),
                    },
                ],
                practical_tip: "Pack a small wheeled bag or backpack - bridges have steps and cobbles. Book San Marco, Accademia, and Scrovegni before departure.".to_string(),
            },
            StayItinerary {
                country_code: "IT".to_string(),
                total_days: 3,
                pace_label: "Long Weekend in Venice".to_string(),
                days: vec![
                    crate::ItineraryDay {
                        day: 1,
                        title: "San Marco & San Polo".to_string(),
                        waypoint_ids: vec!["ve-san-marco".to_string(), "ve-frari".to_string(), "ve-san-rocco".to_string()],
                        note: "San Marco early, cross via traghetto to San Polo for Frari and San Rocco.".to_string(),
                    },
                    crate::ItineraryDay {
                        day: 2,
                        title: "Dorsoduro & Cannaregio".to_string(),
                        waypoint_ids: vec!["ve-accademia".to_string(), "ve-ghetto".to_string()],
                        note: "Accademia morning, Zattere lunch, Cannaregio Ghetto and Misericordia evening.".to_string(),
                    },
                    crate::ItineraryDay {
                        day: 3,
                        title: "Torcello & Burano".to_string(),
                        waypoint_ids: vec!["ve-torcello".to_string()],
                        note: "Line 12 northern lagoon loop before heading back to Santa Lucia.".to_string(),
                    },
                ],
                practical_tip: "A 72-hour ACTV pass covers all three days including the Line 12 lagoon loop.".to_string(),
            },
        ],
        routes: Vec::new(),
        gotchas: vec![
            Gotcha {
                id: "ve-acqua-alta".to_string(),
                title: "Acqua Alta (High Tide)".to_string(),
                season: "Oct-Feb".to_string(),
                advice: "Check Hi!Tide Venice app daily. Piazza San Marco floods first (~90 cm); raised wooden walkways (passerelle) go out above 105 cm. Plastic over-boots are sold everywhere if caught out.".to_string(),
            },
            Gotcha {
                id: "ve-access-fee".to_string(),
                title: "Venice Access Fee (Contributo di Accesso)".to_string(),
                season: "Spring-Summer peak weekends".to_string(),
                advice: "Overnight hotel guests are EXEMPT (you pay the hotel city tax instead), but you still register for a free exemption QR code via cda.ve.it on active peak days.".to_string(),
            },
            Gotcha {
                id: "ve-bridges-luggage".to_string(),
                title: "400+ Bridges & Luggage".to_string(),
                season: "year-round".to_string(),
                advice: "Choose lodging near a vaporetto stop with minimal bridge crossings from Santa Lucia or Piazzale Roma.".to_string(),
            },
        ],
        events: vec![
            CurrentEvent {
                id: "ev-ve-biennale-2026".to_string(),
                title: "61st Venice Art Biennale - In Minor Keys".to_string(),
                venue: "Giardini, Arsenale & Forte Marghera".to_string(),
                category: EventCategory::Exhibition,
                start_date: Some("2026-05-09".to_string()),
                end_date: Some("2026-11-22".to_string()),
                booking_url: "https://www.labiennale.org/en/art/2026".to_string(),
                note: "Curated by Koyo Kouoh; autumn hours 10:00-18:00 (last entry 17:45). Plan separate visits for Giardini and Arsenale.".to_string(),
                source_url: "https://www.labiennale.org/en/art/2026".to_string(),
                verified_at: Some("2026-11-20".to_string()),
            },
            CurrentEvent {
                id: "ev-ve-guggenheim-fucina-2026".to_string(),
                title: "Fucina degli Angeli: Peggy Guggenheim and 20th-Century Artistic Glass".to_string(),
                venue: "Peggy Guggenheim Collection (Dorsoduro)".to_string(),
                category: EventCategory::Exhibition,
                start_date: Some("2026-11-14".to_string()),
                end_date: None,
                booking_url: "https://www.guggenheim-venice.it/en/".to_string(),
                note: "Opened Nov 14, 2026; postwar Murano artistic glass collaborations with Peggy Guggenheim.".to_string(),
                source_url: "https://www.guggenheim-venice.it/en/".to_string(),
                verified_at: Some("2026-11-20".to_string()),
            },
            CurrentEvent {
                id: "ev-ve-dogana-simpson-2026".to_string(),
                title: "Lorna Simpson - Third Person".to_string(),
                venue: "Punta della Dogana (Pinault Collection)".to_string(),
                category: EventCategory::Exhibition,
                start_date: None,
                end_date: Some("2026-11-22".to_string()),
                booking_url: "https://www.pinaultcollection.com/palazzograssi/en".to_string(),
                note: "Final days closing late November 2026 alongside the Biennale closing weekend.".to_string(),
                source_url: "https://www.pinaultcollection.com/palazzograssi/en".to_string(),
                verified_at: Some("2026-11-20".to_string()),
            },
        ],
        events_refreshed_at: Some("2026-11-20".to_string()),
        media: vec![
            MediaAsset {
                id: "media-ve-san-marco".to_string(),
                subject_id: "ve-san-marco".to_string(),
                url: "https://commons.wikimedia.org/wiki/Special:FilePath/Venice_Basilica_di_San_Marco_facade.jpg".to_string(),
                local_path: "media/ve-san-marco.webp".to_string(),
                caption: "West facade of Basilica di San Marco".to_string(),
                license: "CC-BY-SA-4.0".to_string(),
                attribution: "Wikimedia Commons".to_string(),
            },
            MediaAsset {
                id: "media-ve-torcello".to_string(),
                subject_id: "ve-torcello".to_string(),
                url: "https://commons.wikimedia.org/wiki/Special:FilePath/Torcello_Basilica_di_Santa_Maria_Assunta_apside.jpg".to_string(),
                local_path: "media/ve-torcello.webp".to_string(),
                caption: "11th-century Byzantine golden apse mosaic at Torcello".to_string(),
                license: "CC-BY-SA-4.0".to_string(),
                attribution: "Wikimedia Commons".to_string(),
            },
            MediaAsset {
                id: "media-ve-titian-assunta".to_string(),
                subject_id: "art-titian-assumption".to_string(),
                url: "https://commons.wikimedia.org/wiki/Special:FilePath/Tizian_041.jpg".to_string(),
                local_path: "media/ve-titian-assunta.webp".to_string(),
                caption: "Titian, Assumption of the Virgin (1516-1518), Basilica dei Frari".to_string(),
                license: "Public Domain".to_string(),
                attribution: "Wikimedia Commons".to_string(),
            },
        ],
        knowledge_tree: Some(KnowledgeTree {
            country_code: "IT".to_string(),
            title: "Venice Primer".to_string(),
            branches: vec![InfoBranch {
                id: "ve-bacari".to_string(),
                title: "Bacari & Cicchetti Culture".to_string(),
                icon: "wine".to_string(),
                items: vec![InfoTopicItem {
                    id: "ve-bacaro-rule".to_string(),
                    title: "The Venetian Bacaro Rule".to_string(),
                    subtitle: "Stand at the counter, point at 2-3 cicchetti, order an ombra or spritz al Select.".to_string(),
                    tags: vec!["food".to_string(), "bacaro".to_string(), "venice".to_string()],
                    body: "Venetian spritz locals prefer Select (bittersweet ruby, with a green olive) or Campari over tourist Aperol. Classic cicchetti: baccala mantecato (whipped salt cod on polenta), sarde in saor (sweet-sour sardines with pine nuts and raisins), polpette.".to_string(),
                    practical_tips: vec![
                        "Avoid laminated photo menus around San Marco and Rialto bridge.".to_string(),
                        "Rialto produce and fish markets run Tue-Sat mornings (fish closes around 12:00).".to_string(),
                    ],
                    lat: Some(45.4383),
                    lon: Some(12.3340),
                }],
            }],
        }),
    }
}
