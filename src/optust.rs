use crate::bessel_knu;

const PI: f64 = std::f64::consts::PI;
const E: f64 = std::f64::consts::E;

fn ln_gamma(f64: f64) -> f64 {
    // Auxiliary variable when evaluating the `gamma_ln` function
    let gamma_r: f64 = 10.900511;

    // Polynomial coefficients for approximating the `gamma_ln` function
    let gamma_dk: &[f64] = &[
        2.48574089138753565546e-5,
        1.05142378581721974210,
        -3.45687097222016235469,
        4.51227709466894823700,
        -2.98285225323576655721,
        1.05639711577126713077,
        -1.95428773191645869583e-1,
        1.70970543404441224307e-2,
        -5.71926117404305781283e-4,
        4.63399473359905636708e-6,
        -2.71994908488607703910e-9,
    ];

    let x: f64 = f64;

    if x < 0.5 {
        let s = gamma_dk
            .iter()
            .enumerate()
            .skip(1)
            .fold(gamma_dk[0], |s, t| s + *t.1 / ((t.0 as u64) as f64 - x));

        PI.ln()
            - (PI * x).sin().ln()
            - s.ln()
            - (2.0 * (E / PI).powf(0.5)).ln()
            - (0.5 - x) * ((0.5 - x + gamma_r) / E).ln()
    } else {
        let s = gamma_dk
            .iter()
            .enumerate()
            .skip(1)
            .fold(gamma_dk[0], |s, t| {
                s + *t.1 / (x + (t.0 as u64) as f64 - 1.0)
            });

        s.ln()
            + (2.0 * (E / PI).powf(0.5)).ln()
            + (x - 0.5) * ((x - 0.5 + gamma_r) / std::f64::consts::E).ln()
    }
}
pub fn gamma(x: f64) -> f64 {
    ln_gamma(x).exp()
}

pub mod phase {
    use super::*;

    pub fn variance(r0: f64, big_l0: f64) -> f64 {
        let g_11o6: f64 = gamma(11. / 6.);
        let g_5o6: f64 = gamma(5. / 6.);
        let g_6o5: f64 = gamma(6. / 5.);
        let p56: f64 = (24. * g_6o5 / 5.).powf(5. / 6.);
        let pi83: f64 = PI.powf(8. / 3.);
        0.5 * g_11o6 * g_5o6 * p56 * (big_l0 / r0).powf(5. / 3.) / pi83
    }
    pub fn covariance(x: f64, r0: f64, big_l0: f64) -> f64 {
        if x == 0.0 {
            variance(r0, big_l0)
        } else {
            let r = x.abs();
            let g_11o6: f64 = gamma(11. / 6.);
            let g_6o5: f64 = gamma(6. / 5.);
            let p56: f64 = (24. * g_6o5 / 5.).powf(5. / 6.);
            let pi83: f64 = PI.powf(8. / 3.);
            let red = 2. * PI * r / big_l0;
            g_11o6
                * p56
                * (big_l0 / r0).powf(5. / 3.)
                * red.powf(5. / 6.)
                * bessel_knu::fun(5. / 6., red)
                / (pi83 * 2f64.powf(5. / 6.))
        }
    }
    #[allow(dead_code)]
    pub fn structure_function(x: f64, r0: f64, big_l0: f64) -> f64 {
        if x == 0.0 {
            0f64
        } else {
            2. * (variance(r0, big_l0) - covariance(x, r0, big_l0))
        }
    }
    pub fn transfer_function(x: f64, r0: f64, big_l0: f64) -> f64 {
        if x == 0.0 {
            1f64
        } else {
            (covariance(x, r0, big_l0) - variance(r0, big_l0)).exp()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gammaf() {
        println!("gamma: {}", gamma(11. / 6.));
    }
    #[test]
    fn variance() {
        println!("variance: {}", phase::variance(0.15, 25.));
    }
    #[test]
    fn covariance() {
        println!("covariance: {}", phase::covariance(100., 0.15, 25.));
    }
}
