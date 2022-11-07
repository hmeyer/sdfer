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
    fn expression(&self, p: &str) -> String {
        format!("length({}) - {:.8}", p, self.radius)
    }
}
