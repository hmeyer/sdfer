use anyhow::{bail, Result};
use js_sys;
use std::f32::consts::PI;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlUniformLocation,
};

pub struct Mesh {
    vertices: Vec<f32>,
    normals: Vec<f32>,
}

impl Mesh {
    pub fn new(vertices: &[f32], normals: &[f32], indices: &[u32]) -> Result<Mesh> {
        if indices.len() % 3 != 0 {
            bail!("indices.len() is not divisible by 3.")
        }
        let vertices = indices
            .iter()
            .map(|i| i * 3)
            .map(|i| [i, i + 1, i + 2])
            .flatten()
            .map(|i| vertices[i as usize])
            .collect();
        let normals = indices
            .iter()
            .map(|i| i * 3)
            .map(|i| [i, i + 1, i + 2])
            .flatten()
            .map(|i| normals[i as usize])
            .collect();
        Ok(Mesh { vertices, normals })
    }
}

pub struct MeshCanvas {
    canvas: web_sys::HtmlCanvasElement,
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    position_buffer: WebGlBuffer,
    normal_buffer: WebGlBuffer,
    position_location: i32,
    normal_location: i32,
    world_view_projection_location: Option<WebGlUniformLocation>,
    world_inverse_transpose_location: Option<WebGlUniformLocation>,
    color_location: Option<WebGlUniformLocation>,
    reverse_light_direction_location: Option<WebGlUniformLocation>,
}

static VERTEX_SHADER: &'static str = r##" 
attribute vec4 a_position;
attribute vec3 a_normal;

uniform mat4 u_worldViewProjection;
uniform mat4 u_worldInverseTranspose;

varying vec3 v_normal;

void main() {
  // Multiply the position by the matrix.
  gl_Position = u_worldViewProjection * a_position;

  // orient the normals and pass to the fragment shader
  v_normal = mat3(u_worldInverseTranspose) * a_normal;
}
"##;

static FRAGMENT_SHADER: &'static str = r##"
precision mediump float;

// Passed in from the vertex shader.
varying vec3 v_normal;

uniform vec3 u_reverseLightDirection;
uniform vec4 u_color;

void main() {
  // because v_normal is a varying it's interpolated
  // so it will not be a unit vector. Normalizing it
  // will make it a unit vector again
  vec3 normal = normalize(v_normal);

  float light = dot(normal, u_reverseLightDirection);

  gl_FragColor = u_color;

  // Lets multiply just the color portion (not the alpha)
  // by the light
  gl_FragColor.rgb *= light;
}
"##;

impl MeshCanvas {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<MeshCanvas, JsValue> {
        let gl: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .map_err(|e| format!("Cannot get webgl2 gl: {:?}", e.as_string()))?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .or(Err(String::from(
                "Cannot cast gl to WebGl2RenderingContext",
            )))?;

        let vert_shader =
            compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, VERTEX_SHADER)?;

