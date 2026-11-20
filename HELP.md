# TravelCraft (Baedeker 2030) — Help & Field Manual

> **Sovereign Chrono-Cartography Studio & Local-First Travel Guide**
> Pure Rust (`#![forbid(unsafe_code)]`) · Zero Panics · Local SQLite WAL · Offline-First + Opt-In Fleet Companions

---

## 1. What TravelCraft Is & Does

**TravelCraft** reinvents the classic 19th-century Baedeker travel guide for the 2030s as a **1-Engine, 3-Front-End** ArtCraft workstation:

- **Engine (`travelcraft-core` + `travelcraft-baedeker`)**: Compiles curated country editions (`AT` Austria with 8 chrono-waypoints from Hallstatt 800 BC to Habsburg Vienna 1365+; `JP` Japan with 9 chrono-waypoints from Tokaido 1603 to Neo-Tokyo), sensory atmosphere profiles (acoustic, architectural, olfactory, light), geodesic Haversine TSP routing (`giscraft-core`), 1/2/3/5/7/14-day stay itineraries, SightLens sight/artwork identification, Trip Kanban boards, and durable SQLite WAL storage (`%LOCALAPPDATA%\travelcraft\travelcraft.db` or `$TRAVELCRAFT_DATA`).
- **Desktop Workstation (`travelcraft`)**: Native `eframe`/`egui` high-contrast studio with interactive City Maps (Vienna, Tokyo), Chrono-Map, Trip Kanban, Stay Itineraries, SightLens, Sight Chat, Travel Journal, Fleet Transit, and the `Ctrl+J` AI Chat Dock.
- **CLI (`travelcraft-cli`)**: Headless JSON-emitting command-line interface for scripting, itinerary generation, Overpass POI queries, SightLens capture, and mobile iPad dossier serving.
- **MCP Server (`travelcraft-mcp`)**: MCP `2024-11-05` line-delimited JSON-RPC `stdio` server exposing **15 tools** and **2 bundled skill resources** (`skill://travelcraft-advisor/SKILL.md`, `skill://travelcraft-workflow/SKILL.md`) for agentic IDEs.

---

## 2. Get Started (5-Step First-Run Tour)

1. **Launch with Demo State**: Start `travelcraft.exe` (or `cargo run -p travelcraft-app`). The workstation opens on the high-contrast City Map / Chrono-Map viewport with Austria (`AT`) and Japan (`JP`) editions preloaded.
2. **Select an Edition & Inspect Chrono-Waypoints**: Choose `AT` or `JP` in the left inspector, click a waypoint (e.g. *Stephansdom* or *Nihonbashi*), and review its historical era, coordinates, and sensory atmosphere profile.
3. **Plan a Route or Stay Itinerary**: Run `Commands -> Optimize Route (Haversine TSP)` (or open the **Route Planner** / **Itineraries** tab) to compute chrono-ordered legs, distance, travel hours, and carbon estimate for 1, 2, 3, 5, 7, or 14 days. Use **Trip Kanban** to stage stops by city.
4. **Identify a Sight & Log a Journal Entry**: Open **SightLens**, enter GPS coordinates + hint (e.g. `48.2085, 16.3738` + `stephansdom`, or `48.2039, 16.3619` + `caravaggio`), and click **Identify**. Press `Ctrl+J` to open the **AI Chat Dock** for grounded Q&A, and press `Ctrl+S` to persist a reflection to the SQLite Travel Journal.
5. **Export or Serve**: Press `Ctrl+E` (`File -> Export Dossier / HTML`) to generate a standalone Baedeker HTML dossier, or run `travelcraft-cli serve 14130` to browse a mobile-friendly dossier from an iPad.

---

## 3. Functions & Core Modules

