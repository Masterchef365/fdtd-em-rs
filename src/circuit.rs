use std::{
    collections::HashSet, ffi::OsStr, fs::File, path::{Path, PathBuf}
};

use cirmcut::cirmcut_sim::{
    solver::{Solver, SolverConfig, SolverMode}, PrimitiveDiagram, SimOutputs, ThreeTerminalComponent, TwoTerminalComponent
};
use egui::{Color32, DragValue, Key, Layout, Pos2, Rect, RichText, ScrollArea, Vec2, ViewportCommand};

use cirmcut::circuit_widget::{
    draw_grid, egui_to_cellpos, Diagram, DiagramEditor, DiagramState, DiagramWireState,
    VisualizationOptions,
};
use ndarray::Array4;

use crate::{sim::{FdtdSim, FdtdSimConfig}, wire_editor_3d::{WireId, Wiring3D}};

pub struct CircuitApp {
    pub view_rect: Rect,
    pub editor: DiagramEditor,
    pub debug_draw: bool,
    pub current_path: Option<PathBuf>,

    pub current_file: CircuitFile,
    pub vis_opt: VisualizationOptions,

    pub sim: Option<Solver>,

    pub error: Option<String>,

    pub paused: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct CircuitFile {
    diagram: Diagram,
    cfg: SolverConfig,
    dt: f64,
}

impl Default for CircuitApp {
    fn default() -> Self {
        Self {
            vis_opt: VisualizationOptions::default(),
            error: None,
            sim: None,
            editor: DiagramEditor::new(),
            current_file: Default::default(),
            paused: false,
            view_rect: Rect::from_center_size(Pos2::ZERO, Vec2::splat(1000.0)),
            debug_draw: false,
            current_path: None,
        }
    }
}

struct DiagramConversion {
    diagram_state: DiagramState,
    primitive_diagram: PrimitiveDiagram,
    sim_outputs: SimOutputs,
    elec: Array4<f32>,
}

impl DiagramConversion {
    fn convert(diagram: &Diagram, sim: &Solver, wiring: &Wiring3D) -> Self {
        let mut primitive_diagram = diagram.to_primitive_diagram();
        insert_wiring_3d(&mut primitive_diagram, wiring);
        let sim_outputs = sim.state(&primitive_diagram);
        let diagram_state = DiagramState::new(&sim_outputs, &primitive_diagram);

        Self { diagram_state, primitive_diagram, sim_outputs, elec: todo!() }
    }
}

impl CircuitApp {
    pub fn state(&self, wiring: &Wiring3D) -> Option<DiagramConversion> {
        self.sim.as_ref().map(|sim| DiagramConversion::convert(&self.current_file.diagram, sim, wiring))
    }
}

impl CircuitApp {
    pub fn show_config(&mut self, ui: &mut egui::Ui, state: &Option<DiagramState>, rebuild_sim: &mut bool, single_step: &mut bool) {
        let mut rebuild_sim = self.sim.is_none();

        ScrollArea::vertical().show(ui, |ui| {
            ui.strong("Circuit Simulation");
            let text = if self.paused { "Run" } else { "Pause" };
            ui.horizontal(|ui| {
                if ui.button(text).clicked() {
                    self.paused ^= true;
                }
                if self.paused {
                    *single_step |= ui.button("Single-step").clicked();
                }
            });

            rebuild_sim |= ui.button("Reset").clicked();

            ui.add(
                DragValue::new(&mut self.current_file.dt)
                .prefix("dt: ")
                .speed(1e-7)
                .suffix(" s"),
            );

            if let Some(error) = &self.error {
                ui.label(RichText::new(error).color(Color32::RED));
            }

            ui.separator();
            ui.strong("Advanced");

            ui.add(
                DragValue::new(&mut self.current_file.cfg.max_nr_iters)
                .prefix("Max NR iters: "),
            );
            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut self.current_file.cfg.nr_step_size)
                    .speed(1e-6)
                    .prefix("Initial NR step size: "),
                );
                ui.checkbox(
                    &mut self.current_file.cfg.adaptive_step_size,
                    "Adaptive",
                );
            });

