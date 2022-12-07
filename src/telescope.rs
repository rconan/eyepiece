use crate::Observer;
use image::{ImageResult, Rgb, RgbImage};

const N_PX: usize = 128;

#[derive(Debug)]
pub struct Telescope {
    diameter: f64,
    pupil_size: f64,
    pupil_sampling: usize,
    obscuration: Option<f64>,
}

impl Default for Telescope {
    fn default() -> Self {
        Self {
            diameter: 1f64,
            obscuration: Default::default(),
            pupil_size: 2f64,
            pupil_sampling: 2 * N_PX + 1,
        }
    }
}

pub struct TelescopeBuilder {
    diameter: f64,
    obscuration: Option<f64>,
}

impl Telescope {
    pub fn new(diameter: f64) -> TelescopeBuilder {
        TelescopeBuilder {
            diameter,
            obscuration: None,
        }
    }
    pub fn pupil(&self) -> Vec<u8> {
        let s = self.pupil_sampling;
        let s2 = s * s;
        let l = (s - 1) as f64;
        let r_outer = self.diameter * 0.5;
        let r_inner = self.obscuration.unwrap_or_default();
        let mut buffer = vec![0u8; s2];
        for i in 0..s {
            let x = (i as f64 / l - 0.5) * self.pupil_size;
            for j in 0..s {
                let y = (j as f64 / l - 0.5) * self.pupil_size;
                let r = x.hypot(y);
                if r >= r_inner && r <= r_outer {
                    let k = i * s + j;
                    buffer[k] = 1u8;
                }
            }
        }
        buffer
    }
}

impl TelescopeBuilder {
    /// Sets the diameter of the telescope central obscuration
    pub fn obscuration(mut self, obscuration: f64) -> Self {
        self.obscuration = Some(obscuration);
        self
    }
    /// Build the telescope
    pub fn build(self) -> Telescope {
        Telescope {
            diameter: self.diameter,
            obscuration: self.obscuration,
            pupil_size: 2. * self.diameter,
            pupil_sampling: 2 * N_PX + 1,
        }
    }
}

impl Observer for Telescope {
    fn diameter(&self) -> f64 {
        self.diameter
    }

    fn set_pupil_size(&mut self, size: f64) {
        self.pupil_size = dbg!(size);
        let mut s = (N_PX as f64 * self.pupil_size / self.diameter).ceil() as usize;
        s += (s + 1) % 2;
        self.pupil_sampling = s;
    }

    fn intensity(&self) -> Vec<f64> {
        todo!()
    }

    fn show_pupil(&self) -> ImageResult<()> {
        let n = self.pupil_sampling as u32;
        let mut img = RgbImage::new(n, n);
        img.pixels_mut()
            .zip(self.pupil().into_iter())
            .filter(|(px, pup)| *pup > 0)
            .for_each(|(px, _)| *px = Rgb([255, 255, 255]));
        img.save("telescope_pupil.png")
    }
}
