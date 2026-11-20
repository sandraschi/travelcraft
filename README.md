# TravelCraft

TravelCraft (Baedeker 2030) is a local-first chrono-cartography studio and offline travel guide that pairs historical city layers, itineraries, and a SQLite travel journal with an MCP server for agentic IDEs.

## What You Can Do

- **Explore chrono-cartographic editions**: Browse curated Austria (`AT`, 8 waypoints from Hallstatt 800 BC to Habsburg Vienna) and Japan (`JP`, 9 waypoints from Tokaido 1603 to Neo-Tokyo) guides with sensory atmosphere profiles (acoustic, architectural, olfactory, light).
- **Plan stay-length itineraries and routes**: Generate 1, 2, 3, 5, 7, or 14-day Baedeker day-plans (`travel_itinerary`) and geodesic Haversine routes (`travel_plan_route`), or organize stops on the **Trip Kanban** board.
- **Identify landmarks and artworks with SightLens**: Match GPS coordinates and hints offline against landmarks and museum dossiers (Stephansdom, KHM Caravaggio *Rosenkranzmadonna*, Klimt *The Kiss*, Bruegel, Hokusai) and log captures to your journal.
- **Ask questions in Sight Chat & AI Chat Dock**: Press `Ctrl+J` to open the **AI Chat Dock** for grounded answers (using Ollama `:11434` when reachable or offline extractive fallback), with optional voice output via Windows SAPI or `speech-mcp`.
- **Use the 10-item Help menu and Bundled Skills**: Open the in-app **Help Center** (`HELP.md`), browse **Bundled Skills** (`skill://travelcraft-advisor/SKILL.md`, `skill://travelcraft-workflow/SKILL.md`), or call **15 MCP tools** from Claude Desktop.
- **Keep a private offline travel journal**: Save reflections, GPS coordinates, and SightLens captures in a local SQLite WAL database (`%LOCALAPPDATA%\travelcraft\travelcraft.db`), and export standalone HTML dossiers (`Ctrl+E`) or serve a mobile view (`travelcraft-cli serve 14130`).
- **Query nearby OpenStreetMap POIs and fleet transit**: Look up bicycle repair, drinking water, or toilets via Overpass (`travel_nearby`, cached 24 h) and check Vienna/Japan transit via optional local companions (`gtfs-mcp`, `publictransport-mcp`, `qgiscraft`).

## Install

1. Download **`travelcraft-0.2.0-setup.exe`** from the [latest release](https://github.com/sandraschi/travelcraft/releases/latest).
2. Run it. Windows may show a SmartScreen warning because the installer is unsigned: choose **More info > Run anyway**.
3. Open **TravelCraft** from the Start menu.

**Only want the tools inside Claude?** The `.mcpb` bundle registers the MCP server in Claude Desktop but does **not** install the desktop studio. Paste this to Claude:

> Install the travelcraft MCP extension: download `travelcraft-0.2.0.mcpb` from https://github.com/sandraschi/travelcraft/releases/latest and open it with Claude Desktop.

**From a local checkout (Windows):**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-craft.ps1
```

Installs the Windows app, `travelcraft-cli`, and the MCP bundle.

## Example Prompts

> "Plan a 3-day Austria itinerary and summarize each stop with its historical era."

> "What am I looking at near 48.2039, 16.3619 with hint `caravaggio`, and how do I get there?"

> "Log a journal entry: Cafe Berg, rain, Melange — then show my last 5 journal entries."

> "Find bicycle repair shops near Wien using OpenStreetMap."

## Three Ways to Use It

| Surface | Start With | Best For |
|---|---|---|
| **Desktop Studio** | `travelcraft` | Interactive Vienna/Tokyo maps, Chrono-Map, Trip Kanban, SightLens, and `Ctrl+J` Chat |
| **Command Line** | `travelcraft-cli --help` | Headless itineraries, route planning, SightLens capture, and mobile dossier serving |
| **MCP Server** | `travelcraft-mcp` | Calling 15 travel tools and 2 bundled skills from Claude Desktop or your IDE |

## Keyboard Shortcuts (Desktop Studio)

| Shortcut | Action |
|---|---|
| `Ctrl+J` | Toggle the **AI Chat Dock** |
| `Ctrl+S` | Save current reflection to the Travel Journal |
| `Ctrl+E` | Export a standalone Baedeker HTML dossier |

## Scope & Honest Limits

Core editions (`AT` and `JP`), SightLens matching, and the SQLite journal work completely offline. `travel_nearby` uses public OpenStreetMap Overpass mirrors (24 h cache) and returns raw `opening_hours` tags without evaluating whether a place is open right now. SightLens identifies sights from GPS plus a text hint rather than image recognition.

## Documentation

| Document | Contents |
|---|---|
| [HELP.md](HELP.md) | Complete user guide, shortcuts (`Ctrl+J`, `Ctrl+E`, `Ctrl+S`), and SightLens notes |
| [docs/TOOLS.md](docs/TOOLS.md) | Reference for all 15 MCP tools and bundled skill URIs |
| [docs/PRD.md](docs/PRD.md) | Product requirements, crate layout, and `qgiscraft` boundary |
| [docs/TIERED_PLAN.md](docs/TIERED_PLAN.md) | Feature roadmap and tier checklist |
| [docs/CARTRIDGE_AND_NOTEBOOK_SPEC.md](docs/CARTRIDGE_AND_NOTEBOOK_SPEC.md) | Edition cartridge and notebook specification |

## Requirements

- **OS**: Windows 10/11 (x64) for the installer; Linux/macOS supported from source
- **Optional companions**: Ollama (`:11434`) for generative Sight Chat; `qgiscraft`, `gtfs-mcp`, `publictransport-mcp`, `speech-mcp` for live tiles, schedules, and TTS

## License

MIT or Apache-2.0, at your option.
