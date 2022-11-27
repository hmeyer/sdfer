use crate::primitive::Primitive;
use anyhow::{bail, Result};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[macro_use]
extern crate log;
use log::Level;

extern crate isosurface;
extern crate nalgebra as na;

mod mesh_canvas;
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
    let rcanvas = get_canvas(&document, "shader_canvas")?;
    let rcanvas = render_canvas::RenderCanvas::new(rcanvas)?;

    let mcanvas = get_canvas(&document, "mesh_canvas")?;
    let mcanvas = mesh_canvas::MeshCanvas::new(mcanvas)?;
    mcanvas.draw();

    let new_object_callback = move |new_object: &dyn Primitive| {
        if let Err(err) = rcanvas.set_primtive(new_object) {
            error!("{:?}", err);
        };
        rcanvas.draw();
    };
    let run_button = get_button(&document, "run")?;
    let mesh_button = get_button(&document, "mesh")?;
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

fn get_canvas(
    document: &web_sys::Document,
    name: &str,
) -> Result<web_sys::HtmlCanvasElement, JsValue> {
    match document.get_element_by_id(name) {
        Some(e) => e
            .clone()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| format!("Cannot cast {:?} into HtmlCanvasElement.", e).into()),
        None => Err(format!("Did not find {}.", name).into()),
    }
}

fn get_button(
    document: &web_sys::Document,
    name: &str,
) -> Result<web_sys::HtmlButtonElement, JsValue> {
    match document.get_element_by_id(name) {
        Some(e) => e
            .clone()
            .dyn_into::<web_sys::HtmlButtonElement>()
            .map_err(|_| format!("Cannot cast {:?} into HtmlButtonElement.", e).into()),
        None => Err(format!("Did not find {}.", name).into()),
    }
}
