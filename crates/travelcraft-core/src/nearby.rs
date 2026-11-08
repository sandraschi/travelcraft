//! Nearby places from OpenStreetMap via the Overpass API.
//!
//! Online and opt-in: only `TravelCraftEngine::nearby*` touches the network, and
//! every successful answer is cached in the engine's SQLite file (24 h), so a
//! repeat question works offline and a failed refresh falls back to the last
//! answer with `stale: true` and a warning, never silently.
//!
//! The centre is `lat` plus `lon`, or a city from a tiny built-in table.
//!
//! Queries are built from curated presets or a strictly fenced key/value pair;
//! there is no raw Overpass QL passthrough. All OSM text is untrusted data and
//! is stripped of control characters and length-capped before it is returned.

use crate::{TravelCraftEngine, TravelCraftError};
use giscraft_core::geo::{haversine_m, LonLat};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

pub const ATTRIBUTION: &str =
    "Data (c) OpenStreetMap contributors, ODbL 1.0. https://www.openstreetmap.org/copyright";
pub const MIN_RADIUS_M: u32 = 50;
pub const MAX_RADIUS_M: u32 = 5_000;
pub const DEFAULT_RADIUS_M: u32 = 1_500;
pub const DEFAULT_LIMIT: usize = 15;
pub const MAX_LIMIT: usize = 50;
/// Most elements requested from Overpass per query. Results are sorted by
/// distance after the fetch, so a dense area can truncate; `truncated` says so.
pub const FETCH_CAP: usize = 300;
const CACHE_TTL_SECS: i64 = 24 * 3600;
const CACHE_PRUNE_SECS: i64 = 30 * 24 * 3600;
const CACHE_MAX_ROWS: i64 = 200;
const MAX_STR_CHARS: usize = 200;

/// A curated category: a name an agent can use and the OSM tags it stands for.
pub struct Preset {
    pub name: &'static str,
    pub about: &'static str,
    pub filters: &'static [(&'static str, &'static str)],
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "bicycle_repair",
        about: "Bike shops and repair services, plus self-service repair stations",
        filters: &[
            ("shop", "bicycle"),
            ("service:bicycle:repair", "yes"),
            ("amenity", "bicycle_repair_station"),
        ],
    },
    Preset {
        name: "bicycle_rental",
        about: "Bike rental and bike share",
        filters: &[("amenity", "bicycle_rental")],
    },
    Preset {
        name: "pharmacy",
        about: "Pharmacies",
        filters: &[("amenity", "pharmacy")],
    },
    Preset {
        name: "doctors",
        about: "Doctors' practices",
        filters: &[("amenity", "doctors")],
    },
    Preset {
        name: "hospital",
        about: "Hospitals",
        filters: &[("amenity", "hospital")],
    },
    Preset {
        name: "atm",
        about: "Cash machines",
        filters: &[("amenity", "atm")],
    },
    Preset {
        name: "cafe",
        about: "Cafes",
        filters: &[("amenity", "cafe")],
    },
    Preset {
        name: "restaurant",
        about: "Restaurants",
        filters: &[("amenity", "restaurant")],
    },
    Preset {
        name: "bakery",
        about: "Bakeries",
        filters: &[("shop", "bakery")],
    },
    Preset {
        name: "supermarket",
        about: "Supermarkets",
        filters: &[("shop", "supermarket")],
    },
    Preset {
        name: "toilets",
        about: "Public toilets",
        filters: &[("amenity", "toilets")],
    },
    Preset {
        name: "drinking_water",
        about: "Drinking water taps and fountains",
        filters: &[("amenity", "drinking_water")],
    },
    Preset {
        name: "ev_charging",
        about: "EV charging stations",
        filters: &[("amenity", "charging_station")],
    },
    Preset {
        name: "post_office",
        about: "Post offices",
        filters: &[("amenity", "post_office")],
    },
    Preset {
        name: "museum",
        about: "Museums",
        filters: &[("tourism", "museum")],
    },
    Preset {
        name: "viewpoint",
        about: "Viewpoints",
        filters: &[("tourism", "viewpoint")],
    },
    Preset {
        name: "hotel",
        about: "Hotels",
        filters: &[("tourism", "hotel")],
    },
    Preset {
        name: "train_station",
        about: "Railway stations",
        filters: &[("railway", "station")],
    },
];

