use crate::primitive::Primitive;
use crate::script_engine::ScriptEngine;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlTextAreaElement};

pub struct ScriptUI {}

impl ScriptUI {
    pub fn new<E: ScriptEngine + 'static>(
        code_area: HtmlTextAreaElement,
        output_area: HtmlTextAreaElement,
        run_button: HtmlButtonElement,
        mut engine: E,
        on_new_object_callback: impl Fn(&(dyn Primitive)) + 'static,
    ) -> Result<ScriptUI, JsValue> {
        {
            let output_area = output_area.clone();
            engine.on_print(move |s| {
                let mut output = output_area.value();
                output.push_str(s);
                output.push('\n');
                output_area.set_value(&output);
            });
        }
        {
            let code_area = code_area.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
                match engine.eval(&code_area.value()) {
                    Ok(primitive) => on_new_object_callback(&*primitive),
                    Err(e) => info!("was err: {:?}", e),
                }
            });
            run_button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(ScriptUI {})
    }
}
