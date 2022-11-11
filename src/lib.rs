use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[macro_use]
extern crate log;
use log::Level;

use rhai::{Engine, EvalAltResult};
use std::cell::RefCell;
use std::rc::Rc;

extern crate nalgebra as na;

mod primitive;
mod render_canvas;
mod renderer;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug).unwrap();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let canvas = render_canvas::RenderCanvas::new(canvas)?;

    let sphere = Box::new(primitive::Sphere::new(1.0));
    let sphere = Box::new(primitive::Scale::new(
        sphere,
        na::Vector3::new(0.5, 0.8, 1.5),
    ));
    let rbox1 = Box::new(primitive::ExactBox::new(na::Vector3::new(0.4, 0.6, 1.0)));
    let rbox1 = Box::new(primitive::Rotate::from_euler(rbox1, 0.5, 0., 0.));
    let diff = Box::new(primitive::Difference::new(vec![rbox1, sphere])?);
    let rbox2 = Box::new(primitive::RoundBox::new(
        na::Vector3::new(1.0, 0.4, 0.6),
        0.2,
    ));
    let rbox2 = Box::new(primitive::Translate::new(
        rbox2,
        na::Vector3::new(1., 1., 1.),
    ));
    let my_object = primitive::Union::new_with_smoothness(vec![diff, rbox2], 0.2)?;
    canvas.set_primtive(&my_object)?;

    canvas.draw();

    let run_button = document.get_element_by_id("run").unwrap();
    let run_button: web_sys::HtmlButtonElement =
        run_button.dyn_into::<web_sys::HtmlButtonElement>()?;
    let text_program = document.get_element_by_id("program").unwrap();
    let text_program: web_sys::HtmlTextAreaElement =
        text_program.dyn_into::<web_sys::HtmlTextAreaElement>()?;
    let text_output = document.get_element_by_id("output").unwrap();
    let text_output: web_sys::HtmlTextAreaElement =
        text_output.dyn_into::<web_sys::HtmlTextAreaElement>()?;
    let mut engine = Engine::new();
    engine.on_print(move |s| {
        let mut output = text_output.value();
        output.push_str(s);
        output.push('\n');
        text_output.set_value(&output);
    });
    let engine = Rc::new(RefCell::new(engine));

    {
        let text_program = text_program.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
            let result = engine.borrow().eval::<rhai::Dynamic>(&text_program.value());
            info!("click: {:?}", result);
        });
        run_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}
