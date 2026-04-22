use eframe::egui;
use serde::Deserialize;
use std::env;
use std::error::Error;

struct NytApp {
    pops: MostPopular
}

#[derive(Default, Deserialize)]
struct MostPopular {
    results: Vec<Article>
}

#[derive(Deserialize)]
struct Article {
    title: String,
    section: String,
    url: String
}

impl NytApp {
    fn new(_cc: &eframe::CreationContext<'_>, pops: MostPopular) -> Self {
        Self {
            pops
        }
    }
}

macro_rules! MOST_POPULAR_URL {
    () => {
        "https://api.nytimes.com/svc/mostpopular/v2/viewed/1.json?api-key={}"
    };
}

fn new_populars() -> Result<MostPopular, Box<dyn Error>>{
    let nyt_api: String = env::var("NYT_API").expect("API_KEY must be set in .env!");

    let client = reqwest::blocking::Client::new();
    
    let url = format!(MOST_POPULAR_URL!(), nyt_api);

    let pop: MostPopular = client
        .get(&url)
        .send()
        .expect("Failed to fetch story")
        .json()
        .expect("Failed to parse story");

    // do something with status checking here

    Ok(pop)
}

impl eframe::App for NytApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                egui::RichText::new("NYT Popular Articles").size(64.0)
                );
                ui.add_space(50.0);
                if ui.button("Refresh").clicked() {
                    match new_populars() {
                        Ok(pops) => {
                            self.pops = pops;
                        }
                        Err(err) => {
                            eprintln!("Failed to fetch popular articles: {err}");
                        }
                    }
                }
            });


            ui.add_space(20.0);

            let total = self.pops.results.len();
            ui.label(format!("Found {total} articles."));

            ui.add_space(30.0);

            let mut sections = Vec::new();
            for art in self.pops.results.iter() {
                if !sections.contains(&art.section){
                    sections.push(art.section.clone())
                }
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for section in sections.iter() {
                        ui.add_space(20.0);
                        let id = ui.make_persistent_id(section);
                        let state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

                        state.show_header(ui, |ui| {
                            ui.label(egui::RichText::new(section).size(32.0));
                        })
                        .body(|ui| {
                            ui.add_space(5.0);
                            for article in self.pops.results.iter() {
                                if article.section == section.as_str() {
                                    ui.hyperlink_to(
                                        egui::RichText::new(&article.title)
                                            .size(16.0),
                                        &article.url,
                                    );
                                    ui.add_space(5.0);
                                }
                            }
                        });
                        ui.add_space(10.0);
                    }
                });

        });
    }
}


fn main() -> eframe::Result {
    dotenvy::dotenv().ok();

    let pops = new_populars().unwrap_or_else(|err| {
        eprintln!("Failled to load popular articles at startup: {err}");
        MostPopular::default()
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "NYT GUI",
        native_options,
        Box::new(|cc| Ok(Box::new(NytApp::new(cc, pops))),
    ))
}