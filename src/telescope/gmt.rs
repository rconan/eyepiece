use crate::Observer;
use geotrans::{Conic, Segment, SegmentTrait, Transform, M1};

pub struct Gmt;
impl Gmt {
    pub fn new() -> Self {
        Self
    }
}

impl Observer for Gmt {
    fn diameter(&self) -> f64 {
        25.5
    }

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        let m1 = Conic::m1();
        let h = m1.height(x.hypot(y));
        let pt = [x, y, 3.9 + h];
        let r_outer = 8.365 * 0.5;
        let r_inner = 3.6 * 0.5;
        for i in 1..=7 {
            let xyz = pt.fro(Segment::<M1>::new(i)).unwrap();
            let r = xyz[0].hypot(xyz[1]);

            if i == 7 && r < r_inner {
                continue;
            }
            if r <= r_outer {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn inside() {
        let gmt = Gmt::new();
        dbg!(gmt.inside_pupil(0f64, 0.5 * 25.4f64));
        dbg!(gmt.inside_pupil(0f64, 0.5 * 25.5f64));
    }

    #[test]
    fn geotrans() {
        let m1 = Conic::m1();
        let x = 0f64;
        let y = 25.5f64 * 0.5;
        let r = x.hypot(y);
        let h = m1.height(r);
        dbg!(h);
        let pt = [x, y, 3.9 + h];
        let xyz = pt.fro(Segment::<M1>::new(1)).unwrap();
        dbg!(xyz);
    }
}
