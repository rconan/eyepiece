use std::marker::PhantomData;

use super::{AdaptiveOptics, DiffractionLimited, SeeingLimited};
use crate::{atmosphere_transfer_function, AdaptiveOpticsCorrection, SeeingBuilder, ZpDft};
use num_complex::Complex;

/// Observing configurations
pub struct Observing<Mode> {
    fft: Option<ZpDft>,
    ifft: Option<ZpDft>,
    otf: Option<Vec<Complex<f64>>>,
    seeing: Option<SeeingBuilder>,
    adaptive_optics: Option<AdaptiveOpticsCorrection>,
    mode: PhantomData<Mode>,
}
impl Observing<DiffractionLimited> {
    /// Diffraction limited observing mode
    pub fn diffraction_limited() -> Self {
        Self {
            fft: None,
            ifft: None,
            otf: None,
            seeing: None,
            adaptive_optics: None,
            mode: PhantomData,
        }
    }
}
impl Observing<SeeingLimited> {
    /// Seeing limited observing mode
    pub fn seeing_limited(seeing: Option<SeeingBuilder>) -> Self {
        Self {
            fft: None,
            ifft: None,
            otf: None,
            seeing,
            adaptive_optics: None,
            mode: PhantomData,
        }
    }
}
impl Observing<AdaptiveOptics> {
    /// Seeing limited observing mode
    pub fn adaptive_optics(seeing: Option<SeeingBuilder>) -> Self {
        Self {
            fft: None,
            ifft: None,
            otf: None,
            seeing,
            adaptive_optics: None,
            mode: PhantomData,
        }
    }
}
pub trait Intensity {
    fn init_fft(&mut self, n_dft: usize, pupil_resolution: f64);
    fn intensity(
        &mut self,
        pupil: Vec<Complex<f64>>,
        intensity_sampling: usize,
    ) -> Option<Vec<f64>>;
}
impl Intensity for Observing<DiffractionLimited> {
    fn intensity(
        &mut self,
        pupil: Vec<Complex<f64>>,
        intensity_sampling: usize,
    ) -> Option<Vec<f64>> {
        self.fft.as_mut().map(|zp_dft| {
            zp_dft
                .reset()
                .zero_padding(pupil)
                .process()
                .shift()
                .resize(intensity_sampling)
                .norm_sqr()
        })
    }

    fn init_fft(&mut self, n_dft: usize, _pupil_resolution: f64) {
        self.fft = Some(ZpDft::forward(n_dft));
    }
}
impl Intensity for Observing<SeeingLimited> {
    fn init_fft(&mut self, n_dft: usize, pupil_resolution: f64) {
        self.fft = Some(ZpDft::forward(n_dft));
        self.ifft = Some(ZpDft::inverse(n_dft));
        self.otf = match self.seeing {
            Some(SeeingBuilder {
                fried_parameter,
                outer_scale,
            }) => Some(
                atmosphere_transfer_function(fried_parameter, outer_scale, pupil_resolution, n_dft)
                    .into_iter()
                    .map(|o| Complex::new(o, 0f64))
                    .collect(),
            ),
            None => panic!("seeing is not declared"),
        }
    }
    fn intensity(
        &mut self,
        pupil: Vec<Complex<f64>>,
        intensity_sampling: usize,
    ) -> Option<Vec<f64>> {
        self.fft
            .as_mut()
            .zip(self.ifft.as_mut())
            .zip(self.otf.as_ref())
            .map(|((zp_dft, zp_idft), otf)| {
                zp_idft
                    .zero_padding(
                        zp_dft
                            .reset()
                            .zero_padding(pupil)
                            .process()
                            .norm_sqr()
                            .into_iter()
                            .map(|x| Complex::new(x, 0f64))
                            .collect::<Vec<Complex<f64>>>(),
                    )
                    .process()
                    .filter(otf.as_slice());
                zp_dft
                    .zero_padding(zp_idft.buffer())
                    .process()
                    .shift()
                    .resize(intensity_sampling)
                    .norm()
            })
    }
}
impl Intensity for Observing<AdaptiveOptics> {
    fn init_fft(&mut self, n_dft: usize, pupil_resolution: f64) {
        self.fft = Some(ZpDft::forward(n_dft));
        self.ifft = Some(ZpDft::inverse(n_dft));
        self.adaptive_optics = Some(AdaptiveOpticsCorrection::new(n_dft, pupil_resolution));
    }
    fn intensity(
        &mut self,
        pupil: Vec<Complex<f64>>,
        intensity_sampling: usize,
    ) -> Option<Vec<f64>> {
        self.fft
            .as_mut()
            .zip(self.ifft.as_mut())
            .zip(self.seeing)
            .zip(self.adaptive_optics.as_mut())
            .map(
                |(
                    (
                        (zp_dft, zp_idft),
                        SeeingBuilder {
                            fried_parameter,
                            outer_scale,
                        },
                    ),
                    aoc,
                )| {
                    let otf: Vec<_> = aoc
                        .transfer_function(fried_parameter, outer_scale)
                        .into_iter()
                        .map(|o| Complex::new(o, 0f64))
                        .collect();
                    zp_idft
                        .zero_padding(
                            zp_dft
                                .reset()
                                .zero_padding(pupil)
                                .process()
                                .norm_sqr()
                                .into_iter()
                                .map(|x| Complex::new(x, 0f64))
                                .collect::<Vec<Complex<f64>>>(),
                        )
                        .process()
                        .filter(otf.as_slice());
                    zp_dft
                        .zero_padding(zp_idft.buffer())
                        .process()
                        .shift()
                        .resize(intensity_sampling)
                        .norm()
                },
            )
    }
}
