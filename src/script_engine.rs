use crate::primitive::*;
use rhai::{Array, Dynamic, Engine, EvalAltResult};
use wasm_bindgen::prelude::JsValue;

pub trait ScriptEngine {
    fn eval(&self, script: &str) -> Result<Box<dyn Primitive>, JsValue>;
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
            })
            .register_fn("Vector", |x: f32, y: f32, z: i32| {
                na::Vector3::<f32>::new(x as f32, y as f32, z as f32)
            })
            .register_fn("Vector", |x: f32, y: i32, z: f32| {
                na::Vector3::<f32>::new(x as f32, y as f32, z as f32)
            })
            .register_fn("Vector", |x: f32, y: i32, z: i32| {
                na::Vector3::<f32>::new(x as f32, y as f32, z as f32)
            })
            .register_fn("Vector", |x: i32, y: f32, z: f32| {
                na::Vector3::<f32>::new(x as f32, y as f32, z as f32)
            })
            .register_fn("Vector", |x: i32, y: f32, z: i32| {
                na::Vector3::<f32>::new(x as f32, y as f32, z as f32)
            })
            .register_fn("Vector", |x: i32, y: i32, z: f32| {
                na::Vector3::<f32>::new(x as f32, y as f32, z as f32)
            })
            .register_fn("Vector", |x: i32, y: i32, z: i32| {
                na::Vector3::<f32>::new(x as f32, y as f32, z as f32)
            });
        engine
            .register_type_with_name::<Box<dyn Primitive>>("Primitive")
            .register_fn(
                "translate",
                |prim: Box<dyn Primitive>, t: na::Vector3<f32>| prim.translate(t),
            )
            .register_fn(
                "rotate_euler",
                |prim: Box<dyn Primitive>, r: f32, p: f32, y: f32| prim.rotate_euler(r, p, y),
            )
            .register_fn("scale", |prim: Box<dyn Primitive>, s: na::Vector3<f32>| {
                prim.scale(s)
            });
        engine
            .register_type_with_name::<Box<Sphere>>("Sphere")
            .register_fn("Sphere", Sphere::new)
            .register_fn("Sphere", |r: i32| Sphere::new(r as f32));
        engine
            .register_type_with_name::<Box<ExactBox>>("Box")
            .register_fn("Box", ExactBox::new);
        engine
            .register_type_with_name::<Box<RoundBox>>("RoundBox")
            .register_fn("RoundBox", RoundBox::new)
            .register_fn("RoundBox", |extend: na::Vector3<f32>, radius: i32| {
                RoundBox::new(extend, radius as f32)
            });

        engine
            .register_type_with_name::<Box<Boolean>>("Boolean")
            .register_fn(
                "Union",
                |children: rhai::Array| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let children = to_primitive_vec(children)?;
                    Boolean::new_union(children).map_err(|e| e.into())
                },
            )
            .register_fn(
                "Intersection",
                |children: rhai::Array| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let children = to_primitive_vec(children)?;
                    Boolean::new_intersection(children).map_err(|e| e.into())
                },
            )
            .register_fn(
                "Difference",
                |children: rhai::Array| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let children = to_primitive_vec(children)?;
                    Boolean::new_difference(children).map_err(|e| e.into())
                },
            )
            .register_fn(
                "smooth",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Polynomial(k))
                        .map_err(|e: String| Box::<EvalAltResult>::from(e))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_cubic",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::CubicPolynomial(k))
                        .map_err(|e: String| Box::<EvalAltResult>::from(e))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_root",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Root(k))
                        .map_err(|e: String| Box::<EvalAltResult>::from(e))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_exponential",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Exponential(k))
                        .map_err(|e: String| Box::<EvalAltResult>::from(e))?;
                    Ok(b)
                },
            )
            .register_fn(
                "chamfer",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    b.set_kind(BooleanKind::Chamfer(k))
                        .map_err(|e: String| Box::<EvalAltResult>::from(e))?;
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
                        .map_err(|e: String| Box::<EvalAltResult>::from(e))?;
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
        .map(|c| to_primitive(c).map_err(|e| e.into()))
        .collect()
}

fn to_primitive(p: Dynamic) -> Result<Box<dyn Primitive>, String> {
    if p.type_id() == rhai::plugin::TypeId::of::<Box<dyn Primitive>>() {
        return Ok(p.cast::<Box<dyn Primitive>>());
    }
    if p.type_id() == rhai::plugin::TypeId::of::<Box<Boolean>>() {
        return Ok(p.cast::<Box<Boolean>>());
    }
    return Err(format!("Not a primitive: {}", p));
}

impl ScriptEngine for RhaiScriptEngine {
    fn eval(&self, script: &str) -> Result<Box<dyn Primitive>, JsValue> {
        let result = self
            .engine
            .eval::<Dynamic>(script)
            .map_err(|e| format!("{:?}", e))?;
        to_primitive(result).map_err(|e| e.into())
    }
    fn on_print(&mut self, callback: impl Fn(&str) + 'static) {
        self.engine.on_print(callback);
    }
}
