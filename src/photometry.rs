#[derive(Debug)]
pub struct Photometry {
    pub wavelength: f64,
    zeropoint: f64,
    #[allow(dead_code)]
    spectral_bandwidth: f64,
}
impl Photometry {
    pub fn n_photon(&self, magnitude: f64) -> f64 {
        self.zeropoint * 10f64.powf(-0.4 * magnitude)
    }
}

impl From<&str> for Photometry {
    fn from(band: &str) -> Self {
        match band {
            "V" => Photometry {
                wavelength: 0.55e-6,
                zeropoint: 8.97e9,
                spectral_bandwidth: 0.09e-6,
            },
            "R" => Photometry {
                wavelength: 0.64e-6,
                zeropoint: 10.87e9,
                spectral_bandwidth: 0.15e-6,
            },
            "I" => Photometry {
                wavelength: 0.79e-6,
                zeropoint: 7.34e9,
                spectral_bandwidth: 0.15e-6,
            },
            "J" => Photometry {
                wavelength: 1.215e-6,
                zeropoint: 5.16e9,
                spectral_bandwidth: 0.26e-6,
            },
            "H" => Photometry {
                wavelength: 1.654e-6,
                zeropoint: 2.99e9,
                spectral_bandwidth: 0.29e-6,
            },
            "K" => Photometry {
                wavelength: 2.179e-6,
                zeropoint: 1.90e9,
                spectral_bandwidth: 0.41e-6,
            },
            _ => panic!("expected the photometric band: V, R, I, J, H or K, found {band}"),
        }
    }
}
