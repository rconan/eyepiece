const G1_DAT: [f64; 14] = [
    -1.14516408366268311786898152867,
    0.00636085311347084238122955495,
    0.00186245193007206848934643657,
    0.000152833085873453507081227824,
    0.000017017464011802038795324732,
    -6.4597502923347254354668326451e-07,
    -5.1819848432519380894104312968e-08,
    4.5189092894858183051123180797e-10,
    3.2433227371020873043666259180e-11,
    6.8309434024947522875432400828e-13,
    2.8353502755172101513119628130e-14,
    -7.9883905769323592875638087541e-16,
    -3.3726677300771949833341213457e-17,
    -3.6586334809210520744054437104e-20,
];

const G2_DAT: [f64; 15] = [
    1.882645524949671835019616975350,
    -0.077490658396167518329547945212,
    -0.018256714847324929419579340950,
    0.0006338030209074895795923971731,
    0.0000762290543508729021194461175,
    -9.5501647561720443519853993526e-07,
    -8.8927268107886351912431512955e-08,
    -1.9521334772319613740511880132e-09,
    -9.4003052735885162111769579771e-11,
    4.6875133849532393179290879101e-12,
    2.2658535746925759582447545145e-13,
    -1.1725509698488015111878735251e-15,
    -7.0441338200245222530843155877e-17,
    -2.4377878310107693650659740228e-18,
    -7.5225243218253901727164675011e-20,
];
const M_LN10: f64 = std::f64::consts::LN_10;

struct ChebSeries {
    /// coefficients
    c: Vec<f64>,
    /// order of expansion
    order: i32,
    ///lower interval point
    a: f64,
    ///upper interval point
    b: f64,
    ///single precision order
    #[allow(dead_code)]
    order_sp: i32,
}

