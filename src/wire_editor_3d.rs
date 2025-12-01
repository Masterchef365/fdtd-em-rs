use std::collections::HashMap;

use egui::{Color32, DragValue, Pos2, Stroke, Ui, Vec2};
use threegui::{Painter3D, ThreeUi, Vec3};

use crate::{
    common::{IntPos3, espacet},
    sim::Sim,
};

#[derive(Clone, Copy)]
pub struct Wire {
    /// Ohms
    resistance: f32,
}

const DEFAULT_WIRE: Wire = Wire { resistance: 1e-3 };

#[derive(Clone)]
pub struct Port(pub String);

pub type WireId = (IntPos3, IntPos3);

#[derive(Default, Clone)]
pub struct Wiring3D {
    pub wires: HashMap<WireId, Wire>,
    pub ports: HashMap<IntPos3, Port>,
}

pub struct WireEditor3D {
    sel_pos: Option<Selection>,
    undo: Option<Wiring3D>,
}

#[derive(Clone, Copy)]
enum Selection {
    Position(IntPos3),
    WireId((IntPos3, IntPos3)),
}

impl Default for WireEditor3D {
    fn default() -> Self {
        Self { sel_pos: None, undo: None }
    }
}

fn find_closest_grid_point_screenspace(
    width: usize,
    paint: &Painter3D,
    screen_pos: Pos2,
) -> Option<(IntPos3, f32)> {
    let mut closest = None;
    let mut closest_dist = 99e9;

    for i in 0..width {
        for j in 0..width {
            for k in 0..width {
                if let Some(pt_pos) = paint.transform(espacet(width, (i, j, k))) {
                    let dist = pt_pos.distance(screen_pos);
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest = Some((i, j, k));
                    }
                }
            }
        }
    }

    closest.map(|c| (c, closest_dist))
}

fn find_closest_wire_screenspace(
    width: usize,
    wiring: &Wiring3D,
    paint: &Painter3D,
    screen_pos: Pos2,
) -> Option<(WireId, f32)> {
    let mut closest = None;
    let mut closest_dist = 99e9;

    for &wire_id in wiring.wires.keys() {
        if let Some(dist) = screenspace_wire_dist(wire_id, paint, width, screen_pos) {
            if dist < closest_dist {
                closest_dist = dist;
                closest = Some(wire_id);
            }
        }
    }

    closest.map(|c| (c, closest_dist))
}

impl WireEditor3D {
    pub fn draw(&mut self, width: usize, thr: &ThreeUi, wiring: &mut Wiring3D) {
        let paint = thr.painter();

        // Draw wiring
        wiring.draw(width, paint);

        // Projecting the cursor
        let Some(cursor_pos) = paint.egui().ctx().input(|r| r.pointer.latest_pos()) else {
            return;
        };

        let Some((cursor_pos_3d, cursor_grid_dist)) =
            find_closest_grid_point_screenspace(width, paint, cursor_pos)
        else {
            return;
        };

        // Finding the nearest wire
        let closest_wire = find_closest_wire_screenspace(width, wiring, paint, cursor_pos);

        let cursor_circle_size = 10.0;

        let cursor_color = Color32::GREEN;

        // If the wire is closer...
        if let Some((wire_id, wire_dist)) = closest_wire
            && wire_dist < cursor_grid_dist
        {
            let stroke = Stroke::new(1.0, cursor_color);
            let (a, b) = wire_id;
            paint.line(espacet(width, a), espacet(width, b), stroke);

            if thr.resp.clicked() {
                self.sel_pos = Some(Selection::WireId(wire_id));
            }
        } else {
            // If the grid is closer...
            paint.circle(
                espacet(width, cursor_pos_3d),
                cursor_circle_size,
                (1.0, cursor_color),
            );

            if thr.resp.clicked() {
                if thr.resp.ctx.input(|r| r.modifiers.shift) {
                    self.line_to_selection(cursor_pos_3d, wiring, DEFAULT_WIRE);
                    self.undo = Some(wiring.clone());
                }

                self.sel_pos = Some(Selection::Position(cursor_pos_3d));
                return;
            }
        }

        // Undo
        if thr.resp.ctx.input(|r| r.modifiers.ctrl && r.key_released(egui::Key::U)) {
            if let Some(state) = self.undo.take() {
                *wiring = state;
            }
        }

        // Draw selection
        let selection_stroke = Stroke::new(1.0, Color32::YELLOW);

        if let Some(selection) = self.sel_pos {
            match selection {
                Selection::Position(pos) => {
                    paint.circle(espacet(width, pos), cursor_circle_size, selection_stroke);
                }
                Selection::WireId(wire_id) => {
                    let (a, b) = wire_id;
                    paint.line(espacet(width, a), espacet(width, b), selection_stroke);
                }
            }
        }
    }

