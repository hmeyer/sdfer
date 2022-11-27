use super::{shader_vec3, Primitive};
use anyhow::{bail, Result};

#[derive(Clone)]
struct CylinderBounds {
    begin: na::Vector3<f32>,
    end: na::Vector3<f32>,
}

#[derive(Clone)]
pub struct Cylinder {
    radius: f32,
    bounds: Option<CylinderBounds>,
}

impl Cylinder {
    pub fn new_infinite(radius: f32) -> Result<Box<dyn Primitive>> {
        Cylinder::new_impl(radius, None)
    }
    pub fn new(
        radius: f32,
        begin: na::Vector3<f32>,
        end: na::Vector3<f32>,
    ) -> Result<Box<dyn Primitive>> {
        Cylinder::new_impl(radius, Some(CylinderBounds { begin, end }))
    }
    fn new_impl(radius: f32, bounds: Option<CylinderBounds>) -> Result<Box<dyn Primitive>> {
        if radius <= 0.0 {
            bail!("Radius must be > 0 (was {}).", radius);
        }
        if let Some(ref bounds) = bounds {
            let height = (bounds.begin - bounds.end).norm();
            if height == 0. {
                bail!("Height must be > 0.");
            }
        }
        return Ok(Box::new(Cylinder { radius, bounds }));
    }
}

impl Primitive for Cylinder {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        if self.bounds.is_some() {
            shared_code.push(
                r#"
float CappedCylinder(vec3 p, vec3 a, vec3 b, float r) {
    vec3  ba = b - a;
    vec3  pa = p - a;
    float baba = dot(ba,ba);
    float paba = dot(pa,ba);
    float x = length(pa*baba-ba*paba) - r*baba;
    float y = abs(paba-baba*0.5)-baba*0.5;
    float x2 = x*x;
    float y2 = y*y*baba;
    float d = (max(x, y) < 0.0) ? -min(x2, y2) : (((x > 0.0) ? x2 : 0.0) + ((y > 0.0) ? y2 : 0.0));
    return sign(d)*sqrt(abs(d))/baba;
}
"#
                .to_string(),
            );
        }
        if let Some(ref bounds) = self.bounds {
            Ok(format!(
                "CappedCylinder({p}, {a}, {b}, {r:.8})",
                p = p,
                a = shader_vec3(&bounds.begin),
                b = shader_vec3(&bounds.end),
                r = self.radius
            ))
        } else {
            Ok(format!("length(({}).xy) - {:.8}", p, self.radius))
        }
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        if let Some(ref bounds) = self.bounds {
            let ba = bounds.end - bounds.begin;
            let pa = p - bounds.begin;
            let baba = ba.norm_squared();
            let paba = pa.dot(&ba);
            let x = (pa * baba - ba * paba).norm() - self.radius * baba;
            let y = (paba - baba * 0.5).abs() - baba * 0.5;
            let x2 = x * x;
            let y2 = y * y * baba;
            let d = if x.max(y) < 0.0 {
                -(x2.min(y2))
            } else {
                (if x > 0.0 { x2 } else { 0.0 }) + (if y > 0.0 { y2 } else { 0.0 })
            };
            Ok(d.abs().sqrt().copysign(d) / baba)
        } else {
            Ok(p.rows(0, 2).norm() - self.radius)
        }
    }
}

#[derive(Clone)]
pub struct RoundedCylinder {
    main_radius: f32,
    rounding_radius: f32,
    height: f32,
}

impl RoundedCylinder {
    pub fn new(main_radius: f32, rounding_radius: f32, height: f32) -> Result<Box<dyn Primitive>> {
        if main_radius <= 0. {
            bail!("main_radius should be positive (was {}).", main_radius);
        }
        if rounding_radius < 0. {
            bail!(
                "rounding_radius should not be negative (was {}).",
                rounding_radius
            );
        }
        if height < 0. {
            bail!("height should be positive (was {}).", height);
        }
        Ok(Box::new(RoundedCylinder {
            main_radius,
            rounding_radius,
            height,
        }))
    }
}

impl Primitive for RoundedCylinder {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        shared_code.push(
            r#"
float RoundedCylinder(vec3 p, float ra, float rb, float h) {
    vec2 d = vec2(length(p.xy) - 2.0 * ra + rb, abs(p.z) - h);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - rb;
}
"#
            .to_string(),
        );
        Ok(format!(
            "RoundedCylinder({}, {:.8}, {:.8}, {:.8})",
            p, self.main_radius, self.rounding_radius, self.height
        ))
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        let dx = p.rows(0, 2).norm() - 2.0 * self.main_radius + self.rounding_radius;
        let dy = p[2].abs() - self.height;
        Ok(
            dx.max(dy).min(0.0) + na::Vector2::new(dx.max(0.), dy.max(0.)).norm()
                - self.rounding_radius,
        )
    }
}

#[derive(Clone)]
pub struct Capsule {
    radius: f32,
    begin: na::Vector3<f32>,
    end: na::Vector3<f32>,
}

impl Capsule {
    pub fn new(
        radius: f32,
        begin: na::Vector3<f32>,
        end: na::Vector3<f32>,
    ) -> Result<Box<dyn Primitive>> {
        if radius <= 0. {
            bail!("radius should be positive (was {}).", radius);
        }
        Ok(Box::new(Capsule { radius, begin, end }))
    }
}

impl Primitive for Capsule {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        shared_code.push(
            r#"
float Capsule(vec3 p, vec3 a, vec3 b, float r) {
    vec3 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0 );
    return length(pa - ba * h) - r;
}
"#
            .to_string(),
        );
        Ok(format!(
            "Capsule({}, {}, {}, {:.8})",
            p,
            shader_vec3(&self.begin),
            shader_vec3(&self.end),
            self.radius
        ))
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        let pa = p - self.begin;
        let ba = self.end - self.begin;
        let h = (pa.dot(&ba) / ba.norm_squared()).clamp(0., 1.);
        Ok((pa - ba * h).norm() - self.radius)
    }
}