/// What the caller asks for. Give a `category` or a `key` plus `value`, and a
/// centre as `lat` plus `lon` or as a `place` the offline demo geocoder knows.
#[derive(Debug, Clone, Default)]
pub struct NearbyRequest {
    pub category: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub place: Option<String>,
    pub radius_m: Option<u32>,
    pub limit: Option<usize>,
    /// Skip a fresh cache entry and ask Overpass again.
    pub refresh: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NearbyHit {
    pub name: String,
    /// Which requested OSM tags this element matched, e.g. `shop=bicycle`.
    pub matched: Vec<String>,
    pub lat: f64,
    pub lon: f64,
    pub distance_m: u32,
    pub osm_type: String,
    pub osm_id: i64,
    pub osm_url: String,
    /// Opens the OSM editor on this element. Travelcraft itself never edits OSM.
    pub osm_edit_url: String,
    pub address: Option<String>,
    /// Raw `opening_hours` tag. Not evaluated: no "open now" claim is made.
    pub opening_hours: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    /// Useful tags this element lacks. A hint for OSM contributors.
    pub missing_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Center {
    pub lat: f64,
    pub lon: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NearbyResult {
    pub query: String,
    pub center: Center,
    pub radius_m: u32,
    pub count: usize,
    /// Hits found before `limit` was applied.
    pub total_found: usize,
    /// Overpass hit the fetch cap; nearer places may be missing. Shrink the radius.
    pub truncated: bool,
    /// `overpass` (live), `cache` (fresh) or `stale_cache` (live failed).
    pub source: &'static str,
    pub fetched_at: i64,
    pub stale: bool,
    pub warning: Option<String>,
    pub attribution: &'static str,
    pub hits: Vec<NearbyHit>,
}

/// Transport seam so tests and offline runs never need the network.
pub trait OverpassTransport {
    fn query(&self, ql: &str) -> Result<Value, TravelCraftError>;
}

/// Public Overpass instances tried in order. Any one of them can be slow, rate
/// limited or down at a given moment, so a failure moves on to the next.
const MIRRORS: &[&str] = &[
    "https://overpass.openstreetmap.fr/api/interpreter",
    "https://overpass-api.de/api/interpreter",
    "https://overpass.kumi.systems/api/interpreter",
    "https://overpass.private.coffee/api/interpreter",
];
const HTTP_TIMEOUT_SECS: u64 = 15;
/// Server-side query timeout, kept under the client timeout.
const QL_TIMEOUT_SECS: u32 = 12;

/// Live Overpass over HTTPS (blocking, keyless). `TRAVELCRAFT_OVERPASS_URL` pins a
/// single endpoint (your own instance, say) and disables the mirror list.
pub struct HttpOverpass {
    client: reqwest::blocking::Client,
    urls: Vec<String>,
}

impl HttpOverpass {
    pub fn new() -> Result<Self, TravelCraftError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .user_agent(
                "travelcraft-mcp/0.1 (+https://github.com/sandraschi/travelcraft; OSM Overpass)",
            )
            .build()
            .map_err(|e| TravelCraftError::Offline(format!("http client: {e}")))?;
        let urls = match std::env::var("TRAVELCRAFT_OVERPASS_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            Some(one) => vec![one],
            None => MIRRORS.iter().map(|s| (*s).to_string()).collect(),
        };
        Ok(Self { client, urls })
    }
}

fn host_of(url: &str) -> &str {
    url.split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or(url)
}

impl OverpassTransport for HttpOverpass {
    fn query(&self, ql: &str) -> Result<Value, TravelCraftError> {
        let mut failures: Vec<String> = Vec::new();
        for url in &self.urls {
            let host = host_of(url);
            match self.client.post(url).form(&[("data", ql)]).send() {
                Ok(resp) if resp.status().is_success() => {
                    return resp
                        .json::<Value>()
                        .map_err(|e| TravelCraftError::Protocol(format!("{host}: bad json: {e}")));
                }
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    // A 400 means our query is wrong; another mirror will not fix that.
                    if code == 400 {
                        return Err(TravelCraftError::Protocol(format!(
                            "{host}: rejected the query (400)"
                        )));
                    }
                    failures.push(format!("{host}: HTTP {code}"));
                }
                Err(e) if e.is_timeout() => failures.push(format!("{host}: timed out")),
                Err(e) if e.is_connect() => failures.push(format!("{host}: connection failed")),
                Err(e) => failures.push(format!("{host}: request failed ({e})")),
            }
        }
        Err(TravelCraftError::Offline(format!(
            "all Overpass endpoints failed: {}",
            failures.join("; ")
        )))
    }
}
struct Resolved {
    filters: Vec<(String, String)>,
    label: String,
    lat: f64,
    lon: f64,
    center_source: String,
    radius_m: u32,
    limit: usize,
}

/// Approximate centres of major cities, so "Wien" works without coordinates.
/// This is a deliberately tiny table, not a geocoder: anything else must come
/// with lat and lon.
const CITIES: &[(&str, &[&str], f64, f64)] = &[
    ("Wien", &["wien", "vienna"], 48.2082, 16.3738),
    ("Graz", &["graz"], 47.0707, 15.4395),
    ("Salzburg", &["salzburg"], 47.8095, 13.0550),
    ("Innsbruck", &["innsbruck"], 47.2692, 11.4041),
    ("Linz", &["linz"], 48.3069, 14.2858),
    ("Klagenfurt", &["klagenfurt"], 46.6247, 14.3053),
    ("Tokyo", &["tokyo", "tokio"], 35.6812, 139.7671),
    ("Kyoto", &["kyoto"], 35.0116, 135.7681),
    ("Osaka", &["osaka"], 34.6937, 135.5023),
    ("Nara", &["nara"], 34.6851, 135.8048),
];

fn lookup_city(place: &str) -> Option<(&'static str, f64, f64)> {
    let q = place.trim().to_lowercase();
    CITIES
        .iter()
        .find(|(_, aliases, _, _)| aliases.contains(&q.as_str()))
        .map(|(name, _, lat, lon)| (*name, *lat, *lon))
}
/// OSM key/value fence: ASCII letters, digits and `: _ . -`, 1 to 64 chars.
fn tag_part(label: &str, raw: &str) -> Result<String, TravelCraftError> {
    let s = raw.trim();
    let ok = !s.is_empty()
        && s.chars().count() <= 64
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '.' | '-'));
    if ok {
        Ok(s.to_string())
    } else {
        Err(TravelCraftError::Invalid(format!(
            "{label} must be 1 to 64 chars of letters, digits and : _ . -"
        )))
    }
}

