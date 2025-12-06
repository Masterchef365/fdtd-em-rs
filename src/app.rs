use std::collections::HashMap;

use cirmcut::{
    circuit_widget::{Diagram, DiagramState, RichPrimitiveDiagram},
    cirmcut_sim::{
        map::PrimitiveDiagramMapping, solver::{Solver, SolverConfig}, PrimitiveDiagram, SimOutputs
    },
};
use egui::{CentralPanel, Color32, RichText, SidePanel, Ui};
use ndarray::Array4;

use crate::{
    circuit_editor::CircuitEditor, common::IntPos3, fdtd_editor::FdtdEditor, sim::{FdtdSim, FdtdSimConfig}, wire_editor_3d::{WireEditor3D, Wiring3D}
};

/// Every parameter needed for a simulation to proceed, including
/// all wires, components, configuration options, etc.
/// The output of the simulation is a pure function of this struct.
pub struct SimulationParameters {
    fdtd_width: usize,
    fdtd_config: FdtdSimConfig,
    fdtd_wiring: Wiring3D,
    circuit_diagram: Diagram,
    circuit_solver_cfg: SolverConfig,
}

/// Controls for the simulation step (play, pause, single-step).
pub struct SimulationControls {
    dt: f64,
    paused: bool,
    single_step: bool,
}

/// The current, transient state of the simulation and
/// any quantities derived during its creation.
pub struct SimulationState {
    fdtd: FdtdSim,
    circuit_solver: Solver,

    primitive_diagram: PrimitiveDiagram,
    diagram_state: DiagramState,
    nodemap: NodeMap,
    outputs: SimOutputs,
}

/// Current state of the simulation editor.
pub struct SimulationEditor {
    circuit: CircuitEditor,
    fdtd: FdtdEditor,
}

/// Application
pub struct FdtdApp {
    params: SimulationParameters,
    state: SimulationState,
    controls: SimulationControls,
    editor: SimulationEditor,
    /// Any error information from the simulation step is stored here.
    error_shown: Option<String>,
}

impl Default for FdtdApp {
    fn default() -> Self {
        let params = SimulationParameters::default();
        let state = SimulationState::new(&params);
        Self {
            controls: Default::default(),
            editor: SimulationEditor::new(&params),
            error_shown: None,
            state,
            params,
        }
    }
}

impl FdtdApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        /*
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        */

        Default::default()
    }
}

impl eframe::App for FdtdApp {
    /*
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    */

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut needs_rebuild = false;

        SidePanel::left("cfg").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Common");
                needs_rebuild |= self.controls.show_ui(ui);

                if let Some(error) = &self.error_shown {
                    ui.label(RichText::new(error).color(Color32::RED));
                }

                ui.separator();
                needs_rebuild |= self.editor.show_cfg(ui, &mut self.params, &self.state);
                ui.separator();
            });
        });

        SidePanel::right("circuit").show(ctx, |ui| {
            needs_rebuild |= self
                .editor
                .show_circuit_editor(ui, &mut self.params, &self.state);
        });

        CentralPanel::default().show(ctx, |ui| {
            needs_rebuild |= self
                .editor
                .show_fdtd_editor(ui, &mut self.params, &self.state);
        });

        let ret = self.step(needs_rebuild);

        if let Err(e) = ret {
            self.error_shown = Some(e);
        } else {
            self.error_shown = None;
        }
    }
}

impl FdtdApp {
    fn step(&mut self, needs_rebuild: bool) -> Result<(), String> {
        if needs_rebuild {
            self.state = SimulationState::new(&self.params);
        }

        if self.controls.do_step() || needs_rebuild {
            // Create E field from wires 
            let width = self.state.fdtd.width();
            let elec = generate_efield(&self.state.nodemap, &self.params.fdtd_wiring, &self.state.outputs, self.state.fdtd.width());
            let magnetization = Array4::<f64>::zeros((width, width, width, 3));

            // Step FDTD
            //self.state.fdtd.step(&self.params.fdtd_config, &magnetization, &elec);

            // Readback fdtd e-field into the soln vector

            // Step circuit
            self.state.circuit_solver.step(
                self.controls.dt,
                &self.state.primitive_diagram,
                &self.params.circuit_solver_cfg,
            )?;

            let outputs = self
                .state
                .circuit_solver
                .state(&self.state.primitive_diagram);
            self.state.diagram_state = DiagramState::new(&outputs, &self.state.primitive_diagram);
        }

        Ok(())
    }
}

impl SimulationState {
    fn new(params: &SimulationParameters) -> Self {
        let mut rich = params.circuit_diagram.to_primitive_diagram();

        let nodemap = NodeMap::new(&mut rich, &params.fdtd_wiring);

        let circuit_solver = Solver::new(&rich.primitive);
        let outputs = circuit_solver.state(&rich.primitive);
        let diagram_state = DiagramState::new(&outputs, &rich.primitive);

        Self {
            fdtd: FdtdSim::new(params.fdtd_width),
            circuit_solver,
            primitive_diagram: rich.primitive,
            diagram_state,
            nodemap,
            outputs,
        }
    }
}

