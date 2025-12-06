use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
};

use cirmcut::cirmcut_sim::{
    solver::{Solver, SolverConfig, SolverMode},
    PrimitiveDiagram, SimOutputs, ThreeTerminalComponent, TwoTerminalComponent,
};
use egui::{
    Color32, DragValue, Key, Layout, Pos2, Rect, RichText, ScrollArea, Ui, Vec2, ViewportCommand,
};

use cirmcut::circuit_widget::{
    draw_grid, egui_to_cellpos, Diagram, DiagramEditor, DiagramState, DiagramWireState,
    VisualizationOptions,
};

pub struct CircuitEditor {
    view_rect: Rect,
    editor: DiagramEditor,
    debug_draw: bool,
    current_path: Option<PathBuf>,

    vis_opt: VisualizationOptions,
    error: Option<String>,
}

/*
#[derive(serde::Deserialize, serde::Serialize)]
struct CircuitFile {
    diagram: Diagram,
    cfg: SolverConfig,
    dt: f64,
}
*/

impl Default for CircuitEditor {
    fn default() -> Self {
        Self {
            vis_opt: VisualizationOptions::default(),
            error: None,
            editor: DiagramEditor::new(),
            view_rect: Rect::from_center_size(Pos2::ZERO, Vec2::splat(1000.0)),
            debug_draw: false,
            current_path: None,
        }
    }
}

