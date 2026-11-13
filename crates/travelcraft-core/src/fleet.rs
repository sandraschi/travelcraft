//! Local fleet companions travelcraft calls over HTTP.
//!
//! Offline planning never needs these. `travel_transit` is opt-in: short
//! timeouts, fixed paths on localhost, and when a companion is down the
//! answer carries the offline transit blob with `source: offline`.
//! Tests use [`FleetTransport`] and never open a socket.

use crate::TravelCraftError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use travelcraft_baedeker::transit_blob_for_sight;

/// One sibling repo and the HTTP base travelcraft probes.
pub struct Companion {
    pub id: &'static str,
    pub repo: &'static str,
    pub role: &'static str,
    pub default_base: &'static str,
    pub health_path: &'static str,
    pub env: &'static str,
}

/// Companions named by the Baedeker plan. gtfs and publictransport answer
/// transit questions. The others are status-only: their text is already
/// vendored, or another tab already calls them (speech, map tiles).
pub const COMPANIONS: &[Companion] = &[
    Companion {
        id: "gtfs",
        repo: "gtfs-mcp",
        role: "GTFS schedules, stop search, timetable departures",
        default_base: "http://127.0.0.1:10913",
        health_path: "/health",
        env: "TRAVELCRAFT_GTFS_URL",
    },
    Companion {
        id: "publictransport",
        repo: "publictransport-mcp",
        role: "Vienna live departures, disruptions, major stops",
        default_base: "http://127.0.0.1:11170",
        health_path: "/api/health",
        env: "TRAVELCRAFT_PT_URL",
    },
    Companion {
        id: "japanophile",
        repo: "japanophile-mcp",
        role: "Japan primer. Offline text is already vendored in the guide.",
        default_base: "http://127.0.0.1:11193",
        health_path: "/health",
        env: "TRAVELCRAFT_JAPANOPHILE_URL",
    },
    Companion {
        id: "vienna_life",
        repo: "vienna-life-assistant",
        role: "Vienna life tips. Offline text is already vendored in the guide.",
        default_base: "http://127.0.0.1:10922",
        health_path: "/health",
        env: "TRAVELCRAFT_VIENNA_LIFE_URL",
    },
    Companion {
        id: "speech",
        repo: "speech-mcp",
        role: "Voice out (TTS wav on :10909)",
        default_base: "http://127.0.0.1:10909",
        health_path: "/health",
        env: "TRAVELCRAFT_SPEECH_URL",
    },
    Companion {
        id: "giscraft",
        repo: "giscraft",
        role: "Slippy map tiles",
        default_base: "http://127.0.0.1:11254",
        health_path: "/api/health",
        env: "TRAVELCRAFT_GISCRAFT_URL",
    },
];

/// Bases used for one call. [`FleetBases::from_env`] honours overrides.
/// Tests use [`FleetBases::defaults`] so a developer env var cannot redirect them.
#[derive(Debug, Clone)]
pub struct FleetBases {
    pub gtfs: String,
    pub publictransport: String,
    pub japanophile: String,
    pub vienna_life: String,
    pub speech: String,
    pub giscraft: String,
}

impl FleetBases {
    pub fn defaults() -> Self {
        Self::build(false)
    }

    pub fn from_env() -> Self {
        Self::build(true)
    }

    fn build(use_env: bool) -> Self {
        Self {
            gtfs: base_of_id("gtfs", use_env),
            publictransport: base_of_id("publictransport", use_env),
            japanophile: base_of_id("japanophile", use_env),
            vienna_life: base_of_id("vienna_life", use_env),
            speech: base_of_id("speech", use_env),
            giscraft: base_of_id("giscraft", use_env),
        }
    }

    fn of(&self, id: &str) -> &str {
        match id {
            "gtfs" => &self.gtfs,
            "publictransport" => &self.publictransport,
            "japanophile" => &self.japanophile,
            "vienna_life" => &self.vienna_life,
            "speech" => &self.speech,
            "giscraft" => &self.giscraft,
            _ => "",
        }
    }
}

