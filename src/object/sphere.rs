use crate::object::Object;

pub struct Sphere {
    radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Sphere {
        Sphere { radius }
    }
}

impl Object for Sphere {
    fn expression(&self) -> String {
        format!("length(p) - {:.8}", self.radius)
    }
}