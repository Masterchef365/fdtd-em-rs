use egui::{Color32, DragValue, SidePanel, Stroke, Vec2};
use ndarray::{Array3, Array4};
use rand::prelude::Distribution;
use threegui::{threegui, Painter3D, Vec3};

use crate::sim::{Flote, Sim, SimConfig};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    sim: Sim,
    sim_cfg: SimConfig,

    show_grid: bool,
    show_minimal_grid: bool,

    show_e_grid: bool,
    show_h_grid: bool,

    show_e_vect: bool,
    show_h_vect: bool,

    show_e_mag: bool,
    show_h_mag: bool,

    new_width: usize,
    pause: bool,

    vect_scale: f32,
}

fn random_sim(width: usize) -> Sim {
    let mut sim = Sim::new(width);

    let unif = rand::distributions::Uniform::new(-1.0, 1.0);
    let mut rng = rand::thread_rng();

    /*
    sim.e_field
        .iter_mut()
        .for_each(|x| *x = unif.sample(&mut rng));
    sim.h_field
        .iter_mut()
        .for_each(|x| *x = unif.sample(&mut rng));
    */

    sim.e_field[(width/2,width/2,width/2,0)] = 10_f32.into();
    sim.e_field[(width/2,width/2,width/2,1)] = 1_f32.into();
    sim.e_field[(width/2,width/2,width/2,2)] = 5_f32.into();
    /*
    sim.h_field[(width/2,width/2,width/2,1)] = 10.;

    sim.e_field[(0,0,0,0)] = 10.;
    sim.h_field[(0,0,0,0)] = 10.;
    */

    sim
}

impl Default for TemplateApp {
    fn default() -> Self {
        let sim = random_sim(10);
        Self {
            pause: false,
            new_width: sim.width(),

            sim,
            show_grid: false,
            show_minimal_grid: true,

            show_e_grid: false,
            show_h_grid: false,

            show_e_vect: true,
            show_h_vect: true,

            show_e_mag: false,
            show_h_mag: false,

            vect_scale: 0.5,

            sim_cfg: SimConfig {
                dx: 1.,
                dt: 0.1,
                mu: 1.,
                eps: 1.,
            },
        }
    }
}

impl TemplateApp {
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

impl eframe::App for TemplateApp {
    /*
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    */

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.pause {
            ctx.request_repaint();
            self.sim.step(&self.sim_cfg);
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        SidePanel::left("left panel").show(ctx, |ui| {
            ui.strong("Background grid");
            ui.checkbox(&mut self.show_grid, "Show grid");
            ui.checkbox(&mut self.show_minimal_grid, "Show minimal grid");

            ui.strong("Field visualization");
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

            ui.separator();
            ui.strong("State control");
            ui.checkbox(&mut self.pause, "Paused");

            ui.separator();
            ui.strong("Sim settings");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut self.new_width).prefix("Width: "));
                if ui.button("Reset").clicked() {
                    self.sim = random_sim(self.new_width);
                }
            });
            ui.add(
                DragValue::new(&mut self.sim_cfg.dt)
                    .prefix("Δt: ")
                    .speed(1e-3),
            );
            ui.add(
                DragValue::new(&mut self.sim_cfg.dx)
                    .prefix("Δx: ")
                    .speed(1e-3),
            );
            ui.add(
                DragValue::new(&mut self.sim_cfg.eps)
                    .prefix("ε: ")
                    .speed(1e-3),
            );
            ui.add(
                DragValue::new(&mut self.sim_cfg.mu)
                    .prefix("μ: ")
                    .speed(1e-3),
            );

            if ui.button("Step").clicked() {
                self.sim.step(&self.sim_cfg);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                threegui::ThreeWidget::new("E&M torture")
                    .with_desired_size(ui.available_size())
                    .show(ui, |thr| {
                        let paint = thr.painter();
                        if self.show_grid {
                            draw_grid(
                                paint,
                                self.sim.width(),
                                Stroke::new(1., Color32::from_gray(36)),
                            );
                        } 

                        if self.show_minimal_grid {
                            draw_minimal_grid(
                                paint,
                                self.sim.width(),
                                Color32::LIGHT_GRAY,
                            );
                        }

                        let e_color = Stroke::new(1., Color32::YELLOW);
                        let h_color = Stroke::new(1., Color32::RED);
                        if self.show_e_grid {
                            draw_efield_grid(paint, &self.sim, e_color, self.vect_scale);
                        }
                        if self.show_h_grid {
                            draw_hfield_grid(paint, &self.sim, h_color, self.vect_scale);
                        }

                        if self.show_e_vect {
                            draw_efield_vect(paint, &self.sim, e_color, self.vect_scale);
                        }
                        if self.show_h_vect {
                            draw_hfield_vect(paint, &self.sim, h_color, self.vect_scale);
                        }

                        if self.show_e_mag {
                            draw_efield_mag(paint, &self.sim, e_color.color, self.vect_scale*10.);
                        }
                        if self.show_h_mag {
                            draw_hfield_mag(paint, &self.sim, h_color.color, self.vect_scale*10.);
                        }


                    });
            });
        });
    }
}

fn espace(width: usize, v: Vec3) -> Vec3 {
    v - Vec3::splat(width as f32/2.)
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
    field: &Array4<Flote>,
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
                    let base = base + offset * (Vec3::ONE - unit_vect);
                    let extent: f32 = field[(i, j, k, coord)].into();

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
    field: &Array4<Flote>,
    width: usize,
    stroke: Stroke,
    scale: f32,
) {
    for i in 0..width {
        for j in 0..width {
            for k in 0..width {
                let base = Vec3::new(i as f32, j as f32, k as f32);
                let extent = Vec3::new(
                    field[(i, j, k, 0)].into(),
                    field[(i, j, k, 1)].into(),
                    field[(i, j, k, 2)].into(),
                );

                let pos = espace(width, base);
                let end = pos + extent * scale;
                screenspace_arrow(
                    paint,
                    pos,
                    end,
                    stroke,
                )

                
            }
        }
    }
}

fn draw_field_magnitude(
    paint: &Painter3D,
    field: &Array4<Flote>,
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
                    field[(i, j, k, 0)].into(),
                    field[(i, j, k, 1)].into(),
                    field[(i, j, k, 2)].into(),
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



fn screenspace_arrow(paint: &Painter3D, pos: Vec3, end: Vec3, stroke: Stroke) {
    let screen_pos = paint.internal_transform().world_to_egui(pos);
    let screen_end = paint.internal_transform().world_to_egui(end);
    let screen_len = screen_pos.0.to_pos2().distance(screen_end.0.to_pos2());

    paint.arrow(
        pos,
        (end - pos).normalize_or_zero(),
        screen_len,
        stroke,
    );
}

fn trace_mag_vects(
) {
}

/// Returns (E, H)
fn sample(sim: &Sim, pos: Vec3) -> (Vec3, Vec3) {
    let corner = pos.floor();
    let mag_corner = (pos - 0.5).floor();
    todo!()
}
