use clap::Parser;
use serde_json::json;
use travelcraft_core::TravelCraftEngine;
use travelcraft_baedeker::CountryEdition;

#[derive(Parser, Debug)]
#[command(name = "travelcraft", about = "TravelCraft Sovereign Baedeker 2030 Studio")]
struct Args {
    #[arg(long)]
    smoke: bool,
}

pub struct TravelCraftApp {
    engine: TravelCraftEngine,
    selected_country: String,
    countries: Vec<CountryEdition>,
    status_msg: String,
}

impl TravelCraftApp {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let engine = TravelCraftEngine::new_in_memory()?;
        let countries = engine.list_countries();
        Ok(Self {
            engine,
            selected_country: "JP".to_string(),
            countries,
            status_msg: "Baedeker 2030 Chrono-Cartography Ready".to_string(),
        })
    }
}

impl eframe::App for TravelCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🧭 TravelCraft — Baedeker 2030 Chrono-Cartography Studio");
                ui.separator();
                ui.label(format!("Control Port: 14129 | Status: {}", self.status_msg));
            });
        });

        egui::Panel::left("country_selector").show(ui, |ui| {
            ui.heading("Country Editions");
            for country in &self.countries {
                let is_selected = self.selected_country == country.country_code;
                if ui.selectable_label(is_selected, format!("{} {}", country.country_code, country.title)).clicked() {
                    self.selected_country = country.country_code.clone();
                    self.status_msg = format!("Loaded {} Edition", country.country_code);
                }
            }
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if let Ok(country) = self.engine.get_country(&self.selected_country) {
                ui.heading(&country.title);
                ui.label(&country.subtitle);
                ui.add_space(8.0);

                if let Some(route) = country.curated_routes.first() {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(&route.title).strong());
                        ui.label(format!("Estimated Duration: {} days", route.estimated_days));
                        ui.separator();

                        for wp in &route.waypoints {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("[{} AD]", wp.year_ad)).monospace());
                                ui.label(egui::RichText::new(&wp.name).strong());
                                ui.label(format!("({})", wp.region));
                            });
                            ui.label(&wp.historic_narrative);
                            ui.label(format!("🎧 Acoustic: {}", wp.sensory_profile.acoustic_profile));
                            ui.label(format!("🍜 Specialties: {}", wp.culinary_specialty.join(", ")));
                            ui.add_space(4.0);
                        }
                    });
                }
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.smoke {
        println!("[travelcraft] Running headless smoke verification...");
        let engine = TravelCraftEngine::new_in_memory()?;
        let plan = engine.plan_optimized_route("JP")?;
        let smoke_payload = json!({
            "app": "travelcraft",
            "version": "0.1.0",
            "status": "ready",
            "control_port": 14129,
            "success": true,
            "jp_route_distance_km": plan.total_distance_km,
            "jp_waypoints_count": plan.waypoints.len(),
            "capabilities": ["ChronoCartography", "SensoryAtmosphere", "EdoToModern", "HabsburgToAlpine"]
        });
        println!("[travelcraft] Smoke OK: {}", smoke_payload);
        return Ok(());
    }

    let app = TravelCraftApp::new()?;
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("TravelCraft — Baedeker 2030"),
        ..Default::default()
    };

    eframe::run_native(
        "TravelCraft",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )?;
    Ok(())
}
