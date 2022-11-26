use super::renderer;
use super::Primitive;
use shadercanvas::ShaderCanvas;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub struct RenderCanvas {
    shader_canvas: Rc<RefCell<ShaderCanvas>>,
}

impl RenderCanvas {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<RenderCanvas, JsValue> {
        let shader_canvas = Rc::new(RefCell::new(ShaderCanvas::new(canvas.clone())?));
        let world_transform = Rc::new(RefCell::new(na::Matrix4::<f32>::identity()));
        {
            let clone = shader_canvas.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
                clone.borrow().draw();
            });
            canvas
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
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
            canvas
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        Ok(RenderCanvas { shader_canvas })
    }
    pub fn draw(&self) {
        self.shader_canvas.borrow().draw();
    }
    pub fn set_primtive(&self, prim: &dyn Primitive) -> Result<(), JsValue> {
        let shader =
            renderer::generate_renderer_shader(prim).map_err(|e| JsValue::from(e.to_string()))?;
        debug!("setting shader:\n{}", shader);
        let mut shader_canvas = self.shader_canvas.borrow_mut();
        shader_canvas.set_shader(&shader)?;
        // Also reset the world transform.
        let world_transform = na::Matrix4::<f32>::identity();
        shader_canvas.uniform_matrix4fv("iWorldTransform", world_transform.as_slice());
        Ok(())
    }
}
