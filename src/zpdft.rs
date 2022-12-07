use num_complex::Complex;
use rustfft::{num_traits::Zero, Fft, FftPlanner};
use std::sync::Arc;

type Cpx = Complex<f64>;

/// Zero-Padded Discrete Fourier Transform
///
/// Two-dimensions discrete fourier transfrom
pub struct ZpDft {
    zero_padded_buffer: Vec<Cpx>,
    scratch: Vec<Cpx>,
    len: i64,
    fft: Arc<dyn Fft<f64>>,
}
impl ZpDft {
    /// Forward Fourier transfrom
    pub fn forward(len: usize) -> Self {
        let mut planner = FftPlanner::new();
        Self {
            zero_padded_buffer: vec![Complex::zero(); len * len],
            scratch: vec![Complex::zero(); len],
            fft: planner.plan_fft_forward(len),
            len: len as i64,
        }
    }
    /// Inverse Fourier transfrom
    pub fn inverse(len: usize) -> Self {
        let mut planner = FftPlanner::new();
        Self {
            zero_padded_buffer: vec![Complex::zero(); len * len],
            scratch: vec![Complex::zero(); len],
            fft: planner.plan_fft_inverse(len),
            len: len as i64,
        }
    }
    // Zero-pads the FFT buffer
    fn zero_padding(&mut self, buffer: &[Cpx]) {
        let n2 = buffer.len() as i64;
        let n = (n2 as f64).sqrt() as i64;
        assert_eq!(n2, n * n, "DFT input is not a square array");
        self.zero_padded_buffer = vec![Complex::zero(); self.zero_padded_buffer.len()];
        for i in 0..n {
            let ii = (i - n / 2).rem_euclid(self.len);
            for j in 0..n {
                let jj = (j - n / 2).rem_euclid(self.len);
                let k = (i * n + j) as usize;
                let kk = (ii * self.len + jj) as usize;
                self.zero_padded_buffer[kk].re = buffer[k].re;
                self.zero_padded_buffer[kk].im = buffer[k].im;
            }
        }
    }
    // Shift zero frequency back to center
    fn shift(&mut self) {
        let mut buffer: Vec<Cpx> = vec![Complex::zero(); self.zero_padded_buffer.len()];
        for i in 0..self.len {
            let ii = (i + self.len / 2) % self.len;
            for j in 0..self.len {
                let jj = (j + self.len / 2) % self.len;
                let k = (i * self.len + j) as usize;
                let kk = (ii * self.len + jj) as usize;
                buffer[kk] = self.zero_padded_buffer[k];
            }
        }
        self.zero_padded_buffer.copy_from_slice(buffer.as_slice());
    }
    /// Compute the 2D Fourier transfrom
    pub fn process(&mut self, buffer: &[Cpx]) {
        self.zero_padding(buffer);
        self.fft.process_with_scratch(
            self.zero_padded_buffer.as_mut_slice(),
            self.scratch.as_mut_slice(),
        );
        self.zero_padded_buffer = (0..self.len as usize)
            .flat_map(|k| {
                self.zero_padded_buffer
                    .iter()
                    .skip(k)
                    .step_by(self.len as usize)
                    .cloned()
                    .collect::<Vec<Cpx>>()
            })
            .collect();
        self.fft.process_with_scratch(
            self.zero_padded_buffer.as_mut_slice(),
            self.scratch.as_mut_slice(),
        );
        self.shift();
    }
    /// FFT buffer real part
    pub fn real(&self) -> Vec<f64> {
        self.zero_padded_buffer.iter().map(|b| b.re).collect()
    }
    /// FFT buffer imaginary part
    pub fn imag(&self) -> Vec<f64> {
        self.zero_padded_buffer.iter().map(|b| b.im).collect()
    }
    /// FFT buffer norm
    pub fn norm(&self) -> Vec<f64> {
        self.zero_padded_buffer.iter().map(|b| b.norm()).collect()
    }
    /// FFT buffer norm squared
    pub fn norm_sqr(&self) -> Vec<f64> {
        self.zero_padded_buffer
            .iter()
            .map(|b| b.norm_sqr())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_padding() {
        let n = 9;
        let n_fft = 12;
        let mut zp_dft = ZpDft::forward(n_fft);
        let buffer: Vec<Cpx> = vec![Complex::new(1f64, 0f64); n * n];
        zp_dft.zero_padding(buffer.as_slice());
        println!("REAL");
        zp_dft.real().chunks(n_fft).for_each(|c| println!("{c:?}"));
        println!("IMAG");
        zp_dft.imag().chunks(n_fft).for_each(|c| println!("{c:?}"));
        zp_dft.process(buffer.as_slice());
        println!("REAL");
        zp_dft
            .real()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        println!("IMAG");
        zp_dft
            .imag()
            .chunks(n_fft)
            .for_each(|c| println!("{c:.2?}"));
    }
}
