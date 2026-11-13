# TravelCraft — Tiered Execution Plan

**Parent Standard:** [`RUST_ARTCRAFT_STANDARD.md`](../../mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md)  
**Menu Bar Standard:** [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md)  
**Architecture Model:** 4-Tier ArtCraft Incremental Delivery  
**Status:** Active  

---

## Architecture Flow

```
Tier 1: Core Engine ──> Tier 2: MCP / CLI ──> Tier 3: Desktop UI ──> Tier 4: Menu Bar, Help & Packaging
(travelcraft-core)     (travelcraft-mcp)     (travelcraft-app)     (Hardened Workstation)
```

---

## Tier Specifications

### Tier 1 — Core Offline Guide & Routing Engine (`travelcraft-core`)
- [x] L1 `Command::dispatch` universal travel execution bus.
- [x] Baedeker knowledge trees for Austria and Japan editions.
- [x] Haversine route optimizer and multi-day stay itineraries (1–14 days).
- [x] SightLens location identifier and SQLite WAL travel journal.
- [x] Offline sensory atmosphere and historical chrono-waypoints.
- [x] `#![forbid(unsafe_code)]` and zero-panic clippy lints.

### Tier 2 — Headless Driver & FastMCP (`travelcraft-mcp`)
- [x] Unified CLI `travelcraft-cli` with `--json` output and iPad serve mode.
- [x] FastMCP stdio server communicating over JSON-RPC 2.0 (14 tools, including `travel_transit`).
- [x] Nearby sight lookup with Overpass client and local cache.
- [x] Headless `--smoke` verification without window initialization.

### Tier 3 — Desktop Workstation UI (`travelcraft-app`)
- [x] Immediate-mode `egui` 0.37 / `wgpu` workstation.
- [x] Interactive map viewport and chrono-waypoint timeline.
- [x] Sight detail inspector and offline sight chat assistant.

### Tier 4 — Craft Menu Bar Standard, Help Suite & Hardened Verification
- [x] Universal 8-menu bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) with `switch_on_hover`.
- [x] Mandatory 5-dialog Help suite (`Command Palette...`, `Keyboard Shortcuts...`, `Project Files & Docs...`, `Licenses & Notices...`, `About TravelCraft`).
- [x] Fleet UI/UX Pattern 2 adherence: viewport-centric navigation, top button row mapped to View toggles, panel visibility managed via `View` / `Window`.
- [x] Tofu Guard verification (100% font glyph coverage, zero tofu placeholder boxes).
- [x] Headless `--smoke` parity validation.
- [x] Standalone release binary packaging and NSIS installer (`installer/installer.nsi`, `mcpb/manifest.json`).
