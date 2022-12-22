use crate::MastObject;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum GaiaPhotometry {
    V,
    R,
    I,
    J,
    H,
    K,
}
impl GaiaPhotometry {
    /// GAIA magnitude polynomial fit coefficients
    ///
    /// From [Table 5.7](https://gea.esac.esa.int/archive/documentation/GEDR3/Data_processing/chap_cu5pho/cu5pho_sec_photSystem/cu5pho_ssec_photRelations.html) of the EDR3 data guide
    pub fn polynomial(&self) -> Vec<f64> {
        match self {
            GaiaPhotometry::V => vec![-0.02704, 0.01424, -0.2156, 0.01426],
            GaiaPhotometry::R => vec![-0.02275, 0.3961, -0.1243, -0.01396, 0.003775],
            GaiaPhotometry::I => vec![0.01753, 0.76, -0.0991],
            GaiaPhotometry::J => vec![0.01798, 1.389, -0.09338],
            GaiaPhotometry::H => vec![-0.1048, 2.011, -0.1758],
            GaiaPhotometry::K => vec![-0.0981, 2.089, -0.1579],
        }
    }
    pub fn magnitude(&self, object: &MastObject) -> Option<f64> {
        let MastObject {
            gaimag,
            gaiabp,
            gaiarp,
            ..
        } = object;
        gaimag
            .as_ref()
            .zip(gaiabp.as_ref().zip(gaiarp.as_ref()))
            .map(|(g, (g_bp, g_rb))| {
                let bp_rb = g_bp - g_rb;
                let p = self.polynomial();
                let n = p.len();
                let dg = p[0]
                    + p.into_iter().rev().take(n - 1).fold(0f64, |mut a, p| {
                        a = (a + p) * bp_rb;
                        a
                    });
                g - dg
            })
    }
}
impl From<&str> for GaiaPhotometry {
    /// Astronomical photometric bands
    ///
    /// Converts the bands V, R, I, J, H and K into star [Photometry]
    fn from(band: &str) -> Self {
        match band {
            "V" => GaiaPhotometry::V,
            "R" => GaiaPhotometry::R,
            "I" => GaiaPhotometry::I,
            "J" => GaiaPhotometry::J,
            "H" => GaiaPhotometry::H,
            "K" => GaiaPhotometry::K,
            _ => panic!("expected the photometric band: V, R, I, J, H or K, found {band}"),
        }
    }
}
