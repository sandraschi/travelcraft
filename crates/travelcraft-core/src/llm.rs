use crate::skills::{skill_read, skills_list};
use crate::{nearby::NearbyRequest, TravelCraftEngine, TravelJournalEntry};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LlmProvider {
    #[default]
    Ollama,
    LmStudio,
    Gemini,
    OpenAiCompat,
    Anthropic,
}

impl LlmProvider {
    pub const ALL: [Self; 5] = [
        Self::Ollama,
        Self::LmStudio,
        Self::Gemini,
        Self::OpenAiCompat,
        Self::Anthropic,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Ollama => "Ollama (Local)",
            Self::LmStudio => "LM Studio (Local)",
            Self::Gemini => "Google Gemini (Cloud)",
            Self::OpenAiCompat => "OpenAI-Compatible",
            Self::Anthropic => "Anthropic",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            Self::Ollama => "http://127.0.0.1:11434",
            Self::LmStudio => "http://127.0.0.1:1234",
            Self::Gemini => "https://generativelanguage.googleapis.com",
            Self::OpenAiCompat => "https://api.openai.com",
            Self::Anthropic => "https://api.anthropic.com",
        }
    }

    pub fn default_model_hint(self) -> &'static str {
        match self {
            Self::Ollama => "",
            Self::LmStudio => "",
            Self::Gemini => "gemini-2.5-flash",
            Self::OpenAiCompat => "gpt-4o-mini",
            Self::Anthropic => "claude-sonnet-4-20250514",
        }
    }

    pub fn is_local(self, base_url: &str) -> bool {
        match self {
            Self::Ollama | Self::LmStudio => true,
            Self::OpenAiCompat => base_url.contains("localhost") || base_url.contains("127.0.0.1"),
            Self::Gemini | Self::Anthropic => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub user_extra_prompt: String,
    pub tools_enabled: bool,
    pub save_transcripts: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Ollama,
            base_url: "http://127.0.0.1:11434".into(),
            model: String::new(),
            api_key: String::new(),
            user_extra_prompt: String::new(),
            tools_enabled: true,
            save_transcripts: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredModel {
    pub name: String,
    pub parameter_size: String,
    pub quantization: String,
    pub supports_tools: bool,
    pub is_embedding: bool,
    pub is_loaded: bool,
    pub size_gb: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryReport {
    pub ollama_online: bool,
    pub ollama_url: String,
    pub ollama_error: Option<String>,
    pub ollama_models: Vec<DiscoveredModel>,
    pub lm_studio_online: bool,
    pub lm_studio_url: String,
    pub lm_studio_error: Option<String>,
    pub lm_studio_models: Vec<DiscoveredModel>,
    pub env_keys: Vec<(String, bool)>,
    pub recommended_provider: Option<LlmProvider>,
    pub recommended_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_args: Option<Value>,
    pub tool_result: Option<String>,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub struct PendingToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    pub read_only: bool,
}

pub struct PrepromptParts {
    pub full_text: String,
    pub estimated_tokens: usize,
}

pub enum LlmTurnOutput {
    AssistantText(String),
    ToolCalls(Vec<PendingToolCall>),
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("travelcraft")
}

pub fn is_read_only_tool(name: &str) -> bool {
    matches!(
        name,
        "travel_status"
            | "travel_countries"
            | "travel_inspect"
            | "travel_knowledge_tree"
            | "travel_plan_route"
            | "travel_itinerary"
            | "travel_identify_sight"
            | "travel_sight_chat"
            | "travel_journal_list"
            | "travel_nearby"
            | "travel_geocode"
            | "travel_transit"
            | "travelcraft_skill_read"
    )
}

pub fn load_llm_config() -> LlmConfig {
    let path = data_dir().join("chat_config.json");
    if let Ok(raw) = std::fs::read_to_string(path) {
        if let Ok(cfg) = serde_json::from_str::<LlmConfig>(&raw) {
            return cfg;
        }
    }
    LlmConfig::default()
}

pub fn save_llm_config(cfg: &LlmConfig) -> Result<(), String> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create dir failed: {e}"))?;
    let raw = serde_json::to_string_pretty(cfg).map_err(|e| format!("serialize failed: {e}"))?;
    std::fs::write(dir.join("chat_config.json"), raw).map_err(|e| format!("write failed: {e}"))
}

pub fn append_chat_transcript(msg: &ChatMessage) {
    let dir = data_dir().join("chat");
    let _ = std::fs::create_dir_all(&dir);
    let date = Local::now().format("%Y-%m-%d").to_string();
    let path = dir.join(format!("{date}.jsonl"));
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        if let Ok(line) = serde_json::to_string(msg) {
            let _ = writeln!(file, "{line}");
        }
    }
}

pub fn chat_tool_catalog() -> Vec<Value> {
    let t = |name: &str, desc: &str, props: Value, req: &[&str]| {
        json!({
            "name": name,
            "description": desc,
            "inputSchema": { "type": "object", "properties": props, "required": req }
        })
    };
    vec![
        t(
            "travel_status",
            "Server status, editions, and journal count.",
            json!({}),
            &[],
        ),
        t(
            "travel_countries",
            "List available Baedeker country editions (AT, JP).",
            json!({}),
            &[],
        ),
        t(
            "travel_inspect",
            "Inspect full country briefing by country code (AT or JP).",
            json!({ "country": { "type": "string" } }),
            &["country"],
        ),
        t(
            "travel_knowledge_tree",
            "Knowledge tree for AT or JP.",
            json!({ "country": { "type": "string" } }),
            &["country"],
        ),
        t(
            "travel_plan_route",
            "Optimized waypoint route for AT or JP.",
            json!({ "country": { "type": "string" } }),
            &["country"],
        ),
        t(
            "travel_itinerary",
            "Day-by-day stay itinerary (1,2,3,5,7,14 days).",
            json!({ "country": { "type": "string" }, "days": { "type": "integer" } }),
            &["country"],
        ),
        t(
            "travel_identify_sight",
            "Identify sight/artwork from lat/lon and hint.",
            json!({ "lat": { "type": "number" }, "lon": { "type": "number" }, "hint": { "type": "string" } }),
            &[],
        ),
        t(
            "travel_sight_chat",
            "Get blob-grounded Q&A context for a sight.",
            json!({ "question": { "type": "string" }, "about": { "type": "string" } }),
            &["question"],
        ),
        t(
            "travel_journal_list",
            "List recent travel journal entries.",
            json!({ "limit": { "type": "integer" } }),
            &[],
        ),
        t(
            "travel_journal_add",
            "Log a new travel journal entry.",
            json!({ "waypoint_id": { "type": "string" }, "reflection": { "type": "string" } }),
            &["reflection"],
        ),
        t(
            "travel_nearby",
            "Find nearby cafes, museums, or stops around a place or lat/lon.",
            json!({ "category": { "type": "string" }, "place": { "type": "string" }, "limit": { "type": "integer" } }),
            &[],
        ),
        t(
            "travel_geocode",
            "Offline geocoder for a place name.",
            json!({ "query": { "type": "string" } }),
            &["query"],
        ),
        t(
            "travel_transit",
            "Transit status, stops, departures, or live data.",
            json!({ "operation": { "type": "string" }, "query": { "type": "string" } }),
            &["operation"],
        ),
        t(
            "travelcraft_skill_read",
            "Read a bundled TravelCraft domain skill by name.",
            json!({ "name": { "type": "string" } }),
            &["name"],
        ),
    ]
}

pub fn build_preprompt(
    engine: &TravelCraftEngine,
    active_country: &str,
    active_view: &str,
    user_extra: &str,
) -> PrepromptParts {
    let header = "You are the TravelCraft Baedeker 2030 Cultural & Transit Advisor. Answer in the user's language (German or English), concisely and accurately. Never invent transit departures or sightings; use tools when inspecting editions, routes, sights, or journal entries, and ask confirmation before mutating tools.\n\n";

    let mut skills_txt = String::from("## Bundled Domain Skills\n");
    for s in skills_list() {
        let content = skill_read(s.name)
            .ok()
            .and_then(|v| v["content"].as_str().map(str::to_string))
            .unwrap_or_default();
        skills_txt.push_str(&format!("### Skill: {}\n{}\n\n", s.name, content));
    }
    if skills_txt.len() > 28_000 {
        skills_txt.truncate(28_000);
        skills_txt.push_str("...[truncated]\n\n");
    }

    let mut tool_cat = String::from("## Available TravelCraft Tools\n");
    for t in chat_tool_catalog() {
        let name = t["name"].as_str().unwrap_or("");
        let desc = t["description"].as_str().unwrap_or("");
        let ro = if is_read_only_tool(name) {
            "read-only"
        } else {
            "mutating (needs confirm)"
        };
        tool_cat.push_str(&format!("- `{name}` [{ro}]: {desc}\n"));
    }

    let editions: Vec<String> = engine
        .list_countries()
        .into_iter()
        .map(|c| c.country_code)
        .collect();
    let jcount = engine.journal_count().unwrap_or(0);
    let status_sec = format!(
        "\n## Live TravelCraft Status\nEditions: {:?} | Active country: {} | Journal entries: {}\n",
        editions, active_country, jcount
    );

    let selection = format!("\n## Active UI View\n{active_view}\n");
    let extra = if user_extra.trim().is_empty() {
        String::new()
    } else {
        let mut e = format!("\n## User Instructions\n{user_extra}\n");
        if e.len() > 2_000 {
            e.truncate(2_000);
        }
        e
    };

    let full_text = format!("{header}{skills_txt}{tool_cat}{status_sec}{selection}{extra}");
    let estimated_tokens = (full_text.len() / 4).max(1);
    PrepromptParts {
        full_text,
        estimated_tokens,
    }
}

pub fn resolve_api_key(cfg: &LlmConfig) -> Option<String> {
    let manual = cfg.api_key.trim();
    if !manual.is_empty() {
        return Some(manual.to_string());
    }
    let candidates: &[&str] = match cfg.provider {
        LlmProvider::Ollama | LlmProvider::LmStudio => &[],
        LlmProvider::Gemini => &["GEMINI_API_KEY", "GOOGLE_API_KEY"],
        LlmProvider::Anthropic => &["ANTHROPIC_API_KEY"],
        LlmProvider::OpenAiCompat => {
            let lower = cfg.base_url.to_lowercase();
            if lower.contains("openrouter.ai") {
                &["OPENROUTER_API_KEY", "OPENAI_API_KEY"]
            } else if lower.contains("groq.com") {
                &["GROQ_API_KEY", "OPENAI_API_KEY"]
            } else {
                &["OPENAI_API_KEY"]
            }
        }
    };
    for var in candidates {
        if let Ok(val) = std::env::var(var) {
            if !val.trim().is_empty() {
                return Some(val.trim().to_string());
            }
        }
    }
    None
}

fn curl_get_json(
    url: &str,
    connect_timeout_secs: u32,
    max_time_secs: u32,
) -> Result<Value, String> {
    let output = Command::new("curl")
        .arg("-sS")
        .arg("--connect-timeout")
        .arg(connect_timeout_secs.to_string())
        .arg("--max-time")
        .arg(max_time_secs.to_string())
        .arg(url)
        .output()
        .map_err(|e| format!("curl launch failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("HTTP/connect failed (exit {:?})", output.status.code())
        } else {
            stderr
        });
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&raw).map_err(|e| format!("Invalid JSON: {e}"))
}

