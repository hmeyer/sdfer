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
                na::Vector3::<f32>::new(x as f32, y as f32, z as f32)
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
            );
        engine
            .register_type_with_name::<Box<Sphere>>("Sphere")
            .register_fn("Sphere", Sphere::new);
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
            .register_fn("Box", |x: f32, y: f32, z: f32| {
                ExactBox::new(na::Vector3::new(x, y, z))
            });
        engine
            .register_type_with_name::<Box<RoundBox>>("RoundBox")
            .register_fn("RoundBox", |x: f32, y: f32, z: f32, r: f32| {
                RoundBox::new(na::Vector3::new(x, y, z), r)
            });
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
                    b.set_kind(BooleanKind::Stairs(k, n as usize))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
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
