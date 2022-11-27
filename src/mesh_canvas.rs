use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

pub struct MeshCanvas {
    canvas: web_sys::HtmlCanvasElement,
    context: WebGl2RenderingContext,
}

static VERTICES: &'static [f32] = &[
    -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 0.0,
];

impl MeshCanvas {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<MeshCanvas, JsValue> {
        let context: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .map_err(|e| format!("Cannot get webgl2 context: {:?}", e.as_string()))?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .or(Err(String::from(
                "Cannot cast context to WebGl2RenderingContext",
            )))?;

        let mut result = MeshCanvas { canvas, context };
        // result.set_shader(DEFAULT_SHADER)?;
        Ok(result)
    }

    pub fn draw(&self) {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.context.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            (VERTICES.len() / 3) as i32,
        );
    }
}
