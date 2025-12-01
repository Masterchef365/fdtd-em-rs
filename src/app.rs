use egui::{DragValue, SidePanel, TopBottomPanel};
use ndarray::Array4;

use crate::{
    circuit::CircuitApp, field_vis::GridVisualizationConfig, sim::{FdtdSim, FdtdSimConfig}, streamers::{Streamers, StreamersMode}, wire_editor_3d::{WireEditor3D, Wiring3D}
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    sim: FdtdSim,
    sim_cfg: FdtdSimConfig,
    time: f32,

    new_width: usize,

    grid_vis: GridVisualizationConfig,
    streamers: Streamers,
    streamer_step: f32,
    enable_streamers: StreamersMode,

    wire_editor_3d: WireEditor3D,
    wires: Wiring3D,

    magnetization: Array4<f32>,

    circuit: CircuitApp,
}

fn random_sim(width: usize) -> (FdtdSim, Array4<f32>) {
    let sim = FdtdSim::new(width);

    let unif = rand::distributions::Uniform::new(-1.0, 1.0);
    let rng = rand::thread_rng();

    let mut magnetization = Array4::zeros(sim.h_field.dim());

    /*
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
    */

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
            circuit: Default::default(),
            wire_editor_3d: WireEditor3D::default(),

            magnetization,
            streamers: Streamers::new(&sim, 5000),
            enable_streamers: StreamersMode::Off,
            time: 0.,
            streamer_step: 0.01,

            new_width: sim.width(),

            grid_vis: GridVisualizationConfig::default(),

            sim_cfg: FdtdSimConfig {
                dx: 1.,
                dt: 0.005,
                mu: 1.,
                eps: 1.,
            },

            sim,
            wires: Wiring3D::default(),
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
        if !self.circuit.paused {
            ctx.request_repaint();
            self.time += self.sim_cfg.dt;
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        SidePanel::left("left panel").show(ctx, |ui| {
            self.wire_editor_3d
                .show_ui(ui, self.sim.width(), &mut self.wires);

            ui.strong("Background grid");
            ui.checkbox(&mut self.grid_vis.show_grid, "Show grid");
            ui.checkbox(&mut self.grid_vis.show_minimal_grid, "Show minimal grid");

            ui.collapsing("Field visualization", |ui| {
                self.grid_vis.show_ui(ui);
            });

            ui.collapsing("Streamers (visualization)", |ui| {
                //ui.checkbox(&mut self.enable_streamers, "Streamers");
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
            ui.strong("FDTD settings");
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
        });

        let circuit_state = self.circuit.state();

        let (mut rebuild_sim, mut single_step) = (false, false);
        SidePanel::right("right panel").resizable(true).show(ctx, |ui| {
            self.circuit.show_config(ui, &circuit_state, &mut rebuild_sim, &mut single_step);
        });

        TopBottomPanel::bottom("bottom panel").resizable(true).show(ctx, |ui| {
            self.circuit.show_add_components(ui, &mut rebuild_sim, &mut single_step);
            self.circuit.update(ui, &mut rebuild_sim, &mut single_step, &circuit_state);
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

                        rebuild_sim |= self.wire_editor_3d
                            .draw(self.sim.width(), thr, &mut self.wires);
                    });
            });
        });

        self.circuit.step(rebuild_sim, single_step, &mut self.sim, &self.sim_cfg, &self.wires);
    }
}
