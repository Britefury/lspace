use std::ops::{Mul};

use geom::vector2::Vector2;
use geom::point2::Point2;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AffineXform2 {
    pub v: [[f64; 3]; 2],
}

impl AffineXform2 {
    pub fn new(a: f64, b: f64, x: f64, c: f64, d: f64, y: f64) -> AffineXform2 {
        return AffineXform2{v: [[a, b, x], [c, d, y]]};
    }

    pub fn identity() -> AffineXform2 {
        return AffineXform2{v: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]};
    }

    pub fn translate(t: Vector2) -> AffineXform2 {
        return AffineXform2{v: [[1.0, 0.0, t.x], [0.0, 1.0, t.y]]};
    }

    pub fn uniform_scale(s: f64) -> AffineXform2 {
        return AffineXform2{v: [[s, 0.0, 0.0], [0.0, s, 0.0]]};
    }

    pub fn scale(s: Vector2) -> AffineXform2 {
        return AffineXform2{v: [[s.x, 0.0, 0.0], [0.0, s.y, 0.0]]};
    }

    pub fn rotate(theta_radians: f64) -> AffineXform2 {
        let s = theta_radians.sin();
        let c = theta_radians.cos();
        return AffineXform2{v: [[c, -s, 0.0], [s, c, 0.0]]};
    }


    pub fn determinant(&self) -> f64 {
        return self.v[0][0] * self.v[1][1] - self.v[0][1] * self.v[1][0];
    }

    pub fn inverse(&self) -> AffineXform2 {
        // Compute reciprocal of determinant
        let rd = 1.0 / self.determinant();
        // Compute inverse of 2x2 sub-matrix
        let a = self.v[1][1] * rd;
        let b = self.v[0][1] * -rd;
        let c = self.v[1][0] * -rd;
        let d = self.v[0][0] * rd;
        // Compute translation part
        let x = -self.v[0][2] * a - self.v[1][2] * b;
        let y = -self.v[0][2] * c - self.v[1][2] * d;
        return AffineXform2{v: [[a, b, x], [c, d, y]]};
    }
}

// MULTIPLY BY Vector2; apply scale but not translation

impl Mul<Vector2> for AffineXform2 {
    type Output = Vector2;

    fn mul(self, v: Vector2) -> Vector2 {
        return Vector2::new(v.x * self.v[0][0] + v.y * self.v[0][1],
                            v.x * self.v[1][0] + v.y * self.v[1][1]);
    }
}

impl <'a> Mul<Vector2> for &'a AffineXform2 {
    type Output = Vector2;

    fn mul(self, v: Vector2) -> Vector2 {
        return Vector2::new(v.x * self.v[0][0] + v.y * self.v[0][1],
                            v.x * self.v[1][0] + v.y * self.v[1][1]);
    }
}

impl <'a> Mul<&'a Vector2> for AffineXform2 {
    type Output = Vector2;

    fn mul(self, v: &'a Vector2) -> Vector2 {
        return Vector2::new(v.x * self.v[0][0] + v.y * self.v[0][1],
                            v.x * self.v[1][0] + v.y * self.v[1][1]);
    }
}

impl <'a, 'b> Mul<&'b Vector2> for &'a AffineXform2 {
    type Output = Vector2;

    fn mul(self, v: &'b Vector2) -> Vector2 {
        return Vector2::new(v.x * self.v[0][0] + v.y * self.v[0][1],
                            v.x * self.v[1][0] + v.y * self.v[1][1]);
    }
}

// MULTIPLY BY Point2; apply scale and translation

impl Mul<Point2> for AffineXform2 {
    type Output = Point2;

    fn mul(self, p: Point2) -> Point2 {
        return Point2::new(p.x * self.v[0][0] + p.y * self.v[0][1] + self.v[0][2],
                           p.x * self.v[1][0] + p.y * self.v[1][1] + self.v[1][2]);
    }
}

