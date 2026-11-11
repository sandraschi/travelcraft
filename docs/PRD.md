# travelcraft PRD

## What it is

A local-first, offline travel primer for Austria and Japan in the Baedeker
spirit: places placed in time (chrono-waypoints), what they feel like (sensory
atmosphere), how to route between them, and what to know on the ground. Pure
Rust, ArtCraft pattern: one engine, three front ends (CLI, MCP stdio, egui app).

## Users

Sandra and her agents. Agents use the MCP tools to plan stays and answer "what
is this" questions. Sandra uses the egui app and the iPad dossier page.

## Today

- Editions: Japan (9 stops), Austria (8 stops), each with a knowledge tree.
- Planning: haversine route optimiser, stay itineraries (1, 2, 3, 5, 7, 14 days).
- SightLens: identify a sight from GPS plus a hint, log a photo to the durable
  journal (SQLite, WAL). Sight Chat: grounded offline answer, optional Ollama in the GUI.
- Surfaces: `travelcraft` (egui), `travelcraft-cli`, `travelcraft-mcp` (13 tools,
  see TOOLS.md), `travelcraft-cli serve` (iPad dossier page), `publish` to the
  fleet Intel Hub.

## Boundaries

- Not a GIS analyzer. Map maths and the demo geocoder come from giscraft-core.
- No live-tile SLA. Keyless OSM and Photon only.
- No account, no cloud dependency for the core path. Offline is the thesis.

- giscraft dependency: path dependency on giscraft-core for map maths (`geo::haversine_m`, `LonLat`) and the demo geocoder behind `travel_geocode` only. giscraft's Phase 0 (its `docs/PHASE0-foundations.md`, 2026-11-08, designed in another session and not merged) is additive and changes none of those, so nothing here needs to change. Travelcraft keeps its own Overpass client and cache on purpose: giscraft's overpass module is moving to attribute-preserving hits and a command registry. Revisit after that merges; `OverpassHit.props` could then replace the tag parsing in `nearby.rs`. This repo never edits giscraft.

## Gaps (honest)

- Catalogue is tiny; identify matches by hint and distance, not by image.
- Overpass mirror reliability is undiagnosed: from this machine overpass-api.de answers 502 or hangs, kumi.systems and private.coffee hang, while overpass.openstreetmap.fr and overpass.osm.ch answer in about a second. Cause unknown (rate limiting of this IP, routing, or the servers). Needs a proper look with timings per mirror over a day. Until then the French mirror goes first.
- POI search works only through `travel_nearby` (online, public Overpass mirrors that are sometimes down). Place names resolve only for ten AT and JP cities; everything else needs lat and lon. No opening-hours evaluation.
- Live transit (gtfs-mcp) and voice paths are GUI-side and need other services.
- Offline region pack format is not designed yet.
- egui app crate still uses unwrap and expect and is outside the deny-lints.

## Next (days, not weeks)

1. Grow the catalogue and add a catalogue loader instead of compiled-in data.
2. Bring the egui crate under the deny-lints.
3. NSIS current-user installer and an mcpb bundle, per the fleet standard.

## UI/UX Architecture & Menu Bar Standard

In accordance with [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md):
- **Pattern 2 (Viewport-Centric Craft Workstation):** Persistent Baedeker geographical map canvas and chrono-waypoint stage with offline knowledge tree. Top quick button row acts as accelerators into the `View` menu. To add a new regional guide or routing tool:
  1. Register a visibility toggle in the `View` or `Window` menu.
  2. Add an entry in the sidebar Edition / Sights toolbox.
  3. Wire the action directly into the universal `Command::dispatch` bus.
- **Top Menu Bar Hierarchy:**
  - `File`: New Itinerary (`Ctrl+N`), Open Journal (`Ctrl+O`), Open Recent ▸, Save Journal (`Ctrl+S`), Export Dossier / HTML (`Ctrl+E`), Close (`Ctrl+W`), Quit (`Ctrl+Q`).
  - `Edit`: Undo (`Ctrl+Z`), Redo (`Ctrl+Y`), Cut, Copy, Paste, Select All, Preferences (`Ctrl+,`).
  - `View`: Map Canvas / Viewport, Chrono-Waypoint Timeline, SightLens Inspector, Travel Journal, Sight Chat, Theme (`Dark`/`Light`).
  - `Insert`: Add Waypoint, Insert Photo Entry, Add Custom Sight, Import Region Pack.
  - `Modify`: Reorder Itinerary, Edit Waypoint Notes, Adjust Stay Duration, Tune Route Weights.
  - `Commands`: Optimize Route (Haversine TSP), Resolve Nearby Sights, Query Offline Sight Chat, Publish to Intel Hub.
  - `Window`: Reset Layout, Map Viewport, Sights Inspector, Dossier Drawer.
  - `Help` (Mandatory 5-Dialog Suite): `Command Palette...` (`Ctrl+K`), `Keyboard Shortcuts...` (`F1`), `Project Files & Docs...`, `Licenses & Notices...`, `About TravelCraft`.
- **Behavioral Mechanics:** Smooth `switch_on_hover` between top-level menus, zero-unsafe immediate-mode rendering, and headless `--smoke` parity.
