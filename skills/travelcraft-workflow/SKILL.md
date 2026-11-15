---
name: travelcraft-workflow
description: Workflows in TravelCraft for planning an itinerary, capturing a SightLens entry, and checking live transit.
---

# TravelCraft Trip & SightLens Workflows

1. **Plan a trip:** Call `travel_countries` to confirm the edition (`AT` or `JP`), then call `travel_itinerary` for the requested day count and `travel_plan_route` for waypoint ordering.
2. **Identify a sight or artwork:** Call `travel_identify_sight` with GPS coordinates and/or hint, then enrich with `travel_sight_chat`. Only call `travel_sightlens_capture` or `travel_journal_add` after user confirmation.
3. **Check transit:** Call `travel_transit` with `operation: "status"` or `"stops"` / `"live"` / `"departures"` to verify local routes.