impl <'a> Mul<Point2> for &'a AffineXform2 {
    type Output = Point2;

    fn mul(self, p: Point2) -> Point2 {
        return Point2::new(p.x * self.v[0][0] + p.y * self.v[0][1] + self.v[0][2],
                           p.x * self.v[1][0] + p.y * self.v[1][1] + self.v[1][2]);
    }
}

impl <'a> Mul<&'a Point2> for AffineXform2 {
    type Output = Point2;

    fn mul(self, p: &'a Point2) -> Point2 {
        return Point2::new(p.x * self.v[0][0] + p.y * self.v[0][1] + self.v[0][2],
                           p.x * self.v[1][0] + p.y * self.v[1][1] + self.v[1][2]);
    }
}

impl <'a, 'b> Mul<&'b Point2> for &'a AffineXform2 {
    type Output = Point2;

    fn mul(self, p: &'b Point2) -> Point2 {
        return Point2::new(p.x * self.v[0][0] + p.y * self.v[0][1] + self.v[0][2],
                           p.x * self.v[1][0] + p.y * self.v[1][1] + self.v[1][2]);
    }
}

// MULTIPLY BY AffineXform2; matrix multiplication

impl Mul<AffineXform2> for AffineXform2 {
    type Output = AffineXform2;

    fn mul(self, x: AffineXform2) -> AffineXform2 {
        return AffineXform2{v: [
                [
                    self.v[0][0] * x.v[0][0] + self.v[0][1] * x.v[1][0],
                    self.v[0][0] * x.v[0][1] + self.v[0][1] * x.v[1][1],
                    self.v[0][0] * x.v[0][2] + self.v[0][1] * x.v[1][2] + self.v[0][2],
                ],
                [
                    self.v[1][0] * x.v[0][0] + self.v[1][1] * x.v[1][0],
                    self.v[1][0] * x.v[0][1] + self.v[1][1] * x.v[1][1],
                    self.v[1][0] * x.v[0][2] + self.v[1][1] * x.v[1][2] + self.v[1][2],
                ],
            ]};
    }
}

impl <'a> Mul<AffineXform2> for &'a AffineXform2 {
    type Output = AffineXform2;

    fn mul(self, x: AffineXform2) -> AffineXform2 {
        return AffineXform2{v: [
                [
                    self.v[0][0] * x.v[0][0] + self.v[0][1] * x.v[1][0],
                    self.v[0][0] * x.v[0][1] + self.v[0][1] * x.v[1][1],
                    self.v[0][0] * x.v[0][2] + self.v[0][1] * x.v[1][2] + self.v[0][2],
                ],
                [
                    self.v[1][0] * x.v[0][0] + self.v[1][1] * x.v[1][0],
                    self.v[1][0] * x.v[0][1] + self.v[1][1] * x.v[1][1],
                    self.v[1][0] * x.v[0][2] + self.v[1][1] * x.v[1][2] + self.v[1][2],
                ],
            ]};
    }
}

impl <'a> Mul<&'a AffineXform2> for AffineXform2 {
    type Output = AffineXform2;

    fn mul(self, x: &'a AffineXform2) -> AffineXform2 {
        return AffineXform2{v: [
                [
                    self.v[0][0] * x.v[0][0] + self.v[0][1] * x.v[1][0],
                    self.v[0][0] * x.v[0][1] + self.v[0][1] * x.v[1][1],
                    self.v[0][0] * x.v[0][2] + self.v[0][1] * x.v[1][2] + self.v[0][2],
                ],
                [
                    self.v[1][0] * x.v[0][0] + self.v[1][1] * x.v[1][0],
                    self.v[1][0] * x.v[0][1] + self.v[1][1] * x.v[1][1],
                    self.v[1][0] * x.v[0][2] + self.v[1][1] * x.v[1][2] + self.v[1][2],
                ],
            ]};
    }
}