fn base_of_id(id: &str, use_env: bool) -> String {
    let Some(c) = COMPANIONS.iter().find(|c| c.id == id) else {
        return String::new();
    };
    base_of(c, use_env)
}

fn base_of(c: &Companion, use_env: bool) -> String {
    let raw = if use_env {
        std::env::var(c.env)
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| c.default_base.to_string())
    } else {
        c.default_base.to_string()
    };
    raw.trim().trim_end_matches('/').to_string()
}

/// What the caller asked for. Empty strings are treated as absent.
#[derive(Debug, Clone)]
pub struct TransitRequest {
    pub operation: String,
    pub query: Option<String>,
    pub stop_id: Option<String>,
    pub rbl: Option<String>,
    pub feed_id: Option<String>,
    pub limit: Option<usize>,
    pub sight_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionProbe {
    pub id: String,
    pub repo: String,
    pub role: String,
    pub url: String,
    pub up: bool,
    pub http_status: Option<u16>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopHit {
    pub id: String,
    pub name: String,
    pub code: String,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartureHit {
    pub line: String,
    pub destination: String,
    pub when: String,
    pub stop: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHit {
    pub title: String,
    pub lines: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitReport {
    pub success: bool,
    pub operation: String,
    pub online: bool,
    /// `gtfs`, `publictransport`, `fleet`, or `offline`.
    pub source: String,
    pub warning: Option<String>,
    pub offline_blob: Option<String>,
    pub companions: Vec<CompanionProbe>,
    pub stops: Vec<StopHit>,
    pub departures: Vec<DepartureHit>,
    pub alerts: Vec<AlertHit>,
}

impl TransitReport {
    /// Plain text for the desktop tab. The MCP tool returns the struct as JSON.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "operation={} online={} source={} success={}",
            self.operation, self.online, self.source, self.success
        ));
        if let Some(w) = &self.warning {
            lines.push(format!("warning: {w}"));
        }
        for c in &self.companions {
            let mark = if c.up { "up" } else { "down" };
            lines.push(format!(
                "{mark}  {} ({})  {}  {}",
                c.id, c.repo, c.url, c.note
            ));
        }
        for s in &self.stops {
            lines.push(format!(
                "stop  {}  id={}  code={}  {}",
                s.name, s.id, s.code, s.source
            ));
        }
        for d in &self.departures {
            lines.push(format!(
                "dep  {}  {}  {}  [{}]",
                d.line, d.when, d.destination, d.source
            ));
        }
        for a in &self.alerts {
            lines.push(format!("alert  {}  {}", a.title, a.lines));
        }
        if let Some(b) = &self.offline_blob {
            lines.push(b.clone());
        }
        lines.join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct HttpReply {
    pub status: u16,
    pub body: String,
}

/// Transport seam so tests never need the network.
pub trait FleetTransport {
    fn get(&self, url: &str) -> Result<HttpReply, TravelCraftError>;
}

/// Blocking HTTP to localhost companions. Connect fails fast when a server is down.
pub struct HttpFleet {
    client: reqwest::blocking::Client,
}

impl HttpFleet {
    pub fn new() -> Result<Self, TravelCraftError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(6))
            .connect_timeout(Duration::from_millis(800))
            .user_agent("travelcraft/0.2 (local fleet companions)")
            .build()
            .map_err(|e| TravelCraftError::Offline(format!("http client: {e}")))?;
        Ok(Self { client })
    }
}

impl FleetTransport for HttpFleet {
    fn get(&self, url: &str) -> Result<HttpReply, TravelCraftError> {
        match self.client.get(url).send() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp
                    .text()
                    .map_err(|e| TravelCraftError::Protocol(format!("read body: {e}")))?;
                Ok(HttpReply {
                    status,
                    body: cap_bytes(&body, 262_144),
                })
            }
            Err(e) if e.is_timeout() => Err(TravelCraftError::Offline(format!("timed out: {url}"))),
            Err(e) if e.is_connect() => Err(TravelCraftError::Offline(format!(
                "connection failed: {url}"
            ))),
            Err(e) => Err(TravelCraftError::Offline(format!("request failed: {e}"))),
        }
    }
}

