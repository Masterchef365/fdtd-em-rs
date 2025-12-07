
use cirmcut::{
    circuit_widget::{Diagram, DiagramState},
    cirmcut_sim::{
        PrimitiveDiagram, SimOutputs,
        solver::{Solver, SolverConfig},
    },
};
use egui::{CentralPanel, Color32, RichText, Ui};
use ndarray::Array4;

use crate::{
    circuit_editor::CircuitEditor,
    fdtd_editor::FdtdEditor,
    node_map::NodeMap,
    sim::{FdtdSim, FdtdSimConfig},
    wire_editor_3d::Wiring3D,
};

#[derive(Clone, Copy)]
enum Pane {
    CircuitEditor,
    CircuitEditorCfg,
    FdtdEditor,
    FdtdEditorCfg,
    CommonCfg,
}

impl Pane {
    fn name(&self) -> &'static str {
        match self {
            Pane::CommonCfg => "common",
            Pane::CircuitEditor => "Circuit simulation",
            Pane::CircuitEditorCfg => "Circuit configuration",
            Pane::FdtdEditor => "FDTD simulation",
            Pane::FdtdEditorCfg => "FDTD configuration",
        }
    }
}

struct TreeBehavior {
    params: SimulationParameters,
    state: SimulationState,
    controls: SimulationControls,
    editor: SimulationEditor,
    /// Any error information from the simulation step is stored here.
    error_shown: Option<String>,
    needs_rebuild: bool,
}

/// Every parameter needed for a simulation to proceed, including
/// all wires, components, configuration options, etc.
/// The output of the simulation is a pure function of this struct.
#[derive(serde::Serialize, serde::Deserialize)]
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
    tree: egui_tiles::Tree<Pane>,
    behavior: TreeBehavior,
}

impl FdtdApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let params: SimulationParameters = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();

        let state = SimulationState::new(&params);
        let controls = SimulationControls::default();
        let error_shown = None;
        let editor = SimulationEditor::new(&params);

        let behavior = TreeBehavior {
            params,
            state,
            controls,
            editor,
            error_shown,
            needs_rebuild: false,
        };

        Self {
            tree: create_tree(),
            behavior,
        }
    }
}

impl eframe::App for FdtdApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.behavior.params);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //if self.controls.is_step_this_frame() {
        ctx.request_repaint();
        //}

        CentralPanel::default().show(ctx, |ui| {
            self.tree.ui(&mut self.behavior, ui);
        });

        let ret = self.behavior.step();
        if let Err(e) = ret {
            self.behavior.error_shown = Some(e);
        } else {
            self.behavior.error_shown = None;
        }
    }
}

