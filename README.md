# TravelCraft (Baedeker 2030)

**TravelCraft** is an ArtCraft-tier chrono-cartography studio and local-first travel guide platform that reinvents the classic 19th-century Baedeker guide for the 2030s.

Features:
- **Chrono-Cartographic Layers**: Tokaido 1603 vs Neo-Tokyo; Hallstatt 800 BC vs Habsburg Vienna 1365.
- **Sensory Atmosphere Profiles**: Acoustic, architectural, olfactory, light records per waypoint.
- **Local Storage**: Offline SQLite journal + GPS-tagged photos via `DatabaseCraft`.
- **Stay-Length Itineraries**: 1/2/3/5/7/14-day Baedeker day-plans per edition (Itineraries tab, `travelcraft-cli itinerary AT 3`).
- **SightLens Camera**: Point laptop at Stephansdom and click Identify, or load an Immich export. Offline GPS + hint matching (Stephansdom, KHM Caravaggio Rosenkranzmadonna, Klimt Kiss, Bruegel Tower) renders a beautiful dossier and logs it with GPS source.
- **Sight Chat (text + voice)**: After identifying the Caravaggio, ask anything. Grounded in info blob + qgiscraft map blob + GTFS transit blob; online via Ollama `:11434` when reachable, else OFFLINE BLOB extractive answer. Voice out via Windows SAPI Speak or speech-mcp `:10909` TTS wav.
- **Maps + Transit blobs**: qgiscraft slippy tiles/scale/URLs (`:11254`) + preloaded Wiener Linien/JR snapshot; live departures via gtfs-mcp `:10913`.
- **High-Contrast Map**: Mouse-wheel zoom + drag pan, large type, ink/paper palette.
- **FastMCP Protocol**: Port `14129`.

Vendored offline sources (see `crates/travelcraft-baedeker/src/lib.rs SOURCES`):
- `japanophile-mcp/assets/knowledge/japan/travel.html` - JR Pass, konbini, budget, etiquette, regions.
- `vienna-life-assistant` `vienna_tips` (coffee/music/museum), `restaurants_data` Beisl rule, `culture_data` (KHM, Belvedere, Stephansplatz).

---

## Who uses this, and why not just open Google Maps

Travelcraft is an offline, agent-native travel primer. The point is not a better map. It is that an agent in your IDE (Claude Desktop, Claude Code, Cursor, Zed, Antigravity, anything that speaks MCP) can plan, explain and keep a journal for a trip without a browser, an account, an API key or a network on the core path.

| Who | Why not Google Maps | Typical prompt | Works today |
|---|---|---|---|
| Developer or power user with an agentic IDE | Stays in the tool, scriptable, no account, nothing leaves the machine | "Plan 3 days in Austria and put the stops in a table" | Yes, Austria and Japan only |
| Traveller with bad signal (tunnels, Tyrol, flights) | Core data is compiled in, no network needed | "What is the story of Stephansdom? I have no signal" | Yes, small catalogue |
| Culture and history traveller | Places are placed in time (chrono-waypoints), not just on a map | "What am I looking at at 48.2039, 16.3619?" | Yes, a handful of sights |
| Japan first-timer | Curated primer (JR Pass, konbini, etiquette) instead of ten browser tabs | "Plan a Tokaido route through Japan" | Yes, some text is German |
| Private journal keeper | Journal is a local SQLite file, survives crashes, never uploaded | "Log: Cafe Berg, rain, Melange, wet wool" | Yes |
| Builder of craft apps | Small reference for the ArtCraft pattern (engine, CLI, MCP, egui) | read `docs/PRD.md` and `AGENTS.md` | Yes |

### Prompts that work today

| You say | Tool the agent calls | You get |
|---|---|---|
| "Which country editions are there?" | `travel_countries` | Austria and Japan editions |
| "Plan 3 days in Austria" | `travel_itinerary` (AT, 3) | Day-by-day plan (1, 2, 3, 5, 7 or 14 days) |
| "Best route through Japan, oldest sight first" | `travel_plan_route` (JP) | Chrono-ordered legs, distance, hours, carbon estimate |
| "What am I looking at near Stephansplatz?" | `travel_identify_sight` | Sight dossier from the offline catalogue |
| "How do I get to the Caravaggio in the KHM?" | `travel_sight_chat` | Grounded answer plus map and transit snapshot (not live departures) |
| "I photographed the Klimt Kiss, log it" | `travel_sightlens_capture` | Identification plus a durable journal entry |
| "Log: Cafe Berg, rain, Melange" | `travel_journal_add` | Saved entry id |
| "Show my last 10 journal entries" | `travel_journal_list` | Entries, newest first |
| "Coordinates for Wien" | `travel_geocode` | Demo geocoder hit (a handful of places only) |
| "Vienna bicycle repair shops" | `travel_nearby` (bicycle_repair, Wien) | Nearest shops and repair stations from OpenStreetMap with address, phone, raw opening hours, OSM link. **Online**, cached 24 h |
| "Public toilets or drinking water near me" | `travel_nearby` (toilets, drinking_water, lat/lon) | Same, from OSM |
| "Which of these lack opening hours in OSM?" | `travel_nearby` `missing_tags` plus `osm_edit_url` | A to-do list for a mapper; travelcraft itself never edits OSM |

