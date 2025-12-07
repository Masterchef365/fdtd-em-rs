use egui::{Color32, Stroke};
use rand::Rng;
use threegui::{Painter3D, Vec3};

use crate::{
    common::{espace, interp, screenspace_arrow},
    sim::FdtdSim,
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum StreamersMode {
    #[default]
    Off,
    HField,
    EField,
}

pub struct Streamers {
    pub points: Vec<Vec3>,
}

impl Streamers {
    pub fn new(width: usize, n: usize) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            points: (0..n).map(|_| Self::random_pos(width, &mut rng)).collect(),
        }
    }

    fn random_pos(width: usize, rng: &mut impl Rng) -> Vec3 {
        let width = width as f32 - 1.0;
        Vec3::new(
            rng.gen_range(0.0..=width),
            rng.gen_range(0.0..=width),
            rng.gen_range(0.0..=width),
        )
    }

    pub fn step(
        &mut self,
        sim: &FdtdSim,
        paint: &Painter3D,
        dt: f32,
        shimmer: f64,
        scale: f32,
        mode: StreamersMode,
    ) {
        let is_efield = match mode {
            StreamersMode::Off => return,
            StreamersMode::HField => false,
            StreamersMode::EField => true,
        };

        let color = if is_efield {
            Color32::YELLOW
        } else {
            Color32::RED
        };

        let mut rng = rand::thread_rng();
        for point in &mut self.points {
            let width = sim.width();
            let w = width as f32;
            let out_of_bounds = point.to_array().into_iter().any(|x| x < 0.0 || x > w - 1.0);

            if out_of_bounds || rng.gen_bool(shimmer) {
                *point = Self::random_pos(width, &mut rng);
                continue;
            }

            let field = if is_efield {
                sim.e_field()
            } else {
                sim.h_field()
            };
            let field = interp(field, *point);

            let before = *point;
            let after = *point + field * scale;

            *point += field * dt;

            let stroke = Stroke::new(1., color);
            screenspace_arrow(
                paint,
                espace(sim.width(), before),
                espace(sim.width(), after),
                stroke,
            );
            /*
            paint.line(
                espace(sim.width(), before),
                espace(sim.width(), after),
                Stroke::new(1., Color32::WHITE),
            );
            */
        }
    }
}