pub fn probe_ollama(base_url: &str) -> Result<Vec<DiscoveredModel>, String> {
    let base = base_url.trim_end_matches('/');
    let tags = curl_get_json(&format!("{base}/api/tags"), 2, 4)?;
    let loaded_names: Vec<String> = curl_get_json(&format!("{base}/api/ps"), 1, 2)
        .ok()
        .and_then(|ps| ps["models"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|m| m["name"].as_str().map(str::to_string))
        .collect();

    let models = tags["models"]
        .as_array()
        .ok_or_else(|| "Ollama /api/tags missing 'models' array".to_string())?;

    let mut out = Vec::new();
    for m in models {
        let name = m["name"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let param = m["details"]["parameter_size"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let quant = m["details"]["quantization_level"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let caps: Vec<&str> = m["capabilities"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let supports_tools = caps.contains(&"tools");
        let is_embedding = name.to_lowercase().contains("embed")
            || (!caps.is_empty() && !caps.contains(&"completion"));
        let size_bytes = m["size"].as_u64().unwrap_or(0);
        let size_gb = (size_bytes as f64) / 1_073_741_824.0;
        let is_loaded = loaded_names.iter().any(|l| l == &name);

        out.push(DiscoveredModel {
            name,
            parameter_size: param,
            quantization: quant,
            supports_tools,
            is_embedding,
            is_loaded,
            size_gb,
        });
    }
    Ok(out)
}

pub fn probe_openai_models(base_url: &str) -> Result<Vec<DiscoveredModel>, String> {
    let base = base_url.trim_end_matches('/');
    let v = curl_get_json(&format!("{base}/v1/models"), 2, 4)?;
    let data = v["data"]
        .as_array()
        .ok_or_else(|| "/v1/models missing 'data' array".to_string())?;
    let mut out = Vec::new();
    for m in data {
        let name = m["id"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let is_embedding = name.to_lowercase().contains("embed");
        out.push(DiscoveredModel {
            name,
            parameter_size: String::new(),
            quantization: String::new(),
            supports_tools: true,
            is_embedding,
            is_loaded: true,
            size_gb: 0.0,
        });
    }
    Ok(out)
}

pub fn pick_best_ollama_model(models: &[DiscoveredModel]) -> Option<String> {
    let chat_models: Vec<&DiscoveredModel> = models.iter().filter(|m| !m.is_embedding).collect();
    if let Some(m) = chat_models.iter().find(|m| m.is_loaded && m.supports_tools) {
        return Some(m.name.clone());
    }
    if let Some(m) = chat_models.iter().find(|m| m.is_loaded) {
        return Some(m.name.clone());
    }
    if let Some(m) = chat_models
        .iter()
        .find(|m| m.supports_tools && m.size_gb > 0.5 && m.size_gb <= 10.0)
    {
        return Some(m.name.clone());
    }
    if let Some(m) = chat_models.iter().find(|m| m.supports_tools) {
        return Some(m.name.clone());
    }
    chat_models.first().map(|m| m.name.clone())
}

pub fn discover_llms(ollama_url: &str, lm_studio_url: &str) -> DiscoveryReport {
    let ollama_base = if ollama_url.trim().is_empty() {
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://127.0.0.1:11434".into())
    } else {
        ollama_url.trim().to_string()
    };
    let lm_base = if lm_studio_url.trim().is_empty() {
        std::env::var("LM_STUDIO_URL").unwrap_or_else(|_| "http://127.0.0.1:1234".into())
    } else {
        lm_studio_url.trim().to_string()
    };

    let mut report = DiscoveryReport {
        ollama_url: ollama_base.clone(),
        lm_studio_url: lm_base.clone(),
        ..DiscoveryReport::default()
    };

    match probe_ollama(&ollama_base) {
        Ok(models) => {
            report.ollama_online = true;
            report.ollama_models = models;
        }
        Err(e) => {
            report.ollama_error = Some(e);
        }
    }

    match probe_openai_models(&lm_base) {
        Ok(models) => {
            report.lm_studio_online = true;
            report.lm_studio_models = models;
        }
        Err(e) => {
            report.lm_studio_error = Some(e);
        }
    }

    let key_names = [
        "GEMINI_API_KEY",
        "GOOGLE_API_KEY",
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENROUTER_API_KEY",
        "GROQ_API_KEY",
    ];
    for k in key_names {
        let is_set = std::env::var(k)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        report.env_keys.push((k.to_string(), is_set));
    }

    if let Some(m) = pick_best_ollama_model(&report.ollama_models) {
        report.recommended_provider = Some(LlmProvider::Ollama);
        report.recommended_model = Some(m);
    } else if let Some(m) = report.lm_studio_models.iter().find(|m| !m.is_embedding) {
        report.recommended_provider = Some(LlmProvider::LmStudio);
        report.recommended_model = Some(m.name.clone());
    } else if report
        .env_keys
        .iter()
        .any(|(k, ok)| *ok && (k == "GEMINI_API_KEY" || k == "GOOGLE_API_KEY"))
    {
        report.recommended_provider = Some(LlmProvider::Gemini);
        report.recommended_model = Some("gemini-2.5-flash".into());
    } else if report
        .env_keys
        .iter()
        .any(|(k, ok)| *ok && k == "OPENAI_API_KEY")
    {
        report.recommended_provider = Some(LlmProvider::OpenAiCompat);
        report.recommended_model = Some("gpt-4o-mini".into());
    } else if report
        .env_keys
        .iter()
        .any(|(k, ok)| *ok && k == "ANTHROPIC_API_KEY")
    {
        report.recommended_provider = Some(LlmProvider::Anthropic);
        report.recommended_model = Some("claude-sonnet-4-20250514".into());
    }

    report
}

pub fn test_llm_connection(cfg: &LlmConfig) -> Result<String, String> {
    let mut test_cfg = cfg.clone();
    test_cfg.tools_enabled = false;
    let started = Instant::now();
    let msg = ChatMessage {
        role: "user".into(),
        content: "Antworte nur mit: OK".into(),
        tool_name: None,
        tool_args: None,
        tool_result: None,
        is_error: false,
    };
    let out = call_llm_turn(&test_cfg, "You are a connectivity test.", &[msg])?;
    let ms = started.elapsed().as_millis();
    match out {
        LlmTurnOutput::AssistantText(txt) => Ok(format!("OK ({ms} ms): {txt}")),
        LlmTurnOutput::ToolCalls(_) => Ok(format!("OK ({ms} ms): tool call")),
    }
}

/// Call the configured LLM provider via `curl` passing body via stdin.
/// Never returns invented fallback text; errors are returned as `Err`.
pub fn call_llm_turn(
    cfg: &LlmConfig,
    preprompt: &str,
    history: &[ChatMessage],
) -> Result<LlmTurnOutput, String> {
    let mut effective_model = cfg.model.trim().to_string();
    if effective_model.is_empty() {
        match cfg.provider {
            LlmProvider::Ollama => {
                if let Ok(models) = probe_ollama(&cfg.base_url) {
                    if let Some(best) = pick_best_ollama_model(&models) {
                        effective_model = best;
                    }
                }
            }
            LlmProvider::LmStudio => {
                if let Ok(models) = probe_openai_models(&cfg.base_url) {
                    if let Some(first) = models.into_iter().find(|m| !m.is_embedding) {
                        effective_model = first.name;
                    }
                }
            }
            LlmProvider::Gemini | LlmProvider::OpenAiCompat | LlmProvider::Anthropic => {
                effective_model = cfg.provider.default_model_hint().to_string();
            }
        }
    }

    if effective_model.is_empty() {
        return Err(
            "Kein Modell ausgewählt und kein lokales Chat-Modell gefunden. Bitte in 'LLM & Settings' ein Modell wählen.".into(),
        );
    }

    let api_key = resolve_api_key(cfg);

    let (url, body, auth_header) = match cfg.provider {
        LlmProvider::Ollama => {
            let base = cfg.base_url.trim_end_matches('/');
            let url = format!("{base}/api/chat");
            let mut msgs = vec![json!({ "role": "system", "content": preprompt })];
            for m in history {
                msgs.push(json!({ "role": m.role, "content": m.content }));
            }
            let mut payload = json!({
                "model": effective_model,
                "messages": msgs,
                "stream": false,
                "think": false,
                "options": { "num_ctx": 16384 }
            });
            if cfg.tools_enabled {
                let tool_defs: Vec<Value> = chat_tool_catalog()
                    .into_iter()
                    .map(|t| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": t["name"],
                                "description": t["description"],
                                "parameters": t["inputSchema"]
                            }
                        })
                    })
                    .collect();
                payload["tools"] = Value::Array(tool_defs);
            }
            (url, payload.to_string(), None)
        }
        LlmProvider::LmStudio | LlmProvider::OpenAiCompat => {
            if cfg.provider == LlmProvider::OpenAiCompat
                && !cfg.provider.is_local(&cfg.base_url)
                && api_key.is_none()
            {
                return Err(
                    "API-Key fehlt für OpenAI-Compat Cloud-Provider (BYOK in Settings eintragen oder Env-Key setzen)."
                        .into(),
                );
            }
            let base = cfg.base_url.trim_end_matches('/');
            let url = format!("{base}/v1/chat/completions");
            let mut msgs = vec![json!({ "role": "system", "content": preprompt })];
            for m in history {
                let role = if m.role == "tool" { "user" } else { &m.role };
                msgs.push(json!({ "role": role, "content": m.content }));
            }
            let mut payload = json!({
                "model": effective_model,
                "messages": msgs,
                "stream": false
            });
            if cfg.tools_enabled {
                let tool_defs: Vec<Value> = chat_tool_catalog()
                    .into_iter()
                    .map(|t| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": t["name"],
                                "description": t["description"],
                                "parameters": t["inputSchema"]
                            }
                        })
                    })
                    .collect();
                payload["tools"] = Value::Array(tool_defs);
            }
            let hdr = api_key.map(|k| format!("Authorization: Bearer {k}"));
            (url, payload.to_string(), hdr)
        }
        LlmProvider::Gemini => {
            let key = api_key.ok_or_else(|| {
                "Gemini API-Key fehlt. Bitte in 'LLM & Settings' eintragen oder GEMINI_API_KEY setzen.".to_string()
            })?;
            let base = cfg.base_url.trim_end_matches('/');
            let url = format!("{base}/v1beta/models/{effective_model}:generateContent");
            let contents: Vec<Value> = history
                .iter()
                .map(|m| {
                    let role = if m.role == "assistant" {
                        "model"
                    } else {
                        "user"
                    };
                    json!({
                        "role": role,
                        "parts": [{ "text": m.content }]
                    })
                })
                .collect();
            let mut payload = json!({
                "systemInstruction": {
                    "parts": [{ "text": preprompt }]
                },
                "contents": contents
            });
            if cfg.tools_enabled {
                let decls: Vec<Value> = chat_tool_catalog()
                    .into_iter()
                    .map(|t| {
                        json!({
                            "name": t["name"],
                            "description": t["description"],
                            "parameters": t["inputSchema"]
                        })
                    })
                    .collect();
                payload["tools"] = json!([{ "functionDeclarations": decls }]);
            }
            (
                url,
                payload.to_string(),
                Some(format!("x-goog-api-key: {key}")),
            )
        }
        LlmProvider::Anthropic => {
            let key = api_key.ok_or_else(|| {
                "Anthropic API-Key fehlt. Bitte in 'LLM & Settings' eintragen oder ANTHROPIC_API_KEY setzen.".to_string()
            })?;
            let url = "https://api.anthropic.com/v1/messages".to_string();
            let msgs: Vec<Value> = history
                .iter()
                .map(|m| {
                    json!({
                        "role": if m.role == "tool" { "user" } else { &m.role },
                        "content": m.content
                    })
                })
                .collect();
            let payload = json!({
                "model": effective_model,
                "max_tokens": 2048,
                "system": preprompt,
                "messages": msgs
            });
            let hdr = format!("x-api-key: {key}");
            (url, payload.to_string(), Some(hdr))
        }
    };

    let mut cmd = Command::new("curl");
    cmd.arg("-sS")
        .arg("--connect-timeout")
        .arg("5")
        .arg("--max-time")
        .arg("120")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json");

    if cfg.provider == LlmProvider::Anthropic {
        cmd.arg("-H").arg("anthropic-version: 2023-06-01");
    }

    let mut auth_tmp: Option<PathBuf> = None;
    if let Some(h) = auth_header {
        let dir = data_dir();
        let _ = std::fs::create_dir_all(&dir);
        let tmp = dir.join(format!("curl_hdr_{}.tmp", std::process::id()));
        let escaped = h.replace('\\', "\\\\").replace('"', "\\\"");
        std::fs::write(&tmp, format!("header = \"{escaped}\"\n"))
            .map_err(|e| format!("header config write failed: {e}"))?;
        cmd.arg("-K").arg(&tmp);
        auth_tmp = Some(tmp);
    }

    cmd.arg("--data-binary")
        .arg("@-")
        .arg(&url)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            if let Some(p) = auth_tmp {
                let _ = std::fs::remove_file(p);
            }
            return Err(format!("curl launch failed: {e}"));
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(body.as_bytes());
    }

    let output = child.wait_with_output();
    if let Some(p) = auth_tmp {
        let _ = std::fs::remove_file(p);
    }
    let output = output.map_err(|e| format!("curl wait failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "HTTP request failed (curl exit {:?}): {stderr}",
            output.status.code()
        ));
    }

    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    let v: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid JSON from provider: {e} — raw: {raw}"))?;

    if let Some(err) = v.get("error") {
        return Err(format!("Provider error: {err}"));
    }

    match cfg.provider {
        LlmProvider::Ollama => {
            let msg = &v["message"];
            if let Some(calls) = msg.get("tool_calls").and_then(|c| c.as_array()) {
                if !calls.is_empty() {
                    let mut pending = Vec::new();
                    for (idx, c) in calls.iter().enumerate() {
                        let name = c["function"]["name"].as_str().unwrap_or("").to_string();
                        let args = c["function"]["arguments"].clone();
                        pending.push(PendingToolCall {
                            id: format!("call_{idx}"),
                            read_only: is_read_only_tool(&name),
                            name,
                            arguments: args,
                        });
                    }
                    return Ok(LlmTurnOutput::ToolCalls(pending));
                }
            }
            let content = msg["content"].as_str().unwrap_or("").trim().to_string();
            if content.is_empty() {
                return Err("Leere Antwort vom Modell erhalten.".into());
            }
            Ok(LlmTurnOutput::AssistantText(content))
        }
        LlmProvider::LmStudio | LlmProvider::OpenAiCompat => {
            let msg = &v["choices"][0]["message"];
            if let Some(calls) = msg.get("tool_calls").and_then(|c| c.as_array()) {
                if !calls.is_empty() {
                    let mut pending = Vec::new();
                    for (idx, c) in calls.iter().enumerate() {
                        let name = c["function"]["name"].as_str().unwrap_or("").to_string();
                        let args_val = match &c["function"]["arguments"] {
                            Value::String(s) => serde_json::from_str(s).unwrap_or(json!({})),
                            other => other.clone(),
                        };
                        pending.push(PendingToolCall {
                            id: c["id"]
                                .as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| format!("call_{idx}")),
                            read_only: is_read_only_tool(&name),
                            name,
                            arguments: args_val,
                        });
                    }
                    return Ok(LlmTurnOutput::ToolCalls(pending));
                }
            }
            let content = msg["content"].as_str().unwrap_or("").trim().to_string();
            if content.is_empty() {
                return Err("Leere Antwort vom Modell erhalten.".into());
            }
            Ok(LlmTurnOutput::AssistantText(content))
        }
        LlmProvider::Gemini => {
            let parts = v["candidates"][0]["content"]["parts"]
                .as_array()
                .ok_or_else(|| format!("Unerwartete Gemini-Antwort: {raw}"))?;
            let mut pending = Vec::new();
            let mut text_out = String::new();
            for (idx, p) in parts.iter().enumerate() {
                if let Some(fc) = p.get("functionCall") {
                    let name = fc["name"].as_str().unwrap_or("").to_string();
                    let args = fc["args"].clone();
                    pending.push(PendingToolCall {
                        id: format!("gemini_call_{idx}"),
                        read_only: is_read_only_tool(&name),
                        name,
                        arguments: args,
                    });
                } else if let Some(t) = p.get("text").and_then(|x| x.as_str()) {
                    if !text_out.is_empty() {
                        text_out.push('\n');
                    }
                    text_out.push_str(t);
                }
            }
            if !pending.is_empty() {
                return Ok(LlmTurnOutput::ToolCalls(pending));
            }
            let content = text_out.trim().to_string();
            if content.is_empty() {
                return Err("Leere Antwort von Gemini erhalten.".into());
            }
            Ok(LlmTurnOutput::AssistantText(content))
        }
        LlmProvider::Anthropic => {
            let content = v["content"][0]["text"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string();
            if content.is_empty() {
                return Err("Leere Antwort von Anthropic erhalten.".into());
            }
            Ok(LlmTurnOutput::AssistantText(content))
        }
    }
}

