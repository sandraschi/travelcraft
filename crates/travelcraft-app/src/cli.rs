use clap::{Parser, Subcommand};
use travelcraft_core::TravelCraftEngine;

#[derive(Parser)]
#[command(
    name = "travelcraft-cli",
    about = "TravelCraft Local-First Baedeker 2030 CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available country editions
    Countries,
    /// Inspect country edition details (e.g. JP, AT)
    Inspect { country: String },
    /// Plan chrono-optimized route
    Plan { country: String },
    /// Day-plan for a stay length (1/2/3/5/7/14 days)
    Itinerary {
        country: String,
        #[arg(default_value_t = 3)]
        days: u32,
    },
    /// Identify a sight from GPS + hint (offline: Stephansdom, KHM Caravaggio, Klimt Kiss)
    Identify {
        #[arg(default_value_t = 48.2085)]
        lat: f64,
        #[arg(default_value_t = 16.3738)]
        lon: f64,
        #[arg(default_value = "")]
        hint: String,
    },
    /// SightLens capture: identify + log photo with GPS source
    Capture {
        photo: String,
        #[arg(default_value = "manual")]
        gps_source: String,
        #[arg(default_value_t = 48.2085)]
        lat: f64,
        #[arg(default_value_t = 16.3738)]
        lon: f64,
        #[arg(default_value = "")]
        hint: String,
    },
    /// Real places nearby from OpenStreetMap (online; cached 24 h, falls back to the cache)
    Nearby {
        /// Curated category, e.g. bicycle_repair, pharmacy, toilets, cafe
        category: Option<String>,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        value: Option<String>,
        #[arg(long, allow_hyphen_values = true)]
        lat: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        lon: Option<f64>,
        /// Major AT or JP city name, e.g. Wien, Graz, Tokyo, Kyoto
        #[arg(long)]
        place: Option<String>,
        #[arg(long)]
        radius: Option<u32>,
        #[arg(long)]
        limit: Option<usize>,
        /// Skip a fresh cache entry and query live
        #[arg(long)]
        refresh: bool,
    },
    /// List the durable travel journal, newest first
    Journal {
        #[arg(default_value_t = 20)]
        limit: usize,
    },
    /// Chat about a sight (offline blob-grounded; online LLM via Ollama when reachable)
    Chat {
        #[arg(long, default_value = "caravaggio")]
        about: String,
        question: String,
        #[arg(default_value_t = 48.2039)]
        lat: f64,
        #[arg(default_value_t = 16.3619)]
        lon: f64,
    },
    /// Serve a mobile-friendly dossier site for iPad (pair with `tailscale funnel <port>`)
    Serve {
        #[arg(default_value_t = 14130)]
        port: u16,
    },
    /// Publish a sight dossier to the fleet Intel Hub (iPad reads it at :11027)
    Publish {
        #[arg(long, default_value = "caravaggio")]
        about: String,
        #[arg(long, default_value = "http://127.0.0.1:11027")]
        hub_url: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let engine = TravelCraftEngine::open_default()?;

    match cli.command {
        Commands::Countries => {
            let editions = engine.list_countries();
            println!("{}", serde_json::to_string_pretty(&editions)?);
        }
        Commands::Inspect { country } => {
            let edition = engine.get_country(&country)?;
            println!("{}", serde_json::to_string_pretty(&edition)?);
        }
        Commands::Plan { country } => {
            let plan = engine.plan_optimized_route(&country)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Commands::Itinerary { country, days } => {
            let itin = engine.plan_itinerary(&country, days)?;
            println!("{}", serde_json::to_string_pretty(&itin)?);
        }
        Commands::Identify { lat, lon, hint } => match engine.identify_sight(lat, lon, &hint) {
            Ok(hit) => println!("{}", serde_json::to_string_pretty(&hit)?),
            Err(e) => eprintln!("no match: {e}"),
        },
        Commands::Capture {
            photo,
            gps_source,
            lat,
            lon,
            hint,
        } => {
            let res = engine.sightlens_capture(&photo, Some(lat), Some(lon), &gps_source, &hint)?;
            println!("{}", serde_json::to_string_pretty(&res)?);
        }
        Commands::Nearby {
            category,
            key,
            value,
            lat,
            lon,
            place,
            radius,
            limit,
            refresh,
        } => {
            let req = travelcraft_core::nearby::NearbyRequest {
                category,
                key,
                value,
                lat,
                lon,
                place,
                radius_m: radius,
                limit,
                refresh,
            };
            println!("{}", serde_json::to_string_pretty(&engine.nearby(&req)?)?);
        }
        Commands::Journal { limit } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&engine.list_journal(limit)?)?
            );
        }
        Commands::Chat {
            about,
            question,
            lat,
            lon,
        } => {
            let sight = engine
                .identify_sight(lat, lon, &about)
                .or_else(|_| engine.identify_sight(48.2039, 16.3619, "caravaggio"))
                .unwrap_or_else(|_| travelcraft_baedeker::SightHit {
                    id: "fallback_sight".to_string(),
                    name: "Madonna of the Rosary - Caravaggio".to_string(),
                    kind: travelcraft_baedeker::SightKind::Artwork,
                    country_code: "AT".to_string(),
                    lat: 48.2039,
                    lon: 16.3619,
                    distance_m: 0.0,
                    confidence: 0.95,
                    headline: "Caravaggio in the KHM".to_string(),
                    body: "Caravaggio in the KHM".to_string(),
                    practical: "KHM Gemaldegalerie".to_string(),
                });
            let ctx = engine.sight_chat_context(&sight);
            let out = serde_json::json!({
                "sight": sight.name,
                "system_prompt": ctx.system_prompt(),
                "offline_answer": engine.sight_chat_offline(&sight, &question),
                "map_blob": ctx.map_blob,
                "transit_blob": ctx.transit_blob,
                "note": "Online answer: run the GUI Sight Chat tab (Ollama :11434) or POST system_prompt+question to your LLM.",
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        Commands::Serve { port } => {
            run_mobile_server(&engine, port)?;
        }
        Commands::Publish { about, hub_url } => {
            let sight = engine
                .identify_sight(48.2039, 16.3619, &about)
                .or_else(|_| engine.identify_sight(48.2085, 16.3738, &about))
                .or_else(|_| engine.identify_sight(48.2039, 16.3619, "caravaggio"))
                .unwrap_or_else(|_| travelcraft_baedeker::SightHit {
                    id: "fallback_sight".to_string(),
                    name: "Madonna of the Rosary - Caravaggio".to_string(),
                    kind: travelcraft_baedeker::SightKind::Artwork,
                    country_code: "AT".to_string(),
                    lat: 48.2039,
                    lon: 16.3619,
                    distance_m: 0.0,
                    confidence: 0.95,
                    headline: "Caravaggio in the KHM".to_string(),
                    body: "Caravaggio in the KHM".to_string(),
                    practical: "KHM Gemaldegalerie".to_string(),
                });
            let ctx = engine.sight_chat_context(&sight);
            let html = mobile_sight_page(&engine, &sight);
            let payload = serde_json::json!({
                "title": format!("TravelCraft - {}", sight.name),
                "source": "travelcraft",
                "html": html,
                "summary": sight.headline,
                "tags": ["travelcraft", "fleet"],
            });
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()?;
            let url = format!("{}/api/reports/publish", hub_url.trim_end_matches('/'));
            let resp = client.post(&url).json(&payload).send()?;
            if !resp.status().is_success() {
                eprintln!("hub publish failed: HTTP {}", resp.status());
                std::process::exit(1);
            }
            let body: serde_json::Value = resp.json()?;
            println!("{}", serde_json::to_string_pretty(&body)?);
            if let Some(path) = body.get("url_path").and_then(|v| v.as_str()) {
                println!("iPad: open http://<goliath>:11027{path} (Tailscale tailnet) or via funnel URL.");
            }
            let _ = ctx;
        }
    }
    Ok(())
}

/// Escape for the tiny built-in HTML renderer.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Mobile-first dossier page (large type, ultra contrast, iPad-friendly).
fn mobile_sight_page(engine: &TravelCraftEngine, sight: &travelcraft_baedeker::SightHit) -> String {
    let ctx = engine.sight_chat_context(sight);
    let mut itin = String::new();
    for days in [1u32, 3, 7] {
        let cc = sight.country_code.as_str();
        if let Ok(plan) = engine.plan_itinerary(cc, days) {
            itin.push_str(&format!(
                "<h3>{days}-day ({})</h3><ul>",
                esc(&plan.pace_label)
            ));
            for d in &plan.days {
                itin.push_str(&format!(
                    "<li><b>Day {}:</b> {}</li>",
                    d.day,
                    esc(&d
                        .waypoint_ids
                        .iter()
                        .take(4)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", "))
                ));
            }
            itin.push_str("</ul>");
        }
    }
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
        <title>{}</title>\
        <style>body{{background:#050810;color:#fff;font-family:system-ui,sans-serif;margin:0;padding:20px;font-size:19px;line-height:1.6}}h1{{font-size:32px;color:#ffcd69}}h2{{font-size:24px;color:#82f5d2}}h3{{font-size:21px;color:#ffcd69}}.card{{background:#0e1422;border:2px solid #5a82b9;border-radius:10px;padding:16px;margin:14px 0}}.meta{{color:#d0e4fc;font-size:16px}}a{{color:#ffcd69}}</style>\
        </head><body><h1>{}</h1><p class=\"meta\">{}</p>\
        <div class=\"card\"><h2>Why it matters</h2><p>{}</p></div>\
        <div class=\"card\"><h2>Practical</h2><p>{}</p></div>\
        <div class=\"card\"><h2>Map (qgiscraft)</h2><p>{}</p></div>\
        <div class=\"card\"><h2>Transit (GTFS snapshot)</h2><p>{}</p></div>\
        <div class=\"card\"><h2>Stay itineraries</h2>{}</div>\
        <p class=\"meta\">Offline Baedeker blob + qgiscraft map + gtfs-mcp transit snapshot. Live: chat in the TravelCraft desktop app (Sight Chat tab).</p>\
        </body></html>",
        esc(&sight.name),
        esc(&sight.name),
        esc(&sight.headline),
        esc(&ctx.info_body),
        esc(&sight.practical),
        esc(&ctx.map_blob),
        esc(&ctx.transit_blob),
        itin,
    )
}

fn mobile_index(engine: &TravelCraftEngine) -> String {
    let mut cards = String::new();
    for cc in ["AT", "JP"] {
        if let Ok(ed) = engine.get_country(cc) {
            cards.push_str(&format!(
                "<div class=\"card\"><h2>{}</h2><p>{}</p><ul>",
                esc(&ed.title),
                esc(&ed.subtitle)
            ));
            for route in &ed.curated_routes {
                for w in &route.waypoints {
                    cards.push_str(&format!(
                        "<li><a href=\"/sight/{}\">{}</a> <span class=\"meta\">{} AD</span></li>",
                        esc(&w.id),
                        esc(&w.name),
                        w.year_ad
                    ));
                }
            }
            cards.push_str("</ul></div>");
        }
    }
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
        <title>TravelCraft Baedeker 2030</title>\
        <style>body{{background:#050810;color:#fff;font-family:system-ui,sans-serif;margin:0;padding:20px;font-size:19px;line-height:1.6}}h1{{font-size:32px;color:#ffcd69}}h2{{font-size:24px;color:#82f5d2}}.card{{background:#0e1422;border:2px solid #5a82b9;border-radius:10px;padding:16px;margin:14px 0}}.meta{{color:#d0e4fc;font-size:16px}}a{{color:#ffcd69;font-size:20px}}</style>\
        </head><body><h1>TravelCraft Baedeker 2030</h1>\
        <p>Offline dossiers for iPad. Start: <a href=\"/sight/at_stephansdom\">Stephansdom</a> - \
        <a href=\"/sight/khm_caravaggio_rosenkranz\">Caravaggio (KHM)</a></p>{cards}\
        <p class=\"meta\">Desktop app: SightLens + Sight Chat. This page: <a href=\"/health\">/health</a></p>\
        </body></html>"
    )
}

/// Minimal mobile HTTP server (std + tokio, no extra deps).
/// iPad path: run this, then `tailscale funnel <port>` (public) or
/// `tailscale serve` (tailnet-only), open the URL on iPad Safari.
fn run_mobile_server(
    _engine: &TravelCraftEngine,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
        println!("[travelcraft] mobile server on http://127.0.0.1:{port}/");
        println!("[travelcraft] iPad via tailnet: http://<goliath-tailnet-name>:{port}/");
        println!("[travelcraft] iPad via funnel (public): tailscale funnel {port}");
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                continue;
            };
            // Rebuild per request (in-memory engine is cheap).
            let eng = match TravelCraftEngine::new_in_memory() {
                Ok(e) => e,
                Err(_) => continue,
            };
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
                let mut lines = tokio::io::BufReader::new(&mut sock).lines();
                let req = lines.next_line().await.ok().flatten().unwrap_or_default();
                let path = req
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("/")
                    .split('?')
                    .next()
                    .unwrap_or("/")
                    .to_string();
                // Drain headers.
                while let Ok(Some(l)) = lines.next_line().await {
                    if l.trim().is_empty() {
                        break;
                    }
                }
                let (status, body) = if path == "/" {
                    ("200 OK", mobile_index(&eng))
                } else if path == "/health" {
                    ("200 OK", "{\"status\":\"ready\",\"app\":\"travelcraft-mobile\"}".to_string())
                } else if let Some(id) = path.strip_prefix("/sight/") {
                    let id = id.trim().trim_end_matches('/');
                    let hit = travelcraft_baedeker::BaedekerCatalog::identify_sight(48.2085, 16.3738, id)
                        .or_else(|| {
                            // Try every waypoint id + artwork keyword as fallback.
                            travelcraft_baedeker::BaedekerCatalog::all_waypoints()
                                .into_iter()
                                .find(|(_, w)| w.id == id)
                                .map(|(cc, w)| travelcraft_baedeker::SightHit {
                                    id: w.id.clone(),
                                    name: w.name.clone(),
                                    kind: travelcraft_baedeker::SightKind::Landmark,
                                    country_code: cc,
                                    lat: w.lat,
                                    lon: w.lon,
                                    distance_m: 0.0,
                                    confidence: 1.0,
                                    headline: w.name.clone(),
                                    body: w.historic_narrative.clone(),
                                    practical: w.visitor_info.clone(),
                                })
                        });
                    match hit {
                        Some(h) => ("200 OK", mobile_sight_page(&eng, &h)),
                        None => ("404 NOT FOUND", "<h1>Unknown sight</h1><p><a href=\"/\">Back</a></p>".to_string()),
                    }
                } else {
                    ("404 NOT FOUND", "<h1>Not found</h1><p><a href=\"/\">Back</a></p>".to_string())
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(body.as_bytes()).await;
            });
        }
        #[allow(unreachable_code)]
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
