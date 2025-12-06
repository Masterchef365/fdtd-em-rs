use cirmcut::circuit_widget::{DiagramEditor, VisualizationOptions};
use egui::Rect;

pub struct CircuitEditor {
    pub view_rect: Rect,
    pub editor: DiagramEditor,
    pub vis_opt: VisualizationOptions,
    pub error: Option<String>,
}

impl CircuitEditor {
    pub fn show() {
    }
}
