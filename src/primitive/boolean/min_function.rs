use super::Primitive;
use anyhow::{bail, Result};

pub trait MinFunction: Clone {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String>;
}

#[derive(Clone)]
pub struct MinDefault;

impl MinFunction for MinDefault {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String> {
        let local_p = "p";
        let min_exps = children
            .iter()
            .map(|c| c.expression(local_p, shared_code))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|c_expr| format!("    m = min(m, {});", c_expr))
            .collect::<Vec<_>>()
            .join("\n");
        let function_name = format!("MinDefault{}", shared_code.len());
        shared_code.push(format!(
            "
float {function_name}(vec3 {local_p}) {{
    float m = 1e10;
{min_exps}
    return m;
}}",
            function_name = function_name,
            local_p = local_p,
            min_exps = min_exps
        ));
        Ok(format!("{}({})", function_name, p))
    }
}
