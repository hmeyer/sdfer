use crate::object::Object;

pub fn generate_renderer_shader(obj: &dyn Object) -> String {
    let main_renderer = include_str!("renderer.glsl");
    let map = format!(
        "
uniform mat4 iWorldTransform;

float map(in vec3 p) {{
    p = (vec4(p, 1) * iWorldTransform).xyz;
    return {};
}}",
        obj.expression("p")
    );
    let static_code = obj
        .static_code()
        .iter()
        .fold(String::new(), |sum, i| sum + i);
    format!("{}\n{}\n{}", static_code, map, main_renderer)
}
