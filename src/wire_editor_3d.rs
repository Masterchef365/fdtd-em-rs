use egui::{Color32, Pos2, Vec2};
use threegui::{Painter3D, Vec3};

use crate::{common::{espace, espacet}, sim::Sim};

pub struct WireEditor3D {

}

impl Default for WireEditor3D {
    fn default() -> Self {
        Self {
        }
    }
}

fn find_closest_grid_point_screenspace(width: usize, paint: &Painter3D, screen_pos: Pos2) -> Option<(usize, usize, usize)> {
    let mut closest = None;
    let mut closest_dist = 99e9;


    for i in 0..width {
        for j in 0..width {
            for k in 0..width {
                if let Some(pt_pos) = paint.transform(espacet(width, (i, j, k))) {
                    let dist_sq = pt_pos.distance_sq(screen_pos);
                    if dist_sq < closest_dist {
                        closest_dist = dist_sq;
                        closest = Some((i, j, k));
                    }
                }
            }
        }
    }

    closest
}

impl WireEditor3D {
    pub fn draw(&self, sim: &Sim, paint: &Painter3D) {
        let width = sim.width();

        let Some(cursor_pos) = paint.egui().ctx().input(|r| r.pointer.latest_pos()) else { return; };

        let Some(cursor_pos_3d) = find_closest_grid_point_screenspace(width, paint, cursor_pos) else { return; };

        paint.circle(
            espacet(width, cursor_pos_3d),
            10.0,
            (1.0, Color32::GREEN),
        );




    }
}
