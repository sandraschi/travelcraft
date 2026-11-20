---
name: cartridge-compiler
description: Compile grounded, multi-source Country and City Cartridges for TravelCraft (Evergreen history/art/media + Semi-volatile tiered hotels/restaurants/transit + Volatile current events) and import them via travel_cartridge.
---

# TravelCraft Cartridge Compiler

Use this workflow when creating a new Country or City Cartridge (e.g. `it-venice`, `gb-london`, `fr-paris`, `at-salzburg`, `jp-kyoto`) or running a pre-trip refresh on an existing cartridge.

## Core Principles

1. **Ground first, write second.** Never invent coordinates, street addresses, opening hours, restaurant names, hotel names, or exhibition dates from memory. Pull from primary sources first, then synthesize.
2. **Respect the 3 freshness layers.**
   - **Evergreen**: history, chrono-waypoints, architectural eras, museum artworks and room numbers, licensed photos (`media`), cultural rules.
   - **Semi-volatile**: tiered hotels and restaurants (`spots` with `tier: budget | midrange | splurge`), transit passes/fares (`transit`), local traps (`gotchas`).
   - **Volatile (pre-trip)**: current exhibitions, concerts, festivals, strikes, restoration closures (`events` + `events_refreshed_at`).
3. **Baedeker prose, zero brochure fluff.** Ban AI travel clichés ("hidden gem", "vibrant tapestry", "must-see", "bustling hub", "something for everyone"). Prefer concrete architectural periods, sensory detail, exact room numbers, best light/crowd hours, and blunt practical rules.
4. **ASCII hyphens only.** No em dashes in any cartridge JSON field.
5. **Country -> City inheritance.** Always create or verify the parent Country cartridge (`it`, `gb`, `fr`, `at`, `jp`) first so the City cartridge inherits currency, languages, timezone, emergency numbers, rail basics, and national phrasebook without duplicating them.

---

## Step 1: Primary Source Harvest Checklist

Before writing JSON, gather verified facts from multiple independent sources:

| Layer | Sources to Query | Required Verification |
|---|---|---|
| Geography & Zones | Wikipedia, Wikivoyage, OSM | Exact `viewport (lon, lat, zoom)`, `bbox`, and 4-8 named `zones` (districts/quarters) |
| Deep History & Waypoints | Wikipedia, Wikidata, museum sites | `year_ad`, exact WGS84 `lat/lon` inside `bbox`, architectural era, visitor practicals |
| Artworks & Interiors | Official museum catalogs, Wikipedia | Museum name, room/chapel location, year, what to look for |
| Photos (`media`) | Wikimedia Commons, Europeana, museum Open Access (CC0/CC-BY) | Direct image URL, `caption`, `license`, `attribution`, linked `subject_id` |
| Tiered Restaurants & Cafes | 3+ independent lists (e.g. Slow Food / Gambero Rosso / Michelin Bib / local food writers) | Cross-check still open; assign `tier` (`budget`, `midrange`, `splurge`), `booking_note`, `verified_at`, `source_urls` |
| Tiered Hotels (`category: stay`) | Independent hotel/palazzo guides + transit proximity check | 2-3 options per tier (`budget`, `midrange`, `splurge`), note nearest transit stop and bridge/stair reality |
| Transit Snapshot | Official city transit agency site | Single fare vs multi-day pass break-even math, airport links, validation rules |
| Current Events (`events`) | Official museum, opera/concert hall, and city calendars for `trip.start_date..end_date` | Exact `start_date`, `end_date`, `venue`, `booking_url`, `source_url`, `verified_at` |

---

## Step 2: Tiered Rubric for `LocalSpot`

Every restaurant and hotel MUST have a clear `tier` and a blunt `booking_note`:

- **Food (`bacaro`, `restaurant`, `cafe`, `pasticceria`, `market`)**:
  - `budget`: counter snacks, markets, bakeries, local neighborhood lunch spots.
  - `midrange`: honest neighborhood trattorias, bistros, izakayas, wine bars where locals eat.
  - `splurge`: destination dining / historic rooms worth booking weeks ahead.
- **Hotels (`stay`)**:
  - `budget`: clean, well-located character guesthouse, monastic/historic simple stay, or business hotel.
  - `midrange`: boutique hotel or small historic palazzo in a quiet, well-connected district.
  - `splurge`: landmark hotel or canal/park-front grand palazzo.

Every `LocalSpot` should include:
- `booking_note`: e.g. `"Walk-in only; arrive before 12:00"` or `"Book 3 weeks ahead"`.
- `source_urls`: at least one reference URL.
- `verified_at`: `YYYY-MM-DD` (today's verified date from the clock).

---

## Step 3: Chunked Synthesis Order

Do NOT generate a full city cartridge in one giant unfocused output. Compile layer by layer:

1. **Manifest + Zones + Survival Kit + Phrases** -> create or upsert skeleton via `travel_cartridge(operation: "create" | "import")`.
2. **Waypoints + Artworks + Knowledge Tree** -> upsert via `travel_cartridge(operation: "item_upsert", item_type: "waypoint" | "artwork")`.
3. **Tiered Dining + Tiered Hotels** -> upsert via `travel_cartridge(operation: "item_upsert", item_type: "spot")`.
4. **Transit Snapshot + Stay Itineraries (1/3/5/7 days) + Gotchas** -> `update` / `item_upsert`.
5. **Media Assets** -> `item_upsert` with `item_type: "media"`.
6. **Current Events (Pre-trip Volatile Layer)** -> `item_upsert` with `item_type: "event"`, then `update` with `events_refreshed_at: "YYYY-MM-DD"`.

---

## Step 4: Validation & Pre-Trip Refresh

After importing:
1. Call `travel_cartridge(operation: "resolve", id: "<city-id>")` and confirm parent country fields inherited cleanly (`effective_currency`, `effective_survival_kit`, `effective_phrases`, `effective_gotchas`).
2. Verify counts (`waypoint_count`, `artwork_count`, `spot_count`, `itinerary_count`, `event_count`, `media_count`).
3. Before a repeat trip to the same city, ONLY re-run Step 1 (Current Events) + Step 3.6 to refresh `events` and `events_refreshed_at`, leaving Evergreen history and personal Notebook entries untouched.
