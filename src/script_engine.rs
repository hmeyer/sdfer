use super::primitive::*;
use anyhow::{anyhow, bail, Result};
use rhai::{Array, Dynamic, Engine, EvalAltResult};
use std::f32::consts::PI;

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
        engine.set_max_expr_depths(32, 32);
        engine.set_fast_operators(false);
        engine
            .register_type_with_name::<glm::Vec3>("Vector_f32")
            .register_fn("Vector", |x: f32, y: f32, z: f32| glm::vec3(x, y, z))
            .register_indexer_get(|v: &mut glm::Vec3, i: i32| v[i as usize])
            .register_fn("to_string", |v: &mut glm::Vec3| {
                format!("Vector_f32{:?}", v)
            })
            .register_fn("norm", |v: &mut glm::Vec3| v.norm())
            .register_fn("-", |v: glm::Vec3| -v)
            .register_fn("/", |v: glm::Vec3, s: f32| v / s)
            .register_fn("*", |v: glm::Vec3, s: f32| v * s)
            .register_fn("*", |s: f32, v: glm::Vec3| v * s)
            .register_fn("+", |v: glm::Vec3, w: glm::Vec3| v + w)
            .register_fn("-", |v: glm::Vec3, w: glm::Vec3| v - w)
            .register_fn("*", |v: glm::Vec3, w: glm::Vec3| v.component_mul(&w))
            .register_fn("/", |v: glm::Vec3, w: glm::Vec3| v.component_div(&w))
            .register_fn("dot", |v: &mut glm::Vec3, other: glm::Vec3| v.dot(&other))
            .register_fn("cross", |v: &mut glm::Vec3, other: glm::Vec3| {
                v.cross(&other)
            });
        engine
            .register_type_with_name::<glm::I32Vec3>("Vector_i32")
            .register_fn("Vector", |x: i32, y: i32, z: i32| {
                glm::make_vec3::<i32>(&[x, y, z])
            });
        engine
            .register_type_with_name::<Box<dyn Primitive>>("Primitive")
            .register_fn(
                "translate",
                |prim: Box<dyn Primitive>, x: f32, y: f32, z: f32| {
                    prim.translate(glm::vec3(x, y, z))
                },
            )
            .register_fn(
                "rotate_rad",
                |prim: Box<dyn Primitive>, r: f32, p: f32, y: f32| prim.rotate_euler(r, p, y),
            )
            .register_fn(
                "rotate_deg",
                |prim: Box<dyn Primitive>, r: f32, p: f32, y: f32| {
                    prim.rotate_euler(r * PI / 180., p * PI / 180., y * PI / 180.)
                },
            )
            .register_fn(
                "scale",
                |prim: Box<dyn Primitive>, x: f32, y: f32, z: f32| prim.scale(glm::vec3(x, y, z)),
            )
            .register_fn("scale", |prim: Box<dyn Primitive>, s: f32| {
                prim.scale(glm::vec3(s, s, s))
            })
            .register_fn(
                "repeat",
                |prim: Box<dyn Primitive>,
                 bound: glm::Vec3,
                 repeats_min: glm::I32Vec3,
                 repeats_max: glm::I32Vec3|
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
            )
            .register_fn(
                "bend",
                |prim: Box<dyn Primitive>, distance_for_full_circle: f32| {
                    Bend::new(prim, distance_for_full_circle) as Box<dyn Primitive>
                },
            );
        engine.register_fn(
            "Plane",
            |normal: glm::Vec3, d: f32| -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                Plane::new(normal, d).map_err(|e| e.to_string().into())
            },
        );
        engine.register_fn(
            "Sphere",
            |r: f32| -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                Sphere::new(r).map_err(|e| e.to_string().into())
            },
        );
        engine
            .register_fn(
                "Cylinder",
                |r: f32| -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    Cylinder::new_infinite(r).map_err(|e| e.to_string().into())
                },
            )
            .register_fn(
                "Cylinder",
                |r: f32,
                 begin: glm::Vec3,
                 end: glm::Vec3|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    Cylinder::new(r, begin, end).map_err(|e| e.to_string().into())
                },
            );
        engine.register_fn(
            "RoundedCylinder",
            |main_radius: f32,
             rounding_radius: f32,
             height: f32|
             -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                RoundedCylinder::new(main_radius, rounding_radius, height)
                    .map_err(|e| e.to_string().into())
            },
        );
        engine.register_fn(
            "Capsule",
            |r: f32,
             begin: glm::Vec3,
             end: glm::Vec3|
             -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                Capsule::new(r, begin, end).map_err(|e| e.to_string().into())
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
                    ExactBox::new(glm::vec3(x, y, z)).map_err(|e| e.to_string().into())
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
                    RoundBox::new(glm::vec3(x, y, z), r).map_err(|e| e.to_string().into())
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
                    let f = MinPolynomial::new(k)
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    b.set_min_function(Box::new(f))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_cubic",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    let f = MinCubicPolynomial::new(k)
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    b.set_min_function(Box::new(f))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_root",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    let f =
                        MinRoot::new(k).map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    b.set_min_function(Box::new(f))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "smooth_exponential",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    let f = MinExponential::new(k)
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    b.set_min_function(Box::new(f))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "chamfer",
                |b: &mut Box<Boolean>, k: f32| -> Result<Box<Boolean>, Box<EvalAltResult>> {
                    let mut b = b.clone();
                    let f = MinChamfer::new(k)
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    b.set_min_function(Box::new(f))
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
                    let f = MinStairs::new(k, n)
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    b.set_min_function(Box::new(f))
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(b)
                },
            )
            .register_fn(
                "translate",
                |prim: &mut Box<Boolean>, x: f32, y: f32, z: f32| {
                    prim.translate(glm::vec3(x, y, z))
                },
            )
            .register_fn(
                "rotate_rad",
                |prim: &mut Box<Boolean>, r: f32, p: f32, y: f32| prim.rotate_euler(r, p, y),
            )
            .register_fn(
                "rotate_deg",
                |prim: &mut Box<Boolean>, r: f32, p: f32, y: f32| {
                    prim.rotate_euler(r * PI / 180., p * PI / 180., y * PI / 180.)
                },
            )
            .register_fn(
                "scale",
                |prim: &mut Box<Boolean>, x: f32, y: f32, z: f32| prim.scale(glm::vec3(x, y, z)),
            )
            .register_fn("scale", |prim: &mut Box<Boolean>, s: f32| {
                prim.scale(glm::vec3(s, s, s))
            })
            .register_fn(
                "repeat",
                |prim: &mut Box<Boolean>,
                 bound: glm::Vec3,
                 repeats_min: glm::I32Vec3,
                 repeats_max: glm::I32Vec3|
                 -> Result<Box<dyn Primitive>, Box<EvalAltResult>> {
                    let r = Repeat::new(prim.clone(), bound, repeats_min, repeats_max)
                        .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                    Ok(r as Box<dyn Primitive>)
                },
            )
            .register_fn(
                "twist",
                |prim: &mut Box<Boolean>, height_per_rotation: f32| {
                    Twist::new(prim.clone(), height_per_rotation) as Box<dyn Primitive>
                },
            )
            .register_fn(
                "bend",
                |prim: &mut Box<Boolean>, distance_for_full_circle: f32| {
                    Bend::new(prim.clone(), distance_for_full_circle) as Box<dyn Primitive>
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
