use egui::Stroke;
use ndarray::Array4;
use rand::prelude::Distribution;
use threegui::{Painter3D, Vec3};

pub type IntPos3 = (usize, usize, usize);

fn read_array4(field: &Array4<f32>, i: isize, j: isize, k: isize) -> Option<Vec3> {
    if i < 0 || j < 0 || k < 0 {
        return None;
    }

    let [i, j, k] = [i, j, k].map(|x| x as usize);
    Some(Vec3::new(
        *field.get((i, j, k, 0))?,
        *field.get((i, j, k, 1))?,
        *field.get((i, j, k, 2))?,
    ))
}

fn read_array4_or_zero(arr: &Array4<f32>, i: isize, j: isize, k: isize) -> Vec3 {
    read_array4(arr, i, j, k).unwrap_or(Vec3::ZERO)
}

pub fn interp(arr: &Array4<f32>, pos: Vec3) -> Vec3 {
    let fr = pos.fract();
    let [i, j, k] = pos.floor().to_array().map(|x| x as isize);

    read_array4_or_zero(arr, i, j, k)
        .lerp(read_array4_or_zero(arr, i + 1, j, k), fr.x)
        .lerp(
            read_array4_or_zero(arr, i, j + 1, k)
                .lerp(read_array4_or_zero(arr, i + 1, j + 1, k), fr.x),
            fr.y,
        )
        .lerp(
            read_array4_or_zero(arr, i, j, k + 1)
                .lerp(read_array4_or_zero(arr, i + 1, j, k + 1), fr.x)
                .lerp(
                    read_array4_or_zero(arr, i, j + 1, k + 1)
                        .lerp(read_array4_or_zero(arr, i + 1, j + 1, k + 1), fr.x),
                    fr.y,
                ),
            fr.z,
        )
}

pub fn screenspace_arrow(paint: &Painter3D, pos: Vec3, end: Vec3, stroke: Stroke) {
    let screen_pos = paint.internal_transform().world_to_egui(pos);
    let screen_end = paint.internal_transform().world_to_egui(end);
    let screen_len = screen_pos.0.to_pos2().distance(screen_end.0.to_pos2());

    paint.arrow(pos, (end - pos).normalize_or_zero(), screen_len, stroke);
}

pub fn espace(width: usize, v: Vec3) -> Vec3 {
    v - Vec3::splat(width as f32 / 2.)
}

pub fn espacet(width: usize, (x, y, z): IntPos3) -> Vec3 {
    espace(width, Vec3::new(x as f32, y as f32, z as f32))
}

/*
pub fn espace_inv(width: usize, v: Vec3) -> Vec3 {
    v + Vec3::splat(width as f32 / 2.)
}

pub fn espacet_inv(width: usize, v: Vec3) -> IntPos3 {
    let a = espace_inv(width, v);
    (a.x as usize, a.y as usize, a.z as usize)
}
*/
