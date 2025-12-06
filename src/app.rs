use cirmcut::{circuit_widget::Diagram, cirmcut_sim::{solver::{Solver, SolverConfig}, PrimitiveDiagram}};

use crate::{sim::{FdtdSim, FdtdSimConfig}, wire_editor_3d::Wiring3D};

#[derive(Default)]
pub struct SimulationParameters {
    fdtd_width: usize,
    fdtd_config: FdtdSimConfig,
    fdtd_wiring: Wiring3D,
    circuit_diagram: Diagram,
    circuit_solver_cfg: SolverConfig,
}

pub struct SimulationState {
    fdtd: FdtdSim,
    circuit_solver: Solver,
    primitive_diagram: PrimitiveDiagram,
}

pub struct FdtdApp {
    params: SimulationParameters,
    state: SimulationState,
}

impl Default for FdtdApp {
    fn default() -> Self {
        let params = SimulationParameters::default();
        Self {
            state: SimulationState::new(&params),
            params,
        }
    }
}

impl FdtdApp {
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

impl eframe::App for FdtdApp {
    /*
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    */

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

    }
}

impl SimulationState {
    fn new(params: &SimulationParameters) -> Self {
        let mut primitive_diagram = params.circuit_diagram.to_primitive_diagram();
        Self {
            primitive_diagram,
            fdtd: FdtdSim::new(params.fdtd_width),
            circuit_solver: Solver::new(&primitive_diagram),
        }
    }
}