fn cheb_eval_e(cs: &ChebSeries, x: f64) -> (f64, f64) {
    let y = (2.0 * x - cs.a - cs.b) / (cs.b - cs.a);
    let y2 = 2.0 * y;
    let mut e = 0f64;
    let mut dd = 0f64;
    let mut d = 0f64;
    for j in (1..=cs.order as usize).rev() {
        let temp = d;
        d = y2 * d - dd + cs.c[j];
        e += (y2 * temp).abs() + (dd).abs() + cs.c[j].abs();
        dd = temp;
    }
    {
        let temp = d;
        d = y * d - dd + 0.5 * cs.c[0];
        e += (y * temp).abs() + (dd).abs() + 0.5 * (cs.c[0]).abs();
    }
    (d, std::f64::EPSILON * e + cs.c[cs.order as usize].abs())
}
fn temme_gamma(nu: f64) -> (f64, f64, f64, f64) // double * g_1pnu, double * g_1mnu, double * g1, double * g2
{
    let g1_cs: ChebSeries = ChebSeries {
        c: Vec::from(G1_DAT),
        order: 13,
        a: -1.,
        b: 1.,
        order_sp: 7,
    };
    let g2_cs: ChebSeries = ChebSeries {
        c: Vec::from(G2_DAT),
        order: 14,
        a: -1.,
        b: 1.,
        order_sp: 8,
    };
    let anu = nu.abs(); // functions are even
    let x = 4.0 * anu - 1.0;
    let (r_g1, _) = cheb_eval_e(&g1_cs, x);
    let (r_g2, _) = cheb_eval_e(&g2_cs, x);
    let g_1mnu = 1.0 / (r_g2 + nu * r_g1);
    let g_1pnu = 1.0 / (r_g2 - nu * r_g1);
    (g_1pnu, g_1mnu, r_g1, r_g2)
}
fn k_scaled_temme(nu: f64, x: f64) -> (f64, f64, f64) //  double * K_nu, double * K_nup1, double * Kp_nu)
{
    let max_iter = 15000;

    let half_x = 0.5 * x;
    let ln_half_x = half_x.ln();
    let half_x_nu = (nu * ln_half_x).exp();
    let pi_nu = std::f64::consts::PI * nu;
    let sigma = -nu * ln_half_x;
    let sinrat = if pi_nu.abs() < std::f64::EPSILON {
        1.0
    } else {
        pi_nu / pi_nu.sin()
    };
    let sinhrat = if sigma.abs() < std::f64::EPSILON {
        1.0
    } else {
        sigma.sinh() / sigma
    };
    let ex = x.exp();

    let (g_1pnu, g_1mnu, g1, g2) = temme_gamma(nu);

    let mut fk = sinrat * (sigma.cosh() * g1 - sinhrat * ln_half_x * g2);
    let mut pk = 0.5 / half_x_nu * g_1pnu;
    let mut qk = 0.5 * half_x_nu * g_1mnu;
    let mut hk = pk;
    let mut ck = 1.0;
    let mut sum0 = fk;
    let mut sum1 = hk;
    let mut k_usize = 0usize;
    while k_usize < max_iter {
        k_usize += 1;
        let k = k_usize as f64;
        fk = (k * fk + pk + qk) / (k * k - nu * nu);
        ck *= half_x * half_x / k;
        pk /= k - nu;
        qk /= k + nu;
        hk = -k * fk + pk;
        let del0 = ck * fk;
        let del1 = ck * hk;
        sum0 += del0;
        sum1 += del1;
        if del0.abs() < 0.5 * sum0.abs() * std::f64::EPSILON {
            break;
        };
    }

    let k_nu = sum0 * ex;
    let k_nup1 = sum1 * 2.0 / x * ex;
    let kp_nu = -k_nup1 + nu / x * k_nu;

    //stat_iter = ( k == max_iter ? GSL_EMAXITER : GSL_SUCCESS );
    //return GSL_ERROR_SELECT_2(stat_iter, stat_g);
    (k_nu, k_nup1, kp_nu)
}
fn k_scaled_steed_temme_cf2(nu: f64, x: f64) -> (f64, f64, f64) // double * K_nu, double * K_nup1, double * Kp_nu)
{
    let maxiter = 10000;

    //  let i = 1;
    let mut bi = 2.0 * (1.0 + x);
    let mut di = 1.0 / bi;
    let mut delhi = di;
    let mut hi = di;

    let mut qi = 0.0;
    let mut qip1 = 1.0;

    let mut ai = -(0.25 - nu * nu);
    let a1 = ai;
    let mut ci = -ai;
    let mut bqi = -ai;

    let mut s = 1.0 + bqi * delhi;

    for i in 2..=maxiter {
        ai -= 2.0 * (i - 1) as f64;
        ci = -ai * ci / i as f64;
        let tmp = (qi - bi * qip1) / ai;
        qi = qip1;
        qip1 = tmp;
        bqi += ci * qip1;
        bi += 2.0;
        di = 1.0 / (bi + ai * di);
        delhi = (bi * di - 1.0) * delhi;
        hi += delhi;
        let dels = bqi * delhi;
        s += dels;
        if (dels / s).abs() < std::f64::EPSILON {
            break;
        };
    }

    hi *= -a1;

    let k_nu = (std::f64::consts::PI / (2.0 * x)).sqrt() / s;
    let k_nup1 = k_nu * (nu + x + 0.5 - hi) / x;
    let kp_nu = -k_nup1 + nu / x * k_nu;
    /*
    if(i == maxiter)
      GSL_ERROR ("error", GSL_EMAXITER);
    else
      return GSL_SUCCESS;
       */
    (k_nu, k_nup1, kp_nu)
}
pub fn fun(nu: f64, x: f64) -> f64 {
    let bn = (nu + 0.5) as i32;
    let mu = nu - bn as f64; // -1/2 <= mu <= 1/2 */
    //let n = 0;
    //let mut e10 = 0f64;

    let (k_mu, k_mup1, _) = if x < 2.0 {
        k_scaled_temme(mu, x)
    } else {
        k_scaled_steed_temme_cf2(mu, x)
    };

    // recurse forward to obtain K_num1, K_nu */
    let mut k_nu = k_mu;
    let mut k_nup1 = k_mup1;

    for n in 0..bn {
        let mut k_num1 = k_nu;
        k_nu = k_nup1;
        if k_nu.abs() > std::f64::MAX.sqrt() {
            let p = k_nu.abs().ln().floor() / M_LN10;
            let factor = 10f64.powf(p);
            k_num1 /= factor;
            k_nu /= factor;
            //e10 += p;
        };
        k_nup1 = 2.0 * (mu + n as f64 + 1.) / x * k_nu + k_num1;
    }
    k_nu * (-x).exp()
}
#[cfg(test)]
mod tests {
    use super::fun;
    #[test]
    fn it_works() {
        let x = 0.01;
        println!("Knu(5/6,{:.0}): {}",x,fun(5./6.,x));
    }
}
