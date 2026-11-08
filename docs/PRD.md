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
- Surfaces: `travelcraft` (egui), `travelcraft-cli`, `travelcraft-mcp` (12 tools,
  see TOOLS.md), `travelcraft-cli serve` (iPad dossier page), `publish` to the
  fleet Intel Hub.

## Boundaries

- Not a GIS analyzer. Map maths and the demo geocoder come from giscraft-core.
- No live-tile SLA. Keyless OSM and Photon only.
- No account, no cloud dependency for the core path. Offline is the thesis.

## Gaps (honest)

- Catalogue is tiny; identify matches by hint and distance, not by image.
- No POI search ("bicycle repair shops in Vienna"): `travel_geocode` is a demo geocoder. Next candidate: `travel_nearby` over OSM Overpass via giscraft-core, online and opt-in.
- Live transit (gtfs-mcp) and voice paths are GUI-side and need other services.
- Offline region pack format is not designed yet.
- egui app crate still uses unwrap and expect and is outside the deny-lints.

## Next (days, not weeks)

1. `travel_nearby` (Overpass, opt-in, labelled online).
2. Grow the catalogue and add a catalogue loader instead of compiled-in data.
3. Bring the egui crate under the deny-lints.
4. NSIS current-user installer and an mcpb bundle, per the fleet standard.