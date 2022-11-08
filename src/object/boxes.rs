use super::shader_vec3;
use crate::object::Object;
use std::collections::HashSet;

pub struct ExactBox {
    size: na::Vector3<f32>,
}

impl ExactBox {
    pub fn new(size: na::Vector3<f32>) -> ExactBox {
        ExactBox { size }
    }
}

impl Object for ExactBox {
    fn static_code(&self) -> HashSet<String> {
        HashSet::from([r#"
float Box( vec3 p, vec3 b ) {
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}
"#
        .to_string()])
    }
    fn expression(&self, p: &str) -> String {
        format!("Box({}, {})", p, shader_vec3(&self.size))
    }
}

pub struct RoundBox {
    size: na::Vector3<f32>,
    radius: f32,
}

impl RoundBox {
    pub fn new(size: na::Vector3<f32>, radius: f32) -> RoundBox {
        RoundBox { size, radius }
    }
}

impl Object for RoundBox {
    fn static_code(&self) -> HashSet<String> {
        HashSet::from([r#"
float RoundBox( vec3 p, vec3 b, float r ) {
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0) - r;
}
"#
        .to_string()])
    }
    fn expression(&self, p: &str) -> String {
        format!(
            "RoundBox({}, {}, {:.8})",
            p,
            shader_vec3(&self.size),
            self.radius
        )
    }
}
