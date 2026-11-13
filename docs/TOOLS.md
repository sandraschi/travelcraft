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
| `travel_journal_add` | `reflection`; optional `sensory_notes`, `waypoint_id`, `photo`, `lat`, `lon`, `gps_source` | new entry id; text capped at 16000 chars |
| `travel_journal_list` | optional `limit` (1 to 1000, default 20) | entries, newest first |
| `travel_geocode` | `query` | offline demo geocoder hit (giscraft-core); echoes the query at a fixed Wien point, not a place index |
| `travel_nearby` | `category` or `key`+`value`; `lat`+`lon` or `place` (major AT/JP city); optional `radius_m` (50 to 5000, default 1500), `limit` (1 to 50, default 15), `refresh` | **online, opt-in.** Places from OpenStreetMap via Overpass, nearest first, with OSM and edit links, raw opening_hours, `missing_tags`, ODbL attribution, and `source` (overpass, cache, stale_cache) |
| `travel_transit` | `operation` (`status`, `stops`, `departures`, `live`, `disruptions`); optional `query`, `stop_id`, `rbl`, `feed_id` (default `default`), `limit` (1 to 20, default 8), `sight_id` | **online, opt-in, localhost only.** `stops` and `departures` call gtfs-mcp `:10913`. `live` and `disruptions` call publictransport-mcp `:11170` (Vienna). `status` also probes japanophile-mcp `:11193`, vienna-life-assistant `:10922`, speech-mcp `:10909`, and giscraft `:11254`. If the needed companion is down, `source` is `offline` and `offline_blob` is the curated transit note. |

Network: `travel_nearby` calls public Overpass mirrors. `travel_transit` calls only the local companion URLs above (override with `TRAVELCRAFT_GTFS_URL`, `TRAVELCRAFT_PT_URL`, `TRAVELCRAFT_JAPANOPHILE_URL`, `TRAVELCRAFT_VIENNA_LIFE_URL`, `TRAVELCRAFT_SPEECH_URL`, `TRAVELCRAFT_GISCRAFT_URL`). Categories: bicycle_repair, bicycle_rental, pharmacy, doctors, hospital, atm, cafe, restaurant, bakery, supermarket, toilets, drinking_water, ev_charging, post_office, museum, viewpoint, hotel, train_station; or any fenced OSM key and value. No raw Overpass QL is accepted. OSM text is untrusted data: control characters are stripped and fields capped at 200 chars. Answers are cached 24 h in the database; if the live query fails, the last cached answer comes back with `stale: true` and a warning, and with no cache the call fails with `error_type: offline`. Public mirrors are tried in order (overpass.openstreetmap.fr, overpass-api.de, kumi.systems, private.coffee); `TRAVELCRAFT_OVERPASS_URL` pins one endpoint. `truncated: true` means Overpass hit the 300-element fetch cap, so shrink the radius. Opening hours are returned raw and never evaluated.

Limits today: the catalogue is small (Stephansdom, KHM Caravaggio, Klimt Kiss
and the edition waypoints). `travel_sight_chat` does not call an LLM; it returns
the grounding for the caller's own model. Live map tiles are not fetched by the
MCP server. Live transit is `travel_transit`, and only when the local companion
is running.

Wire it in a host (Claude Desktop shape):

```json
{ "mcpServers": { "travelcraft": { "command": "D:\\Dev\\repos\\travelcraft\\target\\release\\travelcraft-mcp.exe" } } }
```