# TravelCraft (Baedeker 2030)

**TravelCraft** is an ArtCraft-tier chrono-cartography studio and sovereign travel guide platform that reinvents the classic 19th-century Baedeker guide for the 2030s.

Features:
- **Chrono-Cartographic Layers**: Tokaido 1603 vs Neo-Tokyo; Hallstatt 800 BC vs Habsburg Vienna 1365.
- **Sensory Atmosphere Profiles**: Acoustic, architectural, olfactory, light records per waypoint.
- **Sovereign Local Storage**: Offline SQLite journal + GPS-tagged photos via `DatabaseCraft`.
- **Stay-Length Itineraries**: 1/2/3/5/7/14-day Baedeker day-plans per edition (Itineraries tab, `travelcraft-cli itinerary AT 3`).
- **SightLens Camera**: Point laptop at Stephansdom and click Identify, or load an Immich export. Offline GPS + hint matching (Stephansdom, KHM Caravaggio Rosenkranzmadonna, Klimt Kiss, Bruegel Tower) renders a beautiful dossier and logs it with GPS source.
- **Sight Chat (text + voice)**: After identifying the Caravaggio, ask anything. Grounded in info blob + qgiscraft map blob + GTFS transit blob; online via Ollama `:11434` when reachable, else OFFLINE BLOB extractive answer. Voice out via Windows SAPI Speak or speech-mcp `:10909` TTS wav.
- **Maps + Transit blobs**: qgiscraft slippy tiles/scale/URLs (`:11254`) + preloaded Wiener Linien/JR snapshot; live departures via gtfs-mcp `:10913`.
- **High-Contrast Map**: Mouse-wheel zoom + drag pan, large type, ink/paper palette.
- **FastMCP Protocol**: Port `14129`.

Vendored offline sources (see `crates/travelcraft-baedeker/src/lib.rs SOURCES`):
- `japanophile-mcp/assets/knowledge/japan/travel.html` — JR Pass, konbini, budget, etiquette, regions.
- `vienna-life-assistant` `vienna_tips` (coffee/music/museum), `restaurants_data` Beisl rule, `culture_data` (KHM, Belvedere, Stephansplatz).

---

## Workspace Structure

1. `crates/travelcraft-baedeker`: Editions (`JapanCraft` 9 stops, `AustriaCraft` 8 stops), stay itineraries, SightLens identify DB (landmarks + KHM/Belvedere artworks).
2. `crates/travelcraft-core`: Geodesic routing, itineraries, SightLens pipeline, DatabaseCraft journal with GPS columns.
3. `crates/travelcraft-mcp`: FastMCP JSON-RPC server on port `14129`.
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

## Lineage — the Storytold craft-app pattern

Storytold ArtCraft ([storytold/artcraft](https://github.com/storytold/artcraft) — "the IDE for artists") was the starting point. The 16 craft apps in the Storytold stable each ship one binary that is both the human CLI and the agent MCP server (stdio), headless-first, with `run.json` + report artifacts. That modus operandi and harness is what this repo implements; the code here is independent, the pattern is shared.