pub fn execute_tool_call(
    engine: &TravelCraftEngine,
    name: &str,
    args: &Value,
) -> Result<String, String> {
    let val = match name {
        "travel_status" => json!({
            "editions": engine.list_countries().iter().map(|c| c.country_code.clone()).collect::<Vec<_>>(),
            "journal_entries": engine.journal_count().ok(),
        }),
        "travel_countries" => {
            serde_json::to_value(engine.list_countries()).map_err(|e| e.to_string())?
        }
        "travel_inspect" => {
            let c = args.get("country").and_then(Value::as_str).unwrap_or("AT");
            let v = engine.get_country(c).map_err(|e| e.to_string())?;
            serde_json::to_value(v).map_err(|e| e.to_string())?
        }
        "travel_knowledge_tree" => {
            let c = args.get("country").and_then(Value::as_str).unwrap_or("AT");
            serde_json::to_value(engine.get_knowledge_tree(c)).map_err(|e| e.to_string())?
        }
        "travel_plan_route" => {
            let c = args.get("country").and_then(Value::as_str).unwrap_or("AT");
            let v = engine.plan_optimized_route(c).map_err(|e| e.to_string())?;
            serde_json::to_value(v).map_err(|e| e.to_string())?
        }
        "travel_itinerary" => {
            let c = args.get("country").and_then(Value::as_str).unwrap_or("AT");
            let d = args.get("days").and_then(Value::as_u64).unwrap_or(3) as u32;
            let v = engine.plan_itinerary(c, d).map_err(|e| e.to_string())?;
            serde_json::to_value(v).map_err(|e| e.to_string())?
        }
        "travel_identify_sight" => {
            let lat = args.get("lat").and_then(Value::as_f64).unwrap_or(48.2085);
            let lon = args.get("lon").and_then(Value::as_f64).unwrap_or(16.3738);
            let hint = args.get("hint").and_then(Value::as_str).unwrap_or("");
            let v = engine
                .identify_sight(lat, lon, hint)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(v).map_err(|e| e.to_string())?
        }
        "travel_sight_chat" => {
            let q = args.get("question").and_then(Value::as_str).unwrap_or("");
            let about = args
                .get("about")
                .and_then(Value::as_str)
                .unwrap_or("caravaggio");
            let hit = engine
                .identify_sight(48.2038, 16.3618, about)
                .map_err(|e| e.to_string())?;
            let ctx = engine.sight_chat_context(&hit);
            json!({ "question": q, "sight": hit, "context": ctx })
        }
        "travel_journal_list" => {
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
            let v = engine.list_journal(limit).map_err(|e| e.to_string())?;
            serde_json::to_value(v).map_err(|e| e.to_string())?
        }
        "travel_journal_add" => {
            let reflection = args
                .get("reflection")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing reflection".to_string())?;
            let entry = TravelJournalEntry {
                entry_id: engine.new_entry_id("JOURNAL"),
                waypoint_id: args
                    .get("waypoint_id")
                    .and_then(Value::as_str)
                    .unwrap_or("general")
                    .to_string(),
                timestamp_epoch: chrono::Utc::now().timestamp(),
                user_reflection: reflection.to_string(),
                sensory_notes: String::new(),
                photo_uris: Vec::new(),
                photo_lat: None,
                photo_lon: None,
                photo_gps_source: None,
            };
            let id = entry.entry_id.clone();
            engine.add_journal_entry(entry).map_err(|e| e.to_string())?;
            json!({ "success": true, "entry_id": id })
        }
        "travel_nearby" => {
            let req = NearbyRequest {
                category: args
                    .get("category")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                key: None,
                value: None,
                lat: args.get("lat").and_then(Value::as_f64),
                lon: args.get("lon").and_then(Value::as_f64),
                place: args
                    .get("place")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                radius_m: None,
                limit: args
                    .get("limit")
                    .and_then(Value::as_u64)
                    .map(|n| n as usize),
                refresh: false,
            };
            let v = engine.nearby(&req).map_err(|e| e.to_string())?;
            serde_json::to_value(v).map_err(|e| e.to_string())?
        }
        "travel_geocode" => {
            let q = args
                .get("query")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing query".to_string())?;
            let v = engine.geocode_waypoint(q).map_err(|e| e.to_string())?;
            serde_json::to_value(v).map_err(|e| e.to_string())?
        }
        "travel_transit" => {
            let req = crate::fleet::TransitRequest {
                operation: args
                    .get("operation")
                    .and_then(Value::as_str)
                    .unwrap_or("status")
                    .to_string(),
                query: args
                    .get("query")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                stop_id: args
                    .get("stop_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                rbl: args.get("rbl").and_then(Value::as_str).map(str::to_string),
                feed_id: args
                    .get("feed_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                limit: args
                    .get("limit")
                    .and_then(Value::as_u64)
                    .map(|n| n as usize),
                sight_id: args
                    .get("sight_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            };
            let v = crate::fleet::transit(&req).map_err(|e| e.to_string())?;
            serde_json::to_value(v).map_err(|e| e.to_string())?
        }
        "travelcraft_skill_read" => {
            let n = args
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing name".to_string())?;
            skill_read(n)?
        }
        other => return Err(format!("unknown tool: {other}")),
    };
    serde_json::to_string_pretty(&val).map_err(|e| e.to_string())
}
