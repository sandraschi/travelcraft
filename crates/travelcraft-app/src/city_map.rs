//! Street maps of Vienna and Tokyo.
//! Tiles are keyless OpenStreetMap, cached on disk by giscraft-core.
//! The country chrono sketch is a different view.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use giscraft_core::tiles::{self, Tile};
use travelcraft_core::cities::{places_in_city, CityPlace, TravelCity};

const PAPER: egui::Color32 = egui::Color32::from_rgb(232, 226, 214);
const INK: egui::Color32 = egui::Color32::from_rgb(12, 16, 28);
const GOLD: egui::Color32 = egui::Color32::from_rgb(180, 120, 20);

#[derive(Clone, Copy)]
struct Camera {
    lng: f64,
    lat: f64,
    zoom: f32,
}

pub struct CityCanvas {
    pub city: TravelCity,
    vienna: Camera,
    tokyo: Camera,
    textures: HashMap<String, egui::TextureHandle>,
    failed: HashSet<String>,
    fetching: Option<PendingTile>,
    tiles_blocked: bool,
    pub tile_note: String,
    selected_id: Option<String>,
}

struct PendingTile {
    key: String,
    rx: Receiver<Result<Vec<u8>, String>>,
}

pub struct MapPick {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

impl CityCanvas {
    pub fn for_country(code: &str) -> Self {
        let city = if code.eq_ignore_ascii_case("AT") {
            TravelCity::Vienna
        } else {
            TravelCity::Tokyo
        };
        Self::new(city)
    }

    pub fn new(city: TravelCity) -> Self {
        let vienna_home = TravelCity::Vienna.home();
        let tokyo_home = TravelCity::Tokyo.home();
        Self {
            city,
            vienna: Camera {
                lng: vienna_home.0,
                lat: vienna_home.1,
                zoom: vienna_home.2,
            },
            tokyo: Camera {
                lng: tokyo_home.0,
                lat: tokyo_home.1,
                zoom: tokyo_home.2,
            },
            textures: HashMap::new(),
            failed: HashSet::new(),
            fetching: None,
            tiles_blocked: false,
            tile_note: String::new(),
            selected_id: None,
        }
    }

    pub fn set_city(&mut self, city: TravelCity) {
        self.city = city;
    }

    pub fn go_home(&mut self) {
        let (lng, lat, zoom) = self.city.home();
        let cam = self.camera_mut();
        cam.lng = lng;
        cam.lat = lat;
        cam.zoom = zoom;
    }

    /// Center a city map on a coordinate. Returns false when the point is
    /// outside both cities (the caller can open the country sketch).
    pub fn show_place(&mut self, lat: f64, lon: f64) -> bool {
        let Some(city) = TravelCity::from_coords(lat, lon) else {
            return false;
        };
        self.city = city;
        let cam = self.camera_mut();
        cam.lat = lat;
        cam.lng = lon;
        cam.zoom = 15.0;
        true
    }

    fn camera(&self) -> Camera {
        match self.city {
            TravelCity::Vienna => self.vienna,
            TravelCity::Tokyo => self.tokyo,
        }
    }

