use crate::primitive::Primitive;
use anyhow::{bail, Result};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[macro_use]
extern crate log;
use log::Level;

extern crate isosurface;
extern crate nalgebra as na;

mod primitive;
mod render_canvas;
mod renderer;
mod script_engine;
mod script_ui;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Info).unwrap();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("shader_canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let canvas = render_canvas::RenderCanvas::new(canvas)?;

    let new_object_callback = move |new_object: &dyn Primitive| {
        if let Err(err) = canvas.set_primtive(new_object) {
            error!("{:?}", err);
        };
        canvas.draw();
    };
    let run_button = get_button(&document, "run").map_err(|e| e.to_string())?;
    let mesh_button = get_button(&document, "mesh").map_err(|e| e.to_string())?;
    let script = document.get_element_by_id("program").unwrap();
    let script: web_sys::HtmlTextAreaElement = script.dyn_into::<web_sys::HtmlTextAreaElement>()?;
    let output = document.get_element_by_id("output").unwrap();
    let output: web_sys::HtmlTextAreaElement = output.dyn_into::<web_sys::HtmlTextAreaElement>()?;
    let engine = script_engine::RhaiScriptEngine::new();
    _ = script_ui::ScriptUI::new(
        script,
        output,
        run_button,
        mesh_button,
        engine,
        new_object_callback,
    )?;

    Ok(())
}

fn get_button(document: &web_sys::Document, name: &str) -> Result<web_sys::HtmlButtonElement> {
    match document.get_element_by_id(name) {
        Some(e) => match e.clone().dyn_into::<web_sys::HtmlButtonElement>() {
            Ok(b) => Ok(b),
            Err(_) => bail!("Cannot cast {:?} into HtmlButtonElement", e),
        },
        None => bail!("Did not find {}.", name),
    }
}
