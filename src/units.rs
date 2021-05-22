use std::f32::consts::LN_2;

/// Used to implement conversions to the Hertz struct
pub trait Units<T> {
    /// From hertz
    fn to_range(self, bottom: T, top: T) -> T;
    fn from_range(self, bottom: T, top: T) -> T;
    fn db_to_lin(self) -> T;
    fn lin_to_db(self) -> T;
    fn sign(self, b: T) -> T;
    fn bw_to_q(self, f0: T, fs: T) -> T;
}

impl Units<f32> for f32 {
    //Just a copy of the f64 version with units swapped
    fn to_range(self, bottom: f32, top: f32) -> f32 {
        self * (top - bottom) + bottom
    }
    fn from_range(self, bottom: f32, top: f32) -> f32 {
        (self - bottom) / (top - bottom)
    }
    fn db_to_lin(self) -> f32 {
        (10.0f32).powf(self * 0.05)
    }
    fn lin_to_db(self) -> f32 {
        self.max(0.0).log(10.0) * 20.0
    }
    fn sign(self, b: f32) -> f32 {
        if b < 0.0 {
            -self
        } else {
            self
        }
    }
    fn bw_to_q(self, _f0: f32, _fs: f32) -> f32 {
        1.0 / (2.0 * (LN_2 / 2.0 * self).sinh())
    }
}

pub fn map_to_freq(n: f32) -> f32 {
    //0-1 to freq
    let n = ((1000.0f32).powf(n) - 1.0) / (1000.0f32 - 1.0);
    n.to_range(20.0, 20000.0)
}

pub fn reverse_map_to_freq(n: f32) -> f32 {
    let n = n.from_range(20.0, 20000.0);
    ((1000.0f32 - 1.0) * n + 1.0).ln() / 1000.0f32.ln()
}