    fn camera_mut(&mut self) -> &mut Camera {
        match self.city {
            TravelCity::Vienna => &mut self.vienna,
            TravelCity::Tokyo => &mut self.tokyo,
        }
    }
}

pub fn show(ui: &mut egui::Ui, canvas: &mut CityCanvas) -> Option<MapPick> {
    ui.horizontal_wrapped(|ui| {
        ui.heading("Street map");
        ui.separator();
        if ui
            .selectable_label(canvas.city == TravelCity::Vienna, "Vienna")
            .clicked()
        {
            canvas.set_city(TravelCity::Vienna);
        }
        if ui
            .selectable_label(canvas.city == TravelCity::Tokyo, "Tokyo")
            .clicked()
        {
            canvas.set_city(TravelCity::Tokyo);
        }
        if ui.button("Whole city").clicked() {
            canvas.go_home();
        }
        if ui.button("Zoom -").clicked() {
            let cam = canvas.camera_mut();
            cam.zoom = (cam.zoom - 0.5).clamp(10.0, 17.0);
        }
        if ui.button("Zoom +").clicked() {
            let cam = canvas.camera_mut();
            cam.zoom = (cam.zoom + 0.5).clamp(10.0, 17.0);
        }
        let cam = canvas.camera();
        ui.label(format!("{}   z {:.1}", canvas.city.label(), cam.zoom));
        if ui.button("Retry tiles").clicked() {
            canvas.failed.clear();
            canvas.fetching = None;
            canvas.tiles_blocked = false;
            canvas.tile_note.clear();
        }
    });
    ui.label(
        egui::RichText::new(
            "OpenStreetMap streets. Drag to pan, wheel to zoom. Gold dots are catalogue stops. Names show up close, or under the cursor.",
        )
        .size(14.0),
    );
    if !canvas.tile_note.is_empty() {
        ui.colored_label(egui::Color32::from_rgb(180, 40, 40), &canvas.tile_note);
    }

    let height = (ui.available_height() - 28.0).max(360.0);
    let (response, painter) = ui.allocate_painter(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click_and_drag(),
    );
    let rect = response.rect;
    painter.rect_filled(rect, 0.0, PAPER);

    let mut cam = canvas.camera();
    let (scroll, zoom_factor, hover) = ui.input(|input| {
        (
            input.smooth_scroll_delta.y,
            input.zoom_delta(),
            input.pointer.hover_pos(),
        )
    });
    if response.hovered() {
        let anchor = hover.unwrap_or(rect.center());
        let target = if zoom_factor != 1.0 {
            Some(cam.zoom + zoom_factor.log2().clamp(-2.0, 2.0))
        } else if scroll != 0.0 {
            Some(cam.zoom + scroll * 0.004)
        } else {
            None
        };
        if let Some(target) = target {
            let center = rect.center();
            let (glng, glat) = tiles::unproject(
                anchor.x, anchor.y, cam.lng, cam.lat, cam.zoom, center.x, center.y,
            );
            cam.zoom = target.clamp(10.0, 17.0);
            let (lng, lat) =
                tiles::recenter(glng, glat, anchor.x, anchor.y, center.x, center.y, cam.zoom);
            cam.lng = lng;
            cam.lat = lat;
        }
    }
    if response.dragged() {
        let delta = response.drag_delta();
        let center = rect.center();
        let (lng, lat) = tiles::unproject(
            center.x - delta.x,
            center.y - delta.y,
            cam.lng,
            cam.lat,
            cam.zoom,
            center.x,
            center.y,
        );
        cam.lng = lng;
        cam.lat = lat;
    }

    paint_tiles(ui, canvas, &painter, rect, cam);

    let places = places_in_city(canvas.city);
    let mut pick = None;
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            pick = nearest_pin(&places, pos, rect, cam, 16.0);
            canvas.selected_id = pick.as_ref().map(|p| p.id.clone());
        }
    }
    let hover_id = hover.and_then(|pos| nearest_pin(&places, pos, rect, cam, 14.0).map(|p| p.id));
    for place in &places {
        draw_pin(
            &painter,
            rect,
            cam,
            place,
            canvas.selected_id.as_deref() == Some(place.id.as_str()),
            hover_id.as_deref() == Some(place.id.as_str()),
        );
    }

    let credit = "(c) OpenStreetMap contributors";
    painter.text(
        rect.left_bottom() + egui::vec2(8.0, -6.0),
        egui::Align2::LEFT_BOTTOM,
        credit,
        egui::FontId::proportional(12.0),
        INK,
    );

    match canvas.city {
        TravelCity::Vienna => canvas.vienna = cam,
        TravelCity::Tokyo => canvas.tokyo = cam,
    }
    pick
}

