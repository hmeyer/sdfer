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
    fn static_code(&self) -> HashSet<&'static str> {
        HashSet::from([r#"
        float Box( vec3 p, vec3 b )
        {
          vec3 q = abs(p) - b;
          return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
        }
        "#])
    }
    fn expression(&self) -> String {
        format!("Box(p, vec3({:.8}, {:.8}, {:.8}));", self.size[0], self.size[1], self.size[2])
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
    fn static_code(&self) -> HashSet<&'static str> {
        HashSet::from([r#"
        float RoundBox( vec3 p, vec3 b, float r )
        {
          vec3 q = abs(p) - b;
          return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0) - r;
        }
        "#])
    }
    fn expression(&self) -> String {
        format!("RoundBox(p, vec3({:.8}, {:.8}, {:.8}), {:.8});", self.size[0], self.size[1], self.size[2], self.radius)
    }
}