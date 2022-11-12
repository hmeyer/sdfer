use crate::script_engine::ScriptEngine;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlTextAreaElement};

pub struct ScriptUI {
    code_area: HtmlTextAreaElement,
    output_area: HtmlTextAreaElement,
}

impl ScriptUI {
    pub fn new<E: ScriptEngine + 'static>(
        code_area: HtmlTextAreaElement,
        output_area: HtmlTextAreaElement,
        run_button: HtmlButtonElement,
        mut engine: E,
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
                let result = engine.eval(&code_area.value());
                info!("click: {:?}", result.is_ok());
            });
            run_button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(ScriptUI {
            code_area,
            output_area,
        })
    }
}