fn resolve(req: &NearbyRequest) -> Result<Resolved, TravelCraftError> {
    let (filters, label) = match (&req.category, &req.key, &req.value) {
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            return Err(TravelCraftError::Invalid(
                "give either category or key plus value, not both".into(),
            ))
        }
        (Some(cat), None, None) => {
            let name = cat.trim().to_lowercase();
            let preset = PRESETS.iter().find(|p| p.name == name).ok_or_else(|| {
                let names: Vec<&str> = PRESETS.iter().map(|p| p.name).collect();
                TravelCraftError::Invalid(format!(
                    "unknown category {name:?}; use one of {} or pass key and value",
                    names.join(", ")
                ))
            })?;
            let f = preset
                .filters
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect();
            (f, preset.name.to_string())
        }
        (None, Some(k), Some(v)) => {
            let (k, v) = (tag_part("key", k)?, tag_part("value", v)?);
            let label = format!("{k}={v}");
            (vec![(k, v)], label)
        }
        _ => {
            return Err(TravelCraftError::Invalid(
                "give a category, or both key and value".into(),
            ))
        }
    };
    let (lat, lon, center_source) = match (req.lat, req.lon, &req.place) {
        (Some(lat), Some(lon), _) => {
            if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) {
                return Err(TravelCraftError::Invalid("lat out of range".into()));
            }
            if !lon.is_finite() || !(-180.0..=180.0).contains(&lon) {
                return Err(TravelCraftError::Invalid("lon out of range".into()));
            }
            (lat, lon, "arguments".to_string())
        }
        (Some(_), None, _) | (None, Some(_), _) => {
            return Err(TravelCraftError::Invalid(
                "give lat and lon together".into(),
            ))
        }
        (None, None, Some(place)) => {
            let (name, plat, plon) = lookup_city(place).ok_or_else(|| {
                TravelCraftError::Invalid(format!(
                    "unknown place {place:?}; the built-in table has only major AT and JP cities, pass lat and lon"
                ))
            })?;
            (
                plat,
                plon,
                format!("built-in city table: {name} (approximate centre)"),
            )
        }
        (None, None, None) => {
            return Err(TravelCraftError::Invalid(
                "give lat and lon, or a place the offline demo geocoder knows".into(),
            ))
        }
    };
    let radius_m = req.radius_m.unwrap_or(DEFAULT_RADIUS_M);
    if !(MIN_RADIUS_M..=MAX_RADIUS_M).contains(&radius_m) {
        return Err(TravelCraftError::Invalid(format!(
            "radius_m must be {MIN_RADIUS_M} to {MAX_RADIUS_M}"
        )));
    }
    let limit = req.limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(TravelCraftError::Invalid(format!(
            "limit must be 1 to {MAX_LIMIT}"
        )));
    }
    Ok(Resolved {
        filters,
        label,
        lat,
        lon,
        center_source,
        radius_m,
        limit,
    })
}

