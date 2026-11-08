//! TravelCraft Baedeker 2030 Engine
//!
//! Provides chrono-cartography, historical layer juxtaposition,
//! and localized hyper-contextual waypoint guidance (JapanCraft & AustriaCraft).
//!
//! Vendored sources (offline copies, see SOURCES below):
//! - Japan practical data: japanophile-mcp/assets/knowledge/japan/travel.html
//!   (JR Pass, konbini, budget, etiquette, regional highlights)
//! - Vienna practical data: vienna-life-assistant/web_sota/vienna_life_assistant/
//!   vienna_life_mcp.py vienna_tips(), restaurants_data.py Beisl rule,
//!   culture_data.py (KHM, Belvedere, Stephansplatz), skills/vienna-transit,
//!   skills/vienna-kaffeehaus
//!
//! SOURCES is the attribution contract. Keep it updated when re-vendoring.

use serde::{Deserialize, Serialize};

/// Attribution for vendored offline content.
pub const SOURCES: &[&str] = &[
    "japanophile-mcp/assets/knowledge/japan/travel.html (JR Pass, konbini, budget, etiquette)",
    "japanophile-mcp/assets/knowledge/japan/culture.html, cuisine.html, geography.html (context)",
    "vienna-life-assistant vienna_tips(coffee/music/museum), restaurants Beisl rule, culture_data KHM/Belvedere/Stephansplatz",
];

/// Japan practical survival kit (vendored, condensed).
pub const JAPAN_PRACTICAL: &str =
    "JR Pass 7/14/21 days pays off in 2-3 Shinkansen rides. IC Suica/Pasmo in cities. \
Konbini (55k stores): onigiri+coffee JPY 300-500, time-sale sushi half price 19-20h. \
Business hotel JPY 5000-8000, guesthouse JPY 2000-4000. Bow slightly, shoes off indoors, \
no nose-blowing in public. Hyperdia + offline maps. Tap water safe. Budget 2 weeks: JPY 80k-120k.";

/// Vienna practical survival kit (vendored, condensed).
pub const VIENNA_PRACTICAL: &str = "Coffee: Hausbrand at Cafe Berg (Alsergrund), Melange + water. \
Music: Staatsoper Stehplatz ~80 min before show EUR 10-15. Museums: KHM + Leopold (Schiele/Klimt). \
Transit: Wiener Linien U1/U3 Stephansplatz, U4 Friedensbruecke, trams D/1/71. \
Beisl rule: no photo menus at door - two alleys further locals eat better/cheaper. \
Avoid taxi short rides; day ticket EUR 8. Safety: harmless fauna except cows on alm + hornviper.";

/// A single topic item within a Baedeker Knowledge Tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoTopicItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub tags: Vec<String>,
    pub body: String,
    pub practical_tips: Vec<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

/// A categorized branch inside a country's Baedeker Knowledge Tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoBranch {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub items: Vec<InfoTopicItem>,
}

/// A comprehensive hierarchical knowledge tree for a country edition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTree {
    pub country_code: String,
    pub title: String,
    pub branches: Vec<InfoBranch>,
}

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
    /// One-line visitor practical (hours, ticket, best light).
    pub visitor_info: String,
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