| Module / Capability | What It Does |
| :--- | :--- |
| **Chrono-GIS & City Maps** | WGS84 vector projection, mouse-wheel zoom & pan, high-contrast Vienna & Tokyo city maps, `giscraft-core` geodesic Haversine distances, and `qgiscraft` (`:11254`) Z16/Z8 slippy-tile math. |
| **Stay Itineraries & Trip Kanban** | Curated 1/2/3/5/7/14-day Baedeker day-plans per edition (`AT`, `JP`), plus a SQLite-backed Trip Kanban board seeded with Vienna and Tokyo places. |
| **SightLens & Artwork Dossiers** | Offline deterministic matching by GPS + hint for landmarks and museum masterpieces (Stephansdom, KHM Caravaggio *Rosenkranzmadonna*, Klimt *The Kiss*, Bruegel, Hokusai, plus edition waypoints). |
| **Sight Chat & `Ctrl+J` AI Chat Dock** | Grounded Q&A combining the sight info blob, `qgiscraft` map blob, and offline/live transit blob. Supports Ollama (`:11434`), LM Studio (`:1234`), Gemini, OpenAI-compatible, and Anthropic, with an automatic `OFFLINE BLOB` extractive fallback and Windows SAPI / `speech-mcp` (`:10909`) voice output. |
| **Route Optimization** | Chrono-ordered Haversine TSP route planner returning leg distances, estimated hours, carbon footprint, and covered Z10 map tiles. |
| **Nearby POIs (`travel_nearby`)** | Opt-in online OpenStreetMap Overpass queries across 18 curated categories (`bicycle_repair`, `pharmacy`, `toilets`, `drinking_water`, `cafe`, `museum`, etc.) or fenced `key`/`value` tags, cached 24 hours in SQLite with stale-cache fallback. |
| **Offline Packs & Vendored Sources** | Dynamic Region Pack JSON import (`Insert -> Import Region Pack...`) and vendored offline knowledge trees from `japanophile-mcp` and `vienna-life-assistant`. |
| **Fleet Transit (`travel_transit`)** | Opt-in localhost companion queries to `gtfs-mcp` (`:10913`) for stops/timetables and `publictransport-mcp` (`:11170`) for Vienna OGD live departures and disruptions, falling back cleanly to `source: "offline"`. |

---

## 4. Usage (GUI, CLI & MCP)

### Desktop GUI (`travelcraft`)
- **8-Menu Bookend Bar**: `File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`.
- **AI Chat Dock (`Ctrl+J`)**: Toggle the right-side assistant dock (or click `Chat (Ctrl+J)` in the top bar). Read-only tools execute automatically; mutating tools (`travel_journal_add`, `travel_sightlens_capture`) prompt for confirmation.
- **Command Palette (`Ctrl+K` / `Ctrl+Shift+P`)**: Fuzzy-search every command, view, and Help Center tab.
- **Headless Smoke Verification**: `travelcraft.exe --smoke` verifies core engine, routing, itineraries, SightLens, SightChat, region packs, and Kanban without opening a window.

### Command Line (`travelcraft-cli`)
```powershell
travelcraft-cli countries
travelcraft-cli inspect JP
travelcraft-cli plan JP
travelcraft-cli itinerary AT 3
travelcraft-cli identify 48.2085 16.3738 stephansdom
travelcraft-cli capture immich-export.jpg immich-exif 48.2085 16.3738 stephansdom
travelcraft-cli nearby bicycle_repair --place Wien --radius 1500
travelcraft-cli journal 10
travelcraft-cli chat --about caravaggio "Who painted the Rosenkranzmadonna and where is it?"
travelcraft-cli transit status
travelcraft-cli serve 14130
```

### MCP Server (`travelcraft-mcp`, `stdio` JSON-RPC `2024-11-05`)
- **Read-Only Tools**: `travel_status`, `travel_countries`, `travel_inspect`, `travel_knowledge_tree`, `travel_plan_route`, `travel_itinerary`, `travel_identify_sight`, `travel_sight_chat`, `travel_journal_list`, `travel_geocode`, `travel_nearby` (online opt-in), `travel_transit` (localhost opt-in), `travelcraft_skill_read`.
- **Mutating Tools**: `travel_journal_add`, `travel_sightlens_capture`.
- **MCP Resources**: `resources/list` and `resources/read` serve `skill://travelcraft-advisor/SKILL.md` and `skill://travelcraft-workflow/SKILL.md`.

---

## 5. Tips & Tricks

- **Fast Keyboard Flow**: Use `Ctrl+K` to jump to any tab or command, `Ctrl+J` to toggle the AI Chat Dock, `Ctrl+F` to focus the selected waypoint on the map, `F1` / `Ctrl+/` for the shortcut cheat sheet, and `Ctrl+E` to export an HTML dossier.
- **Non-Destructive Exploration**: Run `travel_transit` with `operation: "status"` before `live` or `stops` to see which local companions are up, and test `travel_sight_chat` before logging permanent entries with `travel_sightlens_capture`.
- **Avoid Overpass Truncation**: `travel_nearby` caps raw Overpass fetches at 300 elements (`truncated: true`). In dense city centers like Vienna or Tokyo, shrink `radius_m` (e.g. `500`–`1000` m) for complete nearest-first results.
- **Isolate Expedition Databases**: Set `TRAVELCRAFT_DATA` to a trip-specific directory to keep separate SQLite journal, Kanban, and Overpass cache files per journey.
- **Pin an Overpass Mirror**: Set `TRAVELCRAFT_OVERPASS_URL` if a public Overpass instance is rate-limiting you.

