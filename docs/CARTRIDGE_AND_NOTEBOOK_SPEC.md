# TravelCraft Cartridge CRUD and Universal Travel Notebook Spec

Status: Active Design & Implementation (2026-11-20)
Applies to: `travelcraft-baedeker`, `travelcraft-core`, `travelcraft-mcp`, `travelcraft-app`

## 1. Problem

TravelCraft currently compiles Austria (8 stops) and Japan (9 stops) into `travelcraft-baedeker` and hardcodes `TravelCity::{Vienna, Tokyo}` in `travelcraft-core/src/cities.rs`. `RegionPack` is an unwired stub with no CRUD, no city/country hierarchy, no local food/spot layer, no transit snapshot, and no stay itineraries. `travel_journal` is a flat table keyed only by `waypoint_id`, with no trip container and no way to filter notes by trip, city, or country.

To make TravelCraft a real offline travel studio (e.g., planning and logging "A Week in Venice"), it needs two cleanly separated subsystems:

1. **Country & City Cartridges (Reference Layer)**: portable, offline, CRUD-managed destination packs with Country -> City inheritance.
2. **Universal Travel Notebook (Personal Layer)**: one durable SQLite notebook holding Trips and Entries, with scoped views per Trip, City Cartridge, Country, or All.

## 2. Layered Cartridge Model

A **City** cartridge may declare `parent_id` pointing to a **Country** cartridge (e.g., `it-venice` -> `it`).
Resolving a city cartridge merges the parent's country-level survival kit, currency, languages, timezone, phrasebook, and knowledge tree underneath the city's own local overrides.

### 2.1 Cartridge Schema (`Cartridge`)