            ui.add(
                DragValue::new(&mut self.current_file.cfg.nr_tolerance)
                .speed(1e-6)
                .prefix("NR tolerance: "),
            );
            ui.add(
                DragValue::new(&mut self.current_file.cfg.dx_soln_tolerance)
                .speed(1e-6)
                .prefix("Matrix solve tol: "),
            );

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.current_file.cfg.mode,
                    SolverMode::NewtonRaphson,
                    "Newton-Raphson",
                );
                ui.selectable_value(
                    &mut self.current_file.cfg.mode,
                    SolverMode::Linear,
                    "Linear",
                );
            });

            if ui.button("Default cfg").clicked() {
                self.current_file.cfg = Default::default();
            }

            ui.separator();

            if let Some(state) = &state {
                rebuild_sim |=
                    self.editor
                    .edit_component(ui, &mut self.current_file.diagram, state);
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
    }

    pub fn show_add_components(&mut self, ui: &mut egui::Ui, rebuild_sim: &mut bool, single_step: &mut bool) {
        ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Add component: ");
                let pos = egui_to_cellpos(self.view_rect.center());
                if ui.button("Wire").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_twoterminal(
                        &mut self.current_file.diagram,
                        pos,
                        TwoTerminalComponent::Wire,
                    );
                }
                if ui.button("Resistor").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_twoterminal(
                        &mut self.current_file.diagram,
                        pos,
                        TwoTerminalComponent::Resistor(1000.0),
                    );
                }
                if ui.button("Inductor").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_twoterminal(
                        &mut self.current_file.diagram,
                        pos,
                        TwoTerminalComponent::Inductor(1.0, None),
                    );
                }
                if ui.button("Capacitor").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_twoterminal(
                        &mut self.current_file.diagram,
                        pos,
                        TwoTerminalComponent::Capacitor(10e-6),
                    );
                }
                if ui.button("Diode").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_twoterminal(
                        &mut self.current_file.diagram,
                        pos,
                        TwoTerminalComponent::Diode,
                    );
                }
                if ui.button("Battery").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_twoterminal(
                        &mut self.current_file.diagram,
                        pos,
                        TwoTerminalComponent::Battery(5.0),
                    );
                }
                if ui.button("Switch").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_twoterminal(
                        &mut self.current_file.diagram,
                        pos,
                        TwoTerminalComponent::Switch(true),
                    );
                }
                if ui.button("Current source").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_twoterminal(
                        &mut self.current_file.diagram,
                        pos,
                        TwoTerminalComponent::CurrentSource(0.1),
                    );
                }
                if ui.button("PNP").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_threeterminal(
                        &mut self.current_file.diagram,
                        pos,
                        ThreeTerminalComponent::PTransistor(100.0),
                    );
                }
                if ui.button("NPN").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_threeterminal(
                        &mut self.current_file.diagram,
                        pos,
                        ThreeTerminalComponent::NTransistor(100.0),
                    );
                }
                if ui.button("Port").clicked() {
                    *rebuild_sim = true;
                    self.editor.new_port(
                        &mut self.current_file.diagram,
                        pos,
                        "New port".into(),
                    );
                }
                /*
                   if ui.button("Delete").clicked() {
                   self.editor.delete();
                   }
                   ui.checkbox(&mut self.debug_draw, "Debug draw");
                   */
            });
        });
    }

    pub fn update(&mut self, ui: &mut egui::Ui, rebuild_sim: &mut bool, single_step: &mut bool, state: &Option<DiagramConversion>) {
        if ui.button("Reset camera").clicked() {
            self.view_rect = Rect::ZERO;
        }

        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let rect = self.view_rect;
            let resp = egui::Scene::new().show(ui, &mut self.view_rect, |ui| {
                draw_grid(ui, rect, 1.0, Color32::DARK_GRAY);
                if let Some(state) = state {
                    *rebuild_sim |= self.editor.edit(
                        ui,
                        &mut self.current_file.diagram,
                        &state.diagram_state,
                        self.debug_draw,
                        &self.vis_opt,
                    );
                }
            });

            if ui.input(|r| r.key_pressed(Key::Delete)) {
                *rebuild_sim = true;
                self.editor.delete(&mut self.current_file.diagram);
            }

            if resp.response.clicked() || ui.input(|r| r.key_pressed(Key::Escape)) {
                self.editor.reset_selection();
            }
        });
    }

    pub fn step(&mut self, rebuild_sim: bool, single_step: bool, fdtd_sim: &mut FdtdSim, fdtd_cfg: &FdtdSimConfig, state: &DiagramConversion) {
        // Reset
        if rebuild_sim {
            self.sim = Some(Solver::new(
                &state.primitive_diagram
            ));
        }

        if !self.paused || rebuild_sim || single_step {
            //ui.ctx().request_repaint();

            if let Some(circuit_sim) = &mut self.sim {
                //let start = std::time::Instant::now();


                //let external_elec = 

                let magnetization = Array4::<f32>::zeros(fdtd_sim.h_field.dim());
                fdtd_sim.step(fdtd_cfg, &magnetization, &state.elec);

                if let Err(e) = circuit_sim.step(
                    self.current_file.dt,
                    &state.primitive_diagram,
                    &self.current_file.cfg,
                ) {
                    eprintln!("{}", e);
                    self.error = Some(e);
                    self.paused = true;
                } else {
                    self.error = None;
                }
                //println!("Time: {:.03} ms = {:.03} fps", start.elapsed().as_secs_f32() * 1000.0, 1.0 / (start.elapsed().as_secs_f32()));
            }
        }
    }
}

impl Default for CircuitFile {
    fn default() -> Self {
        Self {
            diagram: Diagram::default(),
            dt: 5e-3,
            cfg: Default::default(),
        }
    }
}

fn insert_wiring_3d(prim: &mut PrimitiveDiagram, wiring: &Wiring3D) {
    let mut nodes = HashSet::new();
    for (a, b) in wiring.wires.keys() {
        nodes.insert(*a);
        nodes.insert(*b);
    }
    
    let mut used_ports = vec![];
    for (pt_3d, port3d) in &wiring.ports {
        for (pt_2d, port2d_name) in &prim.ports {
            if &port3d.0 == &port2d_name {
                used_ports.push((pt_2d, pt_3d));
            }
        }
    }

}
