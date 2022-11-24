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

#[derive(Clone)]
pub struct MinExponential {
    k: f32,
}

impl MinExponential {
    pub fn new(k: f32) -> Self {
        MinExponential { k }
    }
}

impl MinFunction for MinExponential {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String> {
        if children.len() < 2 {
            bail!(
                "MinExponential requires at least two arguments (got {}).",
                children.len()
            );
        }
        let local_p = "p";
        let min_exps = children
            .iter()
            .map(|c| c.expression(local_p, shared_code))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|c_expr| format!("    res += exp2(-{:.8} * ({}));", self.k, c_expr))
            .collect::<Vec<_>>()
            .join("\n");
        let function_name = format!("MinExponential{}", shared_code.len());
        shared_code.push(format!(
            "
float {function_name}(vec3 {local_p}) {{
    float res = 0.0;
{min_exps}
    return -log2(res) / {k:.8};
}}",
            function_name = function_name,
            local_p = local_p,
            min_exps = min_exps,
            k = self.k,
        ));
        Ok(format!("{}({})", function_name, p))
    }
}