- **Identity & Spatial Metadata**:
  - `id`: lowercase slug `[a-z0-9-]`, 2..=64 chars (e.g. `it`, `it-venice`, `at-vienna`)
  - `kind`: `country` | `city`
  - `parent_id`: optional `String` (recommended for `city`, forbidden for `country`)
  - `country_code`: ISO 3166-1 alpha-2 uppercase (e.g. `IT`, `AT`, `JP`)
  - `title`, `subtitle`, `description`, `version`, `schema_version` (`1`)
  - `currency`, `languages`, `timezone`, `sources` (attribution)
  - `viewport`: `{ lon, lat, zoom }`
  - `bbox`: `{ min_lat, max_lat, min_lon, max_lon }`
  - `zones`: named districts / islands (e.g. Venice's 6 *sestieri* + Murano/Burano/Torcello/Lido)
- **Survival & Language**:
  - `survival_kit`: emergency number, plugs, tap water, tipping/coperto, transit pass basics, medical/pharmacy notes, general notes
  - `phrases`: compact offline phrasebook (`phrase`, `local`, `pronunciation`, `context`)
- **Baedeker Reference Layers**:
  - `waypoints`: `Vec<ChronoWaypoint>` (coords, `year_ad`, narrative, `SensoryAtmosphere`, food, visitor practicals)
  - `artworks`: `Vec<ArtworkRecord>` (museum, room, story, `look_for` for SightLens + Sight Chat)
  - `spots`: `Vec<LocalSpot>` (curated local places: `bacaro`, `restaurant`, `cafe`, `pasticceria`, `market`, `bookshop`, `viewpoint`, `stay`, `shop`, `other` with tags, price band, local rule/tip, coords, zone)
  - `transit`: `TransitSnapshot` (summary, lines, airport links, passes, crossings/tips)
  - `itineraries`: `Vec<StayItinerary>` (1, 2, 3, 5, 7, 14 day plans)
  - `routes`: `Vec<CuratedRoute>`
  - `gotchas`: `Vec<Gotcha>` (seasonal/practical traps, e.g. *acqua alta*, access fee, museum closure days)
  - `knowledge_tree`: optional `KnowledgeTree` (branches + topic items)

### 2.2 Storage & Portability

- Durable storage lives in SQLite (`cartridges` table in `travelcraft.db`), JSON-serialized payload with indexed metadata columns (`cartridge_id`, `kind`, `parent_id`, `country_code`, `title`, `version`, `updated_at`).
- Portable exchange format: UTF-8 JSON (`.tcpack.json`) validated on import.
- Referential safety: deleting a `country` cartridge that has child `city` cartridges fails with `Invalid` unless `cascade = true`.

### 2.3 CRUD Operations

- **Cartridge CRUD**: `list`, `get`, `resolve` (merges parent), `create`, `update` (metadata/survival/transit), `delete` (with optional `cascade`), `import` (JSON upsert), `export` (pretty JSON), `seed_venice` (installs reference `it` + `it-venice`).
- **Item CRUD** (mutates one item inside a cartridge without rewriting the full JSON by hand):
  - `waypoint`: upsert / remove by `id`
  - `artwork`: upsert / remove by `id`
  - `spot`: upsert / remove by `id`
  - `itinerary`: upsert / remove by `total_days`
  - `gotcha`: upsert / remove by `id`
  - `zone`: upsert / remove by `id`

## 3. Universal Travel Notebook

Personal notes, photos, expenses, and reflections NEVER live inside cartridge files. They live in one universal SQLite store so that:
- Updating, sharing, or deleting a cartridge never touches personal data.
- Multi-leg trips (e.g., Vienna -> Nightjet -> Venice -> Padua day trip -> Vienna) stay unified.
- Cross-trip search works across all places and years.
- Any cartridge or trip still gets a dedicated filtered view (`This Trip | This City | This Country | All`).

### 3.1 `trips` Table

- `trip_id` (PK), `title`, `start_date` (`YYYY-MM-DD`, optional), `end_date` (`YYYY-MM-DD`, optional), `cartridge_ids_json` (e.g. `["it", "it-venice"]`), `status` (`planned | active | completed`), `notes`, `created_at`, `updated_at`.

### 3.2 `notebook_entries` Table

- `entry_id` (PK, generated via `new_entry_id`)
- `trip_id` (optional FK-like link to `trips`)
- `cartridge_id` (optional link to a cartridge, e.g. `it-venice`)
- `country_code` (optional uppercase ISO code, e.g. `IT`)
- `waypoint_id` (optional; freeform notes do not require a pre-existing sight)
- `kind`: `reflection | photo | food | logistics | expense | idea`
- `day_index`: optional 1-based trip day
- `timestamp_epoch`: unix seconds
- `title`, `body`, `sensory_notes`
- `photo_uris_json`, `lat`, `lon`, `tags_json`, `updated_at`

### 3.3 Queries & Export

- `notebook_query` filters by any combination of `trip_id`, `cartridge_id`, `country_code`, `waypoint_id`, `kind`, `tag`, `search` (case-insensitive substring over title/body/sensory_notes), and `limit` (1..=1000).
- `trip_export_markdown(trip_id)` compiles trip metadata, linked cartridge titles, and all entries grouped by `day_index` (then timestamp) into a standalone Markdown travelogue.

## 4. MCP Tool Surface

Three portmanteau tools added to `travelcraft-mcp`:
- `travel_cartridge`: `list | get | resolve | create | update | delete | import | export | seed_venice | item_upsert | item_remove`
- `travel_trip`: `list | get | create | update | delete | export_markdown`
- `travel_notebook`: `add | get | update | delete | query`

## 5. Reference Cartridge: "A Week in Venice" (`it` + `it-venice`)

Ships as a seedable reference pair proving the full schema:
- `it`: Italy survival kit, rail/coperto/coffee rules, phrasebook, historical eras.
- `it-venice`: 6 *sestieri* + lagoon islands, 8 chrono-waypoints, 4 museum/church artworks (Tintoretto, Titian, Carpaccio, Bellini), curated *bacari*/*cicchetti* spots, ACTV vaporetto + *traghetto* + airport transit snapshot, 1/3/7-day itineraries, and *acqua alta* / access-fee / museum-closure gotchas.