fn paint_tiles(
    ui: &mut egui::Ui,
    canvas: &mut CityCanvas,
    painter: &egui::Painter,
    rect: egui::Rect,
    cam: Camera,
) {
    poll_fetch(ui, canvas);
    let template = tiles::tile_url_template();
    let tile_z = (cam.zoom.floor() as i64).clamp(0, 19) as u8;
    let center = rect.center();
    let (west, north) = tiles::unproject(
        rect.min.x, rect.min.y, cam.lng, cam.lat, cam.zoom, center.x, center.y,
    );
    let (east, south) = tiles::unproject(
        rect.max.x, rect.max.y, cam.lng, cam.lat, cam.zoom, center.x, center.y,
    );
    let list = match tiles::bbox_to_tiles(west, south, east, north, tile_z) {
        Ok(list) => list,
        Err(err) => {
            canvas.tile_note = format!("Tiles: {err}");
            return;
        }
    };
    let mut started = false;
    for tile in list {
        let key = format!(
            "{}:{}:{}:{}",
            tiles::template_key(&template),
            tile.z,
            tile.x,
            tile.y
        );
        if !canvas.textures.contains_key(&key) {
            if let Some(png) = read_cached_tile(tile, &template) {
                store_png(ui, canvas, &key, tile, &png);
            }
        }
        if canvas.textures.contains_key(&key) {
            if let Some(handle) = canvas.textures.get(&key) {
                paint_one(painter, rect, cam, tile, handle);
            }
            continue;
        }
        if canvas.failed.contains(&key) || canvas.tiles_blocked || started {
            continue;
        }
        if canvas.fetching.is_some() {
            continue;
        }
        start_fetch(canvas, tile, &key, &template);
        started = true;
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(80));
    }
}

fn poll_fetch(ui: &mut egui::Ui, canvas: &mut CityCanvas) {
    let Some(pending) = canvas.fetching.take() else {
        return;
    };
    match pending.rx.try_recv() {
        Ok(Ok(png)) => {
            store_png(ui, canvas, &pending.key, tile_from_key(&pending.key), &png);
            ui.ctx().request_repaint();
        }
        Ok(Err(err)) => {
            canvas.failed.insert(pending.key);
            canvas.tiles_blocked = true;
            canvas.tile_note = format!(
                "Street tiles did not load ({err}). Pins still work. Retry tiles once the network is up. Cached tiles are reused offline."
            );
        }
        Err(TryRecvError::Empty) => {
            canvas.fetching = Some(pending);
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(80));
        }
        Err(TryRecvError::Disconnected) => {
            canvas.tiles_blocked = true;
            canvas.tile_note = "Street tile fetch stopped before a reply.".to_string();
        }
    }
}

fn start_fetch(canvas: &mut CityCanvas, tile: Tile, key: &str, template: &str) {
    let (tx, rx) = mpsc::channel();
    let template = template.to_string();
    std::thread::spawn(move || {
        let result = tiles::fetch_tile_png_as(tile, &template).map_err(|err| err.to_string());
        let _ = tx.send(result);
    });
    canvas.fetching = Some(PendingTile {
        key: key.to_string(),
        rx,
    });
}

fn store_png(ui: &mut egui::Ui, canvas: &mut CityCanvas, key: &str, tile: Tile, png: &[u8]) {
    match tiles::decode_png_rgba(png) {
        Ok((w, h, rgba)) => {
            let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
            let handle = ui.ctx().load_texture(
                format!("travel-tile-{}-{}-{}", tile.z, tile.x, tile.y),
                image,
                egui::TextureOptions::LINEAR,
            );
            canvas.textures.insert(key.to_string(), handle);
            if canvas.tile_note.starts_with("Street tiles did not load") {
                canvas.tile_note.clear();
            }
        }
        Err(err) => {
            canvas.failed.insert(key.to_string());
            canvas.tile_note = format!("A cached tile could not be read ({err}).");
        }
    }
}

