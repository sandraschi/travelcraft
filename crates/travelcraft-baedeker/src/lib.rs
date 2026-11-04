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