/// Live call. Reads companion URLs from the environment.
pub fn transit(req: &TransitRequest) -> Result<TransitReport, TravelCraftError> {
    transit_with(req, &HttpFleet::new()?, &FleetBases::from_env())
}

/// Same as [`transit`] with an explicit transport and bases (tests).
pub fn transit_with(
    req: &TransitRequest,
    transport: &dyn FleetTransport,
    bases: &FleetBases,
) -> Result<TransitReport, TravelCraftError> {
    let op = req.operation.trim().to_ascii_lowercase();
    let sight = req.sight_id.as_deref().unwrap_or("");
    match op.as_str() {
        "status" | "probe" => Ok(status_report(transport, bases, sight)),
        "stops" | "search" => stops_report(transport, bases, req, sight),
        "departures" | "timetable" => departures_report(transport, bases, req, sight),
        "live" => live_report(transport, bases, req, sight),
        "disruptions" | "alerts" => disruptions_report(transport, bases, sight),
        _ => Err(TravelCraftError::Invalid(
            "operation must be status, stops, departures, live, or disruptions".to_string(),
        )),
    }
}

fn status_report(transport: &dyn FleetTransport, bases: &FleetBases, sight: &str) -> TransitReport {
    let companions = probe_all(transport, bases);
    let gtfs_up = companions.iter().any(|c| c.id == "gtfs" && c.up);
    let pt_up = companions.iter().any(|c| c.id == "publictransport" && c.up);
    TransitReport {
        success: true,
        operation: "status".to_string(),
        online: gtfs_up || pt_up,
        source: "fleet".to_string(),
        warning: None,
        offline_blob: Some(transit_blob_for_sight(sight)),
        companions,
        stops: Vec::new(),
        departures: Vec::new(),
        alerts: Vec::new(),
    }
}

fn probe_all(transport: &dyn FleetTransport, bases: &FleetBases) -> Vec<CompanionProbe> {
    COMPANIONS
        .iter()
        .map(|c| {
            let base = bases.of(c.id);
            let url = format!("{base}{}", c.health_path);
            match transport.get(&url) {
                Ok(r) if (200..300).contains(&r.status) => CompanionProbe {
                    id: c.id.to_string(),
                    repo: c.repo.to_string(),
                    role: c.role.to_string(),
                    url,
                    up: true,
                    http_status: Some(r.status),
                    note: "ok".to_string(),
                },
                Ok(r) => CompanionProbe {
                    id: c.id.to_string(),
                    repo: c.repo.to_string(),
                    role: c.role.to_string(),
                    url,
                    up: false,
                    http_status: Some(r.status),
                    note: format!("HTTP {}", r.status),
                },
                Err(e) => CompanionProbe {
                    id: c.id.to_string(),
                    repo: c.repo.to_string(),
                    role: c.role.to_string(),
                    url,
                    up: false,
                    http_status: None,
                    note: e.to_string(),
                },
            }
        })
        .collect()
}