        let frag_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            FRAGMENT_SHADER,
        )?;

        let program = link_program(&gl, &vert_shader, &frag_shader)?;
        // let locations = get_locations(&gl, &program);
        // let mesh = Mesh::new(_VERTICES, _NORMALS, _INDICES).map_err(|e| e.to_string())?;

        // let buffers = init_buffers(&gl, &mesh)?;

        let position_location = gl.get_attrib_location(&program, "a_position");
        let normal_location = gl.get_attrib_location(&program, "a_normal");
        let world_view_projection_location =
            gl.get_uniform_location(&program, "u_worldViewProjection");
        let world_inverse_transpose_location =
            gl.get_uniform_location(&program, "u_worldInverseTranspose");
        let color_location = gl.get_uniform_location(&program, "u_color");
        let reverse_light_direction_location =
            gl.get_uniform_location(&program, "u_reverseLightDirection");

        let position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));
        set_geometry(&gl);

        let normal_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&normal_buffer));
        set_normals(&gl);

        // let color_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        // gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));
        // set_colors(&gl);

        let mut result = MeshCanvas {
            canvas,
            gl,
            program,
            position_buffer,
            normal_buffer,
            position_location,
            normal_location,
            world_view_projection_location,
            world_inverse_transpose_location,
            color_location,
            reverse_light_direction_location,
        };
        // result.set_shader(DEFAULT_SHADER)?;
        Ok(result)
    }

    pub fn draw(&self) {
        self.gl.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );
        self.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );
        // self.gl.enable(WebGl2RenderingContext::CULL_FACE); // Enable backface culling.
        // self.gl.enable(WebGl2RenderingContext::DEPTH_TEST); // Enable depth testing
        // self.gl.clear_color(0.5, 0.5, 0.5, 1.0);
        // self.gl.clear_depth(1.0); // Clear everything
        // self.gl.depth_func(WebGl2RenderingContext::LEQUAL); // Near things obscure far things
        self.gl.use_program(Some(&self.program));

        {
            self.gl
                .enable_vertex_attrib_array(self.position_location as u32);
            self.gl.bind_buffer(
                WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&self.position_buffer),
            );

            let numComponents = 3;
            let normalize = false;
            let stride = 0;
            let offset = 0;

            self.gl.vertex_attrib_pointer_with_i32(
                self.position_location as u32,
                numComponents,
                WebGl2RenderingContext::FLOAT,
                normalize,
                stride,
                offset,
            );
        }

        {
            self.gl
                .enable_vertex_attrib_array(self.normal_location as u32);
            self.gl.bind_buffer(
                WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&self.normal_buffer),
            );

            let numComponents = 3;
            let normalize = false;
            let stride = 0;
            let offset = 0;

            self.gl.vertex_attrib_pointer_with_i32(
                self.normal_location as u32,
                numComponents,
                WebGl2RenderingContext::FLOAT,
                normalize,
                stride,
                offset,
            );
        }

        let fieldOfView = (60. * PI) / 180.; // in radians
        let aspect = self.canvas.width() as f32 / self.canvas.height() as f32;
        let zNear = 1.0;
        let zFar = 2000.0;
        let projection_matrix = glm::perspective_fov(
            fieldOfView,
            self.canvas.width() as f32,
            self.canvas.height() as f32,
            zNear,
            zFar,
        );
        let eye = glm::vec3(200., 50., -500.);
        let target = glm::vec3(-5.6, -13.1, 0.);
        let up = glm::vec3(0., 1., 0.);
        let camera_matrix = glm::look_at(&eye, &target, &up);
        let view_matrix = camera_matrix.try_inverse().unwrap();
        let view_projection_matrix = projection_matrix * view_matrix;
        let world_matrix = glm::identity::<f32, 4>();
        let world_view_projection_matrix = view_projection_matrix * world_matrix;
        let world_inverse_matrix = world_matrix.try_inverse().unwrap();
        let world_inverse_transpose_matrix = world_inverse_matrix.transpose();

        self.gl.uniform_matrix4fv_with_f32_array(
            self.world_view_projection_location.as_ref(),
            false,
            world_view_projection_matrix.as_slice(),
        );
        self.gl.uniform_matrix4fv_with_f32_array(
            self.world_inverse_transpose_location.as_ref(),
            false,
            world_inverse_transpose_matrix.as_slice(),
        );
        self.gl
            .uniform4f(self.color_location.as_ref(), 0.2, 1.0, 0.2, 1.0);
        self.gl.uniform3f(
            self.reverse_light_direction_location.as_ref(),
            0.5,
            0.7,
            1.0,
        );

        let count = 16 * 6;
        let offset = 0;
        self.gl
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, offset, count);

        //         // Create a perspective matrix, a special matrix that is
        //         // used to simulate the distortion of perspective in a camera.
        //         // Our field of view is 45 degrees, with a width/height
        //         // ratio that matches the display size of the canvas
        //         // and we only want to see objects between 0.1 units
        //         // and 100 units away from the camera.

        //         let fieldOfView = (45. * PI) / 180.; // in radians
        //         let aspect = self.canvas.width() as f32 / self.canvas.height() as f32;
        //         let zNear = 0.1;
        //         let zFar = 100.0;
        //         let projectionMatrix = na::Matrix4::new_perspective(aspect, fieldOfView, zNear, zFar);

        //         let modelViewMatrix = na::Matrix4::new_translation(&na::Vector3::new(0_f32, 0_f32, 6_f32));
        //         let cubeRotation = 0.4;
        //         let modelViewMatrix = modelViewMatrix
        //             * na::Matrix4::from_euler_angles(0_f32, cubeRotation * 0.7, cubeRotation);

        //         let normalMatrix = modelViewMatrix.try_inverse().unwrap().transpose();

        //         {
        //             self.gl
        //                 .enable_vertex_attrib_array(self.locations.vertex_position as u32);
        //                 self.gl.bind_buffer(
        //                     WebGl2RenderingContext::ARRAY_BUFFER,
        //                     Some(&self.buffers.position),
        //                 );
        //               // Tell WebGL how to pull out the positions from the position
        //   // buffer into the vertexPosition attribute
        //             let numComponents = 3;
        //             let normalize = false;
        //             let stride = 0;
        //             let offset = 0;

        //             self.gl.vertex_attrib_pointer_with_i32(
        //                 self.locations.vertex_position as u32,
        //                 numComponents,
        //                 WebGl2RenderingContext::FLOAT,
        //                 normalize,
        //                 stride,
        //                 offset,
        //             );
        //             }
        // //         {
        // //   self.gl
        // //   .enable_vertex_attrib_array(self.locations.vertex_normal as u32);
        // //   self.gl.bind_buffer(
        // //     WebGl2RenderingContext::ARRAY_BUFFER,
        // //     Some(&self.buffers.normal),
        // // );
        // //   // Tell WebGL how to pull out the normals from
        // //   // the normal buffer into the vertexNormal attribute.
        // //   let numComponents = 3;
        // //           let normalize = false;
        // //           let stride = 0;
        // //           let offset = 0;
        // //           self.gl.vertex_attrib_pointer_with_i32(
        // //               self.locations.vertex_normal as u32,
        // //               numComponents,
        // //               WebGl2RenderingContext::FLOAT,
        // //               normalize,
        // //               stride,
        // //               offset,
        // //           );
        // //         }

        //             self.gl.uniform_matrix4fv_with_f32_array(self.locations.projection_matrix.as_ref(), false, projectionMatrix.as_slice());
        //             self.gl.uniform_matrix4fv_with_f32_array(self.locations.model_view_matrix.as_ref(), false, modelViewMatrix.as_slice());
        //             self.gl.uniform_matrix4fv_with_f32_array(self.locations.normal_matrix.as_ref(), false, normalMatrix.as_slice());

        //     {
        //         let triangle_count = (self.mesh.vertices.len() / 3) as i32;
        //         info!("triangle_count: {}", triangle_count);
        //         let offset = 0;
        //         self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, offset, triangle_count);
        //     }
    }
}

