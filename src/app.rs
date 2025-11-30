use egui::{DragValue, SidePanel};
use ndarray::Array4;

use crate::{
    field_vis::GridVisualizationConfig,
    sim::{Sim, SimConfig},
    streamers::{Streamers, StreamersMode},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    sim: Sim,
    sim_cfg: SimConfig,
    time: f32,

    new_width: usize,
    pause: bool,

    grid_vis: GridVisualizationConfig,
    streamers: Streamers,
    streamer_step: f32,
    enable_streamers: StreamersMode,

    magnetization: Array4<f32>,
}

fn random_sim(width: usize) -> (Sim, Array4<f32>) {
    let sim = Sim::new(width);

    let unif = rand::distributions::Uniform::new(-1.0, 1.0);
    let rng = rand::thread_rng();

    let mut magnetization = Array4::zeros(sim.h_field.dim());

    let c = width / 2;

    for i in c - 1..=c + 1 {
        for j in c - 1..=c + 1 {
            for k in c - 1..=c + 1 {
                magnetization[(i, j, k, 0)] = 0.0;
                magnetization[(i, j, k, 1)] = 1.0;
                magnetization[(i, j, k, 2)] = 0.0;
            }
        }
    }

    /*
    sim.e_field
        .iter_mut()
        .for_each(|x| *x = unif.sample(&mut rng));
    sim.h_field
        .iter_mut()
        .for_each(|x| *x = unif.sample(&mut rng));
    */

    //sim.e_field[(width/2,width/2,width/2,1)] = 10.;
    /*
    sim.h_field[(width/2,width/2,width/2,1)] = 10.;

    sim.e_field[(0,0,0,0)] = 10.;
    sim.h_field[(0,0,0,0)] = 10.;
    */

    (sim, magnetization)
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (sim, magnetization) = random_sim(10);
        Self {
            magnetization,
            streamers: Streamers::new(&sim, 5000),
            enable_streamers: StreamersMode::HField,
            time: 0.,
            streamer_step: 0.01,

            pause: false,
            new_width: sim.width(),

            grid_vis: GridVisualizationConfig::default(),

            sim_cfg: SimConfig {
                dx: 1.,
                dt: 0.005,
                mu: 1.,
                eps: 1.,
            },

            sim,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        /*
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        */

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /*
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    */

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.pause {
            ctx.request_repaint();
            self.sim.step(&self.sim_cfg, &self.magnetization);
            let width = self.sim.width();
            self.time += self.sim_cfg.dt;

            let k = (self.time / 3.).cos();

            //self.sim.e_field[(width / 2, width / 2, width / 2, 0)] = 0.1 * k;
            //self.sim.e_field[(width / 2, width / 2, width / 2, 1)] = 10. * k;
            //self.sim.e_field[(width / 2, width / 2, width / 2, 2)] = -0.2 * k;
            //self.sim.e_field[(width / 2, width / 2, width / 2, 0)] = 0.;
            //self.sim.e_field[(width / 2, width / 2, width / 2, 1)] = 0.;
            //self.sim.e_field[(width / 2, width / 2, width / 2, 2)] = 0.;
            //self.sim.e_field[(width/2,width/2,width/2,1)] = 10.;
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        SidePanel::left("left panel").show(ctx, |ui| {
            ui.strong("Background grid");
            ui.checkbox(&mut self.grid_vis.show_grid, "Show grid");
            ui.checkbox(&mut self.grid_vis.show_minimal_grid, "Show minimal grid");

            ui.strong("Field visualization");

            ui.collapsing("Streamers", |ui| {
                self.grid_vis.show_ui(ui);
            });

            ui.collapsing("Streamers", |ui| {
                //ui.checkbox(&mut self.enable_streamers, "Streamers");
                ui.label("Streamers");
                ui.selectable_value(&mut self.enable_streamers, StreamersMode::Off, "Off");
                ui.selectable_value(&mut self.enable_streamers, StreamersMode::HField, "H vects");
                ui.selectable_value(&mut self.enable_streamers, StreamersMode::EField, "E vects");
                ui.add(
                    DragValue::new(&mut self.streamer_step)
                        .prefix("dt: ")
                        .speed(1e-3),
                );
            });

            ui.separator();
            ui.strong("State control");
            ui.checkbox(&mut self.pause, "Paused");

            ui.separator();
            ui.strong("Sim settings");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut self.new_width).prefix("Width: "));
                if ui.button("Reset").clicked() {
                    (self.sim, self.magnetization) = random_sim(self.new_width);
                    self.streamers = Streamers::new(&self.sim, self.streamers.points.len());
                    self.time = 0.0;
                }
            });
            ui.add(
                DragValue::new(&mut self.sim_cfg.dt)
                    .prefix("Δt: ")
                    .speed(1e-3),
            );
            ui.add(
                DragValue::new(&mut self.sim_cfg.dx)
                    .prefix("Δx: ")
                    .speed(1e-3),
            );
            ui.add(
                DragValue::new(&mut self.sim_cfg.eps)
                    .prefix("ε: ")
                    .speed(1e-3),
            );
            ui.add(
                DragValue::new(&mut self.sim_cfg.mu)
                    .prefix("μ: ")
                    .speed(1e-3),
            );

            if ui.button("Step").clicked() {
                self.sim.step(&self.sim_cfg, &self.magnetization);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                threegui::ThreeWidget::new("E&M torture")
                    .with_desired_size(ui.available_size())
                    .show(ui, |thr| {
                        let paint = thr.painter();

                        self.streamers.step(
                            &self.sim,
                            paint,
                            self.streamer_step,
                            0.001,
                            0.2,
                            self.enable_streamers,
                        );

                        self.grid_vis.draw(&self.sim, paint);
                    });
            });
        });
    }
}