fn stops_report(
    transport: &dyn FleetTransport,
    bases: &FleetBases,
    req: &TransitRequest,
    sight: &str,
) -> Result<TransitReport, TravelCraftError> {
    let query = required_text(req.query.as_deref(), "query")?;
    let feed = ident_or(req.feed_id.as_deref(), "default", "feed_id")?;
    let limit = bound_limit(req.limit)?;
    let url = format!(
        "{}/v1/stops/search?feed_id={}&query={}&limit={limit}",
        bases.gtfs,
        feed,
        encode(&query)
    );
    let reply = match transport.get(&url) {
        Ok(r) => r,
        Err(e) => return Ok(offline("stops", sight, e.to_string())),
    };
    if !(200..300).contains(&reply.status) {
        return Ok(offline(
            "stops",
            sight,
            format!("gtfs-mcp HTTP {}", reply.status),
        ));
    }
    let value: Value = serde_json::from_str(&reply.body)
        .map_err(|e| TravelCraftError::Protocol(format!("gtfs stops: bad json: {e}")))?;
    let mut stops = Vec::new();
    for item in as_rows(&value, "stops") {
        if stops.len() >= limit {
            break;
        }
        stops.push(StopHit {
            id: json_str(item, "stop_id"),
            name: json_str(item, "stop_name"),
            code: json_str(item, "stop_code"),
            lat: json_f64(item, "stop_lat"),
            lon: json_f64(item, "stop_lon"),
            source: "gtfs".to_string(),
        });
    }
    Ok(TransitReport {
        success: true,
        operation: "stops".to_string(),
        online: true,
        source: "gtfs".to_string(),
        warning: if stops.is_empty() {
            Some("gtfs-mcp returned no stops for that query".to_string())
        } else {
            None
        },
        offline_blob: None,
        companions: Vec::new(),
        stops,
        departures: Vec::new(),
        alerts: Vec::new(),
    })
}

fn departures_report(
    transport: &dyn FleetTransport,
    bases: &FleetBases,
    req: &TransitRequest,
    sight: &str,
) -> Result<TransitReport, TravelCraftError> {
    let stop_id = required_ident(req.stop_id.as_deref(), "stop_id")?;
    let feed = ident_or(req.feed_id.as_deref(), "default", "feed_id")?;
    let limit = bound_limit(req.limit)?;
    let url = format!(
        "{}/v1/stops/{stop_id}/departures?feed_id={feed}&limit={limit}",
        bases.gtfs
    );
    let reply = match transport.get(&url) {
        Ok(r) => r,
        Err(e) => return Ok(offline("departures", sight, e.to_string())),
    };
    if !(200..300).contains(&reply.status) {
        return Ok(offline(
            "departures",
            sight,
            format!("gtfs-mcp HTTP {}", reply.status),
        ));
    }
    let value: Value = serde_json::from_str(&reply.body)
        .map_err(|e| TravelCraftError::Protocol(format!("gtfs departures: bad json: {e}")))?;
    Ok(TransitReport {
        success: true,
        operation: "departures".to_string(),
        online: true,
        source: "gtfs".to_string(),
        warning: None,
        offline_blob: None,
        companions: Vec::new(),
        stops: Vec::new(),
        departures: parse_gtfs_departures(&value, &stop_id, limit),
        alerts: Vec::new(),
    })
}

