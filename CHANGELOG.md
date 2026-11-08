# Changelog

## Unreleased

- 2026-11-08: real MCP 2024-11-05 stdio server in `travelcraft-mcp` (12 tools); durable SQLite state (journal, chat history, settings) with parameterised SQL and unique entry ids, crash-tested; GUI no longer claims "persisted" for an in-memory DB and no longer seeds a fake journal entry, replacing the banner-only binary. Dependency moved from `qgiscraft-core` to `giscraft-core` after the repo rename. Added AGENTS.md, justfile, .gitattributes (LF), docs/PRD.md, docs/TOOLS.md, workspace clippy deny-lints for the non-GUI crates.

## 0.1.0

- Baedeker editions (Austria, Japan), stay itineraries, SightLens identify, Sight Chat, egui app, CLI.