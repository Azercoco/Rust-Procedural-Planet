use rayon::prelude::*;

mod biome;
mod constant;
mod noise;

use crate::biome::*;
use crate::constant::*;

fn main() {
    for i in 50..100 {
        generate_with_seed(i);
    }
}

fn generate_with_seed(seed: i32) {
    let size = 1024;
    let multisampling = 2;

    let d_multisampling = 2.0 / ((size * multisampling) as f64);

    let mut imgbuf = image::RgbImage::new(size, size);
    let m_square = (multisampling * multisampling) as f64;

    let (biome_map, level) = generate_biome_map(seed, BIOME_MAP_SIZE);

    let pixel_array: Vec<(f64, f64, f64)> = (0..(size * size))
        .into_par_iter()
        .map(|index| {
            let x = index % size;
            let y = index / size;

            let mut red = 0.0;
            let mut green = 0.0;
            let mut blue = 0.0;

            for i in 0..multisampling {
                for j in 0..multisampling {
                    let py =
                        (2.0 * (x as f64) / (size as f64)) - 1.0 + (i as f64) * d_multisampling;
                    let pz =
                        (2.0 * (y as f64) / (size as f64)) - 1.0 + (j as f64) * d_multisampling;
                    let (r, g, b) = ray_trace(py, pz, &biome_map, level, seed);

                    red += r / m_square;
                    green += g / m_square;
                    blue += b / m_square;
                }
            }
            (red, green, blue)
        })
        .collect();

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let index = (x + y * size) as usize;
        let (red, green, blue) = pixel_array[index];
        *pixel = image::Rgb([
            (red * 255.0) as u8,
            (green * 255.0) as u8,
            (blue * 255.0) as u8,
        ]);
    }
    imgbuf.save(format!("result-{}.png", seed)).unwrap();
}

fn ray_trace(
    y: f64,
    z: f64,
    biome_map: &Vec<Vec<(f64, f64, f64)>>,
    level: f64,
    seed: i32,
) -> (f64, f64, f64) {
    let s = RADIUS * RADIUS - y * y - z * z;
    if s < 0.0 {
        return (0.0, 0.0, 0.0);
    } else {
        let x = s.sqrt();

        let (nx, ny, nz, heigth) = normal(x, y, z, seed);
        let (r, g, b) = biome_map[(heigth * (BIOME_MAP_SIZE - 1) as f64) as usize]
            [(biome_value(x, y, z, seed) * (BIOME_MAP_SIZE - 1) as f64) as usize];

        if heigth >= level {
            let ref_vec = reflect_vector((-1.0, 0.0, 0.0), (nx, ny, nz));
            let mut cos_r = cos_vec(ref_vec, NORMAL);

            if cos_r < 0.0 {
                cos_r = 0.0;
            } else {
                cos_r = cos_r.powf(A);
            }

            let mut cos = cos_vec((nx, ny, nz), NORMAL);
            if cos <= 0.05 {
                cos = 0.05;
            }

            let v = (0.9 * cos + 0.1 * cos_r).min(1.0);
            return (v * r, v * g, v * b);
        } else {
            let ref_vec = reflect_vector((-1.0, 0.0, 0.0), (x, y, z));
            let mut cos_r = cos_vec(ref_vec, NORMAL);

            if cos_r < 0.0 {
                cos_r = 0.0;
            } else {
                cos_r = cos_r.powf(A);
            }

            let mut cos = cos_vec((x, y, z), NORMAL);
            if cos <= 0.05 {
                cos = 0.05;
            }

            let v = (0.8 * cos + 0.25 * cos_r).min(1.0);
            return (v * r, v * g, v * b);
        }
    }
}

fn normal(x: f64, y: f64, z: f64, seed: i32) -> (f64, f64, f64, f64) {
    let heigth0 = heigth_value(x, y, z, seed);

    let theta = y.atan2(x);
    let phi = (z / RADIUS).acos();

    let x1 = RADIUS * phi.sin() * (theta + EPS).cos();
    let y1 = RADIUS * phi.sin() * (theta + EPS).sin();
    let z1 = z;

    let heigth1 = heigth_value(x1, y1, z1, seed);

    let x2 = RADIUS * (phi + EPS).sin() * theta.cos();
    let y2 = RADIUS * (phi + EPS).sin() * theta.sin();
    let z2 = RADIUS * (phi + EPS).cos();

    let heigth2 = heigth_value(x2, y2, z2, seed);

    let f0 = 1.0 + heigth0 * H;
    let f1 = 1.0 + heigth1 * H;
    let f2 = 1.0 + heigth2 * H;

    let dx1 = f1 * x1 - f0 * x;
    let dy1 = f1 * y1 - f0 * y;
    let dz1 = f1 * z1 - f0 * z;

    let dx2 = f2 * x2 - f0 * x;
    let dy2 = f2 * y2 - f0 * y;
    let dz2 = f2 * z2 - f0 * z;

    return (
        dy2 * dz1 - dy1 * dz2,
        dx1 * dz2 - dz1 * dx2,
        dx2 * dy1 - dy2 * dx1,
        heigth0,
    );
}

fn reflect_vector(vec1: (f64, f64, f64), normal: (f64, f64, f64)) -> (f64, f64, f64) {
    let (vx, vy, vz) = vec1;
    let (mut nx, mut ny, mut nz) = normal;

    let norm = (nx * nx + ny * ny + nz * nz).sqrt();
    nx /= norm;
    ny /= norm;
    nz /= norm;

    let dot_prod = vx * nx + vy * ny + vz * nz;
    return (
        -2.0 * dot_prod * nx + vx,
        -2.0 * dot_prod * ny + vy,
        -2.0 * dot_prod * nz + vz,
    );
}

fn cos_vec(vec1: (f64, f64, f64), vec2: (f64, f64, f64)) -> f64 {
    let (a1, a2, a3) = vec1;
    let (b1, b2, b3) = vec2;
    return (a1 * b1 + a2 * b2 + a3 * b3)
        / ((a1 * a1 + a2 * a2 + a3 * a3).sqrt() * (b1 * b1 + b2 * b2 + b3 * b3).sqrt());
}

fn heigth_value(x: f64, y: f64, z: f64, seed: i32) -> f64 {
    use crate::noise::perlin;
    let dx = D * perlin(4.0 * x, 4.0 * y, 4.0 * z, 8, 0.5, seed);
    let dy = D * perlin(4.0 * x, 4.0 * y, 4.0 * z, 8, 0.5, seed + 1);
    let dz = D * perlin(4.0 * x, 4.0 * y, 4.0 * z, 8, 0.5, seed + 2);

    return perlin(4.0 * x + dx, 4.0 * y + dy, 4.0 * z + dz, 8, 0.5, seed + 3);
}

fn biome_value(x: f64, y: f64, z: f64, seed: i32) -> f64 {
    use crate::noise::perlin;
    return perlin(4.0 * x, 4.0 * y, 4.0 * z, 8, 0.5, seed + 7);
}
