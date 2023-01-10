use super::*;

impl<T, Mode> Field<T, Mode>
where
    T: Observer,
    Mode: Send,
    Observing<Mode>: Intensity,
{
    /// Computes field-of-view intensity map
    pub fn intensity(&mut self, bar: Option<ProgressBar>) -> Vec<f64> {
        // Telescope Nyquist-Shannon sampling criteria
        // let nyquist = 0.5 * self.photometry.wavelength / self.observer.diameter();
        // Image resolution to sampling criteria ratio
        let b = self
            .pixel_scale
            .to_nyquist_clamped_ratio(&self.observer, &self.photometry);
        // Intensity sampling (oversampled wrt. image by factor b>=1)
        let intensity_sampling = (b * self.field_of_view.to_pixelscale_ratio(self)).ceil() as usize;
        // Pupil size according to intensity angular resolution
        let pupil_size = b * self.photometry.wavelength / self.resolution();
        // FFT sampling based on pupil spatial resolution
        let mut n_dft = (pupil_size / self.observer.resolution()).ceil() as usize;
        // Match parity of FFT and intensity sampling if the latter is larger
        if intensity_sampling > n_dft && intensity_sampling % 2 != n_dft % 2 {
            n_dft += 1;
        }
        log::debug!(
            r"
 . Image sampling: {intensity_sampling}:{b}
 . Pupil size    : {pupil_size:.3}m
 . DFT sampling  : {n_dft}
         "
        );

        // star image stacking buffer
        let mut buffer = vec![0f64; intensity_sampling.pow(2)];
        let n = intensity_sampling as i32;
        let alpha = self.resolution() / b;
        let mut rng = rand::thread_rng();
        for star in self.objects.iter() {
            bar.as_ref().map(|b| b.inc(1));
            // todo: check if star is within FOV (rejection criteria?)
            if !star.inside_box(self.field_of_view() + self.resolution() * 2.) {
                continue;
            }
            let n_photon = self.flux.unwrap_or(
                self.photometry.n_photon(star.magnitude)
                    * self.exposure
                    * self.observer.resolution().powi(2), //  * self.observer.area() ,
            );
            // star coordinates
            let (x, y) = star.coordinates;
            // integer part
            let x0 = -(y / alpha).round();
            let y0 = (x / alpha).round();
            // fractional part
            let fr_x0 = -y.to_radians() - x0 * alpha;
            let fr_y0 = x.to_radians() - y0 * alpha;
            // image fractional translation by Fourier interpolation
            let shift = if intensity_sampling % 2 == 0 {
                Some((
                    0.5 / pupil_size + fr_x0 / self.photometry.wavelength,
                    0.5 / pupil_size + fr_y0 / self.photometry.wavelength,
                ))
            } else {
                Some((
                    fr_x0 / self.photometry.wavelength,
                    fr_y0 / self.photometry.wavelength,
                ))
            };
            // star intensity map
            // Zero-padding discrete Fourier transform
            self.observing_mode
                .init_fft(n_dft, self.observer.resolution());
            let mut pupil = self.observer.pupil(shift);
            pupil.iter_mut().for_each(|p| *p *= n_photon.sqrt());
            let mut intensity = self
                .observing_mode
                .intensity(pupil, intensity_sampling, star)
                .unwrap();
            // intensity set to # of photon & Poisson noise
            // log::debug!("Image flux: {n_photon}");
            if self.poisson_noise {
                intensity.iter_mut().for_each(|i| {
                    if *i == 0f64 {
                        *i = 0f64;
                    } else {
                        let poi = Poisson::new(*i).unwrap();
                        *i = poi.sample(&mut rng)
                    }
                })
            };
            // shift and add star images
            shift_and_add(buffer.as_mut_slice(), x0, y0, n, intensity);
        }
        bar.as_ref().map(|b| b.finish());

        let m = b as usize;
        if m == 1 {
            return buffer;
        }
        // binning
        binning(intensity_sampling, m, buffer)
    }
}
