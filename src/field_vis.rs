use egui::{Color32, DragValue, Stroke, Ui};
use ndarray::Array4;
use threegui::{Painter3D, Vec3};

use crate::{
    common::{espace, screenspace_arrow},
    sim::Sim,
};

pub struct GridVisualizationConfig {
    pub vect_scale: f32,

    pub show_grid: bool,
    pub show_minimal_grid: bool,

    pub show_e_grid: bool,
    pub show_h_grid: bool,

    pub show_e_vect: bool,
    pub show_h_vect: bool,

    pub show_e_mag: bool,
    pub show_h_mag: bool,
}

impl Default for GridVisualizationConfig {
    fn default() -> Self {
        Self {
            show_e_grid: false,
            show_h_grid: false,

            show_e_vect: true,
            show_h_vect: true,

            show_e_mag: false,
            show_h_mag: false,

            show_grid: false,
            show_minimal_grid: true,

            vect_scale: 0.5,
        }
    }
}

impl GridVisualizationConfig {
    pub fn show_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_e_grid, "Show E field grid");
            ui.checkbox(&mut self.show_h_grid, "H grid");
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_e_vect, "Show E field vects");
            ui.checkbox(&mut self.show_h_vect, "H vects");
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_e_mag, "Show E field mag");
            ui.checkbox(&mut self.show_h_mag, "H vects");
        });

        ui.add(
            DragValue::new(&mut self.vect_scale)
                .prefix("Scale: ")
                .speed(1e-3),
        );
    }

    pub fn draw(&self, sim: &Sim, paint: &Painter3D) {
        if self.show_grid {
            draw_grid(paint, sim.width(), Stroke::new(1., Color32::from_gray(36)));
        }

        if self.show_minimal_grid {
            draw_minimal_grid(paint, sim.width(), Color32::LIGHT_GRAY);
        }

        let e_color = Stroke::new(1., Color32::YELLOW);
        let h_color = Stroke::new(1., Color32::RED);
        if self.show_e_grid {
            draw_efield_grid(paint, &sim, e_color, self.vect_scale);
        }
        if self.show_h_grid {
            draw_hfield_grid(paint, &sim, h_color, self.vect_scale);
        }

        if self.show_e_vect {
            draw_efield_vect(paint, &sim, e_color, self.vect_scale);
        }
        if self.show_h_vect {
            draw_hfield_vect(paint, &sim, h_color, self.vect_scale);
        }

        if self.show_e_mag {
            draw_efield_mag(paint, &sim, e_color.color, self.vect_scale * 10.);
        }
        if self.show_h_mag {
            draw_hfield_mag(paint, &sim, h_color.color, self.vect_scale * 10.);
        }
    }
}

fn draw_grid(paint: &Painter3D, width: usize, grid_stroke: Stroke) {
    for i in 0..width {
        for j in 0..width {
            paint.line(
                espace(width, Vec3::new(j as f32, 0.0, i as f32)),
                espace(width, Vec3::new(j as f32, width as f32 - 1., i as f32)),
                grid_stroke,
            );
            paint.line(
                espace(width, Vec3::new(j as f32, i as f32, 0.0)),
                espace(width, Vec3::new(j as f32, i as f32, width as f32 - 1.)),
                grid_stroke,
            );
            paint.line(
                espace(width, Vec3::new(0.0, i as f32, j as f32)),
                espace(width, Vec3::new(width as f32 - 1., i as f32, j as f32)),
                grid_stroke,
            );
        }
    }
}

fn draw_minimal_grid(paint: &Painter3D, width: usize, color: Color32) {
    for i in 0..width {
        for j in 0..width {
            for k in 0..width {
                paint.circle_filled(
                    espace(width, Vec3::new(i as f32, j as f32, k as f32)),
                    1.0,
                    color,
                );
            }
        }
    }
}

fn draw_efield_grid(paint: &Painter3D, sim: &Sim, stroke: Stroke, scale: f32) {
    draw_field_grid(paint, sim.e_field(), sim.width(), stroke, scale, 0.0);
}

fn draw_hfield_grid(paint: &Painter3D, sim: &Sim, stroke: Stroke, scale: f32) {
    draw_field_grid(paint, sim.h_field(), sim.width(), stroke, scale, 0.5);
}

fn draw_field_grid(
    paint: &Painter3D,
    field: &Array4<f32>,
    width: usize,
    stroke: Stroke,
    scale: f32,
    offset: f32,
) {
    for i in 0..width {
        for j in 0..width {
            for k in 0..width {
                for (coord, unit_vect) in [Vec3::X, Vec3::Y, Vec3::Z].into_iter().enumerate() {
                    let base = Vec3::new(i as f32, j as f32, k as f32);
                    let base = base + offset * (Vec3::ONE - unit_vect).abs();
                    let extent = field[(i, j, k, coord)];

                    let pos = espace(width, base);
                    let end = pos + unit_vect * extent * scale;
                    screenspace_arrow(paint, pos, end, stroke);
                }
            }
        }
    }
}

fn draw_efield_vect(paint: &Painter3D, sim: &Sim, stroke: Stroke, scale: f32) {
    draw_field_vect(paint, sim.e_field(), sim.width(), stroke, scale);
}

fn draw_hfield_vect(paint: &Painter3D, sim: &Sim, stroke: Stroke, scale: f32) {
    draw_field_vect(paint, sim.h_field(), sim.width(), stroke, scale);
}

fn draw_field_vect(
    paint: &Painter3D,
    field: &Array4<f32>,
    width: usize,
    stroke: Stroke,
    scale: f32,
) {
    for i in 0..width {
        for j in 0..width {
            for k in 0..width {
                let base = Vec3::new(i as f32, j as f32, k as f32);
                let extent = Vec3::new(
                    field[(i, j, k, 0)],
                    field[(i, j, k, 1)],
                    field[(i, j, k, 2)],
                );

                let pos = espace(width, base);
                let end = pos + extent * scale;
                screenspace_arrow(paint, pos, end, stroke)
            }
        }
    }
}

fn draw_field_magnitude(
    paint: &Painter3D,
    field: &Array4<f32>,
    width: usize,
    color: Color32,
    scale: f32,
    offset: f32,
) {
    for i in 0..width {
        for j in 0..width {
            for k in 0..width {
                let base = Vec3::new(i as f32, j as f32, k as f32);
                let extent = Vec3::new(
                    field[(i, j, k, 0)],
                    field[(i, j, k, 1)],
                    field[(i, j, k, 2)],
                );

                let pos = espace(width, base + offset);
                paint.circle_filled(pos, extent.length() * scale, color)
            }
        }
    }
}

fn draw_efield_mag(paint: &Painter3D, sim: &Sim, color: Color32, scale: f32) {
    draw_field_magnitude(paint, sim.e_field(), sim.width(), color, scale, 0.0);
}

fn draw_hfield_mag(paint: &Painter3D, sim: &Sim, color: Color32, scale: f32) {
    draw_field_magnitude(paint, sim.h_field(), sim.width(), color, scale, 0.5);
}
