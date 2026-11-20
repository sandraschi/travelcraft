//! Vienna and Tokyo extents used by the street map and the trip board.
//! Places come from the catalogue. Nothing is invented.

use giscraft_core::geo::{haversine_m, LonLat};
use travelcraft_baedeker::BaedekerCatalog;

/// A catalogue place that falls inside one of the two city maps.
#[derive(Debug, Clone, PartialEq)]
pub struct CityPlace {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub note: String,
    /// `stop` for a waypoint, `work` for an artwork that is not already
    /// standing on a stop (same building).
    pub kind: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TravelCity {
    Vienna,
    Tokyo,
}

impl TravelCity {
    pub fn id(self) -> &'static str {
        match self {
            Self::Vienna => "vienna",
            Self::Tokyo => "tokyo",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vienna => "Vienna",
            Self::Tokyo => "Tokyo",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "vienna" | "wien" => Some(Self::Vienna),
            "tokyo" => Some(Self::Tokyo),
            _ => None,
        }
    }

    /// `(lng, lat, slippy zoom)` that frames the whole city.
    pub fn home(self) -> (f64, f64, f32) {
        match self {
            // Innere Stadt, wide enough to include the Ring and the Danube canal.
            Self::Vienna => (16.3730, 48.2080, 12.0),
            // Central wards, wide enough for Nihonbashi and Ueno together.
            Self::Tokyo => (139.7600, 35.6900, 11.6),
        }
    }

    /// City proper, not the rest of the country edition.
    /// Vienna stops short of the Wachau and Hallstatt.
    /// Tokyo stops short of Hakone and Nikko.
    pub fn contains(self, lat: f64, lon: f64) -> bool {
        match self {
            Self::Vienna => (48.12..=48.33).contains(&lat) && (16.18..=16.58).contains(&lon),
            Self::Tokyo => (35.53..=35.82).contains(&lat) && (139.60..=139.92).contains(&lon),
        }
    }

    pub fn from_coords(lat: f64, lon: f64) -> Option<Self> {
        if Self::Vienna.contains(lat, lon) {
            Some(Self::Vienna)
        } else if Self::Tokyo.contains(lat, lon) {
            Some(Self::Tokyo)
        } else {
            None
        }
    }
}

/// Waypoints inside the city, then artworks that are not within 120 m of a
/// waypoint (those share the museum pin already).
pub fn places_in_city(city: TravelCity) -> Vec<CityPlace> {
    let mut stops = Vec::new();
    for (_code, waypoint) in BaedekerCatalog::all_waypoints() {
        if !city.contains(waypoint.lat, waypoint.lon) {
            continue;
        }
        stops.push(CityPlace {
            id: waypoint.id,
            name: waypoint.name,
            lat: waypoint.lat,
            lon: waypoint.lon,
            note: clip_chars(&waypoint.visitor_info, 500),
            kind: "stop",
        });
    }
    let mut places = stops.clone();
    for art in BaedekerCatalog::artworks() {
        if !city.contains(art.lat, art.lon) {
            continue;
        }
        let on_a_stop = stops
            .iter()
            .any(|stop| near(stop.lat, stop.lon, art.lat, art.lon, 120.0));
        if on_a_stop {
            continue;
        }
        places.push(CityPlace {
            id: art.id,
            name: format!("{} ({})", art.title, art.museum),
            lat: art.lat,
            lon: art.lon,
            note: clip_chars(&art.look_for, 500),
            kind: "work",
        });
    }
    places
}

fn near(lat_a: f64, lon_a: f64, lat_b: f64, lon_b: f64, meters: f64) -> bool {
    let Ok(a) = LonLat::new(lon_a, lat_a) else {
        return false;
    };
    let Ok(b) = LonLat::new(lon_b, lat_b) else {
        return false;
    };
    haversine_m(a, b) < meters
}

fn clip_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let keep = max.saturating_sub(3);
    let mut out: String = text.chars().take(keep).collect();
    out.push_str("...");
    out
}

/// Extract map pins (waypoints, distinct artworks, and geo-located spots) from any Cartridge.
pub fn places_in_cartridge(
    cartridge: &travelcraft_baedeker::cartridge::Cartridge,
) -> Vec<CityPlace> {
    let mut places: Vec<CityPlace> = cartridge
        .waypoints
        .iter()
        .map(|wp| CityPlace {
            id: wp.id.clone(),
            name: wp.name.clone(),
            lat: wp.lat,
            lon: wp.lon,
            note: clip_chars(&wp.visitor_info, 500),
            kind: "stop",
        })
        .collect();
    let stops = places.clone();
    for art in &cartridge.artworks {
        let on_a_stop = stops
            .iter()
            .any(|s| near(s.lat, s.lon, art.lat, art.lon, 120.0));
        if on_a_stop {
            continue;
        }
        places.push(CityPlace {
            id: art.id.clone(),
            name: format!("{} ({})", art.title, art.museum),
            lat: art.lat,
            lon: art.lon,
            note: clip_chars(&art.look_for, 500),
            kind: "work",
        });
    }
    for spot in &cartridge.spots {
        if let (Some(lat), Some(lon)) = (spot.lat, spot.lon) {
            places.push(CityPlace {
                id: spot.id.clone(),
                name: spot.name.clone(),
                lat,
                lon,
                note: clip_chars(&spot.note, 500),
                kind: "spot",
            });
        }
    }
    places
}
