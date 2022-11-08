use shadercanvas::ShaderCanvas;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[macro_use]
extern crate log;
use log::Level;

extern crate nalgebra as na;

use std::cell::RefCell;
use std::rc::Rc;

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

    let sphere = Box::new(object::Sphere::new(1.0));
    let sphere = Box::new(object::Scale::new(sphere, na::Vector3::new(0.5, 0.8, 1.5)));
    let rbox1 = Box::new(object::RoundBox::new(na::Vector3::new(0.4, 0.6, 1.0), 0.2));
    let rbox1 = Box::new(object::ExactBox::new(na::Vector3::new(0.4, 0.6, 1.0)));
    let rbox1 = Box::new(object::Rotate::from_euler(rbox1, 0.5, 0., 0.));
    let diff = Box::new(object::Difference::new(vec![rbox1, sphere])?);
    let rbox2 = Box::new(object::RoundBox::new(na::Vector3::new(1.0, 0.4, 0.6), 0.2));
    let rbox2 = Box::new(object::Translate::new(rbox2, na::Vector3::new(1., 1., 1.)));
    let object = object::Union::new(vec![diff, rbox2])?;
    let shader = renderer::generate_renderer_shader(&object);
    info!("setting shader:\n{}", shader);
    shader_canvas.borrow_mut().set_shader(&shader)?;

    let world_transform = Rc::new(RefCell::new(na::Matrix4::<f32>::identity()));
    shader_canvas
        .borrow()
        .uniform_matrix4fv("iWorldTransform", world_transform.borrow().as_slice());

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
            let mut world_transform = world_transform.borrow_mut();
            world_transform.prepend_translation_mut(&na::Vector3::new(
                0.,
                0.,
                (-event.delta_y() / 100.0) as f32,
            ));
            let shader_canvas = shader_canvas.borrow();
            shader_canvas
                .uniform_matrix4fv("iWorldTransform", world_transform.transpose().as_slice());
            shader_canvas.draw();
        });
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let shader_canvas = shader_canvas.clone();
        let world_transform = world_transform.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            let mut world_transform = world_transform.borrow_mut();
            match event.buttons() {
                1 => {
                    *world_transform *= na::Matrix4::<f32>::from_euler_angles(
                        event.movement_y() as f32 / 100.0,
                        event.movement_x() as f32 / 100.0,
                        0.,
                    )
                }

                4 => world_transform.prepend_translation_mut(&na::Vector3::new(
                    -event.movement_x() as f32 / 100.0,
                    event.movement_y() as f32 / 100.0,
                    0.,
                )),

                _ => return,
            }
            let shader_canvas = shader_canvas.borrow();
            shader_canvas
                .uniform_matrix4fv("iWorldTransform", world_transform.transpose().as_slice());
            shader_canvas.draw();
        });
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    shader_canvas.borrow().draw();

    Ok(())
}