fn live_report(
    transport: &dyn FleetTransport,
    bases: &FleetBases,
    req: &TransitRequest,
    sight: &str,
) -> Result<TransitReport, TravelCraftError> {
    let limit = bound_limit(req.limit)?;
    let rbl = match req.rbl.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(_) => Some(required_ident(req.rbl.as_deref(), "rbl")?),
        None => None,
    };
    let (stops, chosen) = if let Some(code) = rbl {
        (
            vec![StopHit {
                id: code.clone(),
                name: String::new(),
                code: code.clone(),
                lat: None,
                lon: None,
                source: "publictransport".to_string(),
            }],
            Some(code),
        )
    } else {
        let query = required_text(req.query.as_deref(), "query")?;
        let url = format!("{}/api/stops/major?city=vienna", bases.publictransport);
        let reply = match transport.get(&url) {
            Ok(r) => r,
            Err(e) => return Ok(offline("live", sight, e.to_string())),
        };
        if !(200..300).contains(&reply.status) {
            return Ok(offline(
                "live",
                sight,
                format!("publictransport-mcp HTTP {}", reply.status),
            ));
        }
        let value: Value = serde_json::from_str(&reply.body).map_err(|e| {
            TravelCraftError::Protocol(format!("publictransport stops: bad json: {e}"))
        })?;
        let needle = query.to_lowercase();
        let mut stops = Vec::new();
        for item in as_rows(&value, "stops") {
            let name = json_str(item, "name");
            if !name.to_lowercase().contains(&needle) {
                continue;
            }
            if stops.len() >= limit {
                break;
            }
            stops.push(StopHit {
                id: json_str(item, "rbl"),
                name,
                code: json_str(item, "rbl"),
                lat: json_f64(item, "lat"),
                lon: json_f64(item, "lon"),
                source: "publictransport".to_string(),
            });
        }
        let chosen = stops
            .first()
            .map(|s| s.code.clone())
            .filter(|s| !s.is_empty());
        (stops, chosen)
    };

    let Some(code) = chosen else {
        return Ok(TransitReport {
            success: true,
            operation: "live".to_string(),
            online: true,
            source: "publictransport".to_string(),
            warning: Some(
                "no Vienna major stop matched. Try a hub name (Stephansplatz) or pass rbl."
                    .to_string(),
            ),
            offline_blob: Some(transit_blob_for_sight(sight)),
            companions: Vec::new(),
            stops,
            departures: Vec::new(),
            alerts: Vec::new(),
        });
    };

    let url = format!(
        "{}/api/departures?rbl={}&city=vienna",
        bases.publictransport,
        encode(&code)
    );
    let reply = match transport.get(&url) {
        Ok(r) => r,
        Err(e) => return Ok(offline("live", sight, e.to_string())),
    };
    if !(200..300).contains(&reply.status) {
        return Ok(offline(
            "live",
            sight,
            format!("publictransport-mcp departures HTTP {}", reply.status),
        ));
    }
    let value: Value = serde_json::from_str(&reply.body).map_err(|e| {
        TravelCraftError::Protocol(format!("publictransport departures: bad json: {e}"))
    })?;
    let warning = value.get("error").and_then(Value::as_str).map(clean_text);
    Ok(TransitReport {
        success: warning.is_none(),
        operation: "live".to_string(),
        online: true,
        source: "publictransport".to_string(),
        warning,
        offline_blob: None,
        companions: Vec::new(),
        stops,
        departures: parse_live_departures(&value, limit),
        alerts: Vec::new(),
    })
}

fn disruptions_report(
    transport: &dyn FleetTransport,
    bases: &FleetBases,
    sight: &str,
) -> Result<TransitReport, TravelCraftError> {
    let url = format!("{}/api/disruptions", bases.publictransport);
    let reply = match transport.get(&url) {
        Ok(r) => r,
        Err(e) => return Ok(offline("disruptions", sight, e.to_string())),
    };
    if !(200..300).contains(&reply.status) {
        return Ok(offline(
            "disruptions",
            sight,
            format!("publictransport-mcp HTTP {}", reply.status),
        ));
    }
    let value: Value = serde_json::from_str(&reply.body).map_err(|e| {
        TravelCraftError::Protocol(format!("publictransport disruptions: bad json: {e}"))
    })?;
    let mut alerts = Vec::new();
    for item in as_rows(&value, "alerts") {
        if alerts.len() >= 12 {
            break;
        }
        let lines = item
            .get("lines")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(clean_text)
                    .filter(|s| !s.is_empty())
                    .take(8)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        alerts.push(AlertHit {
            title: json_str(item, "title"),
            lines,
            detail: json_str(item, "description"),
        });
    }
    Ok(TransitReport {
        success: true,
        operation: "disruptions".to_string(),
        online: true,
        source: "publictransport".to_string(),
        warning: None,
        offline_blob: None,
        companions: Vec::new(),
        stops: Vec::new(),
        departures: Vec::new(),
        alerts,
    })
}

fn offline(operation: &str, sight: &str, warning: String) -> TransitReport {
    TransitReport {
        success: true,
        operation: operation.to_string(),
        online: false,
        source: "offline".to_string(),
        warning: Some(warning),
        offline_blob: Some(transit_blob_for_sight(sight)),
        companions: Vec::new(),
        stops: Vec::new(),
        departures: Vec::new(),
        alerts: Vec::new(),
    }
}