---

## 6. Common Pitfalls & Safety

- **Timezone & DST Shifts**: Austria (`Europe/Vienna`, CET/CEST) observes Daylight Saving Time while Japan (`Asia/Tokyo`, JST) does not. Journal timestamps are stored in RFC 3339, whereas offline transit notes and raw OSM `opening_hours` strings reflect local wall-clock time and are **never evaluated** by TravelCraft—always verify seasonal museum/temple hours locally.
- **Stale Transit Schedules**: When `gtfs-mcp` (`:10913`) or `publictransport-mcp` (`:11170`) is offline, `travel_transit` succeeds with `source: "offline"` and returns the preloaded transit blob. Treat offline transit blobs as orientation baselines, not live departure boards.
- **Coordinate & Bounding-Box Errors**: TravelCraft uses `(lat, lon)` WGS84 decimal degrees (e.g. Vienna `48.2085, 16.3738`). Swapping `lat` and `lon`—a common GeoJSON `(lon, lat)` habit—places points in the wrong hemisphere. Note also that **an iPhone hotspot shares internet only, not GPS**: export the original Immich file (preserving EXIF GPS) or enter coordinates manually.
- **Offline Map & Overpass Cache Limits**: `travel_nearby` caches responses for 24 hours; if live mirrors fail, it returns the prior cached payload with `stale: true` and a warning. Overpass text is treated as untrusted data (control characters stripped, fields capped at 200 chars), and `travel_geocode` is only a small offline demo geocoder—pass explicit `lat`/`lon` or a major `place` (`Wien`, `Graz`, `Tokyo`, `Kyoto`) for real POI searches.
- **Database Durability Gate**: SQLite runs in `WAL` mode with `synchronous = FULL`. If the database cannot be opened, `travelcraft-mcp` exits loudly rather than silently falling back to an ephemeral in-memory database.

---

## 7. Planned Extensions

TravelCraft is actively honest about what is **not built yet**:

1. **Additional Country Editions**: Expand `travelcraft-baedeker` beyond Austria (`AT`) and Japan (`JP`) with new chrono-waypoint and sensory atmosphere editions.
2. **Local `opening_hours` Evaluator**: Parse and evaluate raw OSM `opening_hours` tags locally so "is it open now?" works offline.
3. **Expanded Offline SightLens Catalogue**: Grow deterministic GPS + hint coverage for regional museums and architectural landmarks (visual image-model classification remains an explicit non-goal).
4. **Deeper `giscraft` Vector & Terrain Integration**: Richer offline vector tile styling and elevation profiles paired with `giscraft`.

---

## 8. Other Crafts (Ecosystem & Boundaries)

| Craft / Companion | Default Port / Crate | Role & Handoff Boundary |
| :--- | :--- | :--- |
| **GISCraft (`giscraft-core` / `qgiscraft`)** | Crate + `:11254` | Provides WGS84 tile math, geodesic Haversine calculations, and slippy map tiles. Full GIS analysis and OpenStreetMap editing belong in GISCraft, not TravelCraft. |
| **GTFS MCP (`gtfs-mcp`)** | `:10913` | Serves GTFS static transit stops and timetables for `travel_transit` (`stops`, `departures`). |
| **PublicTransport MCP (`publictransport-mcp`)** | `:11170` | Serves live Vienna OGD departures and service disruptions for `travel_transit` (`live`, `disruptions`). |
| **Japanophile MCP (`japanophile-mcp`)** | `:11193` | Companion status probe + vendored Japan travel primer (`travel.html`). |
| **Vienna Life Assistant (`vienna-life-assistant`)** | `:10922` | Companion status probe + vendored Vienna coffeehouse, Beisl, and museum knowledge. |
| **Speech MCP (`speech-mcp`)** | `:10909` | Optional TTS `.wav` synthesis and FunASR voice transcription sidecar (`:10914`) for Sight Chat. |
