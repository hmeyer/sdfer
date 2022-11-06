use shadercanvas::ShaderCanvas;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[macro_use]
extern crate log;
use log::Level;

extern crate nalgebra as na;

use std::rc::Rc;
use std::cell::RefCell;

mod object;
mod renderer;


#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug).unwrap();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let shader_canvas = Rc::new(RefCell::new(ShaderCanvas::new(canvas.clone())?));


    let object = object::Sphere::new(1.0);
    let object = object::RoundBox::new(na::Vector3::new(0.4, 0.6, 1.0), 0.2);
    let shader = renderer::generate_renderer_shader(&object);
    info!("setting shader:\n{}", shader);
    shader_canvas.borrow_mut().set_shader(&shader)?;

    let mut world_transform = Rc::new(RefCell::new(na::Matrix4::<f32>::identity()));
    shader_canvas.borrow().uniform_matrix4fv("iWorldTransform", world_transform.borrow().as_slice());


    {
        let clone = shader_canvas.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
            clone.borrow().draw();
        });
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }


    {
        let shader_canvas = shader_canvas.clone();
        let world_transform = world_transform.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::WheelEvent| {
            event.prevent_default();
            let mut world_transform = world_transform.borrow_mut();
            *world_transform = world_transform.append_translation(&na::Vector3::new(0., 0., (event.delta_y() / 100.0) as f32));
            info!("wheel: {}", world_transform);
            let shader_canvas = shader_canvas.borrow();
            shader_canvas.uniform_matrix4fv("iWorldTransform", world_transform.transpose().as_slice());
            shader_canvas.draw();
        });
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }


    shader_canvas.borrow().draw();

    Ok(())
}
