# Changelog

## Unreleased

- 2026-11-13: `travel_transit` (MCP tool 14) and `travelcraft-cli transit`, plus a Transit tab. Local companions only: gtfs-mcp schedules (`:10913`), publictransport-mcp Vienna live departures and disruptions (`:11170`), and status probes for japanophile-mcp, vienna-life-assistant, speech-mcp, and giscraft. Down companions return the offline transit blob with `source: offline`. Tests use a fake transport and do not open a socket.
- 2026-11-11: Implemented Tier 4 Craft Menu Bar Standard and Help Suite in `travelcraft-app`. 8-menu top bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`), mandatory 5-dialog Help suite (`Command Palette`, `Keyboard Shortcuts`, `Project Files & Docs`, `Licenses & Notices`, `About TravelCraft`), Preferences dialog, and hotkey accelerators (`Ctrl+K`, `F1`, `Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+E`, `Ctrl+F`, `Ctrl+,`).
- 2026-11-11: Enabled workspace deny-lints (`unwrap_used`, `expect_used`, `panic`, `unimplemented`, `todo`) across `travelcraft-app`. Cleaned up dead map tile fields and dead code debt; wired `focus_selected`, `section_heading`, and `epoch_color` for chrono layer visualization.
- 2026-11-11: Added dynamic `RegionPack` format and SQLite loader for offline guide expansion (`import_region_pack`, `list_region_packs`, `load_region_packs_from_dir`). Expanded catalogue with masterworks (Hokusai Great Wave, Sesshu Toyo Sansui, Egon Schiele Wally, Vermeer Art of Painting) with bound-checked lookups.
- 2026-11-11: Added production NSIS current-user installer (`installer/installer.nsi`) and MCPB bundle manifest (`mcpb/manifest.json`). Headless `--smoke` verified with zero unsafe guarantee and command bus parity.

## 0.2.0 (2026-11-08, alpha)

- 2026-11-08: `travel_nearby` (MCP tool 13) and `travelcraft-cli nearby`: real places from OpenStreetMap via Overpass. Curated categories or a fenced key and value, no raw QL. Nearest first, OSM and edit links, `missing_tags`, ODbL attribution. Online and opt-in; 24 h SQLite cache with a stale fallback, mirror list (overpass.openstreetmap.fr first) with 15 s timeouts. OSM text is sanitised as untrusted. Typed error kinds (`error_type`: invalid, offline, protocol, store, ...) replace the generic `engine`. Not done: opening-hours evaluation, GUI panel. Verified live in Claude Desktop for Wien, Graz and Tokyo, with cache hit and refresh. overpass-api.de, kumi.systems and private.coffee were unreachable or returning 502 from the dev machine; cause not yet diagnosed.

- 2026-11-08: real MCP 2024-11-05 stdio server in `travelcraft-mcp` (12 tools); durable SQLite state (journal, chat history, settings) with parameterised SQL and unique entry ids, crash-tested; GUI no longer claims "persisted" for an in-memory DB and no longer seeds a fake journal entry, replacing the banner-only binary. Dependency moved from `qgiscraft-core` to `giscraft-core` after the repo rename. Added AGENTS.md, justfile, .gitattributes (LF), docs/PRD.md, docs/TOOLS.md, workspace clippy deny-lints for the non-GUI crates.

## 0.1.0

- Baedeker editions (Austria, Japan), stay itineraries, SightLens identify, Sight Chat, egui app, CLI.