### Not built yet (do not expect these)

| You say | Why it does not work today | What it would take |
|---|---|---|
| "Is it open now?", prices, booking | `travel_nearby` returns the raw `opening_hours` tag but nothing evaluates it, and there is no price or booking data | An opening-hours evaluator; booking is out of scope |
| "Another country" | Only Austria and Japan editions exist | New editions in `travelcraft-baedeker` |
| "Which painting is in this photo?" | Identification uses GPS plus a hint, not the image | An image model; not planned |
| Contributing back to OpenStreetMap | Travelcraft reads OSM tiles, it does not edit OSM | A mapper workflow belongs in giscraft, not here |

Network use is limited to `travel_nearby` (and the GUI map tiles). Overpass data comes from public instances that are sometimes slow or down; travelcraft tries several mirrors, caches answers for 24 h and says so when it falls back to an old answer. Coverage and opening hours are only as good as OpenStreetMap.

Journal, chat history and settings live in `%LOCALAPPDATA%\travelcraft\travelcraft.db` (override with `TRAVELCRAFT_DATA`).
## Workspace Structure

1. `crates/travelcraft-baedeker`: Editions (`JapanCraft` 9 stops, `AustriaCraft` 8 stops), stay itineraries, SightLens identify DB (landmarks + KHM/Belvedere artworks).
2. `crates/travelcraft-core`: Geodesic routing, itineraries, SightLens pipeline, durable SQLite journal (GPS columns), chat history and settings.
3. `crates/travelcraft-mcp`: MCP 2024-11-05 stdio server (13 tools, see `docs/TOOLS.md`). No port; hosts start the exe.
4. `crates/travelcraft-app`: Native GUI (`eframe`/`egui`) + CLI (`travelcraft-cli`).

---

## Verification & Usage

```powershell
# Run test suite
cargo test --workspace

# Headless smoke test (JP route + AT itinerary + Stephansdom + Caravaggio)
./target/debug/travelcraft.exe --smoke

# CLI
./target/debug/travelcraft-cli.exe plan JP
./target/debug/travelcraft-cli.exe itinerary AT 3
./target/debug/travelcraft-cli.exe identify 48.2085 16.3738 stephansdom
./target/debug/travelcraft-cli.exe identify 48.2039 16.3619 caravaggio
./target/debug/travelcraft-cli.exe capture immich-export.jpg --gps-source immich-exif --lat 48.2085 --lon 16.3738 --hint stephansdom
./target/debug/travelcraft-cli.exe chat --about caravaggio --question "Who painted the Rosenkranzmadonna and where is it?"
```

### Sight Chat notes

- **Text**: Sight Chat tab, context pill shows current sight. Ask anything about the dossier; history (last 6) kept for follow-ups.
- **Online LLM**: Ollama `http://127.0.0.1:11434/api/chat` (model field, default `qwen2.5:7b`). System prompt = info blob + map + transit + sources. If unreachable, OFFLINE BLOB extractive answer is returned and labeled.
- **Voice out**: `Speak last answer` = Windows SAPI offline; `Save last answer (speech-mcp wav)` = POST `:10909/api/v1/tts` to temp wav.
- **Voice in**: type/paste transcript (mic path: speech-mcp `:10909` web UI + FunASR sidecar `:10914`; record wav then paste transcript here in v1).
- **Maps**: every prompt carries `MAP qgiscraft Z16 tile + OSM URL + scale + Z8 bbox count` (offline math, live tiles via qgiscraft `:11254`).
- **Transit**: preloaded blob per sight (Stephansdom U1/U3, KHM D/1/2/71 + U3, Belvedere D/71, Schoenbrunn U4, JP JR/IC hubs) sourced from `gtfs-mcp/data/vienna|wien|default`; live departures via gtfs-mcp `:10913`.

### SightLens GPS notes

- **Windows location**: Settings > Privacy > Location ON; browser/OS prompt once.
- **iPhone as hotspot**: shares internet only, NOT GPS. Enter coords manually or import the Immich original (EXIF GPS preserved).
- **Immich**: export original (not preview), paste path, source `immich-exif`.

## Status - does today / next week / non-goals

**Does today:** 4-crate Rust workspace (baedeker/core/mcp/app); Baedeker
guide model (places, routes, itineraries); map app with fly-to, scale bar,
regional intel; fleet i18n (7 locales); 11 test fns; CLI verbs via
`travelcraft --help`.

**Next week:** `docs/PRD.md` (model + verbs + qgiscraft boundary) plus
`docs/TOOLS.md`; `just ci` (fmt + clippy-deny + test); first democraft
recipe on fixtures.

**Non-goals:** not a GIS analyzer (that's qgiscraft); no live-tile SLA -
keyless OSM/Photon only.

## Lineage - the Storytold craft-app pattern

Storytold ArtCraft ([storytold/artcraft](https://github.com/storytold/artcraft) - "the IDE for artists") was the starting point. The 16 craft apps in the Storytold stable each ship one binary that is both the human CLI and the agent MCP server (stdio), headless-first, with `run.json` + report artifacts. That modus operandi and harness is what this repo implements; the code here is independent, the pattern is shared.
