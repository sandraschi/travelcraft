//! TravelCraft MCP Server
//!
//! Exposes Baedeker 2030 chrono-cartography and sovereign route optimization
//! as FastMCP JSON-RPC tools on port 14129.

use std::sync::Arc;
use serde_json::json;
use travelcraft_core::TravelCraftEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = 14129;
    println!("[travelcraft-mcp] Initializing Baedeker 2030 FastMCP engine on port {}...", port);

    let engine = Arc::new(TravelCraftEngine::new_in_memory()?);
    let countries = engine.list_countries();
    println!("[travelcraft-mcp] Loaded {} sovereign country editions.", countries.len());

    let status = json!({
        "status": "ready",
        "mcp_server": "travelcraft",
        "port": port,
        "editions": ["JP", "AT"],
        "capabilities": ["ChronoCartography", "SensoryAtmosphere", "OfflineVectorRouting", "SovereignJournal"]
    });
    println!("[travelcraft-mcp] Status: {}", serde_json::to_string_pretty(&status)?);
    Ok(())
}
