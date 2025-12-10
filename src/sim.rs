use ndarray::Array4;

pub struct FdtdSim {
    pub e_field: Array4<f64>,
    pub h_field: Array4<f64>,
    width: usize,
}

impl FdtdSim {
    pub fn new(width: usize) -> Self {
        let e_field = Array4::zeros((width, width, width, 3));
        let h_field = Array4::zeros((width, width, width, 3));
        Self {
            e_field,
            h_field,
            width,
        }
    }

    pub fn e_field(&self) -> &Array4<f64> {
        &self.e_field
    }

    pub fn h_field(&self) -> &Array4<f64> {
        &self.h_field
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn step(
        &mut self,
        cfg: &FdtdSimConfig,
        magnetization: &Array4<f64>,
        current: &Array4<f64>,
    ) -> Array4<f64> {
        let prev_e_field = self.e_field.clone();

        self.e_field -= &(cfg.dt * cfg.mu * current);

        half_step(
            &mut self.e_field,
            &(&self.h_field + magnetization),
            cfg.scaling(),
        );

        let width = self.width();
        for xi in 0..width {
            for yi in 0..width {
                self.e_field[(0, xi, yi, 0)] = 0.0;
                self.e_field[(width - 1, xi, yi, 0)] = 0.0;
                self.e_field[(xi, 0, yi, 1)] = 0.0;
                self.e_field[(xi, width - 1, yi, 1)] = 0.0;
                self.e_field[(xi, yi, 0, 2)] = 0.0;
                self.e_field[(xi, yi, width - 1, 2)] = 0.0;
            }
        }

        half_step(
            &mut self.h_field,
            &(&self.e_field + current),
            -cfg.scaling(),
        );

        let width = self.width();
        for xi in 0..width {
            for yi in 0..width {
                self.h_field[(0, xi, yi, 0)] = 0.0;
                self.h_field[(width - 1, xi, yi, 0)] = 0.0;
                self.h_field[(xi, 0, yi, 1)] = 0.0;
                self.h_field[(xi, width - 1, yi, 1)] = 0.0;
                self.h_field[(xi, yi, 0, 2)] = 0.0;
                self.h_field[(xi, yi, width - 1, 2)] = 0.0;
            }
        }

        let induced_current = &self.e_field - &(prev_e_field - &(cfg.dt * curl(&self.h_field)));
        let induced_current = induced_current / (cfg.dt * cfg.mu);

        induced_current
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FdtdSimConfig {
    /// Spacial step (meters)
    pub dx: f64,
    /// Time step (seconds)
    pub dt: f64,
    /// Permittivity N/A^2
    pub mu: f64,
    /// Permeability F/m
    pub eps: f64,
}

impl FdtdSimConfig {
    pub fn scaling(&self) -> f64 {
        self.dt / self.dx / (self.mu * self.eps).sqrt()
    }
}

const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;

/// Some Numerical Techniques for Maxwell's
/// Equations in Different Types of Geometries
/// (Bengt Fornberg)
fn half_step(a: &mut Array4<f64>, b: &Array4<f64>, scale: f64) {
    *a += &(curl(b) * scale);
}

fn curl(b: &Array4<f64>) -> Array4<f64> {
    let mut output = Array4::<f64>::zeros(b.dim());

    let (wx, wy, wz, _) = b.dim();

    let dx = |(xi, yi, zi): (usize, usize, usize), coord: usize| {
        b[(xi + 1, yi, zi, coord)] - b[(xi - 1, yi, zi, coord)]
    };

    let dy = |(xi, yi, zi): (usize, usize, usize), coord: usize| {
        b[(xi, yi + 1, zi, coord)] - b[(xi, yi - 1, zi, coord)]
    };

    let dz = |(xi, yi, zi): (usize, usize, usize), coord: usize| {
        b[(xi, yi, zi + 1, coord)] - b[(xi, yi, zi - 1, coord)]
    };

    for xi in 1..wx - 1 {
        for yi in 1..wy - 1 {
            for zi in 1..wz - 1 {
                let coord = (xi, yi, zi);
                output[(xi, yi, zi, X)] = dy(coord, Z) - dz(coord, Y);
                output[(xi, yi, zi, Y)] = dz(coord, X) - dx(coord, Z);
                output[(xi, yi, zi, Z)] = dx(coord, Y) - dy(coord, X);
            }
        }
    }

    output
}

impl Default for FdtdSimConfig {
    fn default() -> Self {
        Self {
            dx: 1.,
            dt: 0.005,
            mu: 100.,
            eps: 1.,
        }
    }
}
