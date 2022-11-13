use crate::primitive::*;
use rhai;
use wasm_bindgen::prelude::JsValue;
use wasm_bindgen::JsCast;

pub trait ScriptEngine {
    fn eval(&self, script: &str) -> Result<Box<dyn Primitive>, JsValue>;
    fn on_print(&mut self, callback: impl Fn(&str) + 'static);
}

pub struct RhaiScriptEngine {
    engine: rhai::Engine,
}

impl RhaiScriptEngine {
    pub fn new() -> RhaiScriptEngine {
        let mut engine = rhai::Engine::new();
        engine
            .register_type_with_name::<na::Vector3<f32>>("Vector")
            .register_fn("Vector", na::Vector3::<f32>::new);
        engine
            .register_type_with_name::<Box<Sphere>>("Sphere")
            .register_fn("Sphere", Sphere::new);
        engine
            .register_type_with_name::<Box<ExactBox>>("Box")
            .register_fn("Box", ExactBox::new);
        engine
            .register_type_with_name::<Box<RoundBox>>("RoundBox")
            .register_fn("RoundBox", RoundBox::new);
        RhaiScriptEngine { engine }
    }
}

impl ScriptEngine for RhaiScriptEngine {
    fn eval(&self, script: &str) -> Result<Box<dyn Primitive>, JsValue> {
        let result = self
            .engine
            .eval::<rhai::Dynamic>(script)
            .map_err(|e| format!("{:?}", e))?;
        if result.type_id() == rhai::plugin::TypeId::of::<Box<Sphere>>() {
            return Ok(result.cast::<Box<Sphere>>() as Box<dyn Primitive>);
        }
        if result.type_id() == rhai::plugin::TypeId::of::<Box<ExactBox>>() {
            return Ok(result.cast::<Box<ExactBox>>() as Box<dyn Primitive>);
        }
        if result.type_id() == rhai::plugin::TypeId::of::<Box<RoundBox>>() {
            return Ok(result.cast::<Box<RoundBox>>() as Box<dyn Primitive>);
        }
        return Err(format!("Not a primitive: {}", result).into());
    }
    fn on_print(&mut self, callback: impl Fn(&str) + 'static) {
        self.engine.on_print(callback);
    }
}
