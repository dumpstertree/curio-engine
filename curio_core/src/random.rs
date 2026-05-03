use rand::Rng;

use crate::Vector3;

pub struct Random {}
impl Random {
    pub fn range_u8(a_inclusive: u8, b_inclusive: u8) -> u8 {
        if a_inclusive == b_inclusive {
            return a_inclusive;
        }

        let mut rng = rand::rng();

        if a_inclusive < b_inclusive {
            rng.random_range(a_inclusive..b_inclusive)
        } else {
            rng.random_range(b_inclusive..a_inclusive)
        }
    }
    pub fn range_float(a_inclusive: f32, b_inclusive: f32) -> f32 {
        if a_inclusive == b_inclusive {
            return a_inclusive;
        }

        let mut rng = rand::rng();

        if a_inclusive < b_inclusive {
            rng.random_range(a_inclusive..b_inclusive)
        } else {
            rng.random_range(b_inclusive..a_inclusive)
        }
    }
    pub fn range_int(a_inclusive: i32, b_inclusive: i32) -> i32 {
        if a_inclusive == b_inclusive {
            return a_inclusive;
        }

        let mut rng = rand::rng();
        if a_inclusive < b_inclusive {
            rng.random_range(a_inclusive..b_inclusive)
        } else {
            rng.random_range(b_inclusive..a_inclusive)
        }
    }
    pub fn random_bool() -> bool {
        let mut rng = rand::rng();
        rng.random_bool(0.5)
    }
    pub fn direction(use_x: bool, use_y: bool, use_z: bool) -> Vector3 {
        let mut v = Vector3::new(0.0, 0.0, 0.0);
        if use_x {
            v.x = Random::range_float(-1.0, 1.0);
        }
        if use_y {
            v.y = Random::range_float(-1.0, 1.0);
        }
        if use_z {
            v.z = Random::range_float(-1.0, 1.0);
        }

        v.normalize();
        v
    }
}
