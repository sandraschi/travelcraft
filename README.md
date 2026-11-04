# TravelCraft (Baedeker 2030)

**TravelCraft** is an ArtCraft-tier chrono-cartography studio and sovereign travel guide platform that reinvents the classic 19th-century Baedeker guide for the 2030s.

Features:
- **Chrono-Cartographic Layers**: Seamless juxtaposition of historical waypoints (e.g. Tokaido 1603 vs modern Tokyo; Hallstatt Celtic 800 BC vs Habsburg Vienna 1365).
- **Sensory Atmosphere Profiles**: High-fidelity records of acoustic soundscapes, architectural eras, olfactory impressions, and ambient light qualities.
- **Sovereign Local Storage**: Zero cloud lock-in with offline SQLite journal and waypoint storage powered by `DatabaseCraft`.
- **FastMCP Protocol**: Integrates with local AI swarms via port `14129`.

---

## Workspace Structure

1. `crates/travelcraft-baedeker`: Curated country editions (`JapanCraft`, `AustriaCraft`), waypoint models, and sensory taxonomies.
2. `crates/travelcraft-core`: Offline route optimization, great-circle calculations, and DatabaseCraft journal persistence.
3. `crates/travelcraft-mcp`: FastMCP JSON-RPC server on port `14129`.
4. `crates/travelcraft-app`: Native GUI (`eframe`/`egui`) and CLI tool (`travelcraft-cli`).

---

## Verification & Usage

```powershell
# Run test suite
cargo test

# Headless smoke test
./target/debug/travelcraft.exe --smoke

# CLI Route Optimization
./target/debug/travelcraft-cli.exe plan JP
./target/debug/travelcraft-cli.exe inspect AT
```
