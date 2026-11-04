//! TravelCraft Baedeker 2030 Engine
//!
//! Provides chrono-cartography, historical layer juxtaposition,
//! and localized hyper-contextual waypoint guidance (JapanCraft & AustriaCraft).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronoWaypoint {
    pub id: String,
    pub name: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,
    pub year_ad: i32,
    pub historic_narrative: String,
    pub sensory_profile: SensoryAtmosphere,
    pub culinary_specialty: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryAtmosphere {
    pub acoustic_profile: String,
    pub architectural_era: String,
    pub olfactory_notes: String,
    pub light_condition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryEdition {
    pub country_code: String,
    pub title: String,
    pub subtitle: String,
    pub default_currency: String,
    pub curated_routes: Vec<CuratedRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedRoute {
    pub route_id: String,
    pub title: String,
    pub estimated_days: u32,
    pub waypoints: Vec<ChronoWaypoint>,
}

pub struct BaedekerCatalog;

impl BaedekerCatalog {
    pub fn get_japan_craft() -> CountryEdition {
        CountryEdition {
            country_code: "JP".to_string(),
            title: "JapanCraft: Baedeker 2030 Edo-Kyoto-Kansai".to_string(),
            subtitle: "Chrono-cartographic traversal from Tokaido 1832 to Neo-Tokyo Shinjuku 2030".to_string(),
            default_currency: "JPY".to_string(),
            curated_routes: vec![
                CuratedRoute {
                    route_id: "tokaido_chrono".to_string(),
                    title: "The 53 Stations of the Tokaido (Hiroshige Chrono-Path)".to_string(),
                    estimated_days: 7,
                    waypoints: vec![
                        ChronoWaypoint {
                            id: "jp_nihonbashi".to_string(),
                            name: "Nihonbashi Zero Milestone".to_string(),
                            region: "Tokyo / Edo".to_string(),
                            lat: 35.6840,
                            lon: 139.7745,
                            year_ad: 1603,
                            historic_narrative: "The starting point of the Five Routes established by Tokugawa Ieyasu. In 1832, fishmongers crowded the wooden bridge; today, highway overpasses conceal the water, soon to be subterranean in 2035.".to_string(),
                            sensory_profile: SensoryAtmosphere {
                                acoustic_profile: "Distant highway drone fading into river echoes and chime bells".to_string(),
                                architectural_era: "Late Meiji stone arches over Edo timber foundations".to_string(),
                                olfactory_notes: "Dashi broth, hinoki cedar, sea spray from Tokyo Bay".to_string(),
                                light_condition: "Diffused morning sunlight reflected from glass towers".to_string(),
                            },
                            culinary_specialty: vec!["Edomae Nigiri Sushi".to_string(), "Fukagawa Meshi".to_string()],
                        },
                        ChronoWaypoint {
                            id: "jp_hakone".to_string(),
                            name: "Hakone Checkpoint (Sekisho)".to_string(),
                            region: "Kanagawa".to_string(),
                            lat: 35.1917,
                            lon: 139.0253,
                            year_ad: 1840,
                            historic_narrative: "Formidable mountain pass guardhouse where incoming guns and outgoing women hostages were scrutinized by the Shogunate. Steam billows from sulfur thermal onsen valleys.".to_string(),
                            sensory_profile: SensoryAtmosphere {
                                acoustic_profile: "Mountain wind through cryptomeria cedar trees, sulfur geyser bubbling".to_string(),
                                architectural_era: "Blackened cedar wood guardhouses with thatched eaves".to_string(),
                                olfactory_notes: "Volcanic sulfur, damp cedar bark, charcoal hearth smoke".to_string(),
                                light_condition: "Misty mountain fog breaking at sunset over Lake Ashi".to_string(),
                            },
                            culinary_specialty: vec!["Hakone Kuro-tamago (Black Eggs)".to_string(), "Yuba Soba".to_string()],
                        },
                        ChronoWaypoint {
                            id: "jp_gion".to_string(),
                            name: "Kyoto Sanjo Ohashi & Gion Shirakawa".to_string(),
                            region: "Kyoto".to_string(),
                            lat: 35.0088,
                            lon: 135.7725,
                            year_ad: 1185,
                            historic_narrative: "The western terminus of the Tokaido. Bronze giboshi finials with sword cuts from the Bakumatsu rebellion. Lantern-lit stone lanes along weeping willow canals.".to_string(),
                            sensory_profile: SensoryAtmosphere {
                                acoustic_profile: "Rushing Kamo River waters, shamisen plucking behind cedar lattice (machiya)".to_string(),
                                architectural_era: "Edo-period machiya lattices and stone-paved alleys".to_string(),
                                olfactory_notes: "Kyoto incense (Kyara), matcha whisking, tatami reeds".to_string(),
                                light_condition: "Warm paper lantern illumination reflecting in canal water".to_string(),
                            },
                            culinary_specialty: vec!["Kyoto Kaiseki".to_string(), "Yudofu".to_string(), "Uji Matcha Warabi Mochi".to_string()],
                        },
                    ],
                },
            ],
        }
    }

    pub fn get_austria_craft() -> CountryEdition {
        CountryEdition {
            country_code: "AT".to_string(),
            title: "AustriaCraft: Baedeker 2030 Alpine & Imperial".to_string(),
            subtitle: "Habsburg imperial palaces, Salzkammergut salt mines, and high alpine passes".to_string(),
            default_currency: "EUR".to_string(),
            curated_routes: vec![
                CuratedRoute {
                    route_id: "habsburg_salzburg_chrono".to_string(),
                    title: "Imperial Vienna to High Alpine Grossglockner".to_string(),
                    estimated_days: 6,
                    waypoints: vec![
                        ChronoWaypoint {
                            id: "at_stephansdom".to_string(),
                            name: "Stephansdom & Hofburg Imperial Treasury".to_string(),
                            region: "Vienna (Innere Stadt)".to_string(),
                            lat: 48.2085,
                            lon: 16.3738,
                            year_ad: 1365,
                            historic_narrative: "Heart of the Holy Roman and Austro-Hungarian Empires. Gothic vaults holding 650 years of cathedral chimes, steps away from Habsburg treasury containing the Imperial Crown (Reichskrone 962 AD).".to_string(),
                            sensory_profile: SensoryAtmosphere {
                                acoustic_profile: "Pummerin bell resonance, Fiaker horse hooves on cobblestone, cafe murmurs".to_string(),
                                architectural_era: "High Gothic and Ringstrasse Imperial Baroque".to_string(),
                                olfactory_notes: "Roasted Melange coffee, warm Sachertorte chocolate, polished beeswax".to_string(),
                                light_condition: "Stained-glass polychromatic rays piercing dark stone vaults".to_string(),
                            },
                            culinary_specialty: vec!["Wiener Schnitzel vom Kalb".to_string(), "Tafelspitz".to_string(), "Kaiserschmarrn".to_string()],
                        },
                        ChronoWaypoint {
                            id: "at_hallstatt".to_string(),
                            name: "Hallstatt Celtic Salt Mine (Salzwelten)".to_string(),
                            region: "Salzkammergut".to_string(),
                            lat: 47.5622,
                            lon: 13.6493,
                            year_ad: -800,
                            historic_narrative: "Cradle of the Iron Age Hallstatt Culture. 7,000 years of salt mining inside prehistoric alpine tunnels, overlooking reflection of Dachstein glacier in crystal fjord water.".to_string(),
                            sensory_profile: SensoryAtmosphere {
                                acoustic_profile: "Dripping subterranean brine, wooden miner slide creaks, alpine horn echo".to_string(),
                                architectural_era: "Bronze/Iron Age timber supports and 16th-century lakeside timber houses".to_string(),
                                olfactory_notes: "Alpine pine sap, mineral salt brine, crisp glacier air".to_string(),
                                light_condition: "Morning sun hitting alpine mist lifting off mirror-lake".to_string(),
                            },
                            culinary_specialty: vec!["Salzkammergut Reinanke (Lake Trout)".to_string(), "Holzknechtnocken".to_string()],
                        },
                    ],
                },
            ],
        }
    }
}