impl SimulationEditor {
    pub fn show_cfg(
        &mut self,
        ui: &mut Ui,
        params: &mut SimulationParameters,
        state: &SimulationState,
    ) -> bool {
        let mut needs_rebuild = false;

        ui.heading("Circuit");
        needs_rebuild |= self.circuit.show_cfg(
            ui,
            &mut params.circuit_diagram,
            &mut params.circuit_solver_cfg,
            &state.diagram_state,
        );
        ui.separator();
        ui.heading("FDTD");
        needs_rebuild |= self.fdtd.show_cfg(
            ui,
            &state.fdtd,
            &mut params.fdtd_config,
            &mut params.fdtd_wiring,
        );

        needs_rebuild
    }

    pub fn show_circuit_editor(
        &mut self,
        ui: &mut Ui,
        params: &mut SimulationParameters,
        state: &SimulationState,
    ) -> bool {
        self.circuit
            .show_circuit_editor(ui, &mut params.circuit_diagram, &state.diagram_state)
    }

    pub fn show_fdtd_editor(
        &mut self,
        ui: &mut Ui,
        params: &mut SimulationParameters,
        state: &SimulationState,
    ) -> bool {
        self.fdtd.show_editor(
            ui,
            &state.fdtd,
            &mut params.fdtd_config,
            &mut params.fdtd_wiring,
        )
    }
}

impl Default for SimulationControls {
    fn default() -> Self {
        Self {
            paused: true,
            single_step: false,
            dt: 5e-3,
        }
    }
}

impl SimulationEditor {
    fn new(cfg: &SimulationParameters) -> Self {
        Self {
            circuit: CircuitEditor::default(),
            fdtd: FdtdEditor::new(cfg.fdtd_width),
        }
    }
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            fdtd_width: 10,
            fdtd_config: Default::default(),
            fdtd_wiring: Default::default(),
            circuit_diagram: Default::default(),
            circuit_solver_cfg: Default::default(),
        }
    }
}

impl SimulationControls {
    fn show_ui(&mut self, ui: &mut Ui) -> bool {
        ui.strong("Time step");
        ui.horizontal(|ui| {
            ui.label("Time step: ");
            ui.add(egui::DragValue::new(&mut self.dt).speed(1e-7).suffix(" s"));
        });

        let text = if self.paused { "Play" } else { "Pause" };
        if ui.button(text).clicked() {
            self.paused = !self.paused;
        }

        if ui.button("Single step").clicked() {
            self.single_step = true;
        }

        ui.button("Reset Simulation").clicked()
    }

    fn is_step_this_frame(&mut self) -> bool {
        !self.paused || self.single_step
    }

    fn do_step(&mut self) -> bool {
        let ret = self.is_step_this_frame();

        if self.single_step {
            self.single_step = false;
        }

        ret
    }
}

/// Maps 3D nodes to circuit simulator nodes
struct NodeMap {
    pub map: HashMap<IntPos3, usize>,
}

impl NodeMap {
    /// Inserts wires into the diagram, recording where nodes are
    fn new(rich: &mut RichPrimitiveDiagram, wiring: &Wiring3D) -> Self {
        // Helper function
        fn nodemap_insert(map: &mut HashMap<IntPos3, usize>, pos: IntPos3, primitive_diagram: &mut PrimitiveDiagram) -> usize {
            *map.entry(pos).or_insert_with(|| {
                let idx = primitive_diagram.num_nodes;
                primitive_diagram.num_nodes += 1;
                idx
            })
        }

        // Insert resistors for the wires
        let mut map = HashMap::new();
        for ((a, b), wire) in &wiring.wires {
            let a_idx = nodemap_insert(&mut map, *a, &mut rich.primitive);
            let b_idx = nodemap_insert(&mut map, *b, &mut rich.primitive);
            let component = cirmcut::cirmcut_sim::TwoTerminalComponent::Resistor(wire.resistance);
            rich.primitive.two_terminal.push(([a_idx, b_idx], component));
        }

        // Ports
        for (_name, port_indices) in &rich.ports {
            for i in 0..port_indices.len() {
                for j in i + 1..port_indices.len() {
                    let indices = [port_indices[i], port_indices[j]];
                    let comp = cirmcut::cirmcut_sim::TwoTerminalComponent::Wire;
                    rich.primitive.two_terminal.push((indices, comp));
                }
            }
        }

        Self {
            map
        }
    }
}

fn generate_efield(nodemap: &NodeMap, wiring: &Wiring3D, outs: &SimOutputs, width: usize) -> Array4<f64> {
    let mut field = Array4::<f64>::zeros((width, width, width, 3));

    for (a, b) in wiring.wires.keys() {
        let (x, y, z) = *a;
        let (bx, by, bz) = *b;

        assert!(bx > x || by > y || bz > z);

        let a_idx = nodemap.map[a];
        let b_idx = nodemap.map[b];
        let dv = outs.voltages[b_idx] - outs.voltages[a_idx];

        if bx > x {
            field[(x, y, z, 0)] = dv;
        } else if by > y {
            field[(x, y, z, 1)] = dv;
        } else {
            field[(x, y, z, 2)] = dv;
        }
    }

    field
}
