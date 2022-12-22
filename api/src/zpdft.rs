use num_complex::Complex;
use rustfft::{num_traits::Zero, Fft, FftPlanner};
use std::{fmt::Debug, mem, sync::Arc};

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
impl Debug for ZpDft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZpDft")
            .field("zero_padded_buffer", &self.zero_padded_buffer)
            .field("scratch", &self.scratch)
            .field("len", &self.len)
            .field("fft", &())
            .finish()
    }
}
impl ZpDft {
    /// Forward Fourier transfrom
    pub fn forward(len: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(len);
        Self {
            zero_padded_buffer: vec![Complex::zero(); len * len],
            scratch: vec![Complex::zero(); fft.get_inplace_scratch_len()],
            fft,
            len: len as i64,
        }
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn reset(&mut self) -> &mut Self {
        self.zero_padded_buffer = vec![Complex::zero(); (self.len * self.len) as usize];
        self
    }
    pub fn into_buffer(self) -> Vec<Cpx> {
        self.zero_padded_buffer
    }
    pub fn buffer(&self) -> Vec<Cpx> {
        self.zero_padded_buffer.clone()
    }
    /// Inverse Fourier transfrom
    pub fn inverse(len: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_inverse(len);
        Self {
            zero_padded_buffer: vec![Complex::zero(); len * len],
            scratch: vec![Complex::zero(); fft.get_inplace_scratch_len()],
            fft,
            len: len as i64,
        }
    }
    /// Zero-pads the FFT buffer
    pub fn zero_padding(&mut self, mut buffer: Vec<Cpx>) -> &mut Self {
        let n2 = buffer.len() as i64;
        let n = (n2 as f64).sqrt() as i64;
        assert_eq!(n2, n * n, "DFT input is not a square array");
        if n == self.len {
            self.zero_padded_buffer = mem::take(&mut buffer);
        } else {
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
        self
    }
    /// Shift zero frequency back to center
    pub fn shift(&mut self) -> &mut Self {
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
        self
    }
    pub fn filter(&mut self, kernel: &[Cpx]) -> &mut Self {
        self.zero_padded_buffer
            .iter_mut()
            .zip(kernel)
            .for_each(|(b, k)| *b *= k);
        self
    }
    /// Compute the 2D Fourier transfrom
    pub fn process(&mut self) -> &mut Self {
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
        self.zero_padded_buffer
            .iter_mut()
            .for_each(|buffer| *buffer /= self.len as f64);
        self
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
    pub fn crop(&mut self, new_len: usize) -> &mut Self {
        let ij0 = self.len as usize / 2 - new_len / 2;
        let buffer = self.zero_padded_buffer.clone();
        self.zero_padded_buffer = vec![Complex::zero(); new_len * new_len];
        for i in 0..new_len {
            for j in 0..new_len {
                let k = i * new_len + j;
                let kk = (i + ij0) * self.len as usize + j + ij0;
                self.zero_padded_buffer[k] = buffer[kk];
            }
        }
        self
    }
    pub fn resize(&mut self, new_len: usize) -> &mut Self {
        let old_len = self.len as usize;
        if old_len > new_len {
            self.crop(new_len);
        } else {
            let ij0 = (new_len - old_len) / 2;
            let buffer = self.zero_padded_buffer.clone();
            self.zero_padded_buffer = vec![Complex::zero(); new_len * new_len];
            for i in 0..old_len {
                for j in 0..old_len {
                    let k = i * old_len + j;
                    let kk = (i + ij0) * new_len + j + ij0;
                    self.zero_padded_buffer[kk] = buffer[k];
                }
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_padding() {
        let n = 8;
        let n_fft = 16;
        let mut zp_dft = ZpDft::forward(n_fft);
        let buffer: Vec<Cpx> = vec![Complex::new(1f64, 0f64); n * n];
        zp_dft.zero_padding(buffer);
        println!("REAL");
        zp_dft.real().chunks(n_fft).for_each(|c| println!("{c:?}"));
        println!("IMAG");
        zp_dft.imag().chunks(n_fft).for_each(|c| println!("{c:?}"));
        zp_dft.process();
        println!("REAL");
        zp_dft
            .real()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        println!("IMAG");
        zp_dft
            .imag()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        println!("NORM");
        zp_dft
            .norm()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
    }

    #[test]
    fn cropping() {
        let n = 8;
        let n_fft = 16;
        let mut zp_dft = ZpDft::forward(n_fft);
        let buffer: Vec<Cpx> = vec![Complex::new(1f64, 0f64); n * n];
        zp_dft.zero_padding(buffer);
        println!("REAL");
        zp_dft.real().chunks(n_fft).for_each(|c| println!("{c:?}"));
        println!("IMAG");
        zp_dft.imag().chunks(n_fft).for_each(|c| println!("{c:?}"));
        zp_dft.process();
        println!("REAL");
        zp_dft
            .real()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        println!("IMAG");
        zp_dft
            .imag()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        println!("NORM");
        zp_dft
            .norm()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        zp_dft.crop(7);
        println!("NORM");
        zp_dft.norm().chunks(7).for_each(|c| println!("{c:+7.2?}"));
    }

    #[test]
    fn resizing() {
        let n = 7;
        let n_fft = 12;
        let mut zp_dft = ZpDft::forward(n_fft);
        let buffer: Vec<Cpx> = vec![Complex::new(1f64, 0f64); n * n];
        zp_dft.zero_padding(buffer);
        println!("REAL");
        zp_dft.real().chunks(n_fft).for_each(|c| println!("{c:?}"));
        println!("IMAG");
        zp_dft.imag().chunks(n_fft).for_each(|c| println!("{c:?}"));
        zp_dft.process();
        println!("REAL");
        zp_dft
            .real()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        println!("IMAG");
        zp_dft
            .imag()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        println!("NORM");
        zp_dft
            .norm()
            .chunks(n_fft)
            .for_each(|c| println!("{c:+7.2?}"));
        let new_len = 18;
        zp_dft.resize(new_len);
        println!("NORM");
        zp_dft
            .norm()
            .chunks(new_len)
            .for_each(|c| println!("{c:+7.2?}"));
    }
}