/// Overpass QL for the filters around a point. Inputs must already be fenced.
pub fn build_query(filters: &[(String, String)], lat: f64, lon: f64, radius_m: u32) -> String {
    let mut q = format!("[out:json][timeout:{QL_TIMEOUT_SECS}];(");
    for (k, v) in filters {
        q.push_str(&format!(
            "nwr[\"{k}\"=\"{v}\"](around:{radius_m},{lat:.6},{lon:.6});"
        ));
    }
    q.push_str(&format!(");out center tags {FETCH_CAP};"));
    q
}

/// Strip control characters and cap length. OSM text is untrusted.
fn clean(s: &str) -> String {
    let flat: String = s
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(MAX_STR_CHARS)
        .collect();
    flat.trim().to_string()
}

fn tag<'a>(tags: &'a Value, key: &str) -> Option<&'a str> {
    tags.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

struct Parsed {
    hits: Vec<NearbyHit>,
    remark: Option<String>,
    elements: usize,
}

/// Distil an Overpass JSON body into hits sorted by distance. Pure and offline.
fn parse_response(
    body: &Value,
    filters: &[(String, String)],
    lat: f64,
    lon: f64,
) -> Result<Parsed, TravelCraftError> {
    let elements = body
        .get("elements")
        .and_then(Value::as_array)
        .ok_or_else(|| TravelCraftError::Protocol("overpass: no elements array".into()))?;
    let remark = body
        .get("remark")
        .and_then(Value::as_str)
        .map(clean)
        .filter(|s| !s.is_empty());
    let origin = LonLat::new(lon, lat).map_err(|e| TravelCraftError::Gis(e.to_string()))?;
    let mut hits = Vec::new();
    for el in elements {
        let Some(osm_type) = el
            .get("type")
            .and_then(Value::as_str)
            .filter(|t| matches!(*t, "node" | "way" | "relation"))
        else {
            continue;
        };
        let Some(osm_id) = el.get("id").and_then(Value::as_i64) else {
            continue;
        };
        let Some(tags) = el.get("tags").filter(|t| t.is_object()) else {
            continue;
        };
        let (plat, plon) = match (
            el.get("lat").and_then(Value::as_f64),
            el.get("lon").and_then(Value::as_f64),
        ) {
            (Some(a), Some(o)) => (a, o),
            _ => {
                let c = el.get("center");
                match (
                    c.and_then(|c| c.get("lat")).and_then(Value::as_f64),
                    c.and_then(|c| c.get("lon")).and_then(Value::as_f64),
                ) {
                    (Some(a), Some(o)) => (a, o),
                    _ => continue,
                }
            }
        };
        if !(-90.0..=90.0).contains(&plat) || !(-180.0..=180.0).contains(&plon) {
            continue;
        }
        let name = tag(tags, "name")
            .or_else(|| tag(tags, "brand"))
            .map(clean)
            .unwrap_or_else(|| "(unnamed)".to_string());
        let matched = filters
            .iter()
            .filter(|(k, v)| tag(tags, k) == Some(v.as_str()))
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        let street = tag(tags, "addr:street").map(clean);
        let number = tag(tags, "addr:housenumber").map(clean);
        let town = [
            tag(tags, "addr:postcode").map(clean),
            tag(tags, "addr:city").map(clean),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
        let line1 = [street, number]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        let address = [line1, town]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        let address = if address.is_empty() {
            None
        } else {
            Some(address)
        };
        let opening_hours = tag(tags, "opening_hours").map(clean);
        let phone = tag(tags, "phone")
            .or_else(|| tag(tags, "contact:phone"))
            .map(clean);
        let website = tag(tags, "website")
            .or_else(|| tag(tags, "contact:website"))
            .map(clean);
        let mut missing_tags = Vec::new();
        if tag(tags, "name").is_none() {
            missing_tags.push("name".to_string());
        }
        if opening_hours.is_none() {
            missing_tags.push("opening_hours".to_string());
        }
        if phone.is_none() {
            missing_tags.push("phone".to_string());
        }
        if website.is_none() {
            missing_tags.push("website".to_string());
        }
        let point = LonLat::new(plon, plat).map_err(|e| TravelCraftError::Gis(e.to_string()))?;
        let dist = haversine_m(origin, point);
        hits.push(NearbyHit {
            name,
            matched,
            lat: plat,
            lon: plon,
            distance_m: dist.round().clamp(0.0, 1.0e9) as u32,
            osm_type: osm_type.to_string(),
            osm_id,
            osm_url: format!("https://www.openstreetmap.org/{osm_type}/{osm_id}"),
            osm_edit_url: format!("https://www.openstreetmap.org/edit?{osm_type}={osm_id}"),
            address,
            opening_hours,
            phone,
            website,
            missing_tags,
        });
    }
    hits.sort_by(|a, b| {
        a.distance_m
            .cmp(&b.distance_m)
            .then_with(|| a.osm_id.cmp(&b.osm_id))
    });
    Ok(Parsed {
        hits,
        remark,
        elements: elements.len(),
    })
}

#[derive(Serialize, Deserialize)]
struct CachePayload {
    hits: Vec<NearbyHit>,
    truncated: bool,
}

fn cache_key(r: &Resolved) -> String {
    let mut f: Vec<String> = r.filters.iter().map(|(k, v)| format!("{k}={v}")).collect();
    f.sort();
    format!(
        "v1|{}|{:.4}|{:.4}|{}",
        f.join("+"),
        r.lat,
        r.lon,
        r.radius_m
    )
}

impl TravelCraftEngine {
    /// Nearby places via live Overpass, with the SQLite cache described in the module docs.
    pub fn nearby(&self, req: &NearbyRequest) -> Result<NearbyResult, TravelCraftError> {
        // Fail on bad input before building a client or touching the network.
        resolve(req)?;
        self.nearby_with(req, &HttpOverpass::new()?)
    }

    /// Same as [`nearby`](Self::nearby) with an explicit transport (tests, mirrors).
    pub fn nearby_with(
        &self,
        req: &NearbyRequest,
        transport: &dyn OverpassTransport,
    ) -> Result<NearbyResult, TravelCraftError> {
        let r = resolve(req)?;
        let key = cache_key(&r);
        let cached = self.nearby_cache_get(&key)?;
        let now = chrono::Utc::now().timestamp();
        let build = |source: &'static str,
                     fetched_at: i64,
                     p: CachePayload,
                     stale: bool,
                     warning: Option<String>| {
            let total_found = p.hits.len();
            let hits: Vec<NearbyHit> = p.hits.into_iter().take(r.limit).collect();
            NearbyResult {
                query: r.label.clone(),
                center: Center {
                    lat: r.lat,
                    lon: r.lon,
                    source: r.center_source.clone(),
                },
                radius_m: r.radius_m,
                count: hits.len(),
                total_found,
                truncated: p.truncated,
                source,
                fetched_at,
                stale,
                warning,
                attribution: ATTRIBUTION,
                hits,
            }
        };
        if let Some((at, payload)) = &cached {
            if !req.refresh && now - at < CACHE_TTL_SECS {
                return Ok(build("cache", *at, clone_payload(payload), false, None));
            }
        }
        let ql = build_query(&r.filters, r.lat, r.lon, r.radius_m);
        let live = transport.query(&ql).and_then(|body| {
            let parsed = parse_response(&body, &r.filters, r.lat, r.lon)?;
            if parsed.hits.is_empty() {
                if let Some(remark) = &parsed.remark {
                    return Err(TravelCraftError::Protocol(format!(
                        "overpass remark: {remark}"
                    )));
                }
            }
            Ok(parsed)
        });
        match live {
            Ok(parsed) => {
                let payload = CachePayload {
                    truncated: parsed.elements >= FETCH_CAP,
                    hits: parsed.hits,
                };
                let warning = parsed.remark.map(|m| format!("overpass remark: {m}"));
                if warning.is_none() {
                    self.nearby_cache_put(&key, now, &payload)?;
                }
                Ok(build("overpass", now, payload, false, warning))
            }
            Err(e) => match cached {
                Some((at, payload)) => Ok(build(
                    "stale_cache",
                    at,
                    payload,
                    true,
                    Some(format!("live query failed, showing the last answer: {e}")),
                )),
                None => Err(e),
            },
        }
    }

    fn nearby_cache_get(&self, key: &str) -> Result<Option<(i64, CachePayload)>, TravelCraftError> {
        use rusqlite::OptionalExtension;
        let row: Option<(i64, String)> = self
            .lock()
            .query_row(
                "SELECT fetched_at, payload FROM nearby_cache WHERE cache_key = ?1",
                params![key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| TravelCraftError::Store(e.to_string()))?;
        match row {
            // An unreadable cache row is dropped, not trusted.
            Some((at, json)) => Ok(serde_json::from_str::<CachePayload>(&json)
                .ok()
                .map(|p| (at, p))),
            None => Ok(None),
        }
    }

    fn nearby_cache_put(
        &self,
        key: &str,
        at: i64,
        payload: &CachePayload,
    ) -> Result<(), TravelCraftError> {
        let json = serde_json::to_string(payload)?;
        let conn = self.lock();
        let db = |e: rusqlite::Error| TravelCraftError::Store(e.to_string());
        conn.execute(
            "INSERT OR REPLACE INTO nearby_cache (cache_key, fetched_at, payload) VALUES (?1, ?2, ?3)",
            params![key, at, json],
        )
        .map_err(db)?;
        conn.execute(
            "DELETE FROM nearby_cache WHERE fetched_at < ?1",
            params![at - CACHE_PRUNE_SECS],
        )
        .map_err(db)?;
        conn.execute(
            "DELETE FROM nearby_cache WHERE cache_key NOT IN
             (SELECT cache_key FROM nearby_cache ORDER BY fetched_at DESC LIMIT ?1)",
            params![CACHE_MAX_ROWS],
        )
        .map_err(db)?;
        Ok(())
    }
}

fn clone_payload(p: &CachePayload) -> CachePayload {
    CachePayload {
        hits: p.hits.clone(),
        truncated: p.truncated,
    }
}