fn parse_gtfs_departures(value: &Value, stop_id: &str, limit: usize) -> Vec<DepartureHit> {
    let mut out = Vec::new();
    for item in as_rows(value, "departures") {
        if out.len() >= limit {
            break;
        }
        out.push(DepartureHit {
            line: json_str(item, "route_short_name"),
            destination: json_str(item, "trip_headsign"),
            when: json_str(item, "departure_time"),
            stop: {
                let id = json_str(item, "stop_id");
                if id.is_empty() {
                    stop_id.to_string()
                } else {
                    id
                }
            },
            source: "gtfs".to_string(),
        });
    }
    out
}

fn parse_live_departures(value: &Value, limit: usize) -> Vec<DepartureHit> {
    let mut out = Vec::new();
    for item in as_rows(value, "departures") {
        if out.len() >= limit {
            break;
        }
        let when = {
            let real = json_str(item, "time_real");
            if real.is_empty() {
                let countdown = json_str(item, "countdown");
                if countdown.is_empty() {
                    json_str(item, "time_planned")
                } else {
                    format!("{countdown} min")
                }
            } else {
                real
            }
        };
        out.push(DepartureHit {
            line: json_str(item, "line"),
            destination: json_str(item, "destination"),
            when,
            stop: json_str(item, "stop"),
            source: "publictransport".to_string(),
        });
    }
    out
}

fn as_rows<'a>(value: &'a Value, key: &str) -> Vec<&'a Value> {
    let node = if value.is_array() {
        value
    } else {
        match value.get(key) {
            Some(v) => v,
            None => return Vec::new(),
        }
    };
    node.as_array()
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

fn json_str(v: &Value, key: &str) -> String {
    match v.get(key) {
        Some(Value::String(s)) => clean_text(s),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn json_f64(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64).filter(|n| n.is_finite())
}

fn clean_text(raw: &str) -> String {
    let s: String = raw.chars().filter(|c| !c.is_control()).take(240).collect();
    s.trim().to_string()
}

fn required_text(raw: Option<&str>, label: &str) -> Result<String, TravelCraftError> {
    let s = clean_text(raw.unwrap_or(""));
    let n = s.chars().count();
    if n == 0 || n > 80 {
        return Err(TravelCraftError::Invalid(format!(
            "{label} must be 1 to 80 characters"
        )));
    }
    Ok(s)
}

fn required_ident(raw: Option<&str>, label: &str) -> Result<String, TravelCraftError> {
    let s = raw.unwrap_or("").trim();
    if s.is_empty() {
        return Err(TravelCraftError::Invalid(format!(
            "missing or empty string argument: {label}"
        )));
    }
    ident(s, label)
}

fn ident_or(raw: Option<&str>, default: &str, label: &str) -> Result<String, TravelCraftError> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => ident(s, label),
        None => Ok(default.to_string()),
    }
}

fn ident(s: &str, label: &str) -> Result<String, TravelCraftError> {
    if s.chars().count() > 64 {
        return Err(TravelCraftError::Invalid(format!(
            "{label} must be at most 64 characters"
        )));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(TravelCraftError::Invalid(format!(
            "{label} may only contain letters, digits, underscore, hyphen, and dot"
        )));
    }
    Ok(s.to_string())
}

fn bound_limit(n: Option<usize>) -> Result<usize, TravelCraftError> {
    match n {
        None => Ok(8),
        Some(n) if (1..=20).contains(&n) => Ok(n),
        Some(_) => Err(TravelCraftError::Invalid(
            "limit must be 1 to 20".to_string(),
        )),
    }
}

fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                out.push(char::from(b));
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn cap_bytes(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scripted {
        hits: Vec<(&'static str, Result<HttpReply, String>)>,
    }

    impl FleetTransport for Scripted {
        fn get(&self, url: &str) -> Result<HttpReply, TravelCraftError> {
            for (needle, result) in &self.hits {
                if url.contains(needle) {
                    return match result {
                        Ok(r) => Ok(r.clone()),
                        Err(e) => Err(TravelCraftError::Offline(e.clone())),
                    };
                }
            }
            Err(TravelCraftError::Offline(format!("unscripted: {url}")))
        }
    }

    fn ok_body(status: u16, body: &'static str) -> Result<HttpReply, String> {
        Ok(HttpReply {
            status,
            body: body.to_string(),
        })
    }

    #[test]
    fn status_marks_up_and_down() -> Result<(), TravelCraftError> {
        let transport = Scripted {
            hits: vec![
                ("10913/health", ok_body(200, "{\"success\":true}")),
                ("11170/api/health", ok_body(200, "{\"status\":\"ok\"}")),
                ("11193/health", Err("connection failed".to_string())),
                ("10922/health", ok_body(503, "no")),
                ("10909/health", ok_body(200, "ok")),
                ("11254/api/health", ok_body(200, "ok")),
            ],
        };
        let report = transit_with(
            &TransitRequest {
                operation: "status".to_string(),
                query: None,
                stop_id: None,
                rbl: None,
                feed_id: None,
                limit: None,
                sight_id: Some("at_stephansdom".to_string()),
            },
            &transport,
            &FleetBases::defaults(),
        )?;
        assert!(report.online);
        assert_eq!(report.source, "fleet");
        assert!(report.companions.iter().any(|c| c.id == "gtfs" && c.up));
        assert!(report
            .companions
            .iter()
            .any(|c| c.id == "japanophile" && !c.up));
        assert!(report
            .companions
            .iter()
            .any(|c| c.id == "vienna_life" && !c.up));
        let blob = report.offline_blob.unwrap_or_default();
        assert!(blob.contains("Stephans"));
        Ok(())
    }

    #[test]
    fn stops_parse_gtfs_array() -> Result<(), TravelCraftError> {
        let transport = Scripted {
            hits: vec![(
                "/v1/stops/search",
                ok_body(
                    200,
                    r#"[{"stop_id":"123","stop_name":"Stephansplatz","stop_code":"2903","stop_lat":48.2,"stop_lon":16.3}]"#,
                ),
            )],
        };
        let report = transit_with(
            &TransitRequest {
                operation: "stops".to_string(),
                query: Some("Stephansplatz".to_string()),
                stop_id: None,
                rbl: None,
                feed_id: Some("default".to_string()),
                limit: Some(5),
                sight_id: None,
            },
            &transport,
            &FleetBases::defaults(),
        )?;
        assert!(report.online);
        assert_eq!(report.stops.len(), 1);
        assert_eq!(report.stops.first().map(|s| s.id.as_str()), Some("123"));
        assert_eq!(
            report.stops.first().map(|s| s.name.as_str()),
            Some("Stephansplatz")
        );
        Ok(())
    }

    #[test]
    fn departures_parse_and_reject_path_injection() -> Result<(), TravelCraftError> {
        let bad = transit_with(
            &TransitRequest {
                operation: "departures".to_string(),
                query: None,
                stop_id: Some("../health".to_string()),
                rbl: None,
                feed_id: None,
                limit: None,
                sight_id: None,
            },
            &Scripted { hits: vec![] },
            &FleetBases::defaults(),
        );
        assert!(bad.is_err());

        let transport = Scripted {
            hits: vec![(
                "/departures",
                ok_body(
                    200,
                    r#"{"success":true,"departures":[{"route_short_name":"U1","trip_headsign":"Leopoldau","departure_time":"2026-11-13T08:01:00","stop_id":"123"}]}"#,
                ),
            )],
        };
        let report = transit_with(
            &TransitRequest {
                operation: "departures".to_string(),
                query: None,
                stop_id: Some("123".to_string()),
                rbl: None,
                feed_id: None,
                limit: Some(5),
                sight_id: None,
            },
            &transport,
            &FleetBases::defaults(),
        )?;
        assert_eq!(report.departures.len(), 1);
        assert_eq!(
            report.departures.first().map(|d| d.line.as_str()),
            Some("U1")
        );
        assert_eq!(report.source, "gtfs");
        Ok(())
    }

    #[test]
    fn live_matches_major_stop_then_departures() -> Result<(), TravelCraftError> {
        let transport = Scripted {
            hits: vec![
                (
                    "/api/stops/major",
                    ok_body(
                        200,
                        r#"{"stops":[{"name":"Stephansplatz","rbl":"2903","lat":48.2,"lon":16.3},{"name":"Karlsplatz","rbl":"1490"}]}"#,
                    ),
                ),
                (
                    "/api/departures",
                    ok_body(
                        200,
                        r#"{"rbl":"2903","departures":[{"line":"U1","destination":"Leopoldau","countdown":3,"stop":"Stephansplatz"}]}"#,
                    ),
                ),
            ],
        };
        let report = transit_with(
            &TransitRequest {
                operation: "live".to_string(),
                query: Some("stephans".to_string()),
                stop_id: None,
                rbl: None,
                feed_id: None,
                limit: Some(5),
                sight_id: Some("at_stephansdom".to_string()),
            },
            &transport,
            &FleetBases::defaults(),
        )?;
        assert!(report.online);
        assert_eq!(report.source, "publictransport");
        assert_eq!(report.stops.first().map(|s| s.code.as_str()), Some("2903"));
        assert_eq!(
            report.departures.first().map(|d| d.line.as_str()),
            Some("U1")
        );
        assert_eq!(
            report.departures.first().map(|d| d.when.as_str()),
            Some("3 min")
        );
        Ok(())
    }

    #[test]
    fn down_companion_returns_offline_blob() -> Result<(), TravelCraftError> {
        let transport = Scripted {
            hits: vec![("/v1/stops/search", Err("connection failed".to_string()))],
        };
        let report = transit_with(
            &TransitRequest {
                operation: "search".to_string(),
                query: Some("Praterstern".to_string()),
                stop_id: None,
                rbl: None,
                feed_id: None,
                limit: None,
                sight_id: Some("at_stephansdom".to_string()),
            },
            &transport,
            &FleetBases::defaults(),
        )?;
        assert!(!report.online);
        assert_eq!(report.source, "offline");
        assert!(report.warning.unwrap_or_default().contains("connection"));
        assert!(report.offline_blob.unwrap_or_default().contains("TRANSIT"));
        Ok(())
    }

    #[test]
    fn disruptions_cap_and_strip_controls() -> Result<(), TravelCraftError> {
        let body =
            r#"{"alerts":[{"title":"U4  delay\u0001","lines":["U4","U6"],"description":"works"}]}"#;
        let transport = Scripted {
            hits: vec![("/api/disruptions", ok_body(200, body))],
        };
        let report = transit_with(
            &TransitRequest {
                operation: "alerts".to_string(),
                query: None,
                stop_id: None,
                rbl: None,
                feed_id: None,
                limit: None,
                sight_id: None,
            },
            &transport,
            &FleetBases::defaults(),
        )?;
        assert_eq!(report.alerts.len(), 1);
        let title = report
            .alerts
            .first()
            .map(|a| a.title.as_str())
            .unwrap_or("");
        assert!(!title.chars().any(|c| c.is_control()));
        assert_eq!(
            report.alerts.first().map(|a| a.lines.as_str()),
            Some("U4, U6")
        );
        Ok(())
    }

    #[test]
    fn unknown_operation_and_bad_limit_are_invalid() {
        let bases = FleetBases::defaults();
        let transport = Scripted { hits: vec![] };
        let bad_op = transit_with(
            &TransitRequest {
                operation: "shutdown".to_string(),
                query: None,
                stop_id: None,
                rbl: None,
                feed_id: None,
                limit: None,
                sight_id: None,
            },
            &transport,
            &bases,
        );
        assert!(matches!(bad_op, Err(TravelCraftError::Invalid(_))));
        let bad_limit = transit_with(
            &TransitRequest {
                operation: "stops".to_string(),
                query: Some("Wien".to_string()),
                stop_id: None,
                rbl: None,
                feed_id: None,
                limit: Some(99),
                sight_id: None,
            },
            &transport,
            &bases,
        );
        assert!(matches!(bad_limit, Err(TravelCraftError::Invalid(_))));
    }
}
