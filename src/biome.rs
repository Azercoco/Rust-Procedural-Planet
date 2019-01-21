use crate::noise::hash;

pub fn generate_biome_map(seed: i32, size: usize) -> (Vec<Vec<(f64, f64, f64)>>, f64) {
    let mut biome_map = vec![vec![(0.0, 0.0, 0.0); size]; size];

    let (seed, rand_level) = rand(seed);

    let level = ((0.5 * rand_level + 0.3) * (size as f64)) as usize;

    let (seed, color) = random_color(seed);
    let (r, g, b) = color;

    for i in 0..level { // color of water
        let q = (i as f64) / (level as f64);
        for j in 0..size {
            biome_map[i][j] = (
                q.max(0.8)*r / 2.0 ,
                q.max(0.8)*g / 2.0 ,
                q.max(0.8)*b / 2.0 ,
            );
        }
    }

    let (seed, color) = random_color(seed);

    let (seed, (r1, g1, b1)) = random_variation(seed, color, 0.2);
    let (seed, (r2, g2, b2)) = random_variation(seed, color, 0.2);
    let (seed, (r3, g3, b3)) = random_variation(seed, color, 0.2);
    let (seed, (r4, g4, b4)) = random_variation(seed, color, 0.2);

    for i in level..size {
        let s = (i as f64) / ((size - level) as f64);

        let (ra, ga, ba) = (r1 + (r2 - r1) * s, g1 + (g2 - g1) * s, b1 + (b2 - b1) * s);
        let (rb, gb, bb) = (r3 + (r4 - r3) * s, g3 + (g4 - g3) * s, b3 + (b4 - b3) * s);

        for j in 0..size {
            let p = (j as f64) / (size as f64);
            biome_map[i][j] = (ra + (rb - ra) * p, ga + (gb - ga) * p, ba + (bb - ba) * p);
        }
    }

    let (seed, color) = random_color(seed);
    let (mut seed, rand_k) = rand(seed);

    let n_k = (rand_k * 8.0) as usize + 3;

    for _k in 0..n_k {
        let (s, (r, g, b)) = random_variation(seed, color, 0.3);
        seed = s;

        let (s, rand_range) = rand(seed);
        seed = s;
        let (s, rand_x) = rand(seed);
        seed = s;
        let (s, rand_y) = rand(seed);
        seed = s;

        let range = (rand_range * 0.4 + 0.1) * (size as f64);

        let x = (rand_x * (size - level) as f64) + (level as f64);
        let y = rand_y * level as f64;

        for i in level..size {
            for j in 0..size {
                let dist = ((x - i as f64) * (x - i as f64) + (y - j as f64) * (y - j as f64))
                    .sqrt()
                    / range;
                let alpha = (-dist * dist).exp();
                let beta = 1.0 - alpha;

                let (r1, g1, b1) = biome_map[i][j];

                biome_map[i][j] = (
                    alpha * r + beta * r1,
                    alpha * g + beta * g1,
                    alpha * b + beta * b1,
                );
            }
        }
    }

    return (biome_map, level as f64 / size as f64);
}

pub fn rand(mut seed: i32) -> (i32, f64) {
    seed += 17;
    seed = hash(seed);
    let m = 100000000;
    let random_float = (((m + (seed % m)) % m) as f64) / (m as f64);
    seed = hash(seed);
    return (seed, random_float);
}

fn random_color(seed: i32) -> (i32, (f64, f64, f64)) {
    let (seed, red) = rand(seed);
    let (seed, green) = rand(seed);
    let (seed, blue) = rand(seed);
    return (seed, (red, green, blue));
}

fn random_variation(seed: i32, color: (f64, f64, f64), f: f64) -> (i32, (f64, f64, f64)) {
    let (mut r, mut g, mut b) = color;
    let (seed, r1) = rand(seed);
    let (seed, g1) = rand(seed);
    let (seed, b1) = rand(seed);

    r = (r + (2.0 * f * r1) - f).max(0.0).min(1.0);
    g = (g + (2.0 * f * g1) - f).max(0.0).min(1.0);
    b = (b + (2.0 * f * b1) - f).max(0.0).min(1.0);

    return (seed, (r, g, b));
}
