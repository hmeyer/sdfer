use super::script_engine::ScriptEngine;
use super::Primitive;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlTextAreaElement};

pub struct ScriptUI {
    primitive: Option<Box<dyn Primitive>>,
}

impl ScriptUI {
    pub fn new<E: ScriptEngine + 'static>(
        code_area: HtmlTextAreaElement,
        output_area: HtmlTextAreaElement,
        run_button: HtmlButtonElement,
        mesh_button: HtmlButtonElement,
        mut engine: E,
        on_new_object_callback: impl Fn(&(dyn Primitive)) + 'static,
    ) -> Result<Rc<RefCell<ScriptUI>>, JsValue> {
        connect_output_to_engine(&output_area, &mut engine);
        let engine = engine;
        let ui = Rc::new(RefCell::new(ScriptUI { primitive: None }));
        register_run_callback(&ui, &code_area, &run_button, engine, on_new_object_callback)?;
        register_mesh_callback(&ui, &output_area, &mesh_button)?;
        Ok(ui)
    }
}

fn connect_output_to_engine<E: ScriptEngine + 'static>(
    output_area: &HtmlTextAreaElement,
    engine: &mut E,
) {
    let output_area = output_area.clone();
    engine.on_print(move |s| {
        let mut output = output_area.value();
        output.push_str(s);
        output.push('\n');
        output_area.set_value(&output);
    });
}

fn register_run_callback<E: ScriptEngine + 'static>(
    ui: &Rc<RefCell<ScriptUI>>,
    code_area: &HtmlTextAreaElement,
    run_button: &HtmlButtonElement,
    engine: E,
    on_new_object_callback: impl Fn(&(dyn Primitive)) + 'static,
) -> Result<(), JsValue> {
    let code_area = code_area.clone();
    let ui = ui.clone();
    let closure = move |_event: web_sys::MouseEvent| match engine.eval(&code_area.value()) {
        Ok(primitive) => {
            ui.borrow_mut().primitive = Some(primitive.clone());
            on_new_object_callback(&*primitive)
        }
        Err(e) => info!("was err: {:?}", e),
    };
    // Run closure once to create first object.
    closure(web_sys::MouseEvent::new("mock")?);
    let closure = Closure::<dyn FnMut(_)>::new(closure);
    run_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_mesh_callback(
    ui: &Rc<RefCell<ScriptUI>>,
    output_area: &HtmlTextAreaElement,
    mesh_button: &HtmlButtonElement,
) -> Result<(), JsValue> {
    let ui = ui.clone();
    let output_area = output_area.clone();
    let closure = move |_event: web_sys::MouseEvent| match &ui.borrow().primitive {
        Some(p) => {
            let (vertices, indices) = mesh_primitive(p);
            let mut output = output_area.value();
            output.push_str(&format!(
                "Got {} vertices and {} triangles.\n",
                vertices.len() / 6,
                indices.len() / 3
            ));
            output_area.set_value(&output);
        }
        None => {
            let mut output = output_area.value();
            output.push_str("No object found. (Click Run first?)\n");
            output_area.set_value(&output);
        }
    };
    let closure = Closure::<dyn FnMut(_)>::new(closure);
    mesh_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

pub struct PrimitiveSource {
    pub primitive: Box<dyn Primitive>,
}

impl isosurface::source::Source for PrimitiveSource {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        self.primitive.eval(na::Vector3::new(x, y, z))
    }
}

fn mesh_primitive(p: &Box<dyn Primitive>) -> (Vec<f32>, Vec<u32>) {
    let source = Box::new(PrimitiveSource {
        primitive: p.clone(),
    });
    let source = isosurface::source::CentralDifference::new(source);
    let mut marching_cubes =
        isosurface::linear_hashed_marching_cubes::LinearHashedMarchingCubes::new(3);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    marching_cubes.extract_with_normals(&source, &mut vertices, &mut indices);
    (vertices, indices)
}