/// Same cache path giscraft-core writes. A miss just means "not downloaded yet".
fn read_cached_tile(tile: Tile, template: &str) -> Option<Vec<u8>> {
    let mut path = std::env::temp_dir();
    path.push("giscraft-tiles");
    path.push(tiles::template_key(template));
    path.push(format!("{}_{}_{}.png", tile.z, tile.x, tile.y));
    let bytes = std::fs::read(path).ok()?;
    if bytes.is_empty() {
        None
    } else {
        Some(bytes)
    }
}

fn tile_from_key(key: &str) -> Tile {
    let mut parts = key.rsplit(':');
    let y = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let x = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let z = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    Tile { z, x, y }
}

fn paint_one(
    painter: &egui::Painter,
    rect: egui::Rect,
    cam: Camera,
    tile: Tile,
    handle: &egui::TextureHandle,
) {
    let (min_lng, min_lat, max_lng, max_lat) = tiles::tile_bounds(tile);
    let center = rect.center();
    let (x0, y0) = tiles::project(
        min_lng, max_lat, cam.lng, cam.lat, cam.zoom, center.x, center.y,
    );
    let (x1, y1) = tiles::project(
        max_lng, min_lat, cam.lng, cam.lat, cam.zoom, center.x, center.y,
    );
    painter.image(
        handle.id(),
        egui::Rect::from_two_pos(egui::pos2(x0, y0), egui::pos2(x1, y1)),
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
}

fn nearest_pin(
    places: &[CityPlace],
    pos: egui::Pos2,
    rect: egui::Rect,
    cam: Camera,
    radius: f32,
) -> Option<MapPick> {
    let center = rect.center();
    let mut best: Option<(f32, MapPick)> = None;
    for place in places {
        let (x, y) = tiles::project(
            place.lon, place.lat, cam.lng, cam.lat, cam.zoom, center.x, center.y,
        );
        let pin = egui::pos2(x, y);
        if !rect.contains(pin) {
            continue;
        }
        let distance = (pin - pos).length();
        if distance > radius {
            continue;
        }
        let closer = best.as_ref().is_none_or(|(d, _)| distance < *d);
        if closer {
            best = Some((
                distance,
                MapPick {
                    id: place.id.clone(),
                    name: place.name.clone(),
                    lat: place.lat,
                    lon: place.lon,
                },
            ));
        }
    }
    best.map(|(_, pick)| pick)
}

fn draw_pin(
    painter: &egui::Painter,
    rect: egui::Rect,
    cam: Camera,
    place: &CityPlace,
    selected: bool,
    hovered: bool,
) {
    let center = rect.center();
    let (x, y) = tiles::project(
        place.lon, place.lat, cam.lng, cam.lat, cam.zoom, center.x, center.y,
    );
    let pin = egui::pos2(x, y);
    if !rect.contains(pin) {
        return;
    }
    let radius = if selected { 8.0 } else { 5.5 };
    painter.circle_filled(pin, radius + 2.0, egui::Color32::WHITE);
    painter.circle_filled(pin, radius, GOLD);
    painter.circle_stroke(pin, radius, egui::Stroke::new(1.5, INK));
    if selected || hovered || cam.zoom >= 14.0 {
        let font = egui::FontId::proportional(13.0);
        let galley = painter.layout_no_wrap(place.name.clone(), font, INK);
        let origin = pin + egui::vec2(8.0, -12.0 - galley.size().y);
        let chip = egui::Rect::from_min_size(origin, galley.size() + egui::vec2(8.0, 4.0));
        painter.rect_filled(
            chip,
            3.0,
            egui::Color32::from_rgba_unmultiplied(255, 252, 245, 230),
        );
        painter.galley(chip.min + egui::vec2(4.0, 2.0), galley, INK);
    }
}
