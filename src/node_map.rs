use std::collections::HashMap;

use cirmcut::{
    circuit_widget::RichPrimitiveDiagram,
    cirmcut_sim::PrimitiveDiagram,
};

use crate::{
    common::IntPos3,
    wire_editor_3d::{WireId, Wiring3D},
};

pub struct NodeMap {
    pub pos_map: HashMap<IntPos3, usize>,
    pub component_idx_map: HashMap<WireId, usize>,
}

impl NodeMap {
    /// Inserts wires into the diagram, recording where nodes are
    pub fn new(rich: &mut RichPrimitiveDiagram, wiring: &Wiring3D) -> Self {
        // Helper function
        fn nodemap_insert(
            map: &mut HashMap<IntPos3, usize>,
            pos: IntPos3,
            primitive_diagram: &mut PrimitiveDiagram,
        ) -> usize {
            *map.entry(pos).or_insert_with(|| {
                let idx = primitive_diagram.num_nodes;
                primitive_diagram.num_nodes += 1;
                idx
            })
        }

        // Insert resistors for the wires
        let mut pos_map = HashMap::new();
        let mut component_idx_map = HashMap::new();
        for (wire_id @ (a, b), wire) in &wiring.wires {
            let a_idx = nodemap_insert(&mut pos_map, *a, &mut rich.primitive);
            let b_idx = nodemap_insert(&mut pos_map, *b, &mut rich.primitive);
            let component = cirmcut::cirmcut_sim::TwoTerminalComponent::Resistor(wire.resistance);
            let component_idx = rich.primitive.two_terminal.len();
            rich.primitive
                .two_terminal
                .push(([a_idx, b_idx], component));
            component_idx_map.insert(*wire_id, component_idx);
        }

        // Ports
        for (pos, port) in &wiring.ports {
            if let Some(node_idx) = pos_map.get(&pos) {
                rich.ports
                    .entry(port.0.clone())
                    .or_default()
                    .push(*node_idx);
            }
        }

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
            pos_map,
            component_idx_map,
        }
    }
}
