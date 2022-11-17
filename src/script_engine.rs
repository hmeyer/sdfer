use crate::primitive::*;
use anyhow::{anyhow, bail, Result};
use rhai::{Array, Dynamic, Engine, EvalAltResult};

pub trait ScriptEngine {
    fn eval(&self, script: &str) -> Result<Box<dyn Primitive>>;
    fn on_print(&mut self, callback: impl Fn(&str) + 'static);
}

pub struct RhaiScriptEngine {
    engine: Engine,
}

impl RhaiScriptEngine {
    pub fn new() -> RhaiScriptEngine {
        let mut engine = Engine::new();
        engine
            .register_type_with_name::<na::Vector3<f32>>("Vector")
            .register_fn("Vector", |x: f32, y: f32, z: f32| {
                na::Vector3::<f32>::new(x, y, z)
            });
        engine
            .register_type_with_name::<na::Vector3<i32>>("Vector")
            .register_fn("Vector", |x: i32, y: i32, z: i32| {
                na::Vector3::<i32>::new(x, y, z)
            });
        engine
            .register_type_with_name::<Box<dyn Primitive>>("Primitive")
            .register_fn(
                "translate",
                |prim: Box<dyn Primitive>, x: f32, y: f32, z: f32| {
                    prim.translate(na::Vector3::new(x, y, z))
                },
            )
            .register_fn(
                "rotate_euler",
                |prim: Box<dyn Primitive>, r: f32, p: f32, y: f32| prim.rotate_euler(r, p, y),
            )
            .register_fn(
                "scale",
                |prim: Box<dyn Primitive>, x: f32, y: f32, z: f32| {
                    prim.scale(na::Vector3::new(x, y, z))
                },
            )
            .register_fn(
                "repeat",
                |prim: Box<dyn Primitive>,
                 bound: na::Vector3<f32>,
                 repeats_min: na::Vector3<i32>,
                 repeats_max: na::Vector3<i32>|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    let r = Repeat::new(prim, bound, repeats_min, repeats_max)
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(r as Box<dyn Primitive>)
                },
            )
            .register_fn(
                "twist",
                |prim: Box<dyn Primitive>, height_per_rotation: f32| {
                    Twist::new(prim, height_per_rotation) as Box<dyn Primitive>
                },
            );
        engine
            .register_type_with_name::<Box<Sphere>>("Plane")
            .register_fn(
                "Plane",
                |normal: na::Vector3<f32>,
                 d: f32|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    Plane::new(normal, d).map_err(|e| e.to_string().into())
                },
            );
        engine
            .register_type_with_name::<Box<Sphere>>("Sphere")
            .register_fn(
                "Sphere",
                |r: f32| -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    Sphere::new(r).map_err(|e| e.to_string().into())
                },
            );
        engine
            .register_type_with_name::<Box<Sphere>>("Cylinder")
            .register_fn(
                "Cylinder",
                |r: f32| -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    Cylinder::new_infinite(r).map_err(|e| e.to_string().into())
                },
            )
            .register_fn(
                "Cylinder",
                |r: f32,
                 begin: na::Vector3<f32>,
                 end: na::Vector3<f32>|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    Cylinder::new(r, begin, end).map_err(|e| e.to_string().into())
                },
            );
        engine
            .register_type_with_name::<Box<Sphere>>("RoundedCylinder")
            .register_fn(
                "RoundedCylinder",
                |main_radius: f32,
                 rounding_radius: f32,
                 height: f32|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    RoundedCylinder::new(main_radius, rounding_radius, height)
                        .map_err(|e| e.to_string().into())
                },
            );

        engine
            .register_type_with_name::<Box<Torus>>("Torus")
            .register_fn(
                "Torus",
                |inner: f32, outer: f32| -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    Torus::new(inner, outer).map_err(|e| e.to_string().into())
                },
            )
            .register_fn(
                "CappedTorus",
                |inner: f32,
                 outer: f32,
                 cap_angle: f32|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    Torus::new_capped(inner, outer, cap_angle).map_err(|e| e.to_string().into())
                },
            );
        engine
            .register_type_with_name::<Box<ExactBox>>("Box")
            .register_fn(
                "Box",
                |x: f32, y: f32, z: f32| -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    ExactBox::new(na::Vector3::new(x, y, z)).map_err(|e| e.to_string().into())
                },
            );
        engine
            .register_type_with_name::<Box<RoundBox>>("RoundBox")
            .register_fn(
                "RoundBox",
                |x: f32,
                 y: f32,
                 z: f32,
                 r: f32|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    RoundBox::new(na::Vector3::new(x, y, z), r).map_err(|e| e.to_string().into())
                },
            );
        engine
            .register_type_with_name::<Box<Boolean>>("Boolean")
            .register_fn(
                "Union",
                |children: rhai::Array| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let children = to_primitive_vec(children)?;
                    Boolean::new_union(children).map_err(|e| e.to_string().into())
                },
            )
            .register_fn(
                "Intersection",
                |children: rhai::Array| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let children = to_primitive_vec(children)?;
                    Boolean::new_intersection(children).map_err(|e| e.to_string().into())
                },
            )
            .register_fn(
                "Difference",
                |children: rhai::Array| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let children = to_primitive_vec(children)?;
                    Boolean::new_difference(children).map_err(|e| e.to_string().into())
                },
            )
            .register_fn(
                "smooth",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Polynomial(k))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_cubic",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::CubicPolynomial(k))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_root",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Root(k))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_exponential",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Exponential(k))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "chamfer",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Chamfer(k))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "stairs",
                |b: &mut Box<Boolean>,
                 k: f32,
                 n: i32|
                 -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Stairs {
                        d: k,
                        num: n as usize,
                    })
                    .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "translate",
                |prim: &mut Box<Boolean>, x: f32, y: f32, z: f32| {
                    prim.translate(na::Vector3::new(x, y, z))
                },
            )
            .register_fn(
                "rotate_euler",
                |prim: &mut Box<Boolean>, r: f32, p: f32, y: f32| prim.rotate_euler(r, p, y),
            )
            .register_fn(
                "scale",
                |prim: &mut Box<Boolean>, x: f32, y: f32, z: f32| {
                    prim.scale(na::Vector3::new(x, y, z))
                },
            )
            .register_fn(
                "repeat",
                |prim: &mut Box<Boolean>,
                 bound: na::Vector3<f32>,
                 repeats_min: na::Vector3<i32>,
                 repeats_max: na::Vector3<i32>|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    let r = Repeat::new(prim.clone(), bound, repeats_min, repeats_max)
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(r as Box<dyn Primitive>)
                },
            );
        let engine = engine;
        RhaiScriptEngine { engine }
    }
}

fn to_primitive_vec(children: Array) -> Result<Vec<Box<dyn Primitive>>, Box<EvalAltResult>> {
    children
        .into_iter()
        .map(|c| to_primitive(c).map_err(|e| e.to_string().into()))
        .collect()
}

fn to_primitive(p: Dynamic) -> Result<Box<dyn Primitive>> {
    if p.type_id() == rhai::plugin::TypeId::of::<Box<dyn Primitive>>() {
        return Ok(p.cast::<Box<dyn Primitive>>());
    }
    if p.type_id() == rhai::plugin::TypeId::of::<Box<Boolean>>() {
        return Ok(p.cast::<Box<Boolean>>());
    }
    bail!("Not a primitive: {}", p);
}

impl ScriptEngine for RhaiScriptEngine {
    fn eval(&self, script: &str) -> Result<Box<dyn Primitive>> {
        let result = self
            .engine
            .eval::<Dynamic>(script)
            .map_err(|e| anyhow!(e.to_string()))?;
        to_primitive(result)
    }
    fn on_print(&mut self, callback: impl Fn(&str) + 'static) {
        self.engine.on_print(callback);
    }
}
