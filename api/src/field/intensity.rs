use super::{Field, Intensity};
use crate::{Observer, Observing};
use indicatif::ProgressBar;
use rand_distr::{Distribution, Poisson};

fn shift_and_add(buffer: &mut [f64], x0: f64, y0: f64, n: i32, intensity: Vec<f64>) {
    let i0 = x0 as i32;
    let j0 = y0 as i32;
    for i in 0..n {
        let ii = i0 + i;
        if ii < 0 || ii >= n {
            continue;
        }
        for j in 0..n {
            let jj = j0 + j;
            if jj < 0 || jj >= n {
                continue;
            }
            let k = i * n + j;
            let kk = ii * n + jj;
            buffer[kk as usize] += intensity[k as usize];
        }
    }
}

fn binning(intensity_sampling: usize, m: usize, buffer: Vec<f64>) -> Vec<f64> {
    let n = intensity_sampling / m;
    let n_buffer = n * m;
    let h = (intensity_sampling - n_buffer) / 2;
    let mut image = vec![0f64; n * n];
    let rows: Vec<_> = buffer
        .chunks(intensity_sampling)
        .flat_map(|r| r.iter().skip(h).take(n_buffer))
        .collect();
    let matched_buffer: Vec<_> = (0..n_buffer)
        .flat_map(|k| {
            rows.iter()
                .skip(k + h * n_buffer)
                .step_by(n_buffer)
                .take(n_buffer)
                .collect::<Vec<_>>()
        })
        .collect();
    for i in 0..n {
        let ii = i * m;
        for j in 0..n {
            let jj = j * m;
            let mut bin = 0f64;
            for ib in 0..m {
                for jb in 0..m {
                    let kk = (ii + ib) * n_buffer + jj + jb;
                    bin += **matched_buffer[kk];
                }
            }
            let k = i * n + j;
            image[k] = bin;
        }
    }
    image
}

#[cfg(feature = "parallel")]
mod parallel;
#[cfg(not(feature = "parallel"))]
mod serial;