impl <'a, 'b> Mul<&'b AffineXform2> for &'a AffineXform2 {
    type Output = AffineXform2;

    fn mul(self, x: &'b AffineXform2) -> AffineXform2 {
        return AffineXform2{v: [
                [
                    self.v[0][0] * x.v[0][0] + self.v[0][1] * x.v[1][0],
                    self.v[0][0] * x.v[0][1] + self.v[0][1] * x.v[1][1],
                    self.v[0][0] * x.v[0][2] + self.v[0][1] * x.v[1][2] + self.v[0][2],
                ],
                [
                    self.v[1][0] * x.v[0][0] + self.v[1][1] * x.v[1][0],
                    self.v[1][0] * x.v[0][1] + self.v[1][1] * x.v[1][1],
                    self.v[1][0] * x.v[0][2] + self.v[1][1] * x.v[1][2] + self.v[1][2],
                ],
            ]};
    }
}


//
// TESTS
//

#[cfg(test)]
mod tests {
    extern crate test;

    use std::mem;
    use super::*;
    use geom::vector2::Vector2;
    use geom::point2::Point2;

    const ALMOST_EQ_EPSILON: f64 = 1.0e-6;

    macro_rules! assert_almost_eq {
        ($x:expr, $y:expr) => (
            if ($x).abs_sub($y) > ALMOST_EQ_EPSILON {
                panic!("assert_almost_eq failed: {} !=~ {}", $x, $y);
            }
        );
    }

