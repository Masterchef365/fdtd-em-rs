use ndarray::Array4;

pub struct Sim {
    pub e_field: Array4<f32>,
    pub h_field: Array4<f32>,
    width: usize,
}

impl Sim {
    pub fn new(width: usize) -> Self {
        let e_field = Array4::zeros((width, width, width, 3));
        let h_field = Array4::zeros((width, width, width, 3));
        Self {
            e_field,
            h_field,
            width,
        }
    }

    pub fn e_field(&self) -> &Array4<f32> {
        &self.e_field
    }

    pub fn h_field(&self) -> &Array4<f32> {
        &self.h_field
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn step(&mut self, cfg: &SimConfig, magnetization: &Array4<f32>) {

        half_step(&mut self.e_field, &self.h_field, cfg.scaling(), self.width, Some(magnetization));

        let width = self.width();
        for xi in 0..width {
            for yi in 0..width {
                self.e_field[(0, xi, yi, 0)] = 0.0;
                self.e_field[(width-1, xi, yi, 0)] = 0.0;
                self.e_field[(xi, 0, yi, 1)] = 0.0;
                self.e_field[(xi, width-1, yi, 1)] = 0.0;
                self.e_field[(xi, yi, 0, 2)] = 0.0;
                self.e_field[(xi, yi, width-1, 2)] = 0.0;
            }
        }

        half_step(&mut self.h_field, &self.e_field, -cfg.scaling(), self.width, None);

        let width = self.width();
        for xi in 0..width {
            for yi in 0..width {
                self.h_field[(0, xi, yi, 0)] = 0.0;
                self.h_field[(width-1, xi, yi, 0)] = 0.0;
                self.h_field[(xi, 0, yi, 1)] = 0.0;
                self.h_field[(xi, width-1, yi, 1)] = 0.0;
                self.h_field[(xi, yi, 0, 2)] = 0.0;
                self.h_field[(xi, yi, width-1, 2)] = 0.0;
            }
        }
    }
}

pub struct SimConfig {
    /// Spacial step (meters)
    pub dx: f32,
    /// Time step (seconds)
    pub dt: f32,
    /// Permittivity N/A^2
    pub mu: f32,
    /// Permeability F/m
    pub eps: f32,
}

impl SimConfig {
    pub fn scaling(&self) -> f32 {
        self.dt / self.dx / (self.mu * self.eps).sqrt()
    }
}

const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;

/// Some Numerical Techniques for Maxwell's
/// Equations in Different Types of Geometries
/// (Bengt Fornberg)
fn half_step(a: &mut Array4<f32>, b: &Array4<f32>, scale: f32, width: usize, mag: Option<&Array4<f32>>) {
    let get = |coord: (usize, usize, usize, usize)| {
        match mag {
            Some(mag) => mag[coord] + b[coord],
            None => b[coord],
        }
    };

    let dx = |(xi, yi, zi): (usize, usize, usize), coord: usize| {
        get((xi + 1, yi, zi, coord)) - get((xi - 1, yi, zi, coord))
    };

    let dy = |(xi, yi, zi): (usize, usize, usize), coord: usize| {
        get((xi, yi + 1, zi, coord)) - get((xi, yi - 1, zi, coord))
    };

    let dz = |(xi, yi, zi): (usize, usize, usize), coord: usize| {
        get((xi, yi, zi + 1, coord)) - get((xi, yi, zi - 1, coord))
    };

    for xi in 1..width - 1 {
        for yi in 1..width - 1 {
            for zi in 1..width - 1 {
                let coord = (xi, yi, zi);
                a[(xi, yi, zi, X)] += scale * (dy(coord, Z) - dz(coord, Y));
                a[(xi, yi, zi, Y)] += scale * (dz(coord, X) - dx(coord, Z));
                a[(xi, yi, zi, Z)] += scale * (dx(coord, Y) - dy(coord, X));
            }
        }
    }
}