fn set_normals(gl: &WebGl2RenderingContext) {
    let normals = [
        // left column front
        0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, // top rung front
        0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, // middle rung front
        0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, // left column back
        0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, // top rung back
        0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, // middle rung back
        0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, // top
        0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, // top rung right
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, // under top rung
        0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0,
        // between top rung and middle
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, // top of middle rung
        0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, // right of middle rung
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, // bottom of middle rung.
        0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, // right of bottom
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, // bottom
        0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, // left side
        -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0,
    ]
    .iter()
    .map(|x| *x as f32)
    .collect::<Vec<_>>();
    let normal_array = js_sys::Float32Array::from(&normals[..]);
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &normal_array,
        WebGl2RenderingContext::STATIC_DRAW,
    );
}

fn set_geometry(gl: &WebGl2RenderingContext) {
    let geometry = &[
        // left column front
        0, 0, 0, 0, 150, 0, 30, 0, 0, 0, 150, 0, 30, 150, 0, 30, 0, 0, // top rung front
        30, 0, 0, 30, 30, 0, 100, 0, 0, 30, 30, 0, 100, 30, 0, 100, 0, 0,
        // middle rung front
        30, 60, 0, 30, 90, 0, 67, 60, 0, 30, 90, 0, 67, 90, 0, 67, 60, 0,
        // left column back
        0, 0, 30, 30, 0, 30, 0, 150, 30, 0, 150, 30, 30, 0, 30, 30, 150, 30,
        // top rung back
        30, 0, 30, 100, 0, 30, 30, 30, 30, 30, 30, 30, 100, 0, 30, 100, 30, 30,
        // middle rung back
        30, 60, 30, 67, 60, 30, 30, 90, 30, 30, 90, 30, 67, 60, 30, 67, 90, 30, // top
        0, 0, 0, 100, 0, 0, 100, 0, 30, 0, 0, 0, 100, 0, 30, 0, 0, 30, // top rung right
        100, 0, 0, 100, 30, 0, 100, 30, 30, 100, 0, 0, 100, 30, 30, 100, 0, 30,
        // under top rung
        30, 30, 0, 30, 30, 30, 100, 30, 30, 30, 30, 0, 100, 30, 30, 100, 30, 0,
        // between top rung and middle
        30, 30, 0, 30, 60, 30, 30, 30, 30, 30, 30, 0, 30, 60, 0, 30, 60, 30,
        // top of middle rung
        30, 60, 0, 67, 60, 30, 30, 60, 30, 30, 60, 0, 67, 60, 0, 67, 60, 30,
        // right of middle rung
        67, 60, 0, 67, 90, 30, 67, 60, 30, 67, 60, 0, 67, 90, 0, 67, 90, 30,
        // bottom of middle rung.
        30, 90, 0, 30, 90, 30, 67, 90, 30, 30, 90, 0, 67, 90, 30, 67, 90, 0,
        // right of bottom
        30, 90, 0, 30, 150, 30, 30, 90, 30, 30, 90, 0, 30, 150, 0, 30, 150, 30, // bottom
        0, 150, 0, 0, 150, 30, 30, 150, 30, 0, 150, 0, 30, 150, 30, 30, 150, 0, // left side
        0, 0, 0, 0, 0, 30, 0, 150, 30, 0, 0, 0, 0, 150, 30, 0, 150, 0,
    ][..]
        .chunks(3)
        .map(|p| [p[0] - 50, p[1] - 75, p[2] - 15])
        .flatten()
        .map(|x| x as f32)
        .collect::<Vec<_>>();
    let vs = geometry[..]
        .chunks(3)
        .map(|p| glm::vec3(p[0], p[1], p[2]))
        .collect::<Vec<_>>();
    let mean_v = vs.iter().sum::<glm::Vec3>() / vs.len() as f32;
    info!("mean: {:?}", mean_v);
    let geometry_array = js_sys::Float32Array::from(&geometry[..]);
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &geometry_array,
        WebGl2RenderingContext::STATIC_DRAW,
    );
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    gl: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
