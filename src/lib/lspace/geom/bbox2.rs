use std::ops::Add;
use layout::lalloc::LAlloc;
use geom::fastminmax::{fast_min, fast_max};
use geom::vector2::Vector2;
use geom::point2::Point2;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BBox2 {
    pub lower: Point2,
    pub upper: Point2,
}

impl BBox2 {
    pub fn new(lower: Point2, upper: Point2) -> BBox2 {
        return BBox2{lower: lower, upper: upper};
    }

    pub fn from_centre_size(centre: Point2, size: Vector2) -> BBox2 {
        let half_size = size * 0.5;
        return BBox2{lower: &centre - &half_size, upper:
            &centre + &half_size};
    }

    pub fn from_lower_size(lower: Point2, size: Vector2) -> BBox2 {
        return BBox2{lower: lower, upper: &lower + size};
    }

    pub fn from_allocs(x_alloc: &LAlloc, y_alloc: &LAlloc) -> BBox2 {
        let lower = Point2::new(x_alloc.pos_in_parent(), y_alloc.pos_in_parent());
        return BBox2{lower: lower,
                     upper: Point2::new(lower.x + x_alloc.actual_size(),
                                        lower.y + y_alloc.actual_size())};
    }

    pub fn centre(&self) -> Point2 {
        return Point2::new((self.lower.x + self.upper.x) * 0.5,
                           (self.lower.y + self.upper.y) * 0.5);
    }

    pub fn size(&self) -> Vector2 {
        return self.upper - self.lower;
    }

    pub fn offset(&self, v: &Vector2) -> BBox2 {
        return BBox2{lower: self.lower + v, upper: self.upper + v};
    }

    pub fn contains(&self, p: &Point2) -> bool {
        return p.x >= self.lower.x && p.x <= self.upper.x &&
               p.y >= self.lower.y && p.y <= self.upper.y;
    }

    pub fn intersects(&self, r: &BBox2) -> bool {
        return self.lower.x < r.upper.x && self.lower.y < r.upper.y &&
               self.upper.x > r.lower.x && self.upper.y > r.lower.y;
    }

    pub fn intersection(&self, r: &BBox2) -> Option<BBox2> {
        let lx = fast_max(self.lower.x, r.lower.x);
        let ly = fast_max(self.lower.y, r.lower.y);
        let ux = fast_min(self.upper.x, r.upper.x);
        let uy = fast_min(self.upper.y, r.upper.y);
        if lx <= ux && ly <= uy {
            return Some(BBox2::new(Point2::new(lx, ly), Point2::new(ux, uy)));
        } else {
            return None;
        }
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
    use layout::lalloc::LAlloc;

    #[test]
    fn test_constructor() {
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).lower,
                  Point2::new(1.0, 2.0));
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).upper,
                  Point2::new(10.0, 20.0));

        assert_eq!(BBox2::from_lower_size(Point2::new(1.0, 2.0), Vector2::new(10.0, 20.0)),
                   BBox2::new(Point2::new(1.0, 2.0), Point2::new(11.0, 22.0)));
        assert_eq!(BBox2::from_centre_size(Point2::new(10.0, 11.0), Vector2::new(10.0, 16.0)),
                   BBox2::new(Point2::new(5.0, 3.0), Point2::new(15.0, 19.0)));

        assert_eq!(BBox2::from_allocs(&LAlloc::new(2.0, 10.0, 10.0),
                                      &LAlloc::new(10.0, 30.0, 30.0)),
                   BBox2::new(Point2::new(2.0, 10.0), Point2::new(12.0, 40.0)));
    }

    #[test]
    fn test_dims() {
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).centre(),
                    Point2::new(5.5, 11.0));
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).size(),
                    Vector2::new(9.0, 18.0));
    }

    #[test]
    fn test_offset() {
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).offset(
                   &Vector2::new(2.0, 3.0)),
                   BBox2::new(Point2::new(3.0, 5.0), Point2::new(12.0, 23.0)));
    }

    #[test]
    fn test_contains() {
        assert!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).contains(
                &Point2::new(5.0, 5.0)));

        assert!(!BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).contains(
                &Point2::new(-1.0, 5.0)));
        assert!(!BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).contains(
                &Point2::new(15.0, 5.0)));
        assert!(!BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).contains(
                &Point2::new(5.0, -1.0)));
        assert!(!BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).contains(
                &Point2::new(5.0, 25.0)));
    }

    #[test]
    fn test_intersects() {
        assert!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0))));

        assert!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(9.0, 2.0), Point2::new(20.0, 20.0))));
        assert!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(-9.0, 2.0), Point2::new(2.0, 20.0))));
        assert!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(1.0, -20.0), Point2::new(10.0, 4.0))));
        assert!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(1.0, 18.0), Point2::new(10.0, 40.0))));

        assert!(!BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(11.0, 2.0), Point2::new(20.0, 20.0))));
        assert!(!BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(-9.0, 2.0), Point2::new(0.0, 20.0))));
        assert!(!BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(1.0, -20.0), Point2::new(10.0, 1.0))));
        assert!(!BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersects(
                &BBox2::new(Point2::new(1.0, 22.0), Point2::new(10.0, 40.0))));
    }

    #[test]
    fn test_intersection() {
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0))),
                Some(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0))));

        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(9.0, 2.0), Point2::new(20.0, 20.0))),
                Some(BBox2::new(Point2::new(9.0, 2.0), Point2::new(10.0, 20.0))));
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(-9.0, 2.0), Point2::new(2.0, 20.0))),
                Some(BBox2::new(Point2::new(1.0, 2.0), Point2::new(2.0, 20.0))));
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(1.0, -20.0), Point2::new(10.0, 4.0))),
                Some(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 4.0))));
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(1.0, 18.0), Point2::new(10.0, 40.0))),
                Some(BBox2::new(Point2::new(1.0, 18.0), Point2::new(10.0, 20.0))));

        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(11.0, 2.0), Point2::new(20.0, 20.0))), None);
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(-9.0, 2.0), Point2::new(0.0, 20.0))), None);
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(1.0, -20.0), Point2::new(10.0, 1.0))), None);
        assert_eq!(BBox2::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0)).intersection(
                        &BBox2::new(Point2::new(1.0, 22.0), Point2::new(10.0, 40.0))), None);
    }
}