/// Day-by-day itinerary for a given stay length.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StayItinerary {
    pub country_code: String,
    pub total_days: u32,
    pub pace_label: String,
    pub days: Vec<ItineraryDay>,
    pub practical_tip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItineraryDay {
    pub day: u32,
    pub title: String,
    pub waypoint_ids: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SightKind {
    Landmark,
    Museum,
    Artwork,
    District,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SightHit {
    pub id: String,
    pub name: String,
    pub kind: SightKind,
    pub country_code: String,
    pub lat: f64,
    pub lon: f64,
    pub distance_m: f64,
    pub confidence: f32,
    pub headline: String,
    pub body: String,
    pub practical: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkRecord {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: String,
    pub museum: String,
    pub room: String,
    pub lat: f64,
    pub lon: f64,
    pub story: String,
    pub look_for: String,
}

fn sensory(acoustic: &str, arch: &str, olfactory: &str, light: &str) -> SensoryAtmosphere {
    SensoryAtmosphere {
        acoustic_profile: acoustic.to_string(),
        architectural_era: arch.to_string(),
        olfactory_notes: olfactory.to_string(),
        light_condition: light.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
fn wp(
    id: &str,
    name: &str,
    region: &str,
    lat: f64,
    lon: f64,
    year: i32,
    narrative: &str,
    s: SensoryAtmosphere,
    food: &[&str],
    visitor: &str,
) -> ChronoWaypoint {
    ChronoWaypoint {
        id: id.to_string(),
        name: name.to_string(),
        region: region.to_string(),
        lat,
        lon,
        year_ad: year,
        historic_narrative: narrative.to_string(),
        sensory_profile: s,
        culinary_specialty: food.iter().map(|x| x.to_string()).collect(),
        visitor_info: visitor.to_string(),
    }
}

fn topic(
    id: &str,
    title: &str,
    subtitle: &str,
    tags: &[&str],
    body: &str,
    tips: &[&str],
    coords: Option<(f64, f64)>,
) -> InfoTopicItem {
    InfoTopicItem {
        id: id.to_string(),
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        tags: tags.iter().map(|s| s.to_string()).collect(),
        body: body.to_string(),
        practical_tips: tips.iter().map(|s| s.to_string()).collect(),
        lat: coords.map(|(lat, _)| lat),
        lon: coords.map(|(_, lon)| lon),
    }
}

pub struct BaedekerCatalog;

impl BaedekerCatalog {
    pub fn get_japan_craft() -> CountryEdition {
        CountryEdition {
            country_code: "JP".to_string(),
            title: "JapanCraft: Baedeker 2030 Edo-Kyoto-Kansai".to_string(),
            subtitle: "Chrono-cartographic traversal from Tokaido 1832 to Neo-Tokyo Shinjuku 2030".to_string(),
            default_currency: "JPY".to_string(),
            curated_routes: vec![CuratedRoute {
                route_id: "tokaido_chrono".to_string(),
                title: "The 53 Stations of the Tokaido (Hiroshige Chrono-Path)".to_string(),
                estimated_days: 7,
                waypoints: vec![
                    wp("jp_nihonbashi", "Nihonbashi Zero Milestone", "Tokyo / Edo", 35.6840, 139.7745, 1603,
                        "The starting point of the Five Routes established by Tokugawa Ieyasu. In 1832, fishmongers crowded the wooden bridge; today, highway overpasses conceal the water, soon to be subterranean in 2035.",
                        sensory("Distant highway drone fading into river echoes and chime bells", "Late Meiji stone arches over Edo timber foundations", "Dashi broth, hinoki cedar, sea spray from Tokyo Bay", "Diffused morning sunlight reflected from glass towers"),
                        &["Edomae Nigiri Sushi", "Fukagawa Meshi"],
                        "Free, open air. Best at dawn before traffic. Metro Mitsukoshimae."),
                    wp("jp_hakone", "Hakone Checkpoint (Sekisho)", "Kanagawa", 35.1917, 139.0253, 1840,
                        "Formidable mountain pass guardhouse where incoming guns and outgoing women hostages were scrutinized by the Shogunate. Steam billows from sulfur thermal onsen valleys.",
                        sensory("Mountain wind through cryptomeria cedar trees, sulfur geyser bubbling", "Blackened cedar wood guardhouses with thatched eaves", "Volcanic sulfur, damp cedar bark, charcoal hearth smoke", "Misty mountain fog breaking at sunset over Lake Ashi"),
                        &["Hakone Kuro-tamago (Black Eggs)", "Yuba Soba"],
                        "JPY 500. Hakone Freepass covers transit. Morning fog lifts ~10h."),
                    wp("jp_fushimi", "Fushimi Inari Thousand Torii", "Kyoto", 34.9671, 135.7727, 711,
                        "Shinto shrine of rice and business prosperity. Ten thousand vermilion torii donated by merchants since Edo. Night ascent avoids crowds; fox messengers (kitsune) guard every turn.",
                        sensory("Cicada chorus, wooden geta on stone steps, distant prayer bells", "Vermilion lacquered timber gates layered up the mountain", "Incense smoke, cedar, fried aburaage offerings", "Torii-filtered amber light, best after 17h"),
                        &["Inari Sushi", "Yudofu"],
                        "Free, 24h. JR Nara line Inari. 2-3h loop. Bring coins."),
                    wp("jp_nara", "Nara Todaiji Daibutsu", "Nara", 34.6890, 135.8398, 752,
                        "Great Eastern Temple of Emperor Shomu. Fifteen-meter bronze Buddha in the largest wooden hall on earth. Sika deer bow for shika-senbei crackers in Nara Park.",
                        sensory("Temple gongs, deer snorts, wind through 1000-year pines", "Nara-period monumental timber bracketing (daibutsuyo)", "Incense, old wood, rain on moss", "Side windows rake across Buddha face ~9h"),
                        &["Kakinoha Sushi", "Miwa Somen"],
                        "JPY 600. JR Nara + walk. Deer crackers JPY 200, bow back."),
                    wp("jp_gion", "Kyoto Sanjo Ohashi & Gion Shirakawa", "Kyoto", 35.0088, 135.7725, 1185,
                        "The western terminus of the Tokaido. Bronze giboshi finials with sword cuts from the Bakumatsu rebellion. Lantern-lit stone lanes along weeping willow canals.",
                        sensory("Rushing Kamo River waters, shamisen plucking behind cedar lattice (machiya)", "Edo-period machiya lattices and stone-paved alleys", "Kyoto incense (Kyara), matcha whisking, tatami reeds", "Warm paper lantern illumination reflecting in canal water"),
                        &["Kyoto Kaiseki", "Yudofu", "Uji Matcha Warabi Mochi"],
                        "Free to stroll. Gion after 18h, silent streets, no tripod."),
                    wp("jp_osaka", "Osaka Dotonbori & Kuidaore", "Osaka", 34.6689, 135.5023, 1615,
                        "Eat-till-you-drop canal strip. Glico runner neon since 1935, kuidaore puppet drumming. Takoyaki, kushikatsu, 551 Horai pork buns shoulder to shoulder.",
                        sensory("Canal splash, sizzling takoyaki grills, arcade jingles", "Showa neon over Taisho brick theater fronts", "Takoyaki sauce, hot oil, Doton river damp", "Neon double-light after 19h, reflections on water"),
                        &["Takoyaki", "Kushikatsu", "Okonomiyaki"],
                        "Free strip. Namba walk. Cash for stalls, JPY 500-1000 each."),
                    wp("jp_hiroshima", "Hiroshima Peace Memorial", "Hiroshima", 34.3955, 132.4536, 1945,
                        "Genbaku Dome skeleton kept as found after 6 Aug 1945. Museum documents the city rebuilt around the river delta. Paper cranes from schoolchildren worldwide.",
                        sensory("River murmur, tram bells, paper crane rustle, hushed museum halls", "Exposed 1915 steel dome ribs + post-war concrete memorial park", "River air, fresh-cut memorial grass", "Soft overcast light suits the site; avoid midday glare"),
                        &["Hiroshima Okonomiyaki", "Anago Meshi"],
                        "Park free, museum JPY 200. Tram from station. Quiet conduct."),
                    wp("jp_kanazawa", "Kanazawa Kenrokuen & Higashi Chaya", "Ishikawa", 36.5613, 136.6562, 1676,
                        "One of the Three Great Gardens. Pine roots propped with yukitsuri ropes in winter. Higashi Chaya geisha lanes keep Edo facades; gold leaf everything, even ice cream.",
                        sensory("Garden streams, wooden clogs on chaya lanes, temple drums", "Maeda clan stroll garden + preserved chaya lattice houses", "Pine sap, matcha, gold-leaf sweets", "Opening 7h in season, mist on pond"),
                        &["Jibuni Stew", "Gold-leaf Soft Cream"],
                        "Garden JPY 320. Hokuriku Shinkansen. Yukitsuri Nov-Mar."),
                    wp("jp_nikko", "Nikko Toshogu Shrine", "Tochigi", 36.7581, 139.5967, 1617,
                        "Tokugawa Ieyasu mausoleum under ancient cedars. Yomeimon gate carved with 500+ figures, sleeping cat (Nemuri-neko) above the passage. Lake Chuzenji autumn colors above.",
                        sensory("Cedar wind roar, shrine drums, waterfall echo from Kegon", "Gongen-zukuri lacquer + gold leaf under 400-year cedars", "Cedar resin, incense, lake mist", "Morning sun through cedars, autumn peak Nov"),
                        &["Yuba Soba", "Kanpyo Maki"],
                        "JPY 1300 combo. Tobu from Asakusa. Arrive before 9h."),
                ],
            }],
        }
    }

    pub fn get_austria_craft() -> CountryEdition {
        CountryEdition {
            country_code: "AT".to_string(),
            title: "AustriaCraft: Baedeker 2030 Alpine & Imperial".to_string(),
            subtitle: "Habsburg imperial palaces, Salzkammergut salt mines, and high alpine passes".to_string(),
            default_currency: "EUR".to_string(),
            curated_routes: vec![CuratedRoute {
                route_id: "habsburg_salzburg_chrono".to_string(),
                title: "Imperial Vienna to High Alpine Grossglockner".to_string(),
                estimated_days: 6,
                waypoints: vec![
                    wp("at_stephansdom", "Stephansdom & Stephansplatz", "Vienna (Innere Stadt)", 48.2085, 16.3738, 1365,
                        "Heart of Vienna since 1147, Gothic south tower (Steffl, 136m) raised by Rudolf IV. Pummerin bell recast from Turkish cannon bronze 1711/1957. Roof tiles form Habsburg eagle in 230k glazed bricks. Catacombs hold plague pits + Habsburg urns. U1/U3 Stephansplatz, trams 1/2/D nearby.",
                        sensory("Pummerin bell resonance, Fiaker horse hooves on cobblestone, cafe murmurs", "High Gothic + Romanesque Giant's Door, Baroque chapels inside", "Roasted Melange coffee, warm Sachertorte, polished beeswax", "Stained-glass polychromatic rays in dark nave; roof glows at golden hour"),
                        &["Wiener Schnitzel vom Kalb", "Tafelspitz", "Kaiserschmarrn"],
                        "Nave free, tower EUR 6.50, catacombs EUR 12. Roof photo from south tower at 18h. Mass quiet hours."),
                    wp("at_hofburg", "Hofburg Imperial Treasury & Apartments", "Vienna (Innere Stadt)", 48.2065, 16.3655, 1279,
                        "Habsburg winter residence 600+ years. Treasury holds Imperial Crown (Reichskrone c.962), Holy Lance, Order of the Golden Fleece. Sisi Museum + Silver Collection next door. Neue Burg museums row faces Heldenplatz.",
                        sensory("Courtyard gravel crunch, Lipizzaner hooves from Spanish Riding School", "Baroque + Ringstrasse historicism wings around medieval core", "Beeswax, old velvet, horse leather", "Courtyard glare midday; interiors dim vitrine light"),
                        &["Backhendl", "Gulasch"],
                        "Treasury EUR 14. Closed Tue. Combine with KHM across Ring."),
                    wp("at_khm", "Kunsthistorisches Museum (KHM)", "Vienna (Mariahilf/Ring)", 48.2039, 16.3619, 1891,
                        "Emperor Franz Joseph's palace of art. Staircase Klimt murals, then Bruegel room (Tower of Babel, Hunters in Snow), Caravaggio room: Rosenkranzmadonna (1606-07, his last Neapolitan altarpiece), David with the Head of Goliath, Madonna of the Rosary drama in chiaroscuro. Dome cafe for Melange break.",
                        sensory("Marble echo, hushed gallery air, dome cafe cup clink", "Ringstrasse neo-Renaissance palace by Semper/Hasenauer", "Old varnish, marble dust, coffee from dome cafe", "Top-lit galleries; dome cafe brightest at noon"),
                        &["Cafe im Kunsthistorischen: Melange + Gugelhupf"],
                        "EUR 21, Thu-Tue. Caravaggio: Staircase I, Saal X. Thu evening quiet."),
                    wp("at_belvedere", "Belvedere Palace & Klimt Kiss", "Vienna (Landstrasse)", 48.1915, 16.3809, 1723,
                        "Prince Eugene's summer palace, Upper + Lower + gardens. Home of Klimt's Kiss (1908-09), Judith, plus Schiele, Kokoschka. 1955 State Treaty signed here (Figl: Oesterreich ist frei!). Gardens free, view over Innere Stadt.",
                        sensory("Gravel garden paths, fountain splash, palace hall hush", "High Baroque palace + landscaped terraced gardens", "Boxwood hedges, palace wax, summer lime blossom", "Upper Belvedere Kiss room sidelit; gardens sunset-facing"),
                        &["Salm Braeu next door", "Kaffee Alt Wien"],
                        "EUR 16.90 combo. Gardens free dawn-dusk. Staatsvertrag room upstairs."),
                    wp("at_schoenbrunn", "Schoenbrunn Palace & Gardens", "Vienna (Hietzing)", 48.1858, 16.3122, 1696,
                        "Habsburg summer seat, 1441 rooms, Gloriette hill panorama. Oldest zoo in world (Tiergarten 1752) in park. Maria Theresa mornings, Mozart child concerts in mirror room. U4 Hietzing/Schoenbrunn.",
                        sensory("Gravel + peacock calls, Gloriette wind, palace clock chimes", "Baroque palace + Rococo interiors + English/French gardens", "Mown park grass, palace beeswax, zoo hay", "Gloriette panorama best 17-19h, palace backlit noon"),
                        &["Gerstner K.u.K. Hofzuckerb.: Torte", "Schoenbrunn Kipferl"],
                        "Park free, Grand Tour EUR 24. Timeslot booking. Zoo separate EUR 26."),
                    wp("at_kaffeehaus", "Cafe Berg & Alsergrund Kaffeehaus Mile", "Vienna (Alsergrund)", 48.2175, 16.3575, 1873,
                        "Viennese coffee house institution: marble tables, Thonet chairs, newspapers on holders, Melange + glass of water. Berg draws writers + Zeitung readers; AIDA + Diglas nearby. U4 Friedensbruecke, trams D/5/33 Julius-Tandler-Platz.",
                        sensory("Cup clink, newspaper rustle, low German murmur, tram rumble outside", "Ringstrasse + Gruenderzeit corner house, bentwood + marble", "Hausbrand roast, Apfelstrudel cinnamon, newsprint", "Big windows, morning side bright, evening amber lamps"),
                        &["Melange", "Einspaenner", "Apfelstrudel"],
                        "Melange EUR 5-6. Hausbrand at Berg. Cash + card. Laptop tolerated off-peak."),
                    wp("at_hallstatt", "Hallstatt Celtic Salt Mine (Salzwelten)", "Salzkammergut", 47.5622, 13.6493, -800,
                        "Cradle of the Iron Age Hallstatt Culture. 7,000 years of salt mining inside prehistoric alpine tunnels, overlooking reflection of Dachstein glacier in crystal fjord water.",
                        sensory("Dripping subterranean brine, wooden miner slide creaks, alpine horn echo", "Bronze/Iron Age timber supports and 16th-century lakeside timber houses", "Alpine pine sap, mineral salt brine, crisp glacier air", "Morning sun hitting alpine mist lifting off mirror-lake"),
                        &["Salzkammergut Reinanke (Lake Trout)", "Holzknechtnocken"],
                        "Mine EUR 36. Slide + train. Book slot. No large bags underground."),
                    wp("at_wachau", "Wachau Melk Abbey & Danube Terraces", "Wachau", 48.2297, 15.3319, 1089,
                        "Benedictine baroque abbey above Danube loop. Apricot (Marille) terraces, Riesling + Gruener Veltliner stone terraces. Melk to Krems bike 39km, Duerenstein Richard Lionheart ruin mid-way.",
                        sensory("Danube lap, abbey bells, bike tires on riverside path", "High Baroque abbey church + medieval Duerenstein ruin", "Apricot blossom Apr, fermenting must Oct, river damp", "Terraces glow 18h; abbey library top-lit 10-14h"),
                        &["Marillenknodel", "Wachauer Laberl + Gruener Veltliner"],
                        "Abbey EUR 14.50. Bike rental Melk/Krems. Apricot Jul-Aug."),
                ],
            }],
        }
    }

    /// All waypoints flattened for search/identify.
    pub fn all_waypoints() -> Vec<(String, ChronoWaypoint)> {
        let mut out = Vec::new();
        for c in [Self::get_japan_craft(), Self::get_austria_craft()] {
            for r in &c.curated_routes {
                for w in &r.waypoints {
                    out.push((c.country_code.clone(), w.clone()));
                }
            }
        }
        out
    }

    /// Offline artwork index (KHM + Belvedere highlights).
    pub fn artworks() -> Vec<ArtworkRecord> {
        vec![
            ArtworkRecord {
                id: "khm_caravaggio_rosenkranz".to_string(),
                title: "Madonna of the Rosary (Rosenkranzmadonna)".to_string(),
                artist: "Michelangelo Merisi da Caravaggio".to_string(),
                year: "1606-1607".to_string(),
                museum: "KHM Gemaldegalerie, Saal X".to_string(),
                room: "Staircase I / Caravaggio room".to_string(),
                lat: 48.2039, lon: 16.3619,
                story: "Painted in Naples while Caravaggio fled a murder charge from Rome. Dominicans commissioned it, then refused it as too undignified - the Virgin dusty-footed among plague poor. Rubens' circle bought it for Antwerp Dominicans; Emperor Joseph II seized it for Vienna. Chiaroscuro drama: begging hands reach from darkness into Madonna's light.".to_string(),
                look_for: "Bare dirty feet, outstretched beggar hands, red canopy like a theater curtain.".to_string(),
            },
            ArtworkRecord {
                id: "khm_caravaggio_david".to_string(),
                title: "David with the Head of Goliath".to_string(),
                artist: "Caravaggio".to_string(),
                year: "c.1607".to_string(),
                museum: "KHM Gemaldegalerie".to_string(),
                room: "Caravaggio room".to_string(),
                lat: 48.2039, lon: 16.3619,
                story: "David ties Goliath's head by the hair - some read Goliath as Caravaggio's own self-portrait, a confession letter in paint while begging papal pardon.".to_string(),
                look_for: "Rope cutting into forehead, David's pitying frown, blood still wet.".to_string(),
            },
            ArtworkRecord {
                id: "khm_bruegel_turm".to_string(),
                title: "The Tower of Babel".to_string(),
                artist: "Pieter Bruegel the Elder".to_string(),
                year: "1563".to_string(),
                museum: "KHM, Bruegel room (world's largest Bruegel holding)".to_string(),
                room: "Saal XII".to_string(),
                lat: 48.2039, lon: 16.3619,
                story: "Bruegel paints Rome's Colosseum spiralling into clouds - a warning about imperial hubris Vienna's Habsburgs read personally. Look for tiny workers like ants.".to_string(),
                look_for: "Colosseum arches half-built, tiny cranes, clouds swallowing the top.".to_string(),
            },
            ArtworkRecord {
                id: "belvedere_klimt_kuss".to_string(),
                title: "The Kiss (Der Kuss)".to_string(),
                artist: "Gustav Klimt".to_string(),
                year: "1908-1909".to_string(),
                museum: "Belvedere Upper".to_string(),
                room: "Klimt room".to_string(),
                lat: 48.1915, lon: 16.3809,
                story: "Golden-phase lovers kneeling on a flower cliff. Austria's Mona Lisa - bought before it was finished, 25,000 crowns, the highest price ever paid for an Austrian painting then.".to_string(),
                look_for: "Gold leaf rectangles (man) vs circles (woman), toes curling over the edge.".to_string(),
            },
        ]
    }

    /// Nearest-sight identification: GPS + optional text hint (artwork aware).
    pub fn identify_sight(lat: f64, lon: f64, hint: &str) -> Option<SightHit> {
        let h = hint.to_lowercase();
        // 1. Artwork keyword fast-path (no GPS needed).
        if h.contains("caravaggio") || h.contains("rosenkranz") || h.contains("rosary") {
            let a = Self::artworks()[0].clone();
            return Some(SightHit {
                id: a.id,
                name: format!("{} - {}", a.title, a.artist),
                kind: SightKind::Artwork,
                country_code: "AT".to_string(),
                lat: a.lat,
                lon: a.lon,
                distance_m: 0.0,
                confidence: 0.97,
                headline: "Caravaggio in the KHM - fugitive masterpiece".to_string(),
                body: a.story,
                practical: format!("{} | Look for: {}", a.museum, a.look_for),
            });
        }
        if h.contains("kuss") || h.contains("kiss") && h.contains("klimt") || h == "klimt" {
            let a = Self::artworks()[3].clone();
            return Some(SightHit {
                id: a.id,
                name: format!("{} - {}", a.title, a.artist),
                kind: SightKind::Artwork,
                country_code: "AT".to_string(),
                lat: a.lat,
                lon: a.lon,
                distance_m: 0.0,
                confidence: 0.96,
                headline: "Klimt's Kiss at Belvedere Upper".to_string(),
                body: a.story,
                practical: format!("{} | Look for: {}", a.museum, a.look_for),
            });
        }
        if h.contains("bruegel") || h.contains("babel") {
            let a = Self::artworks()[2].clone();
            return Some(SightHit {
                id: a.id,
                name: format!("{} - {}", a.title, a.artist),
                kind: SightKind::Artwork,
                country_code: "AT".to_string(),
                lat: a.lat,
                lon: a.lon,
                distance_m: 0.0,
                confidence: 0.95,
                headline: "Bruegel's Tower of Babel - KHM".to_string(),
                body: a.story,
                practical: format!("{} | Look for: {}", a.museum, a.look_for),
            });
        }
        if h.contains("stephansdom") || h.contains("steffl") {
            let (_, w) = Self::all_waypoints()
                .into_iter()
                .find(|(_, w)| w.id == "at_stephansdom")?;
            return Some(SightHit {
                id: w.id.clone(),
                name: w.name.clone(),
                kind: SightKind::Landmark,
                country_code: "AT".to_string(),
                lat: w.lat,
                lon: w.lon,
                distance_m: 0.0,
                confidence: 0.98,
                headline: "Stephansdom - Gothic heart of Vienna".to_string(),
                body: w.historic_narrative.clone(),
                practical: w.visitor_info.clone(),
            });
        }
        // 2. GPS nearest within 2 km (haversine approx).
        let mut best: Option<(String, ChronoWaypoint, f64)> = None;
        for (cc, w) in Self::all_waypoints() {
            let dlat = (w.lat - lat).to_radians();
            let dlon = (w.lon - lon).to_radians();
            let a = (dlat / 2.0).sin().powi(2)
                + lat.to_radians().cos() * w.lat.to_radians().cos() * (dlon / 2.0).sin().powi(2);
            let d = 2.0 * 6371000.0 * a.sqrt().asin();
            let is_better = match &best {
                None => true,
                Some((_, _, bd)) => d < *bd,
            };
            if is_better {
                best = Some((cc, w, d));
            }
        }
        let (cc, w, d) = best?;
        if d > 2000.0 {
            return None;
        }
        let confidence = (1.0 - (d / 2000.0) as f32).clamp(0.4, 0.98);
        Some(SightHit {
            id: w.id.clone(),
            name: w.name.clone(),
            kind: if cc == "AT" && w.id.contains("khm") {
                SightKind::Museum
            } else {
                SightKind::Landmark
            },
            country_code: cc,
            lat: w.lat,
            lon: w.lon,
            distance_m: d,
            confidence,
            headline: format!("{} - {:.0}m away", w.name, d),
            body: w.historic_narrative.clone(),
            practical: w.visitor_info.clone(),
        })
    }

    /// Day-plan builder for 1..14 day stays. Slices waypoints into digestible days.
    pub fn itinerary_for_stay(country_code: &str, days: u32) -> StayItinerary {
        let days = days.clamp(1, 14);
        let edition = match country_code.to_uppercase().as_str() {
            "AT" | "AUSTRIA" => Self::get_austria_craft(),
            _ => Self::get_japan_craft(),
        };
        let all: Vec<ChronoWaypoint> = edition
            .curated_routes
            .iter()
            .flat_map(|r| r.waypoints.clone())
            .collect();
        let pace = match days {
            1 => "Blitz - highlights only",
            2..=3 => "Compact - essentials at walking pace",
            4..=7 => "Classic - one base + day trips",
            _ => "Slow Baedeker - live like a local",
        };
        let practical = if edition.country_code == "JP" {
            JAPAN_PRACTICAL.to_string()
        } else {
            VIENNA_PRACTICAL.to_string()
        };
        if all.is_empty() {
            return StayItinerary {
                country_code: edition.country_code,
                total_days: days,
                pace_label: pace.to_string(),
                days: Vec::new(),
                practical_tip: practical,
            };
        }
        // Distribute waypoints round-robin across days so 1-day = best-of, 14-day = deep.
        let mut per_day: Vec<Vec<String>> = vec![Vec::new(); days as usize];
        for (i, w) in all.iter().enumerate() {
            let slot = i % days as usize;
            per_day[slot].push(w.id.clone());
        }
        // For 1-day stays keep max 3 stops.
        if days == 1 {
            per_day[0].truncate(3);
        }
        let mut out_days = Vec::new();
        for (i, ids) in per_day.into_iter().enumerate() {
            if ids.is_empty() {
                continue;
            }
            let names: Vec<String> = ids
                .iter()
                .filter_map(|id| all.iter().find(|w| &w.id == id))
                .map(|w| w.name.clone())
                .collect();
            out_days.push(ItineraryDay {
                day: (i + 1) as u32,
                title: format!(
                    "Day {} - {}",
                    i + 1,
                    names.first().cloned().unwrap_or_default()
                ),
                waypoint_ids: ids,
                note: if edition.country_code == "JP" {
                    "Start 8h, konbini breakfast, JR + IC, time-sale dinner.".to_string()
                } else {
                    "Start 9h Melange, U-Bahn day ticket, Beisl lunch off the main street."
                        .to_string()
                },
            });
        }
        StayItinerary {
            country_code: edition.country_code,
            total_days: days,
            pace_label: pace.to_string(),
            days: out_days,
            practical_tip: practical,
        }
    }

    /// Full Austrian Knowledge Tree (extensive Vienna gastronomy, culture, imperial, transit).
    pub fn get_austria_knowledge_tree() -> KnowledgeTree {
        KnowledgeTree {
            country_code: "AT".to_string(),
            title: "AustriaCraft: Sovereign Baedeker Knowledge Tree".to_string(),
            branches: vec![
                InfoBranch {
                    id: "at_branch_restaurants".to_string(),
                    title: "Vienna Restaurant & Gastronomy Guide (Extensive)".to_string(),
                    icon: "🍽️".to_string(),
                    items: vec![
                        topic(
                            "at_item_beisl_rules",
                            "The Golden Vienna Beisl Rules & Culture",
                            "Etiquette, ordering terminology, tipping, and the 2-alleys rule",
                            &["etiquette", "beisl", "beer", "spritzer", "tipping"],
                            "Keine Fotokarten vor der Tür — zwei Gassen weiter essen Einheimische besser und günstiger. Ein echtes Wiener Beisl zeichnet sich durch schlichte Holztäfelung, Schanktisch, traditionelle Wiener Hausmannskost und den Stammgast-Tisch (Stammtisch) aus. Niemals an einen freien Tisch mit Stammtisch-Schild setzen. Bier-Einheiten: Ein Pfiff (0.2L, traditionell zu Trzesniewski-Brötchen), ein Seidel (0.3L), ein Krügerl (0.5L). Beim Spritzer unterscheidet man: Weißer Spritzer (1/2 Grüner Veltliner, 1/2 Sodawasser), 'Süßer Spritzer' (mit Zitronenlimonade oder Almdudler), 'Saurer Spritzer' (mit saurem Mineralwasser). Trinkgeld (Tip): 10% ist der Höflichkeitsstandard; man rundet direkt beim Kellner (Herr Ober) auf und nennt den Gesamtbetrag ('24 Euro bitte' oder 'Stimmt so').",
                            &[
                                "Never sit at a table marked 'Stammtisch' unless invited.",
                                "Order 'Ein Seidel' (0.3L) or 'Ein Krügerl' (0.5L) — avoid saying just 'ein Bier'.",
                                "Tip ~10% by naming the rounded sum to the waiter directly.",
                                "Skip any place displaying photo menus or tourist boards outside."
                            ],
                            None,
                        ),
                        topic(
                            "at_item_falstaff_michelin",
                            "Falstaff 2026 & Michelin Stars (Vienna Haute Cuisine)",
                            "Steirereck (3*), Amador (3*), Filippou (2*), Mraz & Sohn (2*), Silvio Nickol, Tian, Pramerl & the Wolf",
                            &["michelin", "falstaff", "fine-dining", "haute-cuisine"],
                            "Wien ist eine europäische Spitzenmetropole der Hochgastronomie (Guide MICHELIN Austria 2026 & Falstaff Restaurantguide 2026):\n\
                            • Steirereck im Stadtpark (3* Michelin, 99 Falstaff-Punkte, Chef Heinz Reitbauer): Weltklasse der alpinen Produktforschung, legendärer Brotwagen mit 25 Sorten, im Bienenwachs gegarter Saibling.\n\
                            • Restaurant Amador (3* Michelin, 98 Falstaff, Juan Amador): In einem gewölbten Ziegelkeller in Grinzing, meisterhafte französisch-spanische Avantgarde.\n\
                            • Konstantin Filippou (2* Michelin, Dominikanerbastei): Griechisch-österreichischer Purismus mit Meeresfokus (z.B. Langustine mit Schnecke und Knochenmark).\n\
                            • Mraz & Sohn (2* Michelin, Wallensteinstraße 59, 1200): Punk-Haute-Cuisine ohne steife Attitüde, explosives Überraschungsmenü.\n\
                            • Silvio Nickol Gourmet Restaurant (2* Michelin, Palais Coburg): Speisen über den 60.000 Flaschen fassenden historischen Weinkellern.\n\
                            • TIAN Restaurant (1* Michelin, Himmelpfortgasse 23, 1010): Paul Ivićs radikale fleischlose Hochküche, eines der weltbesten vegetarischen Sternerestaurants.\n\
                            • Pramerl & the Wolf (1* Michelin, Pramergasse 21, 1090 Servitenviertel): Intimes Neo-Bistro in einem ehemaligen Wiener Vorstadt-Beisl.",
                            &[
                                "Book Steirereck and Amador 2-3 months ahead.",
                                "TIAN offers an entirely plant-based Michelin tasting experience.",
                                "Pramerl & the Wolf is located in the charming Servitenviertel (1090)."
                            ],
                            Some((48.2036, 16.3808)),
                        ),
                        topic(
                            "at_item_traditional_beisln",
                            "Authentic Beisln & Gasthäuser (Where Locals Dine)",
                            "Gasthaus Orlik, Gasthaus Pöschl, Woracziczky, Glacis Beisl, Servitenwirt, Plachutta Tafelspitz",
                            &["beisl", "gasthaus", "schnitzel", "tafelspitz", "servitenviertel"],
                            "Die wahren Schätze der Wiener Küche abseits der Touristenströme:\n\
                            • Gasthaus Orlik (Spittelauer Platz 1, 1090 Wien): Das urige Servitenviertel-Wirtshaus. Legendäre Tageskarte (mittag.at), knuspriges Wiener Schnitzel, saftiger Zwiebelrostbraten und ehrliche Hausmannskost.\n\
                            • Gasthaus Pöschl (Weihburggasse 17, 1010 Wien): Ein unprätentiöses Gasthaus am Franziskanerplatz. Mittags sitzen hier Stammgäste und Künstler; Schnitzel €18–22 von hoher Qualität.\n\
                            • Gasthaus Woracziczky (Spengergasse 52, 1050 Margareten): Das Paradebeispiel des Wiener Neo-Beisls. Keine feste Karte, sondern täglich wechselnde Schiefertafel mit Marktfrische (Kalbsbries, Hirschgulasch, gefüllte Paprika). Unbedingt reservieren!\n\
                            • Glacis Beisl (Breite Gasse 4, 1070 Wien): Versteckt hinter dem MuseumsQuartier unter Schatten spendenden Walnussbäumen. Legendäre Grammelknödel auf Veltlinerkraut.\n\
                            • Plachutta Stammhaus Hietzing (Auhofstraße 1, 1130) & Wollzeile (1010): Die Pilgerstätte des Wiener Tafelspitz. Serviert im glänzenden Kupfertopf mit Rindsuppe, Markknochen auf getoastetem Schwarzbrot, Apfelkren und Schnittlauchsauce.\n\
                            • Schachtelwirt (Judengasse 5, 1010): Hausmannskost modern interpretiert im kompostierbaren Karton — Schweinsbraten, Knödel und Krautfleckerl in Spitzenqualität to go.",
                            &[
                                "Woracziczky in the 5th requires reservation 3-5 days ahead.",
                                "Plachutta is the benchmark for boiled beef (Tafelspitz) and bone marrow.",
                                "Gasthaus Orlik has one of the best value lunch menus in the 9th district."
                            ],
                            Some((48.2268, 16.3556)),
                        ),
                        topic(
                            "at_item_wuerstelstand",
                            "Würstelstand & Kult-Imbisse (Late Night Institution)",
                            "Bitzinger Albertina, Alles Wurscht, Zum scharfen René, Leberkas-Pepi, LEO",
                            &["wuerstelstand", "kaesekrainer", "bosna", "leberkaese", "streetfood"],
                            "Der Würstelstand ist Wiens nächtliches Wohnzimmer und sozialer Schmelztiegel, an dem Nachtschwärmer, Bauarbeiter und Staatsopern-Besucher im Frack Schulter an Schulter stehen:\n\
                            • Bitzinger bei der Albertina (Augustinerstraße 1, 1010 Wien): Die Legende hinter der Oper. Berühmt für Käsekrainer ('Eitrige'), Bosna, serviert wahlweise mit Gösser Bier oder eisgekühltem Taittinger Champagner.\n\
                            • Alles Wurscht (Börseplatz 1, 1010 Wien): Der Sieger im 'Der Standard' Test 2026. Handwerklich hergestellte Würste, fermentiertes Kimchi, Calamari fritti und die beste Gourmet-Bosna der Stadt.\n\
                            • Zum scharfen René (Schwarzenbergplatz, 1010 Wien): Wiens Schärfe-Hotspot. Berühmt für hausgemischte Chili-Currysaucen bis zur Carolina Reaper Schärfestufe 10+.\n\
                            • Würstelstand LEO (Döblinger Gürtel 2, 1190 Wien): Wiens ältester Würstelstand (seit 1928 an der U6 Nußdorfer Straße). Bekannt für die Riesenkäsekrainer 'Big Mama'.\n\
                            • Leberkas-Pepi (Operngasse 12 & Hauptbahnhof): Wiens Leberkäse-Tempel mit 12 wechselnden Sorten von oberösterreichischen Bauernhöfen: Trüffel-Steinpilz, Röstzwiebel, Chili, Käse und traditioneller Pferdeleberkäse in der knusprigen Kaisersemmel.",
                            &[
                                "Classic order: 'A Eitrige mit an Schoafn und an 16er-Blech' (Käsekrainer with hot mustard and Ottakringer can).",
                                "Leberkas-Pepi at Operngasse is ideal after an evening performance.",
                                "Alles Wurscht offers artisan sourdough bread with its sausages."
                            ],
                            Some((48.2048, 16.3683)),
                        ),
                        topic(
                            "at_item_broetchen",
                            "Trzesniewski & Wiener Brötchen-Kultur",
                            "Dorotheergasse Stammhaus (1902) & Tauber Brötchen Manufaktur",
                            &["broetchen", "trzesniewski", "dorotheergasse", "snack"],
                            "Trzesniewski (Dorotheergasse 1, 1010 Wien): Gegründet 1902 von Franciszek Trzesniewski. 'Die unaussprechlich guten Brötchen' sind Wiens traditionsreichstes Fingerfood. Über 25 verschiedene, streng geheim gehaltene Aufstriche auf saftigem Vollkorn-Schwarzbrot (Klassiker: Speck mit Ei, Ei mit Pfefferoni, Krebsensalat). Traditionell bestellt man 3-4 Brötchen und trinkt dazu einen 'Pfiff' Bier (0.2L im Spezialglas).\n\n\
                            Tauber Brötchen Manufaktur: Gegründet 1987, mit 13 Standorten die moderne handwerkliche Alternative mit frischen Sandwich-Boxen und warmer Theke (Fleischlaibchen, warmer Leberkäse).",
                            &[
                                "Trzesniewski flagship in Dorotheergasse has no chairs — stand at the brass counter.",
                                "Order 3-4 rolls per person with a Pfiff of beer.",
                                "Try 'Speck mit Ei' (unchanged since 1902)."
                            ],
                            Some((48.2081, 16.3698)),
                        ),
                        topic(
                            "at_item_tourist_traps",
                            "Tourist Traps & Clever Alternatives (Graetzl Intel)",
                            "Figlmüller vs Pöschl, Café Sacher vs Diglas/Demel",
                            &["traps", "insider", "graetzl", "advice"],
                            "Touristenfalle 1: Figlmüller (Wollzeile / Bäckerstraße). Das tellergroße Schnitzel ist historisch belegt, aber durch 2-stündige Schlangen und industrielle Massenabfertigung geprägt. Zudem ist das berühmte Figlmüller-Schnitzel vom Schwein (nicht Kalb!). Echte Alternative: Gasthaus Pöschl (Weihburggasse) oder Gasthaus Orlik (1090) für butterzartes Wiener Kalbsschnitzel ohne Hektik.\n\n\
                            Touristenfalle 2: Café Sacher (Philharmonikerstraße). Die Schlange für die Sachertorte steht oft 100 Meter auf der Straße. Echte Alternative: Café Diglas (Wollzeile 10), K.u.K. Hofzuckerbäckerei Demel (Kohlmarkt) oder Konditorei Sluka (Rathausplatz) für handwerkliche Mehlspeisen in historischem Ambiente ohne Wartezeit.",
                            &[
                                "Real Wiener Schnitzel is strictly veal (Kalb); pork is labeled 'Wiener Art'.",
                                "Never stand in line for Sacher; Viennese pastries elsewhere are fresher and less crowded."
                            ],
                            None,
                        ),
                        topic(
                            "at_item_heurige",
                            "Heurige & Historische Weinkeller",
                            "Fuhrgassl-Huber (Neustift am Walde) & Zwölf-Apostelkeller (1327)",
                            &["wine", "heuriger", "gemischter-satz", "cellar"],
                            "Wiener Gemischter Satz DAC ist das weltweit einzigartige Phänomen eines Qualitätsweins aus Weinbergen innerhalb einer Millionenstadt:\n\
                            • Fuhrgassl-Huber (Neustift am Walde 68, 1190 Wien): Malerischer Winzerhof mit Schanigarten am Hang der Weinberge. Selbstbedienungsbuffet mit hausgemachten Aufstrichen, Kümmelbraten, Surschnitzel und jungem Wein.\n\
                            • Zwölf-Apostelkeller (Sonnenfelsgasse 3, 1010 Wien): Unterirdischer gotischer Keller aus dem 14. Jahrhundert, der sich über drei Stockwerke in die Tiefe erstreckt.",
                            &[
                                "Take Bus 35A from Spittelau out to Neustift am Walde for Heurige.",
                                "Order Wiener Gemischter Satz DAC.",
                                "At a traditional Heuriger, order drinks at the table and warm food at the buffet counter."
                            ],
                            Some((48.2536, 16.3052)),
                        ),
                        topic(
                            "at_item_wiener_schnitzel",
                            "Wiener Schnitzel: The Sacred Veal & Panier Law",
                            "Austrian food codex, Butterschmalz soufflé, Erdäpfelsalat rules",
                            &["schnitzel", "veal", "codex", "tradition", "butterschmalz"],
                            "Nach dem Österreichischen Lebensmittelbuch darf die Bezeichnung 'Wiener Schnitzel' ausschließlich für Kalbfleisch verwendet werden (Schwein muss als 'Schnitzel nach Wiener Art' deklariert sein). Die Zubereitung ist ein strenges Handwerk:\n\
                            • Fleisch: Aus der Kalbsnuss oder der Kaiserteil-Schale, behutsam auf 3–4 mm plattiert (niemals mit scharfem Zackenklopfer zerfasert!).\n\
                            • Panier (Wiener Panier): Mehl (glatt), geschlagenes Ei (mit einem Esslöffel Obers oder Wasser verquirlt), feine Bäcker-Semmelbrösel (niemals industrielles Paniermehl!). Die Panier darf nicht fest angedrückt werden, damit sie beim Backen 'souffliert' — d.h. wellenförmig und luftig vom Fleisch abhebt.\n\
                            • Backen: Schwimmend in reichlich geklärtem Schweineschmalz oder Butterschmalz bei 160–180°C unter ständigem Schwenken (Welligkeit).\n\
                            • Beilage: Wiener Erdäpfelsalat (mit Rindsuppe, Zwiebeln, Essig und Öl warm mariniert — Mayonnaise ist im Wiener Beisl verpönt!), Preiselbeerkompott und eine frische Zitronenspalte.",
                            &[
                                "Look for soufflierte (wavy, detached) golden crust.",
                                "Order with warm Austrian potato salad (Erdäpfelsalat), never french fries.",
                                "Squeeze lemon directly on the crust before the first bite."
                            ],
                            Some((48.2085, 16.3738)),
                        ),
                        topic(
                            "at_item_sachertorte_rivalry",
                            "The Great Sachertorte Feud: Hotel Sacher vs Demel",
                            "The 25-year courtroom battle (1938–1965) between Hotel Sacher and K.u.K. Hofzuckerbäckerei Demel",
                            &["sachertorte", "demel", "sacher", "rivalry", "history"],
                            "Der legendäre 'Wiener Sachertortenstreit' beschäftigte über ein Vierteljahrhundert lang die Gerichte:\n\
                            1832 kreierte der 16-jährige Kochlehrling Franz Sacher am Hofe des Fürsten Metternich eine Schokoladentorte mit Marillenmarmelade. Sein Sohn Eduard Sacher vollendete die Rezeptur während seiner Ausbildung bei der K.u.K. Hofzuckerbäckerei Demel am Kohlmarkt. Nach der Pleite des Hotel Sacher in den 1930er Jahren verkaufte Eduards Sohn das Originalrezept an Demel.\n\
                            Es folgte ein erbitterter Rechtsstreit um die Namensrechte:\n\
                            • Hotel Sacher: 'Original Sacher-Torte' — zeichnet sich durch zwei Lagen Marillenmarmelade (eine in der Mitte durchgeschnitten, eine unter der Glasur) und ein rundes Schokoladensiegel aus.\n\
                            • Hofzuckerbäckerei Demel: 'Eduard-Sacher-Torte' (Demel's Sachertorte) — besitzt nur eine einzige Marillenschicht direkt unter der dunklen Schokoglasur und ein dreieckiges Siegel.\n\
                            Beide schmecken hervorragend, sollten jedoch traditionell stets mit einer großzügigen Portion ungesüßtem Schlagobers (Schlagsahne) serviert werden.",
                            &[
                                "Demel's torte has one jam layer directly under glaze; Hotel Sacher's is sliced through the middle with two layers.",
                                "Always eat Sachertorte with unsweetened Schlagobers (whipped cream) to balance the chocolate density.",
                                "Demel on Kohlmarkt lets you watch master pastry chefs through glass walls in the salon."
                            ],
                            Some((48.2086, 16.3670)),
                        ),
                    ],
                },
                InfoBranch {
                    id: "at_branch_coffee".to_string(),
                    title: "Viennese Kaffeehaus Culture & Etiquette".to_string(),
                    icon: "☕".to_string(),
                    items: vec![
                        topic(
                            "at_item_coffee_types",
                            "Viennese Coffee Menu & Lexicon",
                            "Melange, Verlängerter, Einspänner, Kleiner Brauner, Franziskaner",
                            &["kaffeehaus", "coffee", "melange", "einspaenner"],
                            "Bestellen wie ein Wiener:\n\
                            • Wiener Melange: Halb milder Espresso, halb heiße Milch mit Milchschaumhaube.\n\
                            • Verlängerter: Espresso mit der doppelten Menge heißem Wasser gestreckt (schwarz oder braun mit Milch).\n\
                            • Einspänner: Doppelter Mokka im Henkelglas mit üppiger kalter Schlagobershaube (ursprünglich für Kutscher, hält Kaffee warm).\n\
                            • Kleiner / Großer Schwarzer (Mokka): Reiner Espresso ohne Milch.\n\
                            • Kleiner / Großer Brauner: Mokka serviert mit einem kleinen Kännchen Obers (Milch) zum Selbstgießen.\n\
                            • Franziskaner: Melange getoppt mit Schlagobers statt Milchschaum.\n\
                            • Fiaker: Großer Mokka im Glas mit einem Schuss Kirschwasser oder Rum.",
                            &[
                                "Never order just 'Kaffee' — specify the precise preparation.",
                                "Every coffee is served on a silver tray with a glass of tap water and an inverted spoon on top.",
                                "Drink a sip of water before the coffee to cleanse the palate."
                            ],
                            None,
                        ),
                        topic(
                            "at_item_coffee_etiquette",
                            "Kaffeehaus Etiquette & The Herr Ober",
                            "Newspaper sticks, marble tables, and lingering without rush",
                            &["etiquette", "culture", "herr-ober", "unesco"],
                            "Wiener Kaffeehauskultur ist seit 2011 immaterielles UNESCO-Kulturerbe: 'Ein Ort, in dem Raum und Zeit verzehrt werden, aber nur der Kaffee auf der Rechnung steht.'\n\
                            Der Oberkellner wird traditionell als 'Herr Ober' angesprochen. Nach dem Servieren des Kaffees kann man stundenlang ungestört an den Thonet-Holzstühlen und Marmortischen verweilen, in den Holzstangen-Zeitungen (Internationale Presse) blättern oder schreiben, ohne nachbestellen zu müssen.",
                            &[
                                "Address the waiter as 'Herr Ober'.",
                                "Reading newspapers on wooden sticks is completely free.",
                                "Take your time — rushing in a traditional Kaffeehaus is considered poor form."
                            ],
                            None,
                        ),
                        topic(
                            "at_item_iconic_cafes",
                            "Legendary Coffeehouses (Central, Hawelka, Sperl, Berg, Aida)",
                            "Historic institutions across Vienna's districts",
                            &["cafes", "hawelka", "sperl", "central", "aida"],
                            "• Café Central (Herrengasse 14, 1010): Neorenaissance-Arkaden, einst Treffpunkt von Trozki, Freud und Altenberg.\n\
                            • Café Hawelka (Dorotheergasse 6, 1010): Legendäre Künstler-Kaffeestube; berühmt für warme Buchteln mit Powidl, die erst ab 22:00 Uhr aus dem Ofen kommen.\n\
                            • Café Sperl (Gumpendorfer Str. 11, 1060): 1880 eröffnet, original erhaltene Billardtische und Plüschlogen.\n\
                            • Café Berg (Berggasse 8, 1090 Alsergrund): Intellektueller Treffpunkt unweit des Sigmund-Freud-Museums mit eigenem Hausbrand.\n\
                            • Aida Konditoreien (34 Filialen, Stamm Wollzeile): Wiens älteste pinke Konditoreikette seit 1913; führte 1946 die erste Espressomaschine Österreichs ein.",
                            &[
                                "Arrive at Hawelka after 22:00 to catch fresh Buchteln.",
                                "Café Sperl has pristine historic Carambole billiards tables.",
                                "Café Central has queues at peak times; visit early morning before 10:00."
                            ],
                            Some((48.2104, 16.3653)),
                        ),
                    ],
                },
                InfoBranch {
                    id: "at_branch_art".to_string(),
                    title: "Museums & Masterpieces (Caravaggio, Klimt, Bruegel)".to_string(),
                    icon: "🎨".to_string(),
                    items: vec![
                        topic(
                            "at_item_khm_art",
                            "Kunsthistorisches Museum: Caravaggio & Bruegel",
                            "Rosenkranzmadonna, David with Goliath's Head, Tower of Babel",
                            &["khm", "caravaggio", "bruegel", "renaissance"],
                            "Das KHM beherbergt eine der weltbedeutendsten Gemäldegalerien der Renaissance und des Barock:\n\
                            • Caravaggio-Saal (Saal X): 'Rosenkranzmadonna' (1606-07) — Caravaggios letztes großes neapolitanisches Altarbild im Chiaroscuro-Dunkel mit bettelnden nackten Füßen; und 'David mit dem Haupt des Goliath' (sein Selbstporträt als geköpfter Riese).\n\
                            • Bruegel-Saal (Saal XII): Die weltgrößte Sammlung von Pieter Bruegel d. Ä., darunter 'Der Turm zu Babel' (1563), 'Die Jäger im Schnee' und 'Bauernhochzeit'.\n\
                            • Kuppelcafé: Eines der prachtvollsten Museumscafés der Welt für eine Melange-Pause unter der monumentalen Neorenaissance-Kuppel.",
                            &[
                                "Caravaggio room: Staircase I, Saal X.",
                                "Bruegel collection: Saal XII.",
                                "Visit Thursday evenings for late opening until 21:00 with fewer crowds."
                            ],
                            Some((48.2039, 16.3619)),
                        ),
                        topic(
                            "at_item_belvedere_art",
                            "Belvedere Palace: Klimt's Kiss & Schiele",
                            "Upper Belvedere, Der Kuss (1908-09), Egon Schiele, Marmorsaal",
                            &["belvedere", "klimt", "schiele", "gold-phase"],
                            "Das Obere Belvedere von Prinz Eugen von Savoyen beherbergt Gustav Klimts 'Der Kuss' (1908-09), die weltberühmte Ikone der Wiener Moderne aus Blattgold, Silber und Öl auf Leinwand. Daneben Meisterwerke von Egon Schiele ('Tod und Mädchen') und Oskar Kokoschka. Im historischen Marmorsaal wurde 1955 der Österreichische Staatsvertrag unterzeichnet.",
                            &[
                                "Book timeslot online for Upper Belvedere.",
                                "The palace baroque gardens connecting Upper and Lower Belvedere are free to stroll dawn to dusk."
                            ],
                            Some((48.1915, 16.3809)),
                        ),
                    ],
                },
                InfoBranch {
                    id: "at_branch_imperial".to_string(),
                    title: "Imperial Vienna & Classical Music".to_string(),
                    icon: "👑".to_string(),
                    items: vec![
                        topic(
                            "at_item_hofburg_schatzkammer",
                            "Hofburg Imperial Treasury & Crown Jewels",
                            "Reichskrone (962 AD), Holy Lance, Order of the Golden Fleece",
                            &["hofburg", "treasury", "schatzkammer", "habsburg"],
                            "Die Kaiserliche Schatzkammer in der Wiener Hofburg birgt die Insignien des Heiligen Römischen Reiches Deutscher Nation: Die achteckige Reichskrone aus reinem Gold und Edelsteinen (um 962 AD), die Heilige Lanze und das Reichskreuz. Dazu die Österreichische Kaiserkrone Rudolfs II. (1602) und der Schatz des Ordens vom Goldenen Vlies.",
                            &[
                                "Closed on Tuesdays.",
                                "Audio guide is highly recommended for the medieval coronation regalia."
                            ],
                            Some((48.2065, 16.3655)),
                        ),
                        topic(
                            "at_item_staatsoper_stehplatz",
                            "Staatsoper Standing Room Insider Guide (€10–€15)",
                            "World-class opera for pocket change: Stehplatz protocol",
                            &["staatsoper", "opera", "stehplatz", "budget"],
                            "Die Wiener Staatsoper bietet eines der besten Musik-Schnäppchen der Welt: Rund 80 Minuten vor Vorstellungsbeginn öffnen die Stehplatzkassen an der Operngasse. Für €10 bis €15 erhält man Stehplätze im Parterre oder auf der Galerie. Wiener Brauch: Man bindet einen Seidenschal um die Messingstange des Platzes, um ihn während der Pausen zu reservieren.",
                            &[
                                "Queue ~80-90 minutes before curtain at the Operngasse entrance.",
                                "Bring a scarf or cloth to tie to the brass railing to reserve your spot during intermission.",
                                "Smart casual dress code recommended even for standing room."
                            ],
                            Some((48.2030, 16.3691)),
                        ),
                        topic(
                            "at_item_schoenbrunn_palace",
                            "Schönbrunn Imperial Residence & Gloriette",
                            "1,441 Rococo rooms, Maria Theresa, Gloriette panorama, Tiergarten 1752",
                            &["schoenbrunn", "palace", "gloriette", "zoo"],
                            "Sommerresidenz der Habsburger mit 1.441 Zimmern, prunkvollen Rokoko-Appartements Maria Theresias und des jungen Mozart im Spiegelsaal. Der weitläufige Park umfasst die Aussichtskolonnade der Gloriette und den 1752 gegründeten Tiergarten Schönbrunn — den ältesten bestehenden Zoo der Welt.",
                            &[
                                "Park grounds and hike to Gloriette are free.",
                                "Pre-book Grand Tour timeslot online to skip ticket lines.",
                                "Gloriette café has panoramic views over Vienna at golden hour."
                            ],
                            Some((48.1858, 16.3122)),
                        ),
                    ],
                },
                InfoBranch {
                    id: "at_branch_transit".to_string(),
                    title: "Wiener Linien & Regional Corridors".to_string(),
                    icon: "🚋".to_string(),
                    items: vec![
                        topic(
                            "at_item_wiener_linien",
                            "Wiener Linien U-Bahn, Ring Trams & Day Ticket",
                            "U1-U6 grid, Trams 1/2/71/D, 24h weekend service, €8.00 day ticket",
                            &["transit", "u-bahn", "tram", "tickets"],
                            "Wiens öffentliches Verkehrsnetz zählt zu den effizientesten der Welt. Die Ring-Straßenbahnen (Linien 1, 2, 71, D) umrunden die historische Altstadt entlang Prachtbauten wie Parlament, Rathaus und Burgtheater. An Wochenenden verkehren die U-Bahn-Linien durchgehend 24 Stunden im 15-Minuten-Takt. Das 24-Stunden-Netzticket kostet €8.00.",
                            &[
                                "Buy 24h or 48h ticket at red ticket machines or WienMobil app.",
                                "Honesty system: validate paper ticket once before first ride; heavy €105 penalty for unvalidated rides.",
                                "Take Tram D from Hauptbahnhof via Ringstraße directly to Nussdorf."
                            ],
                            None,
                        ),
                        topic(
                            "at_item_alpine_corridor",
                            "Wachau Danube Terraces & Hallstatt Salt Mine",
                            "ÖBB Railjet, Wachau wine path, prehistoric salt galleries",
                            &["danube", "wachau", "hallstatt", "alpine"],
                            "Die zwei großen Chrono-Korridore Österreichs:\n\
                            • Wachau (UNESCO-Welterbe): Flusslandschaft der Donau zwischen Stift Melk und Krems, steile Riesling-Terrassen und die Ruine Dürnstein (wo Richard Löwenherz gefangen saß).\n\
                            • Hallstatt (Salzkammergut): 7.000 Jahre prähistorischer Salzabbau in den Salzwelten über dem fjordartigen Dachstein-Gletschersee.",
                            &[
                                "ÖBB Railjet connects Wien Hbf to Melk in 50 minutes.",
                                "Rent a bike in Melk and cycle the 39km Danube trail to Krems.",
                                "Hallstatt salt mine requires booking a funicular + mine entry slot."
                            ],
                            Some((47.5622, 13.6493)),
                        ),
                    ],
                },
                InfoBranch {
                    id: "at_branch_alpine".to_string(),
                    title: "Alpine Culture, Hut Etiquette & Mountain Safety".to_string(),
                    icon: "⛰️".to_string(),
                    items: vec![
                        topic(
                            "at_item_huettenordnung",
                            "Alpine Hut Etiquette (Hüttenordnung & Bergwacht)",
                            "DAV/ÖAV rules, Hüttenschlafsack, Hüttenruhe 22:00, Emergency 140",
                            &["alps", "hiking", "safety", "huette", "oeav"],
                            "Auf den über 500 Alpenvereinshütten (ÖAV & DAV) in den österreichischen Alpen gelten feste Regeln der Bergkameradschaft:\n\
                            • Hüttenschlafsack-Pflicht: Auf Bettenlagern und Matratzenlagern ist aus hygienischen Gründen ein eigener dünner Seiden- oder Baumwollschlafsack zwingend vorgeschrieben (Hüttendecken sind vorhanden).\n\
                            • Bergschuhe ausziehen: Bergschuhe gehören sofort im Eingangsbereich in den Trockenraum (Schuhregal); in der Hütte trägt man Hüttenschuhe oder Pantoletten.\n\
                            • Hüttenruhe: Punkt 22:00 Uhr herrscht absolute Hüttenruhe. Bergsteiger brechen oft vor Sonnenaufgang (05:00 Uhr) auf.\n\
                            • Müll mitnehmen: Auf Berghütten gibt es keine städtische Müllabfuhr — jeder Bergsteiger nimmt seinen gesamten Müll restlos wieder ins Tal.\n\
                            • Bargeld: Viele alpine Schutzhütten haben kein Mobilfunknetz oder Kartenterminal — immer ausreichend Bargeld mitführen!\n\
                            • Alpine Notrufnummer: 140 (Österreichischer Bergrettungsdienst), 112 (Euro-Notruf).",
                            &[
                                "Pack a silk sleeping bag liner (Hüttenschlafsack).",
                                "Alpine emergency number is 140.",
                                "Take every scrap of personal trash back down into the valley.",
                                "Carry cash for all food and overnight hut fees."
                            ],
                            Some((47.0742, 12.6947)),
                        ),
                        topic(
                            "at_item_salzkammergut_lakes",
                            "Salzkammergut Lakes & Dachstein Glacier",
                            "Hallstättersee, Lake Wolfgang, Traunsee & Holzknechtnocken",
                            &["lakes", "salzkammergut", "dachstein", "hallstatt"],
                            "Das Salzkammergut umfasst 76 glasklare Alpenseen, eingebettet zwischen dem Kalksteinmassiv des Dachsteins und sanften Voralpenhügeln:\n\
                            • Hallstätter See: Fjordartiges Gewässer mit Weltkulturerbe-Blick. Mit der Holzplätte (traditionelles Flachboot) über das spiegelnde Wasser gleiten.\n\
                            • Wolfgangsee & Schafbergbahn: Die steilste Zahnrad-Dampfbahn Österreichs (seit 1893) erklimmt den 1.783m hohen Schafberg mit 360°-Blick über sieben Seen.\n\
                            • Holzknechtnocken: Traditionelle Mahlzeit der Holzarbeiter aus Mehl, Butterschmalz und Wasser, wahlweise süß mit Apfelmus oder herzhaft serviert.",
                            &[
                                "Ride the historic Schafbergbahn steam cog railway in St. Wolfgang.",
                                "Water temperature in mountain lakes peaks around 20-22°C in late July.",
                                "Take the Dachstein Krippenstein cable car to the Giant Ice Cave (Rieseneishöhle)."
                            ],
                            Some((47.5622, 13.6493)),
                        ),
                    ],
                },
            ],
        }
    }

    /// Full Japanese Knowledge Tree (Transit, Shinkansen, Konbini, Shrine Etiquette, Regions).
    pub fn get_japan_knowledge_tree() -> KnowledgeTree {
        KnowledgeTree {
            country_code: "JP".to_string(),
            title: "JapanCraft: Sovereign Baedeker Knowledge Tree".to_string(),
            branches: vec![
                InfoBranch {
                    id: "jp_branch_transit".to_string(),
                    title: "Transit, Shinkansen & IC Cards".to_string(),
                    icon: "🚅".to_string(),
                    items: vec![
                        topic(
                            "jp_item_jr_pass",
                            "JR Pass Rules, Luggage Restrictions & Strategy",
                            "7/14/21 day national vs regional passes, Nozomi surcharge, oversized baggage",
                            &["transit", "jr-pass", "shinkansen", "luggage"],
                            "Der Japan Rail Pass (7, 14 oder 21 Tage) deckt Shinkansen-Schnellzüge, JR-Regionallinien und den Narita Express ab. Beachte:\n\
                            • Nozomi & Mizuho: Für die schnellsten Shinkansen-Züge ist seit 2023 ein Zusatzticket erforderlich; Hikari und Sakura sind voll abgedeckt.\n\
                            • Großgepäck (Oversized Baggage): Gepäckstücke mit Gesamtumfang (L+B+H) über 160 cm bis 250 cm erfordern zwingend eine kostenlose Sitzplatzreservierung mit Gepäckbereich in den Tokaido/Sanyo/Kyushu Shinkansen.\n\
                            • Regionalpässe (z.B. JR Kansai Wide Area Pass oder JR Hokuriku Arch) sind oft wirtschaftlicher als der landesweite Pass.",
                            &[
                                "Exchange your voucher at JR Travel Service Centers (Tokyo/Shinagawa/KIX).",
                                "Reserve oversized luggage seats when picking up Shinkansen tickets.",
                                "Use Japan Travel by NAVITIME app with 'JR Pass' filter enabled."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_ic_cards",
                            "Suica, Pasmo & Icoca Cards (Mobile Wallet Guide)",
                            "Contactless tap-and-go for subways, local buses, konbini & coin lockers",
                            &["transit", "suica", "pasmo", "icoca", "ic-card"],
                            "IC-Karten (Suica, Pasmo im Kanto-Raum; Icoca in Kansai) sind vollständig interoperabel und der Schlüssel zum bargeldlosen Reisen. Man tippt sie an Bahn-Drehkreuzen, städtischen Bussen, Konbini-Kassen, Getränkeautomaten und Bahnhof-Schließfächern (Coin Lockers). Die digitale Suica / Pasmo kann in Apple Wallet oder Google Wallet hinzugefügt und mit Kreditkarte aufgeladen werden.",
                            &[
                                "Add Suica/Pasmo directly to your phone wallet for zero-wait boarding.",
                                "Physical 'Welcome Suica' cards are available at Haneda/Narita with 28-day validity.",
                                "Keep at least JPY 1,500 balance for hassle-free transfers."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_shinkansen_etiquette",
                            "Shinkansen Manners, Platform Markings & Eki-melo",
                            "Queuing lines, silent carriages, eki-bento culture, luggage racks",
                            &["shinkansen", "etiquette", "eki-bento", "manners"],
                            "Shinkansen-Reisen folgt einem präzisen Ritual:\n\
                            • Auf dem Bahnsteig: Linienmarkierungen auf dem Boden zeigen exakt Wagentür und Wartenummer.\n\
                            • Im Zug: Telefongespräche sind im Passagierabteil tabu — wechsle in den Vorraum zwischen den Waggons.\n\
                            • Eki-bento: Vor der Abfahrt kauft man am Bahnhof ein kunstvolles Ekiben (Bahnhofs-Bento mit regionalen Spezialitäten) und isst es genussvoll während der Fahrt.",
                            &[
                                "Line up strictly at your car door marking on the platform.",
                                "Switch phones to 'Manner Mode' (silent) and use earphones.",
                                "Recline your seat only after politely asking the passenger behind you."
                            ],
                            None,
                        ),
                    ],
                },
                InfoBranch {
                    id: "jp_branch_dining".to_string(),
                    title: "Konbini & Dining Etiquette".to_string(),
                    icon: "🍱".to_string(),
                    items: vec![
                        topic(
                            "jp_item_konbini_survival",
                            "Konbini Guide: 7-Eleven, Lawson, FamilyMart & 19h Time Sales",
                            "Onigiri, tamago sando, hot cabinets, and 20-50% evening discounts",
                            &["konbini", "7-eleven", "lawson", "familymart", "budget"],
                            "Japans 55.000 Konbini sind kulinarische Wunderkammern des Alltags:\n\
                            • 7-Eleven: Weltberühmtes Tamago Sando (Eiersalat-Sandwich auf butterweichem Shokupan) und Premium-Onigiri (Lachs, Umeboshi, Tuna-Mayo).\n\
                            • Lawson: Legendäres 'Karaage-kun' (knusprige panierte Hähnchenhappen an der Heißtheke).\n\
                            • FamilyMart: 'FamiChiki' frittiertes Hähnchenfilet.\n\
                            • 19:00–20:00 Time Sales (Timuri): Bento-Boxen und frisches Sushi werden abends mit gelben Aufklebern um 20% bis 50% rabattiert.",
                            &[
                                "Pinch pull-tabs 1-2-3 on onigiri packaging to keep nori seaweed crisp.",
                                "Ask 'Atatamemasu ka?' for staff to microwave bentos in 30 seconds.",
                                "Hot canned coffees and teas are in the red-shelf heated cabinets."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_ramen_slurping",
                            "Ramen, Soba & Slurping Etiquette (Susuru)",
                            "Shokkenki ticket vending machines, cooling noodles, zero lingering",
                            &["ramen", "soba", "susuru", "noodles"],
                            "Das geräuschvolle Schlürfen (Susuru) von Ramen und Soba ist in Japan kein Verstoß, sondern erwünscht: Es kühlt die heißen Nudeln im Mund ab, verstärkt das Aroma der Brühe und zeigt dem Koch Wertschätzung. Fast alle Ramen-Bars arbeiten mit Shokkenki (Ticketautomaten am Eingang). Man wirft Bargeld ein, drückt den Nudel-Button und gibt den Beleg am Tresen ab.",
                            &[
                                "Buy your ticket at the machine before sitting down.",
                                "Slurp vigorously — it cools the broth and aerates aromatics.",
                                "Do not linger on your phone after finishing if people are waiting outside."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_izakaya_otoshi",
                            "Izakaya Rules, Otoshi Seating Fee & Pouring Etiquette",
                            "Table charge snacks, shouting 'Sumimasen!', and communal kanpai",
                            &["izakaya", "otoshi", "kanpai", "beer"],
                            "Die Izakaya ist die traditionelle japanische Kneipe:\n\
                            • Otoshi: Nach dem Hinsetzen erhält jeder Gast automatisch ein kleines Schälchen mit einer Vorspeise (Otoshi). Dies dient als gesetzlicher Sitzplatzaufschlag (ca. JPY 300–500).\n\
                            • Bestellung: Rufe laut und selbstbewusst 'Sumimasen!' (Entschuldigung) zum Servicepersonal.\n\
                            • Einschenken: Schenke niemals dein eigenes Bierglas voll — schenke zuerst deinem Gegenüber ein; dieser revanchiert sich.",
                            &[
                                "Expect the small Otoshi cover charge on your bill.",
                                "Say 'Kanpai!' only after everyone has been served.",
                                "Pour drinks with two hands for elder or senior companions."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_kaiseki_washoku",
                            "Kaiseki Seasonal Gastronomy (Shun Philosophy)",
                            "Seasonal washoku harmony, five flavors, and ceramic artistry",
                            &["kaiseki", "washoku", "unesco", "seasonal"],
                            "Kaiseki-Ryori ist die Krönung der japanischen Kulinarik, verwurzelt in der Teezeremonie von Kyoto. Das Herzstück ist 'Shun' — die Verwendung von Zutaten im exakten Moment ihres jahreszeitlichen Höhepunkts (z.B. Bambussprossen im Frühling, Ayu-Süßwasserfisch im Sommer, Matsutake-Pilze im Herbst). Jedes Gericht wird auf kunstvoll abgestimmter Keramik und Lackware serviert.",
                            &[
                                "Kaiseki courses take 2 to 3 hours; savor each course slowly.",
                                "Eat smaller side garnishes in the order presented.",
                                "Avoid strong perfume that masks subtle dashi and cedar aromas."
                            ],
                            None,
                        ),
                    ],
                },
                InfoBranch {
                    id: "jp_branch_culture".to_string(),
                    title: "Culture & Shinto/Buddhist Etiquette".to_string(),
                    icon: "⛩️".to_string(),
                    items: vec![
                        topic(
                            "jp_item_shrine_vs_temple",
                            "Shinto Shrine (Jinja) vs Buddhist Temple (Otera)",
                            "Torii vs Sanmon, Kami spirits vs Buddha, incense vs bells",
                            &["shinto", "buddhism", "shrine", "temple"],
                            "Shinto-Schreine (Jinja) erkennt man am roten oder hölzernen Torii-Tor am Eingang; sie ehren Naturgeister (Kami). Buddhistische Tempel (Otera) besitzen monumentale Sanmon-Tore, Pagoden und große Weihrauchbecken (Jokoro). Shinto gehört dem Leben (Geburten, Hochzeiten, Neujahr); Buddhismus begleitet das Jenseits und die Ahnen.",
                            &[
                                "Bow slightly before stepping under a Shinto Torii gate.",
                                "Walk along the sides of the path (Sando); the center is reserved for Kami.",
                                "Wave fragrant smoke from temple Jokoro incense toward your head for health."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_temizuya_bowing",
                            "Temizuya Purification & 2-Bows 2-Claps Shrine Ritual",
                            "Hand cleansing pavilion and the prayer ritual at the altar",
                            &["temizuya", "ritual", "worship", "clapping"],
                            "Vor dem Gebet am Schrein reinigt man Geist und Körper am Temizuya-Wasserbecken:\n\
                            1. Schöpfkelle mit rechter Hand füllen, linke Hand waschen.\n\
                            2. Kelle in linke Hand wechseln, rechte Hand waschen.\n\
                            3. Kelle wieder rechts: Wasser in die linke Handfläche gießen und Mund ausspülen (niemals direkt aus der Kelle trinken!).\n\
                            4. Kelle senkrecht aufstellen, damit das Restwasser den Griff für den Nächsten abspült.\n\n\
                            Am Altar: Münze in den Opferkasten werfen (5-Yen-Münze mit Loch bringt 'Go-en' / Glück). 2 tiefe Verbeugungen (Ni-rei), 2 Händeklatschen (Ni-hai), stilles Gebet, 1 abschließende tiefe Verbeugung (Ichi-rei).",
                            &[
                                "Remember the rule: 2 bows, 2 claps, 1 bow (at Shinto shrines only; never clap at Buddhist temples!).",
                                "Toss a 5-yen coin for good fortune connection ('go-en').",
                                "Do not drink directly from the purification ladle."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_genkan_shoes",
                            "Footwear (Genkan) Protocol & Restroom Slippers",
                            "Raised tatami thresholds, shoe direction, and the toilet slipper trap",
                            &["genkan", "shoes", "tatami", "slippers"],
                            "Überall dort, wo ein erhöhter Holzboden oder Tatami-Matten beginnen (Ryokan, Tempel, traditionelle Lokale), müssen Straßenschuhe zwingend ausgezogen werden. Man tritt aus den Schuhen direkt auf den Holzsteg, ohne mit den Strümpfen den Steinboden zu berühren, und dreht die Schuhe mit den Spitzen zur Tür um. In Toiletten stehen spezielle WC-Pantoletten bereit — diese niemals im Wohnbereich anlassen!",
                            &[
                                "Always wear clean, hole-free socks when traveling in Japan.",
                                "Never let bare feet or socks touch the outdoor stone genkan floor.",
                                "Remember to step out of restroom slippers before returning to tatami rooms."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_bowing_angles",
                            "The Three Bows: Eshaku (15°), Keirei (30°), Saikeirei (45°)",
                            "Degrees of Japanese bowing from casual greeting to deep reverence",
                            &["bowing", "ojigi", "manners", "respect"],
                            "Verbeugungen (Ojigi) drücken Respekt und Harmonie aus:\n\
                            • Eshaku (15°): Beiläufiges Nicken zur Begrüßung im Vorbeigehen oder beim Betreten von Geschäften.\n\
                            • Keirei (30°): Höfliche Standard-Verbeugung für Kunden, Geschäftspartner und Schreine.\n\
                            • Saikeirei (45°): Tiefe Verbeugung bei aufrichtiger Entschuldigung oder höchstem Respekt.",
                            &[
                                "Keep your back straight while bowing from the hips.",
                                "Men keep hands at their sides; women clasp hands in front.",
                                "A gentle 15° nod suffices for almost all everyday travel situations."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_public_manners",
                            "Public Etiquette: Tabearuki, Silent Commutes & Trash Protocol",
                            "No walking while eating, whisper-quiet trains, carrying rubbish home",
                            &["manners", "trains", "trash", "tabearuki"],
                            "Drei goldene Verhaltensregeln in der Öffentlichkeit:\n\
                            • Kein Essen im Gehen (Tabearuki): Verzehre Snacks direkt am Konbini oder Stand.\n\
                            • Ruhe in Zügen: Kein lautes Sprechen, kein Telefonieren in der Bahn.\n\
                            • Müll-Disziplin: Öffentliche Mülleimer sind selten; nimm eine kleine Plastiktüte mit, um deinen Müll im Hotel oder Konbini zu entsorgen.",
                            &[
                                "Carry a small plastic bag for personal trash in your daypack.",
                                "Never talk on the phone inside trains or subways.",
                                "Blow your nose in private or restrooms, never loudly in public."
                            ],
                            None,
                        ),
                        topic(
                            "jp_item_onsen_ryokan_protocol",
                            "Onsen & Ryokan Protocol: The Sacred Bathing Ritual",
                            "Total nudity, pre-wash washing stool, tattoo rules, and yukata folding",
                            &["onsen", "ryokan", "bathing", "etiquette", "yukata"],
                            "Das traditionelle japanische Onsen-Bad ist eine uralte Reinigungszeremonie in geothermalem Thermalwasser:\n\
                            1. Auskleiden: Vor dem Betreten des Nassbereichs zieht man sich im Umkleideraum komplett nackt aus. Badekleidung ist im Onsen streng verboten!\n\
                            2. Waschen VOR dem Becken: Man setzt sich auf einen kleinen Plastikhocker vor den Duscharmaturen und wäscht sich gründlich von Kopf bis Fuß mit Seife und Shampoo, bevor man auch nur einen Zeh ins Thermalbecken taucht.\n\
                            3. Das kleine Handtuch (Tenugui): Darf niemals das heiße Onsen-Wasser berühren. Man faltet es auf dem Kopf oder legt es an den Beckenrand.\n\
                            4. Tattoos: Viele traditionelle Onsen verbieten Tätowierungen wegen historischer Yakuza-Assoziationen. Kleinere Tattoos können mit wasserfesten hautfarbenen Pflastern (Seal) überklebt werden; alternativ bucht man ein Ryokan mit privatem Bad (Kashikiri-Buro).\n\
                            5. Yukata-Kimonos: Im Ryokan trägt man Yukata. Goldene Lebensregel: Die LINKE Seite wird ÜBER die rechte Seite geschlagen! Rechts-über-links ist im Shinto/Buddhismus ausschließlich Leichen bei der Bestattung vorbehalten.",
                            &[
                                "Always fold yukata left side over right side (left-over-right = life; right-over-left = burial).",
                                "Never submerge your head or small hand towel in onsen water.",
                                "Wash and rinse completely on the stool before entering the communal pool.",
                                "Book a ryokan with 'kashikiri' (private rentable onsen) if heavily tattooed."
                            ],
                            Some((35.1917, 139.0253)),
                        ),
                        topic(
                            "jp_item_kyoto_walking_trails",
                            "Kyoto Chrono-Walks: Philosopher's Path & Arashiyama",
                            "Tetsugaku no Michi stone canal path, bamboo grove, and dawn crowd avoidance",
                            &["kyoto", "walks", "philosophers-path", "arashiyama", "temples"],
                            "Die erhabensten historischen Spaziergänge in Kyoto:\n\
                            • Philosopher's Path (Tetsugaku no Michi): Ein 2 km langer, von Steinplatten gesäumter Fußweg entlang eines von Kirschbäumen beschatteten Kanals am Fuße der östlichen Higashiyama-Berge. Benannt nach Philosoph Nishida Kitaro, der hier täglich zur Meditation wandelte. Verbindet den Silbernen Pavillon (Ginkaku-ji) mit dem Nanzen-ji.\n\
                            • Arashiyama Bambuswald: Ein märchenhafter Hohlweg durch meterhohe, im Wind knarrende Bambusstangen. Tipp: Unbedingt vor 07:30 Uhr morgens besuchen, bevor die Ausflugsbusse eintreffen.\n\
                            • Fushimi Inari Dämmerungspfad: Den 4 km langen Rundweg durch die 10.000 Torii-Tore nach 17:30 Uhr im Abendlicht besteigen — die Scharen schwinden, Laternen glimmen, und Steinfüchse wachen im Schatten.",
                            &[
                                "Walk Philosopher's Path from Ginkaku-ji south toward Nanzen-ji in late afternoon.",
                                "Arashiyama bamboo grove is completely quiet only before 07:30.",
                                "Fushimi Inari shrine never closes — climb at twilight or night with flashlights."
                            ],
                            Some((35.0272, 135.7953)),
                        ),
                        topic(
                            "jp_item_matcha_tea_ceremony",
                            "Matcha & The Way of Tea (Chado / Sado)",
                            "Wabi-sabi aesthetics, Sen no Rikyu, and chawan bowl rotation etiquette",
                            &["tea", "matcha", "chado", "wabi-sabi", "zen"],
                            "Die japanische Teezeremonie (Chado) wurde im 16. Jahrhundert von Teemeister Sen no Rikyu in Kyoto vollendet. Sie verkörpert vier Zen-Ideale: Harmonie (Wa), Respekt (Kei), Reinheit (Sei) und Stille (Jaku):\n\
                            • Die Schale (Chawan): Die wertvollste, kunstvoll bemalte Seite der Keramikschale (die 'Schauseite') zeigt bei der Übergabe zum Gast.\n\
                            • Drehung der Schale: Man nimmt die Chawan mit der rechten Hand, setzt sie auf die linke Handfläche und dreht sie zweimal im Uhrzeigersinn um etwa 90°, damit man mit den Lippen nicht direkt auf die Schauseite trinkt (ein Zeichen der Demut).\n\
                            • Trinken: Man trinkt den schaumig geschlagenen Koicha oder Usucha in drei bis vier Schlucken. Der letzte Schluck wird hörbar geschlürft, um zu signalisieren, dass man den Tee genossen hat.\n\
                            • Rand abwischen: Nach dem Trinken wischt man die Trinkstelle mit Daumen und Zeigefinger der rechten Hand sanft ab und reinigt die Finger am mitgebrachten Kaishi-Papier.",
                            &[
                                "Eat the sweet wagashi confection BEFORE drinking the matcha to contrast the bitterness.",
                                "Turn the tea bowl clockwise twice before sipping.",
                                "Audibly slurp the final sip to indicate to the host that you have finished."
                            ],
                            Some((35.0088, 135.7725)),
                        ),
                    ],
                },
                InfoBranch {
                    id: "jp_branch_regions".to_string(),
                    title: "Regions & Chrono-Corridors".to_string(),
                    icon: "🗾".to_string(),
                    items: vec![
                        topic(
                            "jp_item_tokaido_highway",
                            "The 53 Stations of the Tokaido & Edo-Period Travel",
                            "Hiroshige's prints, Nihonbashi zero milestone, Hakone sekisho barrier",
                            &["tokaido", "edo", "hiroshige", "history"],
                            "Die Tokaido war die wichtigste der fünf Poststraßen (Gokaido) des Tokugawa-Shogunats und verband das politische Machtzentrum Edo (Tokyo) mit der kaiserlichen Residenzstadt Kyoto. Reisende passierten 53 Poststationen zu Fuß oder per Sänfte. Utagawa Hiroshiges Farbholzschnitt-Serie 'Die 53 Stationen der Tokaido' (1832) verewigte die Rastplätze, Gasthäuser und Gebirgspässe für die Ewigkeit.",
                            &[
                                "Start at Nihonbashi Zero Milestone in central Tokyo.",
                                "Visit Hakone Sekisho to see the preserved wooden checkpoint inspection halls.",
                                "Follow the modern Tokaido Shinkansen path retracing the ancient Edo route."
                            ],
                            Some((35.6840, 139.7745)),
                        ),
                        topic(
                            "jp_item_kansai_cultural_core",
                            "Kyoto Imperial Capital, Osaka Kitchen & Nara Antiquity",
                            "Over 1,200 years of cultural heritage in western Japan",
                            &["kansai", "kyoto", "osaka", "nara"],
                            "Die Kansai-Region ist das historische Herz Japans:\n\
                            • Kyoto: Kaiserstadt von 794 bis 1868 mit über 2.000 Tempeln, Schreinen und Geisha-Vierteln (Gion, Pontocho).\n\
                            • Osaka: 'Tenka no Daidokoro' (Küche des Reiches) mit unvergleichlicher Streetfood-Kultur (Takoyaki, Okonomiyaki) und Dotonbori-Neon.\n\
                            • Nara: Japans erste feste Hauptstadt (710 AD) mit dem Großen Buddha im Todaiji und den zahmen Sika-Hirschen im Nara-Park.",
                            &[
                                "JR Special Rapid trains connect Osaka and Kyoto in just 28 minutes.",
                                "In Nara, buy Shika-senbei crackers and bow to the deer before feeding.",
                                "Explore Gion in the early evening for lantern-lit wooden machiya alleys."
                            ],
                            Some((35.0088, 135.7725)),
                        ),
                        topic(
                            "jp_item_hokuriku_chugoku",
                            "Kanazawa Geisha Teahouses & Hiroshima Delta Rebirth",
                            "Kenrokuen garden, gold leaf craftsmanship, and the Seto Inland Sea",
                            &["kanazawa", "hiroshima", "hokuriku", "chugoku"],
                            "• Kanazawa: Residenz des wohlhabenden Maeda-Clans mit dem Kenrokuen-Landschaftsgarten, erhaltenen Samurai-Vierteln (Nagamachi) und Higashi Chaya Geisha-Teehäusern. Zentrum der japanischen Blattgold-Herstellung.\n\
                            • Hiroshima: Am Seto-Binnenmeer gelegen, Symbol des Friedens mit dem Genbaku Dome und dem Friedenspark, berühmt für Hiroshima-Okonomiyaki mit Nudelschicht.",
                            &[
                                "Hokuriku Shinkansen connects Tokyo directly to Kanazawa in 2h 30m.",
                                "Taste gold-leaf soft cream in Kanazawa's Higashi Chaya district.",
                                "Take the short ferry from Hiroshima to Miyajima's floating Torii gate."
                            ],
                            Some((36.5613, 136.6562)),
                        ),
                    ],
                },
            ],
        }
    }

    /// Retrieve knowledge tree for a given country code ("AT" or "JP").
    pub fn get_knowledge_tree(country_code: &str) -> KnowledgeTree {
        match country_code.to_uppercase().as_str() {
            "AT" | "AUSTRIA" => Self::get_austria_knowledge_tree(),
            _ => Self::get_japan_knowledge_tree(),
        }
    }

    /// Lookup a specific item by ID across all branches in a country's knowledge tree.
    pub fn find_tree_item(country_code: &str, item_id: &str) -> Option<InfoTopicItem> {
        let tree = Self::get_knowledge_tree(country_code);
        for branch in tree.branches {
            for item in branch.items {
                if item.id == item_id {
                    return Some(item);
                }
            }
        }
        None
    }

    /// Return all items in a country's knowledge tree flattened.
    pub fn all_tree_items(country_code: &str) -> Vec<InfoTopicItem> {
        let tree = Self::get_knowledge_tree(country_code);
        tree.branches.into_iter().flat_map(|b| b.items).collect()
    }
}

/// Preloaded transit snapshot (offline blob).
/// Curated from gtfs-mcp data/vienna + data/wien + data/default
/// (stops.txt/routes.txt: Wiener Linien U1/U2/U3/U4, D/1/2/71, 1A/2A)
/// plus JR/Tokyo Metro hubs for JapanCraft.
/// Full live departures: gtfs-mcp backend :10913.
pub fn transit_blob_for_sight(sight_id: &str) -> String {
    match sight_id {
        s if s.contains("stephansdom") => "TRANSIT Stephansdom/Stephansplatz (1010 Wien): U1+U3 Stephansplatz (elevators), buses 1A/2A Singerstrasse, trams 1/2/D Kaeerntner Ring (5 min walk). Day ticket EUR 8. Live: gtfs-mcp :10913 /v1/stops/search?q=Stephansplatz.".to_string(),
        s if s.contains("khm") || s.contains("caravaggio") || s.contains("bruegel") || s.contains("rosenkranz") || s.contains("david") => "TRANSIT KHM/Maria-Theresien-Platz (1010 Wien): U3 Volkstheater, trams D/1/2/71 Burgring, U2 Museumsquartier. Dome cafe break. Live: gtfs-mcp :10913 stops near 48.2039,16.3619.".to_string(),
        s if s.contains("belvedere") || s.contains("kuss") || s.contains("klimt") => "TRANSIT Belvedere (1030 Wien): tram D/71 + S Hauptbahnhof, bus 69A. Gardens free dawn-dusk. Live: gtfs-mcp :10913 stops near 48.1915,16.3809.".to_string(),
        s if s.contains("schoenbrunn") => "TRANSIT Schoenbrunn (1130 Wien): U4 Schoenbrunn/Hietzing, trams 10/58, bus 10A. Live: gtfs-mcp :10913 stops near 48.1858,16.3122.".to_string(),
        s if s.contains("hofburg") => "TRANSIT Hofburg/Heldenplatz: U3 Herrengasse, trams D/1/2/71 Burgring, bus 2A. Combine with KHM across Ring. Live: gtfs-mcp :10913.".to_string(),
        s if s.contains("kaffeehaus") || s.contains("berg") => "TRANSIT Cafe Berg/Alsergrund: U4 Friedensbruecke, trams D/5/33 Julius-Tandler-Platz. Live: gtfs-mcp :10913.".to_string(),
        s if s.contains("hallstatt") => "TRANSIT Hallstatt: OBB rail Attnang-Puchheim + ferry across lake, bus 543/544. Salt mine funicular. Live: OBB Scotty (no GTFS in blob).".to_string(),
        s if s.contains("wachau") => "TRANSIT Wachau: OBB Krems/Melk, Danube bike 39km, DDSG boat Apr-Oct. Live: OBB.".to_string(),
        s if s.starts_with("jp_") => "TRANSIT Japan hub: JR Pass 7/14/21d, IC Suica/Pasmo, Hyperdia. Nihonbashi: Ginza/Tozai/Hanzomon Mitsukoshimae. Fushimi Inari: JR Nara line Inari. Nara: JR/Kintetsu Nara. Osaka: Midosuji Namba. Hiroshima: JR tram. Kanazawa: Hokuriku Shinkansen. Nikko: Tobu from Asakusa.".to_string(),
        _ => "TRANSIT: Vienna day ticket EUR 8 (Wiener Linien U1/U2/U3/U4, D/1/2/71). Japan: JR Pass + IC. Live departures: gtfs-mcp :10913.".to_string(),
    }
}

/// Grounded chat context: info blob + map + transit + sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SightChatContext {
    pub sight_id: String,
    pub sight_name: String,
    pub info_body: String,
    pub practical: String,
    pub map_blob: String,
    pub transit_blob: String,
    pub sources: Vec<String>,
}

impl SightChatContext {
    pub fn system_prompt(&self) -> String {
        format!(
            "You are TravelCraft Baedeker 2030, a sovereign offline-first guide. Answer ONLY from the INFO BLOB, MAP blob, and TRANSIT blob below unless the user explicitly asks for web knowledge (then label it WEB BEYOND BLOB). Be concise, high-contrast plain text, no markdown tables. Cite practical info.\n\nSIGHT: {}\nINFO: {}\nPRACTICAL: {}\nMAP: {}\nTRANSIT: {}\nSOURCES: {}",
            self.sight_name,
            self.info_body,
            self.practical,
            self.map_blob,
            self.transit_blob,
            self.sources.join("; ")
        )
    }
}

/// Offline extractive fallback: pick blob sentences matching question keywords.
/// Used when Ollama/local-llm is unreachable; the UI labels it OFFLINE BLOB.
pub fn offline_chat_answer(ctx: &SightChatContext, question: &str) -> String {
    let q = question.to_lowercase();
    let keywords: Vec<&str> = q
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
        .collect();
    let mut pool: Vec<String> = Vec::new();
    for chunk in ctx
        .info_body
        .split(". ")
        .chain(ctx.practical.split(". "))
        .chain(ctx.transit_blob.split(". "))
        .chain(ctx.map_blob.split(". "))
    {
        let c = chunk.trim();
        if c.is_empty() {
            continue;
        }
        let cl = c.to_lowercase();
        let score = keywords.iter().filter(|k| cl.contains(**k)).count();
        if score > 0 || pool.len() < 2 {
            pool.push(format!("({}) {}", score, c));
        }
    }
    pool.sort_by(|a, b| b.cmp(a));
    let top: Vec<String> = pool
        .into_iter()
        .take(4)
        .map(|s| {
            s.split_once(')')
                .map(|(_, rest)| rest.trim().to_string())
                .unwrap_or(s)
        })
        .collect();
    if top.is_empty() {
        return format!(
            "OFFLINE BLOB ({}): {}\n\nPractical: {}\n\n{}",
            ctx.sight_name, ctx.info_body, ctx.practical, ctx.transit_blob
        );
    }
    format!(
        "OFFLINE BLOB ({} - grounded in info blob, no web):\n- {}\n\nPractical: {}\n{}\nSources: {}",
        ctx.sight_name,
        top.join("\n- "),
        ctx.practical,
        ctx.transit_blob,
        ctx.sources.join("; ")
    )
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_austria_knowledge_tree_vienna_restaurants() {
        let tree = BaedekerCatalog::get_austria_knowledge_tree();
        assert_eq!(tree.country_code, "AT");
        let resto_branch = tree
            .branches
            .iter()
            .find(|b| b.id == "at_branch_restaurants")
            .expect("Must have restaurants branch");
        assert!(resto_branch.items.len() >= 6);

        // Verify key restaurants & Beisl rules are present
        let ids: Vec<&str> = resto_branch.items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"at_item_beisl_rules"));
        assert!(ids.contains(&"at_item_falstaff_michelin"));
        assert!(ids.contains(&"at_item_traditional_beisln"));
        assert!(ids.contains(&"at_item_wuerstelstand"));
        assert!(ids.contains(&"at_item_broetchen"));
        assert!(ids.contains(&"at_item_wiener_schnitzel"));
        assert!(ids.contains(&"at_item_sachertorte_rivalry"));

        // Check specific intel content
        let orlik_beisl = resto_branch
            .items
            .iter()
            .find(|i| i.id == "at_item_traditional_beisln")
            .unwrap();
        assert!(orlik_beisl.body.contains("Gasthaus Orlik"));
        assert!(orlik_beisl.body.contains("Plachutta"));
        assert!(orlik_beisl.body.contains("Woracziczky"));

        let bitzinger = resto_branch
            .items
            .iter()
            .find(|i| i.id == "at_item_wuerstelstand")
            .unwrap();
        assert!(bitzinger.body.contains("Bitzinger"));
        assert!(bitzinger.body.contains("Alles Wurscht"));
        assert!(bitzinger.body.contains("Leberkas-Pepi"));

        let schnitzel = resto_branch
            .items
            .iter()
            .find(|i| i.id == "at_item_wiener_schnitzel")
            .unwrap();
        assert!(schnitzel.body.contains("Kalbfleisch"));
        assert!(schnitzel.body.contains("souffliert"));

        // Verify Alpine branch
        let alpine_branch = tree
            .branches
            .iter()
            .find(|b| b.id == "at_branch_alpine")
            .expect("Must have alpine branch");
        assert!(alpine_branch
            .items
            .iter()
            .any(|i| i.id == "at_item_huettenordnung"));
    }

    #[test]
    fn test_japan_knowledge_tree() {
        let tree = BaedekerCatalog::get_japan_knowledge_tree();
        assert_eq!(tree.country_code, "JP");
        assert!(tree.branches.len() >= 4);

        let transit = tree
            .branches
            .iter()
            .find(|b| b.id == "jp_branch_transit")
            .expect("Must have transit branch");
        assert!(transit.items.iter().any(|i| i.id == "jp_item_jr_pass"));

        let dining = tree
            .branches
            .iter()
            .find(|b| b.id == "jp_branch_dining")
            .expect("Must have dining branch");
        assert!(dining
            .items
            .iter()
            .any(|i| i.id == "jp_item_konbini_survival"));

        let culture = tree
            .branches
            .iter()
            .find(|b| b.id == "jp_branch_culture")
            .expect("Must have culture branch");
        assert!(culture
            .items
            .iter()
            .any(|i| i.id == "jp_item_temizuya_bowing"));
        assert!(culture.items.iter().any(|i| i.id == "jp_item_genkan_shoes"));
        assert!(culture
            .items
            .iter()
            .any(|i| i.id == "jp_item_onsen_ryokan_protocol"));
        assert!(culture
            .items
            .iter()
            .any(|i| i.id == "jp_item_kyoto_walking_trails"));
    }

    #[test]
    fn test_find_tree_item() {
        let item = BaedekerCatalog::find_tree_item("AT", "at_item_beisl_rules");
        assert!(item.is_some());
        assert_eq!(
            item.unwrap().title,
            "The Golden Vienna Beisl Rules & Culture"
        );

        let jp_item = BaedekerCatalog::find_tree_item("JP", "jp_item_jr_pass");
        assert!(jp_item.is_some());
    }
}
