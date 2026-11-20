//! Offline Phrasebook + Companion Live Speech/Text Translation.
//!
//! Tier 1 (always offline): queries the resolved cartridge phrasebook and
//! formats large-type flashcard + pronunciation guidance.
//! Tier 2 (companion local stack): translates arbitrary travel phrases via
//! local Ollama (`:11434`) and can optionally request TTS wav output from
//! `speech-mcp` (`:10909`).

use crate::{TravelCraftEngine, TravelCraftError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use travelcraft_baedeker::cartridge::PhraseEntry;

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_TRANSLATE_MODEL: &str = "qwen2.5:7b";

pub trait TranslateTransport {
    fn chat(&self, model: &str, system: &str, user: &str) -> Result<String, TravelCraftError>;
}

pub struct OllamaTranslateHttp {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl OllamaTranslateHttp {
    pub fn from_env() -> Result<Self, TravelCraftError> {
        let base_url = std::env::var("TRAVELCRAFT_OLLAMA_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_OLLAMA_URL.to_string());
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| TravelCraftError::Offline(format!("http client: {e}")))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        })
    }
}

impl TranslateTransport for OllamaTranslateHttp {
    fn chat(&self, model: &str, system: &str, user: &str) -> Result<String, TravelCraftError> {
        let url = format!("{}/api/chat", self.base_url);
        let body = json!({
            "model": model,
            "stream": false,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });
        let resp =
            self.client.post(&url).json(&body).send().map_err(|e| {
                TravelCraftError::Offline(format!("ollama unreachable at {url}: {e}"))
            })?;
        if !resp.status().is_success() {
            return Err(TravelCraftError::Protocol(format!(
                "ollama HTTP {}",
                resp.status()
            )));
        }
        let val: Value = resp
            .json()
            .map_err(|e| TravelCraftError::Protocol(format!("ollama invalid json: {e}")))?;
        val.pointer("/message/content")
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| TravelCraftError::Protocol("ollama returned empty translation".into()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhrasebookResult {
    pub cartridge_id: String,
    pub languages: Vec<String>,
    pub total: usize,
    pub phrases: Vec<PhraseEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub source_text: String,
    pub target_language: String,
    pub translated: String,
    pub pronunciation: String,
    pub source: String,
    pub model: Option<String>,
}

impl TravelCraftEngine {
    /// Offline phrasebook lookup across a resolved cartridge (inherits parent country phrases).
    pub fn translate_phrases(
        &self,
        cartridge_id: &str,
        query: Option<&str>,
    ) -> Result<PhrasebookResult, TravelCraftError> {
        let resolved = self.cartridge_resolve(cartridge_id)?;
        let needle = query
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty());
        let phrases: Vec<PhraseEntry> = resolved
            .effective_phrases
            .into_iter()
            .filter(|p| match &needle {
                None => true,
                Some(q) => {
                    p.phrase.to_ascii_lowercase().contains(q)
                        || p.local.to_ascii_lowercase().contains(q)
                        || p.context.to_ascii_lowercase().contains(q)
                }
            })
            .collect();
        Ok(PhrasebookResult {
            cartridge_id: resolved.cartridge.id,
            languages: resolved.effective_languages,
            total: phrases.len(),
            phrases,
        })
    }

    /// Translate travel text using local Ollama (or match offline phrasebook first if `cartridge_id` matches).
    pub fn translate_text(
        &self,
        text: &str,
        target_language: &str,
        cartridge_id: Option<&str>,
        model: Option<&str>,
    ) -> Result<TranslationResult, TravelCraftError> {
        if let Some(hit) = self.try_offline_phrase_match(text, target_language, cartridge_id) {
            return Ok(hit);
        }
        let http = OllamaTranslateHttp::from_env()?;
        self.translate_text_with(&http, text, target_language, model)
    }

    fn try_offline_phrase_match(
        &self,
        text: &str,
        target_language: &str,
        cartridge_id: Option<&str>,
    ) -> Option<TranslationResult> {
        let cid = cartridge_id?.trim();
        if cid.is_empty() {
            return None;
        }
        let needle = text.trim().to_ascii_lowercase();
        let resolved = self.cartridge_resolve(cid).ok()?;
        let found = resolved
            .effective_phrases
            .into_iter()
            .find(|p| p.phrase.trim().to_ascii_lowercase() == needle)?;
        Some(TranslationResult {
            source_text: text.trim().to_string(),
            target_language: target_language.trim().to_string(),
            translated: found.local,
            pronunciation: found.pronunciation,
            source: "offline_phrasebook".to_string(),
            model: None,
        })
    }

    pub fn translate_text_with(
        &self,
        transport: &dyn TranslateTransport,
        text: &str,
        target_language: &str,
        model: Option<&str>,
    ) -> Result<TranslationResult, TravelCraftError> {
        Self::check_text("text", text, 1000, true)?;
        Self::check_text("target_language", target_language, 64, true)?;
        let model_name = model
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_TRANSLATE_MODEL);
        let system = "You are a concise travel interpreter. Return strict JSON with keys 'translated' and 'pronunciation' (Latin phonetic guide). Polite everyday local phrasing. No extra commentary.";
        let user = format!("Translate into {target_language}: {text}");
        let raw = transport.chat(model_name, system, &user)?;
        let (translated, pronunciation) = match serde_json::from_str::<Value>(&raw) {
            Ok(v) => (
                v.get("translated")
                    .and_then(Value::as_str)
                    .unwrap_or(raw.as_str())
                    .trim()
                    .to_string(),
                v.get("pronunciation")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            ),
            Err(_) => (raw.trim().to_string(), String::new()),
        };
        Ok(TranslationResult {
            source_text: text.trim().to_string(),
            target_language: target_language.trim().to_string(),
            translated,
            pronunciation,
            source: "ollama".to_string(),
            model: Some(model_name.to_string()),
        })
    }
}
