use anyhow::{anyhow, Result};
use eframe::egui;
use tracing::{event, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod process;
#[path = "tracer/lib.rs"]
mod tracer;

fn main() -> Result<()> {
    let collector = tracer::EventCollector::level(Level::DEBUG);
    tracing_subscriber::registry()
        .with(collector.clone())
        .init();
    eframe::run_native(
        "Converter",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 600.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_zoom_factor(1.1);
            Box::<App>::new(App::with_collector(collector))
        }),
    )
    .map_err(|e| anyhow!(e.to_string()))
}

#[derive(Debug)]
struct App {
    tracer_collector: tracer::EventCollector,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Converter");
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui.button("Convert to CSV").clicked()
                    || ctx.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    match process::deserialize_json() {
                        Ok((skill, mut path)) => {
                            if let Err(e) = process::serialize_csv_to_file(skill, &mut path) {
                                event!(Level::ERROR, "Failed to serialize CSV: {}", e);
                            }
                        }
                        Err(e) => event!(Level::ERROR, "Failed to deserialize JSON: {}", e),
                    }
                }
            });
            ui.separator();
            ui.add(tracer::ui_tracer::LogUi::new(self.tracer_collector.clone()));
        });
    }
}

impl App {
    fn with_collector(collector: tracer::EventCollector) -> Self {
        Self {
            tracer_collector: collector,
        }
    }
}
