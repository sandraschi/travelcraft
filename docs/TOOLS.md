# travelcraft MCP tools

Server: `travelcraft-mcp` (MCP 2024-11-05, line-delimited JSON-RPC on stdio).
stdout carries protocol only, logs go to stderr. Notifications get no reply.
Errors: `{success:false, error, error_type}` with `isError: true`.
State is durable: journal, chat history and settings live in a SQLite file (WAL, synchronous=FULL),
`%LOCALAPPDATA%\travelcraft\travelcraft.db` or `$TRAVELCRAFT_DATA\travelcraft.db`. A crash loses nothing
that was acknowledged. If the database cannot be opened the server exits with an error; it does not
fall back to memory.

| Tool | Arguments | Returns |
|---|---|---|
| `travel_status` | none | editions, persistence flag, db path, journal count, vendored source list |
| `travel_countries` | none | all country editions (large, about 14 KB) |
| `travel_inspect` | `country` (AT, JP) | one edition: waypoints, atmosphere, curated routes |
| `travel_knowledge_tree` | `country` | primer topic tree (large, about 18 KB for JP) |
| `travel_plan_route` | `country` | chrono-optimised route: legs, distance, hours, carbon, map tiles |
| `travel_itinerary` | `country`, `days` (1, 2, 3, 5, 7, 14; default 3) | day-by-day plan |
| `travel_identify_sight` | `lat`, `lon`, `hint` (all optional) | sight or artwork from the offline catalogue |
| `travel_sight_chat` | `question`; optional `about`, `lat`, `lon` | offline answer plus system prompt, map blob, transit blob |
| `travel_sightlens_capture` | `photo`; optional `gps_source`, `lat`, `lon`, `hint` | identification plus a durable journal entry |
| 	ravel_journal_add | eflection; optional sensory_notes, waypoint_id, photo, lat, lon, gps_source | new entry id; text capped at 16000 chars |
| 	ravel_journal_list | optional limit (1 to 1000, default 20) | entries, newest first |
| 	ravel_geocode | query | offline demo geocoder hits (giscraft-core) |

Limits today: the catalogue is small (Stephansdom, KHM Caravaggio, Klimt Kiss
and the edition waypoints). `travel_sight_chat` does not call an LLM; it returns
the grounding for the caller's own model. Live transit and live map tiles are
not fetched by the MCP server.

Wire it in a host (Claude Desktop shape):

```json
{ "mcpServers": { "travelcraft": { "command": "D:\\Dev\\repos\\travelcraft\\target\\release\\travelcraft-mcp.exe" } } }
```