    #[test]
    fn test_constructor() {
        assert_eq!(AffineXform2::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).v[0][0], 1.0);
        assert_eq!(AffineXform2::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).v[0][1], 2.0);
        assert_eq!(AffineXform2::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).v[0][2], 3.0);
        assert_eq!(AffineXform2::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).v[1][0], 4.0);
        assert_eq!(AffineXform2::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).v[1][1], 5.0);
        assert_eq!(AffineXform2::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).v[1][2], 6.0);
    }

    #[test]
    fn test_identity() {
        let a = AffineXform2::identity();
        assert_eq!(a.v[0][0], 1.0);
        assert_eq!(a.v[0][1], 0.0);
        assert_eq!(a.v[0][2], 0.0);
        assert_eq!(a.v[1][0], 0.0);
        assert_eq!(a.v[1][1], 1.0);
        assert_eq!(a.v[1][2], 0.0);
    }

    #[test]
    fn test_translate() {
        let a = AffineXform2::translate(Vector2::new(3.0, 4.0));
        assert_eq!(a.v[0][0], 1.0);
        assert_eq!(a.v[0][1], 0.0);
        assert_eq!(a.v[0][2], 3.0);
        assert_eq!(a.v[1][0], 0.0);
        assert_eq!(a.v[1][1], 1.0);
        assert_eq!(a.v[1][2], 4.0);

        assert_eq!(a * Vector2::new(10.0, 20.0), Vector2::new(10.0, 20.0));
        assert_eq!(a * Point2::new(10.0, 20.0), Point2::new(13.0, 24.0));
    }

    #[test]
    fn test_uniform_scale() {
        let a = AffineXform2::uniform_scale(2.0);
        assert_eq!(a.v[0][0], 2.0);
        assert_eq!(a.v[0][1], 0.0);
        assert_eq!(a.v[0][2], 0.0);
        assert_eq!(a.v[1][0], 0.0);
        assert_eq!(a.v[1][1], 2.0);
        assert_eq!(a.v[1][2], 0.0);

        assert_eq!(a * Vector2::new(10.0, 20.0), Vector2::new(20.0, 40.0));
        assert_eq!(a * Point2::new(10.0, 20.0), Point2::new(20.0, 40.0));
    }

    #[test]
    fn test_scale() {
        let a = AffineXform2::scale(Vector2::new(3.0, 4.0));
        assert_eq!(a.v[0][0], 3.0);
        assert_eq!(a.v[0][1], 0.0);
        assert_eq!(a.v[0][2], 0.0);
        assert_eq!(a.v[1][0], 0.0);
        assert_eq!(a.v[1][1], 4.0);
        assert_eq!(a.v[1][2], 0.0);

        assert_eq!(a * Vector2::new(10.0, 20.0), Vector2::new(30.0, 80.0));
        assert_eq!(a * Point2::new(10.0, 20.0), Point2::new(30.0, 80.0));
    }

    #[test]
    fn test_rotate() {
        let a = AffineXform2::rotate((90.0 as f64).to_radians());
        assert_almost_eq!(a.v[0][0], 0.0);
        assert_almost_eq!(a.v[0][1], -1.0);
        assert_almost_eq!(a.v[0][2], 0.0);
        assert_almost_eq!(a.v[1][0], 1.0);
        assert_almost_eq!(a.v[1][1], 0.0);
        assert_almost_eq!(a.v[1][2], 0.0);

        assert_almost_eq!((a * Vector2::new(10.0, 20.0)).x, -20.0);
        assert_almost_eq!((a * Vector2::new(10.0, 20.0)).y, 10.0);
        assert_almost_eq!((a * Point2::new(10.0, 20.0)).x, -20.0);
        assert_almost_eq!((a * Point2::new(10.0, 20.0)).y, 10.0);
    }

    #[test]
    fn test_inverse() {
        {
            let a = AffineXform2::identity();
            let b = a.inverse();

            assert_almost_eq!(b.v[0][0], 1.0);
            assert_almost_eq!(b.v[0][1], 0.0);
            assert_almost_eq!(b.v[0][2], 0.0);
            assert_almost_eq!(b.v[1][0], 0.0);
            assert_almost_eq!(b.v[1][1], 1.0);
            assert_almost_eq!(b.v[1][2], 0.0);
        }

        {
            let a = AffineXform2::translate(Vector2::new(2.0, 4.0));
            let b = a.inverse();

            assert_almost_eq!(b.v[0][0], 1.0);
            assert_almost_eq!(b.v[0][1], 0.0);
            assert_almost_eq!(b.v[0][2], -2.0);
            assert_almost_eq!(b.v[1][0], 0.0);
            assert_almost_eq!(b.v[1][1], 1.0);
            assert_almost_eq!(b.v[1][2], -4.0);
        }

        {
            let a = AffineXform2::uniform_scale(4.0);
            let b = a.inverse();

            assert_almost_eq!(b.v[0][0], 0.25);
            assert_almost_eq!(b.v[0][1], 0.0);
            assert_almost_eq!(b.v[0][2], 0.0);
            assert_almost_eq!(b.v[1][0], 0.0);
            assert_almost_eq!(b.v[1][1], 0.25);
            assert_almost_eq!(b.v[1][2], 0.0);
        }

        {
            let a = AffineXform2::scale(Vector2::new(2.0, 4.0));
            let b = a.inverse();

            assert_almost_eq!(b.v[0][0], 0.5);
            assert_almost_eq!(b.v[0][1], 0.0);
            assert_almost_eq!(b.v[0][2], 0.0);
            assert_almost_eq!(b.v[1][0], 0.0);
            assert_almost_eq!(b.v[1][1], 0.25);
            assert_almost_eq!(b.v[1][2], 0.0);
        }

        {
            let a = AffineXform2::rotate((90.0 as f64).to_radians());
            let b = a.inverse();

            assert_almost_eq!(b.v[0][0], 0.0);
            assert_almost_eq!(b.v[0][1], 1.0);
            assert_almost_eq!(b.v[0][2], 0.0);
            assert_almost_eq!(b.v[1][0], -1.0);
            assert_almost_eq!(b.v[1][1], 0.0);
            assert_almost_eq!(b.v[1][2], 0.0);
        }

        {
            let a = AffineXform2::new(2.0, 3.0, -7.0, 5.0, 11.0, 13.0);
            let ainv = a.inverse();
            let b = ainv * a;

            let p0 = Point2::new(5.0, 3.0);
            let p1 = a * p0;
            let p2 = ainv * p1;

            assert_almost_eq!(p0.x, p2.x);
            assert_almost_eq!(p0.y, p2.y);

            assert_almost_eq!(b.v[0][0], 1.0);
            assert_almost_eq!(b.v[0][1], 0.0);
            assert_almost_eq!(b.v[0][2], 0.0);
            assert_almost_eq!(b.v[1][0], 0.0);
            assert_almost_eq!(b.v[1][1], 1.0);
            assert_almost_eq!(b.v[1][2], 0.0);
        }
    }

    #[test]
    fn test_mul() {
        assert_eq!(AffineXform2::new(2.0, 3.0, 5.0, 7.0, 11.0, 13.0) * Vector2::new(2.0, 3.0),
                Vector2::new(13.0, 47.0));
        assert_eq!(&AffineXform2::new(2.0, 3.0, 5.0, 7.0, 11.0, 13.0) * Vector2::new(2.0, 3.0),
                Vector2::new(13.0, 47.0));
        assert_eq!(AffineXform2::new(2.0, 3.0, 5.0, 7.0, 11.0, 13.0) * &Vector2::new(2.0, 3.0),
                Vector2::new(13.0, 47.0));
        assert_eq!(&AffineXform2::new(2.0, 3.0, 5.0, 7.0, 11.0, 13.0) * &Vector2::new(2.0, 3.0),
                Vector2::new(13.0, 47.0));

        assert_eq!(AffineXform2::new(2.0, 3.0, 5.0, 7.0, 11.0, 13.0) * Point2::new(2.0, 3.0),
                Point2::new(18.0, 60.0));
        assert_eq!(&AffineXform2::new(2.0, 3.0, 5.0, 7.0, 11.0, 13.0) * Point2::new(2.0, 3.0),
                Point2::new(18.0, 60.0));
        assert_eq!(AffineXform2::new(2.0, 3.0, 5.0, 7.0, 11.0, 13.0) * &Point2::new(2.0, 3.0),
                Point2::new(18.0, 60.0));
        assert_eq!(&AffineXform2::new(2.0, 3.0, 5.0, 7.0, 11.0, 13.0) * &Point2::new(2.0, 3.0),
                Point2::new(18.0, 60.0));

        assert_eq!(AffineXform2::new(2.0, 3.0, 5.0,
                                     7.0, 11.0, 13.0) *
                    AffineXform2::new(3.0, 2.0, 4.0,
                                      8.0, 7.0, 6.0),
                AffineXform2::new(30.0, 25.0, 31.0,
                                  109.0, 91.0, 107.0));
        assert_eq!(&AffineXform2::new(2.0, 3.0, 5.0,
                                      7.0, 11.0, 13.0) *
                    AffineXform2::new(3.0, 2.0, 4.0,
                                      8.0, 7.0, 6.0),
                AffineXform2::new(30.0, 25.0, 31.0,
                                  109.0, 91.0, 107.0));
        assert_eq!(AffineXform2::new(2.0, 3.0, 5.0,
                                     7.0, 11.0, 13.0) *
                    &AffineXform2::new(3.0, 2.0, 4.0,
                                      8.0, 7.0, 6.0),
                AffineXform2::new(30.0, 25.0, 31.0,
                                  109.0, 91.0, 107.0));
        assert_eq!(&AffineXform2::new(2.0, 3.0, 5.0,
                                       7.0, 11.0, 13.0) *
                    &AffineXform2::new(3.0, 2.0, 4.0,
                                       8.0, 7.0, 6.0),
                AffineXform2::new(30.0, 25.0, 31.0,
                                  109.0, 91.0, 107.0));
    }
}
