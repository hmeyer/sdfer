use std::collections::HashSet;

pub trait Primitive: PrimitiveClone {
    fn expression(&self, p: &str) -> String;
    fn static_code(&self) -> HashSet<String>;
    fn translate(&self, vector: na::Vector3<f32>) -> Box<dyn Primitive> {
        Translate::new(self.clone_box(), vector)
    }
    fn rotate_euler(&self, r: f32, p: f32, y: f32) -> Box<dyn Primitive> {
        Rotate::from_euler(self.clone_box(), r, p, y)
    }
    fn scale(&self, scale: na::Vector3<f32>) -> Box<dyn Primitive> {
        Scale::new(self.clone_box(), scale)
    }
}

pub trait PrimitiveClone {
    /// Clone ```Box<Primitive>```.
    fn clone_box(&self) -> Box<dyn Primitive>;
}

impl<T> PrimitiveClone for T
where
    T: 'static + Primitive + Clone,
{
    fn clone_box(&self) -> Box<dyn Primitive> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Primitive> {
    fn clone(&self) -> Box<dyn Primitive> {
        self.clone_box()
    }
}

fn shader_vec3(v: &na::Vector3<f32>) -> String {
    format!("vec3({:.8}, {:.8}, {:.8})", v[0], v[1], v[2])
}

fn shader_mat3(m: &na::Matrix3<f32>) -> String {
    let m = m.as_slice();
    format!(
        "mat3({:.8}, {:.8}, {:.8},
                  {:.8}, {:.8}, {:.8},
                  {:.8}, {:.8}, {:.8})",
        m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8]
    )
}

mod plane;
pub use plane::Plane;

mod sphere;
pub use sphere::Sphere;

mod torus;
pub use torus::Torus;

mod boxes;
pub use boxes::{ExactBox, RoundBox};

mod cylinder;
pub use cylinder::{Cylinder, RoundedCylinder};

mod boolean;
pub use boolean::{Boolean, BooleanKind};

mod transformations;
pub use transformations::{Rotate, Scale, Translate};

mod repeat;
pub use repeat::Repeat;

mod warp;
pub use warp::{Bend, Twist};
