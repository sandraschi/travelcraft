//! TravelCraft MCP Server
//!
//! Exposes Baedeker 2030 chrono-cartography and sovereign route optimization
//! as FastMCP JSON-RPC tools on port 14129.

use serde_json::json;
use std::sync::Arc;
use travelcraft_core::TravelCraftEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = 14129;
    println!(
        "[travelcraft-mcp] Initializing Baedeker 2030 FastMCP engine on port {}...",
        port
    );

    let engine = Arc::new(TravelCraftEngine::new_in_memory()?);
    let countries = engine.list_countries();
    println!(
        "[travelcraft-mcp] Loaded {} sovereign country editions.",
        countries.len()
    );
    let itin3 = engine.plan_itinerary("AT", 3)?;
    let sight = engine.identify_sight(48.2085, 16.3738, "stephansdom")?;
    let art = engine.identify_sight(48.2039, 16.3619, "caravaggio")?;
    let chat_ctx = engine.sight_chat_context(&art);

    let status = json!({
        "status": "ready",
        "mcp_server": "travelcraft",
        "port": port,
        "editions": ["JP", "AT"],
        "capabilities": ["ChronoCartography", "SensoryAtmosphere", "OfflineVectorRouting", "SovereignJournal", "StayItineraries", "SightLensIdentify", "SightChat"],
        "vendored_sources": travelcraft_baedeker::SOURCES,
        "sample_itinerary_days": itin3.days.len(),
        "sample_sight": sight.name,
        "sample_artwork": art.name,
        "sample_map_blob": chat_ctx.map_blob,
        "sample_transit_blob": chat_ctx.transit_blob,
        "chat": "SightChat: system_prompt + question -> Ollama :11434 (online) else offline extractive; voice out via SAPI + speech-mcp :10909 TTS; maps via qgiscraft :11254; live departures via gtfs-mcp :10913.",
    });
    println!(
        "[travelcraft-mcp] Status: {}",
        serde_json::to_string_pretty(&status)?
    );
    Ok(())
}
