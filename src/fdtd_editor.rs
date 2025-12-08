use cirmcut::cirmcut_sim::SimOutputs;
use egui::{DragValue, Ui};

use crate::{
    field_vis::GridVisualizationConfig,
    node_map::NodeMap,
    sim::{FdtdSim, FdtdSimConfig},
    streamers::{Streamers, StreamersMode},
    wire_editor_3d::{WireEditor3D, Wiring3D},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FdtdEditor {
    grid_vis: GridVisualizationConfig,
    streamers: Streamers,
    streamer_step: f32,
    enable_streamers: StreamersMode,
    wire_editor_3d: WireEditor3D,
}

impl FdtdEditor {
    pub fn new(width: usize) -> Self {
        Self {
            wire_editor_3d: WireEditor3D::default(),

            streamers: Streamers::new(width, 5000),
            enable_streamers: StreamersMode::HField,
            streamer_step: 0.01,

            grid_vis: GridVisualizationConfig::default(),
        }
    }
}

impl FdtdEditor {
    /// Returns true if the change would require an external update
    pub fn show_edit_wire(
        &mut self,
        ui: &mut Ui,
        sim: &FdtdSim,
        wires: &mut Wiring3D,
    ) -> bool {
        self.wire_editor_3d.show_ui(ui, sim.width(), wires)
    }


    /// Returns true if the change would require an external update
    pub fn show_cfg(
        &mut self,
        ui: &mut Ui,
        cfg: &mut FdtdSimConfig,
    ) -> bool {
        let rebuild = false;

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
        ui.add(DragValue::new(&mut cfg.dt).prefix("Δt: ").speed(1e-3));
        ui.add(DragValue::new(&mut cfg.dx).prefix("Δx: ").speed(1e-3));
        ui.add(DragValue::new(&mut cfg.eps).prefix("ε: ").speed(1e-3));
        ui.add(DragValue::new(&mut cfg.mu).prefix("μ: ").speed(1e-3));

        rebuild
    }

    /// Returns true if the simulation should be rebuilt
    pub fn show_editor(
        &mut self,
        ui: &mut Ui,
        sim: &FdtdSim,
        wires: &mut Wiring3D,
        nodemap: &NodeMap,
        soln: &SimOutputs,
    ) -> bool {
        let mut rebuild_sim = false;

        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            threegui::ThreeWidget::new("E&M torture")
                .with_desired_size(ui.available_size())
                .show(ui, |thr| {
                    let paint = thr.painter();

                    // TODO: Make this configurable
                    self.streamers.step(
                        sim,
                        paint,
                        self.streamer_step,
                        0.001,
                        0.2,
                        self.enable_streamers,
                    );

                    self.grid_vis.draw(&sim, paint);

                    self.wire_editor_3d
                        .draw_current(thr, wires, nodemap, soln, sim.width());
                    rebuild_sim |= self.wire_editor_3d.edit(sim.width(), thr, wires);
                });
        });

        rebuild_sim
    }
}
