use crate::primitive::Primitive;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[macro_use]
extern crate log;
use log::Level;

extern crate nalgebra as na;

mod primitive;
mod render_canvas;
mod renderer;
mod script_engine;
mod script_ui;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug).unwrap();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let canvas = render_canvas::RenderCanvas::new(canvas)?;

    let new_object_callback = move |new_object: &dyn Primitive| {
        if let Err(err) = canvas.set_primtive(new_object) {
            error!("{:?}", err);
        };
        canvas.draw();
    };
    let run_button = document.get_element_by_id("run").unwrap();
    let run_button: web_sys::HtmlButtonElement =
        run_button.dyn_into::<web_sys::HtmlButtonElement>()?;
    let script = document.get_element_by_id("program").unwrap();
    let script: web_sys::HtmlTextAreaElement = script.dyn_into::<web_sys::HtmlTextAreaElement>()?;
    let output = document.get_element_by_id("output").unwrap();
    let output: web_sys::HtmlTextAreaElement = output.dyn_into::<web_sys::HtmlTextAreaElement>()?;
    let engine = script_engine::RhaiScriptEngine::new();
    _ = script_ui::ScriptUI::new(script, output, run_button, engine, new_object_callback)?;

    Ok(())
}