impl CircuitEditor {
    /// Returns true if the sim should be rebuilt
    fn update_cfg(
        &mut self,
        ui: &mut Ui,
        diagram: &mut Diagram,
        cfg: &mut SolverConfig,
        state: Option<&DiagramState>,
    ) -> bool {
        let mut rebuild_sim = false;

        let mut single_step = false;

        ScrollArea::vertical().show(ui, |ui| {
            rebuild_sim |= ui.button("Reset").clicked();

            /*
            ui.add(
                DragValue::new(&mut self.current_file.dt)
                    .prefix("dt: ")
                    .speed(1e-7)
                    .suffix(" s"),
            );
            */

            if let Some(error) = &self.error {
                ui.label(RichText::new(error).color(Color32::RED));
            }

            ui.separator();
            ui.strong("Advanced");

            ui.add(DragValue::new(&mut cfg.max_nr_iters).prefix("Max NR iters: "));
            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut cfg.nr_step_size)
                        .speed(1e-6)
                        .prefix("Initial NR step size: "),
                );
                ui.checkbox(&mut cfg.adaptive_step_size, "Adaptive");
            });

            ui.add(
                DragValue::new(&mut cfg.nr_tolerance)
                    .speed(1e-6)
                    .prefix("NR tolerance: "),
            );
            ui.add(
                DragValue::new(&mut cfg.dx_soln_tolerance)
                    .speed(1e-6)
                    .prefix("Matrix solve tol: "),
            );

            ui.horizontal(|ui| {
                ui.selectable_value(&mut cfg.mode, SolverMode::NewtonRaphson, "Newton-Raphson");
                ui.selectable_value(&mut cfg.mode, SolverMode::Linear, "Linear");
            });

            if ui.button("Default cfg").clicked() {
                *cfg = Default::default();
            }

            ui.separator();

            if let Some(state) = &state {
                rebuild_sim |= self.editor.edit_component(ui, diagram, state);
            }

            ui.separator();
            ui.strong("Visualization");
            ui.add(
                DragValue::new(&mut self.vis_opt.voltage_scale)
                    .prefix("Voltage scale: ")
                    .speed(1e-2),
            );
            ui.add(
                DragValue::new(&mut self.vis_opt.current_scale)
                    .prefix("Current scale: ")
                    .speed(1e-2),
            );
            if ui.button("Auto scale").clicked() {
                if let Some(state) = &state {
                    let all_wires = state.two_terminal.iter().copied().flatten();
                    self.vis_opt.voltage_scale = all_wires
                        .clone()
                        .map(|wire| wire.voltage.abs())
                        .max_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap_or(VisualizationOptions::default().voltage_scale);
                    self.vis_opt.current_scale = all_wires
                        .map(|wire| wire.current.abs())
                        .max_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap_or(VisualizationOptions::default().current_scale);
                }
                //self.vis_opt.voltage_scale =
            }
        });

        ui.label("Add component: ");
        let pos = egui_to_cellpos(self.view_rect.center());
        if ui.button("Wire").clicked() {
            rebuild_sim = true;
            self.editor
                .new_twoterminal(diagram, pos, TwoTerminalComponent::Wire);
        }
        if ui.button("Resistor").clicked() {
            rebuild_sim = true;
            self.editor
                .new_twoterminal(diagram, pos, TwoTerminalComponent::Resistor(1000.0));
        }
        if ui.button("Inductor").clicked() {
            rebuild_sim = true;
            self.editor
                .new_twoterminal(diagram, pos, TwoTerminalComponent::Inductor(1.0, None));
        }
        if ui.button("Capacitor").clicked() {
            rebuild_sim = true;
            self.editor
                .new_twoterminal(diagram, pos, TwoTerminalComponent::Capacitor(10e-6));
        }
        if ui.button("Diode").clicked() {
            rebuild_sim = true;
            self.editor
                .new_twoterminal(diagram, pos, TwoTerminalComponent::Diode);
        }
        if ui.button("Battery").clicked() {
            rebuild_sim = true;
            self.editor
                .new_twoterminal(diagram, pos, TwoTerminalComponent::Battery(5.0));
        }
        if ui.button("Switch").clicked() {
            rebuild_sim = true;
            self.editor
                .new_twoterminal(diagram, pos, TwoTerminalComponent::Switch(true));
        }
        if ui.button("Current source").clicked() {
            rebuild_sim = true;
            self.editor
                .new_twoterminal(diagram, pos, TwoTerminalComponent::CurrentSource(0.1));
        }
        if ui.button("PNP").clicked() {
            rebuild_sim = true;
            self.editor
                .new_threeterminal(diagram, pos, ThreeTerminalComponent::PTransistor(100.0));
        }
        if ui.button("NPN").clicked() {
            rebuild_sim = true;
            self.editor
                .new_threeterminal(diagram, pos, ThreeTerminalComponent::NTransistor(100.0));
        }
        if ui.button("Port").clicked() {
            rebuild_sim = true;
            self.editor.new_port(diagram, pos, "New port".into());
        }
        /*
        if ui.button("Delete").clicked() {
            self.editor.delete();
        }
        ui.checkbox(&mut self.debug_draw, "Debug draw");
        */

        rebuild_sim
    }

    /// Returns true if the sim should be rebuilt
    fn show_circuit_editor(
        &mut self,
        ui: &mut Ui,
        diagram: &mut Diagram,
        state: Option<&DiagramState>,
    ) -> bool {
        let mut rebuild_sim = false;

        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let rect = self.view_rect;
            let resp = egui::Scene::new().show(ui, &mut self.view_rect, |ui| {
                draw_grid(ui, rect, 1.0, Color32::DARK_GRAY);
                if let Some(state) = state {
                    rebuild_sim |=
                        self.editor
                            .edit(ui, diagram, &state, self.debug_draw, &self.vis_opt);
                }
            });

            if ui.input(|r| r.key_pressed(Key::Delete)) {
                rebuild_sim = true;
                self.editor.delete(diagram);
            }

            if resp.response.clicked() || ui.input(|r| r.key_pressed(Key::Escape)) {
                self.editor.reset_selection();
            }
        });

        rebuild_sim
    }
}

/*
impl Default for CircuitFile {
    fn default() -> Self {
        Self {
            diagram: Diagram::default(),
            dt: 5e-3,
            cfg: Default::default(),
        }
    }
}
*/
