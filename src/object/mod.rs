use std::collections::HashSet;

pub trait Object {
    fn expression(&self) -> String;
    fn static_code(&self) -> HashSet<&'static str> {
        HashSet::new()
    }
}

mod sphere;
pub use sphere::Sphere;

mod boxes;
pub use boxes::ExactBox;
pub use boxes::RoundBox;