    pub fn show_ui(&mut self, ui: &mut Ui, width: usize, wiring: &mut Wiring3D) {
        ui.strong("Wires");

        if ui.button("Add wire").clicked() {
            if let Some(Selection::Position(pos @ (x, y, z))) = self.sel_pos {
                let b = if z + 1 < width {
                    (x, y, z + 1)
                } else {
                    (x, y, z - 1)
                };

                let line = (pos, b);
                wiring.insert(line, DEFAULT_WIRE);
                self.sel_pos = Some(Selection::WireId(line));
            }
        }
        ui.label("A quicker way is to select a point, then hold shift and select another.");
        ui.separator();

        ui.strong("Editing wire");
        if let Some(Selection::WireId(wire_id)) = self.sel_pos {
            if let Some(wire) = wiring.wires.get_mut(&wire_id) {
                wire.show_ui(ui);
                if ui.button("Delete").clicked() {
                    wiring.wires.remove(&wire_id);
                    self.sel_pos = None;
                }
            }
        }

        ui.separator();
    }

    fn line_to_selection(&mut self, start: IntPos3, wiring: &mut Wiring3D, wire: Wire) {
        let Some(Selection::Position(end)) = self.sel_pos else {
            return;
        };

        let (sx, sy, sz) = start;
        let (ex, ey, ez) = end;

        if sx < ex {
            for x in sx..ex {
                wiring.insert(((x, sy, sz), (x + 1, sy, sz)), wire);
            }
        } else {
            for x in ex..sx {
                wiring.insert(((x, sy, sz), (x - 1, sy, sz)), wire);
            }
        }

        if sy < ey {
            for y in sy..ey {
                wiring.insert(((ex, y, sz), (ex, y + 1, sz)), wire);
            }
        } else {
            for y in ey..sy {
                wiring.insert(((ex, y, sz), (ex, y - 1, sz)), wire);
            }
        }

        if sz < ez {
            for z in sz..ez {
                wiring.insert(((ex, ey, z), (ex, ey, z + 1)), wire);
            }
        } else {
            for z in ez..sz {
                wiring.insert(((ex, ey, z), (ex, ey, z - 1)), wire);
            }
        }






        
    }
}

impl Wiring3D {
    pub fn insert(&mut self, pos: (IntPos3, IntPos3), wire: Wire) -> Option<Wire> {
        self.wires.insert(pos, wire)
    }

    pub fn get(&self, wire_id @ (a, b): WireId) -> Option<&Wire> {
        self.wires.get(&wire_id).or_else(|| self.wires.get(&(b, a)))
    }

    pub fn get_mut(&mut self, wire_id @ (a, b): WireId) -> Option<&mut Wire> {
        if self.wires.contains_key(&wire_id) {
            self.wires.get_mut(&wire_id)
        } else {
            self.wires.get_mut(&(b, a))
        }
    }

    pub fn remove(&mut self, wire_id @ (a, b): WireId) {
        self.wires.remove(&wire_id);
        self.wires.remove(&(b, a));
    }

    pub fn draw(&self, width: usize, paint: &Painter3D) {
        let stroke = Stroke::new(1.0, Color32::GRAY);
        for (a, b) in self.wires.keys() {
            paint.line(espacet(width, *a), espacet(width, *b), stroke);
        }
    }
}

impl Wire {
    pub fn show_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Resistance: ");
            ui.add(DragValue::new(&mut self.resistance).suffix("Ohms"));
        });
    }
}

fn screenspace_wire_dist(
    wire_id: WireId,
    paint: &Painter3D,
    width: usize,
    pt: Pos2,
) -> Option<f32> {
    let (a, b) = wire_id;

    let pa = paint.transform(espacet(width, a))?;
    let pb = paint.transform(espacet(width, b))?;

    let u = pt - pa;
    let v = pb - pa;
    let p = proj(u, v);

    if p >= 0.0 && p <= 1.0 {
        Some(((p * v) - u).length())
    } else {
        let dist_a = pt.distance(pa);
        let dist_b = pt.distance(pb);

        Some(dist_a.min(dist_b))
    }
}

fn proj(u: Vec2, v: Vec2) -> f32 {
    return u.dot(v) / v.dot(v);
}