impl TreeBehavior {
    fn step(&mut self) -> Result<(), String> {
        let do_rebuild = self.needs_rebuild;
        if do_rebuild {
            self.state = SimulationState::new(&self.params);
            self.needs_rebuild = false;
        }

        // Unconditionally rebuild the primitive diagram from the diagram;
        // this allows operating the switches at runtime.
        self.state.rewire(&self.params);

        if self.controls.do_step() || do_rebuild {
            // Create E field from wires
            let width = self.state.fdtd.width();
            let elec = generate_efield(
                &mut self.state.fdtd,
                &self.state.nodemap,
                &self.params.fdtd_wiring,
                &self.state.outputs,
            );
            let magnetization = Array4::<f64>::zeros((width, width, width, 3));

            // Step FDTD
            self.state
                .fdtd
                .step(&self.params.fdtd_config, &magnetization, &elec);

            // Copy the fdtd e-field into the soln vector
            let external_params = readback_efield(
                self.state.fdtd.e_field(),
                &self.state.nodemap,
                &self.params.fdtd_wiring,
                &self.state.circuit_solver,
            );

            // Step circuit
            self.state.circuit_solver.step(
                self.controls.dt,
                &self.state.primitive_diagram,
                &self.params.circuit_solver_cfg,
                Some(&external_params),
            )?;

            self.state.outputs = self
                .state
                .circuit_solver
                .state(&self.state.primitive_diagram);

            self.state.diagram_state =
                DiagramState::new(&self.state.outputs, &self.state.primitive_diagram);
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

    fn rewire(&mut self, params: &SimulationParameters) {
        let mut rich = params.circuit_diagram.to_primitive_diagram();

        self.nodemap = NodeMap::new(&mut rich, &params.fdtd_wiring);
        self.primitive_diagram = rich.primitive;
    }
}

impl SimulationEditor {
    pub fn show_circuit_cfg(
        &mut self,
        ui: &mut Ui,
        params: &mut SimulationParameters,
        state: &SimulationState,
    ) -> bool {
        self.circuit.show_cfg(
            ui,
            &mut params.circuit_diagram,
            &mut params.circuit_solver_cfg,
            &state.diagram_state,
        )
    }

    pub fn show_fdtd_cfg(
        &mut self,
        ui: &mut Ui,
        params: &mut SimulationParameters,
        state: &SimulationState,
    ) -> bool {
        self.fdtd.show_cfg(
            ui,
            &state.fdtd,
            &mut params.fdtd_config,
            &mut params.fdtd_wiring,
        )
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
            &mut params.fdtd_wiring,
            &state.nodemap,
            &state.outputs,
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

fn generate_efield(
    fdtd: &mut FdtdSim,
    nodemap: &NodeMap,
    wiring: &Wiring3D,
    outs: &SimOutputs,
) -> Array4<f64> {
    let width = fdtd.width();
    let mut external_field = Array4::<f64>::zeros((width, width, width, 3));

    for (a, b) in wiring.wires.keys() {
        let (x, y, z) = *a;
        let (bx, by, bz) = *b;

        assert!(bx > x || by > y || bz > z);

        let a_idx = nodemap.pos_map[a];
        let b_idx = nodemap.pos_map[b];
        let dv = outs.voltages[b_idx] - outs.voltages[a_idx];

        let dim = if bx > x {
            0
        } else if by > y {
            1
        } else {
            2
        };

        let coord = (x, y, z, dim);
        external_field[coord] = dv;
        fdtd.e_field[coord] = 0.0;
    }

    external_field
}

fn readback_efield(
    field: &Array4<f64>,
    nodemap: &NodeMap,
    wiring: &Wiring3D,
    outs: &Solver,
) -> Vec<f64> {
    let n = outs.map().vector_size();
    let mut external_params = vec![0_f64; n];

    for wire_id @ (a, b) in wiring.wires.keys() {
        let (x, y, z) = *a;
        let (bx, by, bz) = *b;

        assert!(bx > x || by > y || bz > z);

        let voltage_drop = if bx > x {
            field[(x, y, z, 0)]
        } else if by > y {
            field[(x, y, z, 1)]
        } else {
            field[(x, y, z, 2)]
        };

        let component_idx = nodemap.component_idx_map.get(wire_id).unwrap();
        let soln_vec_idx = outs
            .map
            .state_map
            .voltage_drops()
            .nth(*component_idx)
            .unwrap();

        external_params[soln_vec_idx] = -voltage_drop;
    }

    external_params
}

fn create_tree() -> egui_tiles::Tree<Pane> {
    let mut tiles = egui_tiles::Tiles::default();

    //let [common, fdtd, circuit, fdtd_cfg, cricuit_cfg] = [Pane::CommonCfg, Pane::FdtdEditor, Pane::CircuitEditor, Pane::FdtdEditorCfg, Pane::CircuitEditorCfg].map(|pane| tiles.insert_tab_tile(vec![tiles.insert_pane(pane)]));
    let [common, fdtd, circuit, fdtd_cfg, circuit_cfg] = [
        Pane::CommonCfg,
        Pane::FdtdEditor,
        Pane::CircuitEditor,
        Pane::FdtdEditorCfg,
        Pane::CircuitEditorCfg,
    ]
    .map(|pane| tiles.insert_pane(pane));

    let left_bar = tiles.insert_vertical_tile(vec![common, fdtd_cfg]);
    let left = tiles.insert_horizontal_tile(vec![left_bar, fdtd]);

    let right = tiles.insert_horizontal_tile(vec![circuit_cfg, circuit]);

    let root = tiles.insert_horizontal_tile(vec![left, right]);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.name().into()
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::CommonCfg => {
                self.needs_rebuild |= self.controls.show_ui(ui);
                if ui.button("Reset everything").clicked() {
                    self.params = SimulationParameters::default();
                    self.needs_rebuild = true;
                }
                if let Some(error) = &self.error_shown {
                    ui.label(RichText::new(error).color(Color32::RED));
                }
            }
            Pane::FdtdEditorCfg => {
                self.needs_rebuild |= self.editor.show_fdtd_cfg(ui, &mut self.params, &self.state);
            }
            Pane::CircuitEditorCfg => {
                self.needs_rebuild |=
                    self.editor
                        .show_circuit_cfg(ui, &mut self.params, &self.state);
            }
            Pane::CircuitEditor => {
                self.needs_rebuild |=
                    self.editor
                        .show_circuit_editor(ui, &mut self.params, &self.state);
            }
            Pane::FdtdEditor => {
                self.needs_rebuild |=
                    self.editor
                        .show_fdtd_editor(ui, &mut self.params, &self.state);
            }
        }

        egui_tiles::UiResponse::None
    }
}
