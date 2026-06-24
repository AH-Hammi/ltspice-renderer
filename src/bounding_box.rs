use crate::shape::Point;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundingBox {
    pub(crate) top_left: (f32, f32),
    pub(crate) bottom_right: (f32, f32),
}

impl BoundingBox {
    pub(crate) fn new() -> Self {
        BoundingBox {
            top_left: (f32::MAX, f32::MAX),
            bottom_right: (f32::MIN, f32::MIN),
        }
    }

    pub(crate) fn add_point(&mut self, point: (f32, f32)) {
        if point.0 < self.top_left.0 {
            self.top_left.0 = point.0;
        }
        if point.1 < self.top_left.1 {
            self.top_left.1 = point.1;
        }
        if point.0 > self.bottom_right.0 {
            self.bottom_right.0 = point.0;
        }
        if point.1 > self.bottom_right.1 {
            self.bottom_right.1 = point.1;
        }
    }

    pub(crate) fn add_point_i32(&mut self, point: Point) {
        let point_f32 = (point.x as f32, point.y as f32);
        self.add_point(point_f32);
    }

    pub(crate) fn merge(&mut self, other: &BoundingBox) {
        self.add_point(other.top_left);
        self.add_point(other.bottom_right);
    }

    pub(crate) fn translate(&mut self, offset: (f32, f32)) {
        self.top_left.0 += offset.0;
        self.top_left.1 += offset.1;
        self.bottom_right.0 += offset.0;
        self.bottom_right.1 += offset.1;
    }

    pub(crate) fn expand(&mut self, amount: f32) {
        self.top_left.0 -= amount;
        self.top_left.1 -= amount;
        self.bottom_right.0 += amount;
        self.bottom_right.1 += amount;
    }
}
