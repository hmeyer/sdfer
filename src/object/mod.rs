use std::collections::HashSet;

pub trait Object {
    fn expression(&self) -> String;
    fn static_code(&self) -> HashSet<String> {
        HashSet::new()
    }
}

mod sphere;
pub use sphere::Sphere;

mod boxes;
pub use boxes::ExactBox;
pub use boxes::RoundBox;

mod boolean;
pub use boolean::Difference;
pub use boolean::Intersection;
pub use boolean::Union;
