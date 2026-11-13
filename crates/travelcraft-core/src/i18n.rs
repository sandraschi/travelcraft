//! TravelCraft Internationalization (i18n) Engine.
//!
//! Follows the fleet standard: typed locale enums, zero-panic string lookups,
//! multi-language translations (EN, DE, FR, ES, JA, KO, ZH),
//! and cultural Baedeker guide terminology localization.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppLocale {
    #[default]
    En, // English (Global Default)
    De, // German (Austrian / Baedeker heritage native)
    Fr, // French
    Es, // Spanish
    Ja, // Japanese (Tokaido / Edo heritage native)
    Ko, // Korean
    Zh, // Chinese
}

impl AppLocale {
    pub fn all() -> &'static [AppLocale] {
        &[
            AppLocale::En,
            AppLocale::De,
            AppLocale::Fr,
            AppLocale::Es,
            AppLocale::Ja,
            AppLocale::Ko,
            AppLocale::Zh,
        ]
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppLocale::En => "en",
            AppLocale::De => "de",
            AppLocale::Fr => "fr",
            AppLocale::Es => "es",
            AppLocale::Ja => "ja",
            AppLocale::Ko => "ko",
            AppLocale::Zh => "zh",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AppLocale::En => "English (US/UK)",
            AppLocale::De => "Deutsch (DE/AT/CH)",
            AppLocale::Fr => "Français (FR)",
            AppLocale::Es => "Español (ES/LATAM)",
            AppLocale::Ja => "日本語 (JA)",
            AppLocale::Ko => "한국어 (KO)",
            AppLocale::Zh => "中文 (ZH)",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            AppLocale::En => "🇬🇧",
            AppLocale::De => "🇩🇪",
            AppLocale::Fr => "🇫🇷",
            AppLocale::Es => "🇪🇸",
            AppLocale::Ja => "🇯🇵",
            AppLocale::Ko => "🇰🇷",
            AppLocale::Zh => "🇨🇳",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum I18nKey {
    // Navigation Tabs
    TabChronoMap,
    TabWaypointDetail,
    TabSensoryAtmosphere,
    TabRoutePlanner,
    TabItineraries,
    TabSightLens,
    TabSightChat,
    TabJournal,
    TabTransit,

    // Country Editions
    EditionJapan,
    EditionAustria,

    // Actions & Common
    ActionOptimizeRoute,
    ActionAddJournal,
    ActionIdentifySight,
    ActionSendChat,
    StatusReady,
}

pub fn t(key: I18nKey, locale: AppLocale) -> &'static str {
    match (key, locale) {
        // Navigation Tabs
        (I18nKey::TabChronoMap, AppLocale::En) => "🗺️ Chrono-Cartography",
        (I18nKey::TabChronoMap, AppLocale::De) => "🗺️ Chrono-Kartographie",
        (I18nKey::TabChronoMap, AppLocale::Fr) => "🗺️ Chrono-Cartographie",
        (I18nKey::TabChronoMap, AppLocale::Es) => "🗺️ Crono-Cartografía",
        (I18nKey::TabChronoMap, AppLocale::Ja) => "🗺️ 歴史地図 (クロノマップ)",
        (I18nKey::TabChronoMap, AppLocale::Ko) => "🗺️ 연대기 지도",
        (I18nKey::TabChronoMap, AppLocale::Zh) => "🗺️ 历史纪元时空地图",

        (I18nKey::TabWaypointDetail, AppLocale::En) => "📍 Waypoint Dossier",
        (I18nKey::TabWaypointDetail, AppLocale::De) => "📍 Wegpunkt-Dossier",
        (I18nKey::TabWaypointDetail, AppLocale::Fr) => "📍 Dossier d'étape",
        (I18nKey::TabWaypointDetail, AppLocale::Es) => "📍 Expediente de punto",
        (I18nKey::TabWaypointDetail, AppLocale::Ja) => "📍 宿場・名所詳細",
        (I18nKey::TabWaypointDetail, AppLocale::Ko) => "📍 경유지 상세정보",
        (I18nKey::TabWaypointDetail, AppLocale::Zh) => "📍 地标档案详情",

        (I18nKey::TabSensoryAtmosphere, AppLocale::En) => "🌿 Sensory Atmosphere",
        (I18nKey::TabSensoryAtmosphere, AppLocale::De) => "🌿 Sensorische Atmosphäre",
        (I18nKey::TabSensoryAtmosphere, AppLocale::Fr) => "🌿 Atmosphère sensorielle",
        (I18nKey::TabSensoryAtmosphere, AppLocale::Es) => "🌿 Atmósfera sensorial",
        (I18nKey::TabSensoryAtmosphere, AppLocale::Ja) => "🌿 五感の風情・情景",
        (I18nKey::TabSensoryAtmosphere, AppLocale::Ko) => "🌿 오감 분위기",
        (I18nKey::TabSensoryAtmosphere, AppLocale::Zh) => "🌿 感官风土与氛围",

        (I18nKey::TabRoutePlanner, AppLocale::En) => "🧭 Route Optimizer",
        (I18nKey::TabRoutePlanner, AppLocale::De) => "🧭 Routenplaner",
        (I18nKey::TabRoutePlanner, AppLocale::Fr) => "🧭 Optimiseur d'itinéraire",
        (I18nKey::TabRoutePlanner, AppLocale::Es) => "🧭 Optimizador de rutas",
        (I18nKey::TabRoutePlanner, AppLocale::Ja) => "🧭 街道ルート最適化",
        (I18nKey::TabRoutePlanner, AppLocale::Ko) => "🧭 경로 최적화",
        (I18nKey::TabRoutePlanner, AppLocale::Zh) => "🧭 路线规划优化器",

        (I18nKey::TabItineraries, AppLocale::En) => "📅 Curated Itineraries",
        (I18nKey::TabItineraries, AppLocale::De) => "📅 Baedeker-Reisepläne",
        (I18nKey::TabItineraries, AppLocale::Fr) => "📅 Itinéraires recommandés",
        (I18nKey::TabItineraries, AppLocale::Es) => "📅 Itinerarios seleccionados",
        (I18nKey::TabItineraries, AppLocale::Ja) => "📅 厳選日程プラン",
        (I18nKey::TabItineraries, AppLocale::Ko) => "📅 추천 여행 일정",
        (I18nKey::TabItineraries, AppLocale::Zh) => "📅 经典行程规划",

        (I18nKey::TabSightLens, AppLocale::En) => "📷 SightLens AI",
        (I18nKey::TabSightLens, AppLocale::De) => "📷 SightLens KI",
        (I18nKey::TabSightLens, AppLocale::Fr) => "📷 SightLens IA",
        (I18nKey::TabSightLens, AppLocale::Es) => "📷 SightLens IA",
        (I18nKey::TabSightLens, AppLocale::Ja) => "📷 SightLens 史跡認識",
        (I18nKey::TabSightLens, AppLocale::Ko) => "📷 SightLens AI 인식",
        (I18nKey::TabSightLens, AppLocale::Zh) => "📷 SightLens 视觉识别",

        (I18nKey::TabSightChat, AppLocale::En) => "💬 Baedeker Voice Chat",
        (I18nKey::TabSightChat, AppLocale::De) => "💬 Baedeker Sprach-Chat",
        (I18nKey::TabSightChat, AppLocale::Fr) => "💬 Guide vocal Baedeker",
        (I18nKey::TabSightChat, AppLocale::Es) => "💬 Chat de voz Baedeker",
        (I18nKey::TabSightChat, AppLocale::Ja) => "💬 ベデカー音声案内",
        (I18nKey::TabSightChat, AppLocale::Ko) => "💬 베데커 음성 안내",
        (I18nKey::TabSightChat, AppLocale::Zh) => "💬 贝德克智能导游问答",

        (I18nKey::TabJournal, AppLocale::En) => "📖 Travel Journal",
        (I18nKey::TabJournal, AppLocale::De) => "📖 Reisetagebuch",
        (I18nKey::TabJournal, AppLocale::Fr) => "📖 Journal de voyage",
        (I18nKey::TabJournal, AppLocale::Es) => "📖 Diario de viaje",
        (I18nKey::TabJournal, AppLocale::Ja) => "📖 旅日記・紀行文",
        (I18nKey::TabJournal, AppLocale::Ko) => "📖 여행 일지",
        (I18nKey::TabJournal, AppLocale::Zh) => "📖 游记手账",

        (I18nKey::TabTransit, AppLocale::En) => "Transit",
        (I18nKey::TabTransit, AppLocale::De) => "Verkehr",
        (I18nKey::TabTransit, AppLocale::Fr) => "Transports",
        (I18nKey::TabTransit, AppLocale::Es) => "Transporte",
        (I18nKey::TabTransit, AppLocale::Ja) => "交通",
        (I18nKey::TabTransit, AppLocale::Ko) => "교통",
        (I18nKey::TabTransit, AppLocale::Zh) => "交通",

        // Country Editions
        (I18nKey::EditionJapan, AppLocale::En) => "🇯🇵 Japan (Tokaido Edo-2030)",
        (I18nKey::EditionJapan, AppLocale::De) => "🇯🇵 Japan (Tokaido Edo-2030)",
        (I18nKey::EditionJapan, AppLocale::Fr) => "🇯🇵 Japon (Tokaido Edo-2030)",
        (I18nKey::EditionJapan, AppLocale::Es) => "🇯🇵 Japón (Tokaido Edo-2030)",
        (I18nKey::EditionJapan, AppLocale::Ja) => "🇯🇵 日本 (東海道五十三次 江戸〜現代)",
        (I18nKey::EditionJapan, AppLocale::Ko) => "🇯🇵 일본 (토카이도 에도~현대)",
        (I18nKey::EditionJapan, AppLocale::Zh) => "🇯🇵 日本 (东海道五十三次 江户至现代)",

        (I18nKey::EditionAustria, AppLocale::En) => "🇦🇹 Austria (Habsburg Imperial)",
        (I18nKey::EditionAustria, AppLocale::De) => "🇦🇹 Österreich (Habsburger Kaiserreich)",
        (I18nKey::EditionAustria, AppLocale::Fr) => "🇦🇹 Autriche (Empire des Habsbourg)",
        (I18nKey::EditionAustria, AppLocale::Es) => "🇦🇹 Austria (Imperio de los Habsburgo)",
        (I18nKey::EditionAustria, AppLocale::Ja) => "🇦🇹 オーストリア (ハプスブルク帝国)",
        (I18nKey::EditionAustria, AppLocale::Ko) => "🇦🇹 오스트리아 (합스부르크 제국)",
        (I18nKey::EditionAustria, AppLocale::Zh) => "🇦🇹 奥地利 (哈布斯堡帝国与阿尔卑斯)",

        // Actions
        (I18nKey::ActionOptimizeRoute, AppLocale::En) => "Calculate Optimal Geodesic Path",
        (I18nKey::ActionOptimizeRoute, AppLocale::De) => "Optimale geodätische Route berechnen",
        (I18nKey::ActionOptimizeRoute, AppLocale::Fr) => "Calculer le trajet géodésique optimal",
        (I18nKey::ActionOptimizeRoute, AppLocale::Es) => "Calcular ruta geodésica óptima",
        (I18nKey::ActionOptimizeRoute, AppLocale::Ja) => "最適測地線ルートを計算",
        (I18nKey::ActionOptimizeRoute, AppLocale::Ko) => "최적 측지선 경로 계산",
        (I18nKey::ActionOptimizeRoute, AppLocale::Zh) => "计算最优大圆测地线路径",

        (I18nKey::ActionAddJournal, AppLocale::En) => "Save Journal Entry",
        (I18nKey::ActionAddJournal, AppLocale::De) => "Tagebucheintrag speichern",
        (I18nKey::ActionAddJournal, AppLocale::Fr) => "Enregistrer l'entrée",
        (I18nKey::ActionAddJournal, AppLocale::Es) => "Guardar entrada del diario",
        (I18nKey::ActionAddJournal, AppLocale::Ja) => "日記を保存",
        (I18nKey::ActionAddJournal, AppLocale::Ko) => "일지 저장",
        (I18nKey::ActionAddJournal, AppLocale::Zh) => "保存游记记录",

        (I18nKey::ActionIdentifySight, AppLocale::En) => "Identify Sight / Artwork",
        (I18nKey::ActionIdentifySight, AppLocale::De) => {
            "Sehenswürdigkeit / Kunstwerk identifizieren"
        }
        (I18nKey::ActionIdentifySight, AppLocale::Fr) => "Identifier le monument / l'œuvre",
        (I18nKey::ActionIdentifySight, AppLocale::Es) => "Identificar monumento u obra de arte",
        (I18nKey::ActionIdentifySight, AppLocale::Ja) => "史跡・美術品をAI照合",
        (I18nKey::ActionIdentifySight, AppLocale::Ko) => "명소 / 예술작품 식별",
        (I18nKey::ActionIdentifySight, AppLocale::Zh) => "识别名胜古迹 / 艺术杰作",

        (I18nKey::ActionSendChat, AppLocale::En) => "Ask Baedeker Guide",
        (I18nKey::ActionSendChat, AppLocale::De) => "Baedeker-Führer fragen",
        (I18nKey::ActionSendChat, AppLocale::Fr) => "Demander au guide Baedeker",
        (I18nKey::ActionSendChat, AppLocale::Es) => "Preguntar al guía Baedeker",
        (I18nKey::ActionSendChat, AppLocale::Ja) => "ベデカーに尋ねる",
        (I18nKey::ActionSendChat, AppLocale::Ko) => "베데커 가이드에게 질문",
        (I18nKey::ActionSendChat, AppLocale::Zh) => "向贝德克向导提问",

        (I18nKey::StatusReady, AppLocale::En) => "Baedeker 2030 Chrono-Cartography Ready",
        (I18nKey::StatusReady, AppLocale::De) => "Baedeker 2030 Chrono-Kartographie bereit",
        (I18nKey::StatusReady, AppLocale::Fr) => "Chrono-Cartographie Baedeker 2030 prête",
        (I18nKey::StatusReady, AppLocale::Es) => "Crono-Cartografía Baedeker 2030 lista",
        (I18nKey::StatusReady, AppLocale::Ja) => "ベデカー2030 歴史地図 準備完了",
        (I18nKey::StatusReady, AppLocale::Ko) => "베데커 2030 연대기 지도 준비완료",
        (I18nKey::StatusReady, AppLocale::Zh) => "贝德克2030时空地图系统就绪",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_travelcraft_i18n_locales() {
        assert_eq!(AppLocale::all().len(), 7);
        for loc in AppLocale::all() {
            assert!(!loc.code().is_empty());
            assert!(!loc.display_name().is_empty());
            assert!(!loc.flag().is_empty());
            assert!(!t(I18nKey::TabChronoMap, *loc).is_empty());
            assert!(!t(I18nKey::TabWaypointDetail, *loc).is_empty());
            assert!(!t(I18nKey::TabSensoryAtmosphere, *loc).is_empty());
            assert!(!t(I18nKey::TabRoutePlanner, *loc).is_empty());
            assert!(!t(I18nKey::TabItineraries, *loc).is_empty());
            assert!(!t(I18nKey::TabSightLens, *loc).is_empty());
            assert!(!t(I18nKey::TabSightChat, *loc).is_empty());
            assert!(!t(I18nKey::TabJournal, *loc).is_empty());
            assert!(!t(I18nKey::TabTransit, *loc).is_empty());
            assert!(!t(I18nKey::StatusReady, *loc).is_empty());
        }
    }
}
