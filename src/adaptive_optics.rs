use num_complex::Complex;
use num_traits::Zero;

use crate::{optust, Star, ZpDft};

const DELTA_0: f64 = 2.5e-2;

#[derive(Debug)]
struct TurbulenceProfile {
    height: Vec<f64>,
    weight: Vec<f64>,
}
impl TurbulenceProfile {
    pub fn new() -> Self {
        Self {
            height: vec![25., 275., 425., 1250., 4000., 8000., 13000.],
            weight: vec![0.1257, 0.0874, 0.0666, 0.3498, 0.2273, 0.0681, 0.0751],
        }
    }
}
impl IntoIterator for TurbulenceProfile {
    type Item = (f64, f64);

    type IntoIter = std::iter::Zip<std::vec::IntoIter<f64>, std::vec::IntoIter<f64>>;

    fn into_iter(self) -> Self::IntoIter {
        self.height.into_iter().zip(self.weight.into_iter())
    }
}

#[derive(Debug)]
struct TransferFunction {
    fft: ZpDft,
    d: f64,
    n_otf: usize,
    kappa: usize,
    fitting_cutoff: f64,
}
impl Clone for TransferFunction {
    fn clone(&self) -> Self {
        Self {
            fft: ZpDft::forward(self.fft.len()),
            d: self.d.clone(),
            n_otf: self.n_otf.clone(),
            kappa: self.kappa.clone(),
            fitting_cutoff: self.fitting_cutoff.clone(),
        }
    }
}
impl TransferFunction {
    pub fn new(n_otf: usize, d: f64) -> Self {
        if d < DELTA_0 {
            panic!("Pupil sampling is too small, must be greater or equal to 2.5cm")
        }
        let kappa = (d / DELTA_0).ceil() as usize;
        let n = usize::max(kappa * n_otf, 4096);
        Self {
            fft: ZpDft::forward(n),
            n_otf,
            d,
            kappa,
            fitting_cutoff: 0f64,
        }
    }
    pub fn fitting_cutoff_frequency(
        &mut self,
        strehl_ratio: f64,
        fried_parameter: f64,
        outer_scale: f64,
    ) -> &mut Self {
        let var_lim = 2f64 * (1f64 - strehl_ratio.sqrt());
        let n = 4096 * 2;
        let delta = 1e-2_f64;
        for i in 0..n {
            let df = 1f64 / (delta * (n - 1) as f64);
            let mut var: f64 = (i..n)
                .map(|i| {
                    let f = i as f64 * df;
                    f * optust::phase::spectrum(f, fried_parameter, outer_scale)
                })
                .sum();
            var *= 2f64 * std::f64::consts::PI * df;
            if var <= var_lim {
                self.fitting_cutoff = i as f64 * df;
                break;
            }
        }
        self
        /*         let var_analytic = optust::phase::variance(fried_parameter, outer_scale);
        dbg!(var_analytic);
        dbg!(var); */
    }
}
#[derive(Debug, Clone)]
pub struct AdaptiveOpticsCorrection {
    strehl_ratio: f64,
    guide_star: Option<Star>,
    transfer_function: Option<TransferFunction>,
}
impl AdaptiveOpticsCorrection {
    pub fn new(strehl_ratio: f64, guide_star: Option<Star>) -> Self {
        Self {
            strehl_ratio,
            guide_star,
            transfer_function: None,
        }
    }
    pub fn init_transfer_function(
        &mut self,
        n_otf: usize,
        d: f64,
        fried_parameter: f64,
        outer_scale: f64,
    ) -> &mut Self {
        self.transfer_function = Some(TransferFunction::new(n_otf, d));
        self.transfer_function
            .as_mut()
            .map(|tf| tf.fitting_cutoff_frequency(self.strehl_ratio, fried_parameter, outer_scale));
        self
    }
    /*     pub fn variance_check(&mut self, fried_parameter: f64, outer_scale: f64) {
        let n = 4096;
        let delta = 2.5e-2_f64;
        let df = 1f64 / (delta * (n - 1) as f64);
        let mut var: f64 = (0..n)
            .map(|i| {
                let f = i as f64 * df;
                f * optust::phase::spectrum(f, fried_parameter, outer_scale)
            })
            .sum();
        var *= 2f64 * std::f64::consts::PI * df;
        let var_analytic = optust::phase::variance(fried_parameter, outer_scale);
        dbg!(var_analytic);
        dbg!(var);
    }
    pub fn psd_sampling(&mut self, fried_parameter: f64, outer_scale: f64) -> Vec<Complex<f64>> {
        let f_max = 1e2;
        let kappa = (f_max * 2. * self.d).ceil() as usize;
        let n = 4096;
        let mut psd: Vec<Complex<f64>> = vec![Complex::zero(); n * n];
        let df = 0.1f64.recip() / (n - 1) as f64;
        dbg!(df);
        for i in 0..n {
            let q = i as i32 - n as i32 / 2;
            let x = q as f64 * df;
            let ii = if q < 0i32 {
                (q + n as i32) as usize
            } else {
                q as usize
            };
            for j in 0..n {
                let q = j as i32 - n as i32 / 2;
                let y = q as f64 * df;
                let jj = if q < 0i32 {
                    (q + n as i32) as usize
                } else {
                    q as usize
                };
                let f = x.hypot(y);
                let kk = ii * n + jj;
                psd[kk].re = optust::phase::spectrum(f, fried_parameter, outer_scale);
            }
        }
        let sum_psd: f64 = psd.iter().map(|x| x.re).sum::<f64>() * df * df;
        dbg!(sum_psd);
        psd
    } */
    pub fn transfer_function(
        &mut self,
        fried_parameter: f64,
        outer_scale: f64,
        star: &Star,
    ) -> Vec<Complex<f64>> {
        let TransferFunction {
            fft,
            d,
            n_otf,
            kappa,
            fitting_cutoff,
        } = self.transfer_function.as_mut().unwrap();
        let d = *d;
        let kappa = *kappa;
        let fitting_cutoff = *fitting_cutoff;
        let n = fft.len();
        let df = (d / kappa as f64).recip() / (n - 1) as f64;
        let mut psd: Vec<Complex<f64>> = vec![Complex::zero(); n * n];
        for i in 0..n {
            let q = i as i32 - n as i32 / 2;
            let x = q as f64 * df;
            let ii = if q < 0i32 {
                (q + n as i32) as usize
            } else {
                q as usize
            };
            for j in 0..n {
                let q = j as i32 - n as i32 / 2;
                let y = q as f64 * df;

                let jj = if q < 0i32 {
                    (q + n as i32) as usize
                } else {
                    q as usize
                };
                let f = x.hypot(y);

                let buffer = optust::phase::spectrum(f, fried_parameter, outer_scale);
                let (x_star, y_star) = star.coordinates;
                let (x_gs, y_gs) = self.guide_star.unwrap_or_default().coordinates;
                let delta_x = x_star - x_gs;
                let delta_y = y_star - y_gs;
                let anisoplanatism = TurbulenceProfile::new()
                    .into_iter()
                    .map(|(h, w)| {
                        let red = 2. * std::f64::consts::PI * h * (x * delta_x + y * delta_y);
                        w * (1. - red.cos())
                    })
                    .sum::<f64>()
                    * buffer;
                let kk = ii * n + jj;
                psd[kk].re = if f < fitting_cutoff {
                    anisoplanatism
                } else {
                    buffer + anisoplanatism
                };
            }
        }
        let covariance = fft.zero_padding(psd).process().buffer();

        let n_otf = *n_otf;
        let mut cov: Vec<Complex<f64>> = vec![Complex::zero(); n_otf * n_otf];
        for i in 0..(n_otf + 1) / 2 {
            let ii = i * kappa;
            for j in 0..(n_otf + 1) / 2 {
                let jj = j * kappa;
                let k = i * n_otf + j;
                let kk = ii * n + jj;
                cov[k] = covariance[kk] * df * df;
            }
        }
        for i in 1..n_otf / 2 + 1 {
            let ii = n - i * kappa;
            for j in 1..n_otf / 2 + 1 {
                let jj = n - j * kappa;
                let k = (n_otf - i) * n_otf + n_otf - j;
                let kk = ii * n + jj;
                cov[k] = covariance[kk] * df * df;
            }
        }
        for i in 1..n_otf / 2 + 1 {
            let ii = n - i * kappa;
            for j in 0..(n_otf + 1) / 2 {
                let jj = j * kappa;
                let k = (n_otf - i) * n_otf + j;
                let kk = ii * n + jj;
                cov[k] = covariance[kk] * df * df;
            }
        }
        for i in 0..(n_otf + 1) / 2 {
            let ii = i * kappa;
            for j in 1..n_otf / 2 + 1 {
                let jj = n - j * kappa;
                let k = i * n_otf + n_otf - j;
                let kk = ii * n + jj;
                cov[k] = covariance[kk] * df * df;
            }
        }
        let var = cov[0];
        cov.into_iter().map(|cov| (cov - var).exp()).collect()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::atmosphere_transfer_function;

    #[test]
    fn strehl() {
        dbg!(optust::phase::variance(15e-2, 25.));
        let n = 5;
        let d = 0.1_f64;
        let mut ao = AdaptiveOpticsCorrection::new(n, d);
        ao.fitting_cutoff_frequency(0.5, 15e-2, 25.);
        println!("d: {}", 0.5 / ao.fitting_cutoff);
    }
    /*     #[test]
    fn variance() {
        dbg!(optust::phase::variance(15e-2, 25.));
        let n = 5;
        let d = 0.1_f64;
        let mut ao = AdaptiveOpticsCorrection::new(n, d);
        ao.variance_check(15e-2, 25.);
    }
    #[test]
    fn psd() {
        dbg!(optust::phase::variance(15e-2, 25.));
        let n = 9;
        let d = 2.5e-2_f64;
        let mut ao = AdaptiveOpticsCorrection::new(n, d);
        ao.variance_check(15e-2, 25.);
        // let psd = ao.psd_sampling(15e-2, 25.);
        let otf = ao.transfer_function(15e-2, 25.);
        otf.chunks(n).for_each(|o| println!("{:8.3?}", o));
    } */
}
