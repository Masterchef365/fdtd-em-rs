use std::collections::HashSet;

use egui::{Color32, Pos2, Vec2};
use threegui::{Painter3D, ThreeUi, Vec3};

use crate::{common::{espace, espacet, IntPos3}, sim::Sim};

struct Wiring3D {
    wires: HashSet<(IntPos3, IntPos3)>,
    connections: Vec<(String, IntPos3)>,
}

enum Selection {
    Position(IntPos3),
    WireId(usize),
}

pub struct WireEditor3D {
    sel_pos: Option<Selection>,
}

impl Default for WireEditor3D {
    fn default() -> Self {
        Self {
            sel_pos: None,
        }
    }
}

fn find_closest_grid_point_screenspace(width: usize, paint: &Painter3D, screen_pos: Pos2) -> Option<(IntPos3, f32)> {
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

impl WireEditor3D {
    pub fn draw(&mut self, sim: &Sim, thr: &ThreeUi) {
        let paint = thr.painter();

        let width = sim.width();

        let Some(cursor_pos) = paint.egui().ctx().input(|r| r.pointer.latest_pos()) else { return; };

        let Some((cursor_pos_3d, cursor_grid_dist)) = find_closest_grid_point_screenspace(width, paint, cursor_pos) else { return; };

        let cursor_circle_size = 10.0; 
        paint.circle(
            espacet(width, cursor_pos_3d),
            cursor_circle_size,
            (1.0, Color32::GREEN),
        );

        if let Some(Selection::Position(pos)) = self.sel_pos {
            paint.circle(
                espacet(width, pos),
                cursor_circle_size,
                (1.0, Color32::YELLOW),
            );
        }

        if cursor_grid_dist < cursor_circle_size && thr.resp.clicked() {
            self.sel_pos = Some(Selection::Position(cursor_pos_3d));
            return;
        }


    }
}
