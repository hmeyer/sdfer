use crate::primitive::Primitive;
use rhai;
use wasm_bindgen::prelude::JsValue;

pub trait ScriptEngine {
    fn eval(&self, script: &str) -> Result<Box<dyn Primitive>, JsValue>;
    fn on_print(&mut self, callback: impl Fn(&str) + 'static);
}

pub struct RhaiScriptEngine {
    engine: rhai::Engine,
}

impl RhaiScriptEngine {
    pub fn new() -> RhaiScriptEngine {
        let engine = rhai::Engine::new();
        RhaiScriptEngine { engine }
    }
}

impl ScriptEngine for RhaiScriptEngine {
    fn eval(&self, script: &str) -> Result<Box<dyn Primitive>, JsValue> {
        let result = self.engine.eval::<rhai::Dynamic>(script);
        Err("Error".into())
    }
    fn on_print(&mut self, callback: impl Fn(&str) + 'static) {
        self.engine.on_print(callback);
    }
}
