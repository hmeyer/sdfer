use super::Primitive;
use std::collections::HashSet;

pub fn generate_renderer_shader(obj: &dyn Primitive) -> String {
    let main_renderer = include_str!("renderer.glsl");
    let mut static_code = HashSet::new();
    let map = format!(
        "
uniform mat4 iWorldTransform;

float map(in vec3 p) {{
    p = (vec4(p, 1) * iWorldTransform).xyz;
    return {};
}}",
        obj.expression("p", &mut static_code)
    );
    let static_code = static_code.iter().fold(String::new(), |mut sum, i| {
        sum.push_str(i);
        sum
    });
    format!("{}\n{}\n{}", static_code, map, main_renderer)
}
