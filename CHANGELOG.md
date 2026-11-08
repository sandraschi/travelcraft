# Changelog

## Unreleased

- 2026-11-08: `travel_nearby` (MCP tool 13) and `travelcraft-cli nearby`: real places from OpenStreetMap via Overpass. Curated categories or a fenced key and value, no raw QL. Nearest first, OSM and edit links, `missing_tags`, ODbL attribution. Online and opt-in; 24 h SQLite cache with a stale fallback, mirror list (overpass.openstreetmap.fr first) with 15 s timeouts. OSM text is sanitised as untrusted. Typed error kinds (`error_type`: invalid, offline, protocol, store, ...) replace the generic `engine`. Not done: opening-hours evaluation, GUI panel. Verified live in Claude Desktop for Wien, Graz and Tokyo, with cache hit and refresh. overpass-api.de, kumi.systems and private.coffee were unreachable or returning 502 from the dev machine; cause not yet diagnosed.

- 2026-11-08: real MCP 2024-11-05 stdio server in `travelcraft-mcp` (12 tools); durable SQLite state (journal, chat history, settings) with parameterised SQL and unique entry ids, crash-tested; GUI no longer claims "persisted" for an in-memory DB and no longer seeds a fake journal entry, replacing the banner-only binary. Dependency moved from `qgiscraft-core` to `giscraft-core` after the repo rename. Added AGENTS.md, justfile, .gitattributes (LF), docs/PRD.md, docs/TOOLS.md, workspace clippy deny-lints for the non-GUI crates.

## 0.1.0

- Baedeker editions (Austria, Japan), stay itineraries, SightLens identify, Sight Chat, egui app, CLI.