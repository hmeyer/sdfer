use super::Primitive;
use anyhow::{bail, Result};

pub trait SdfMin: Clone {
    fn function_body(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> String;
}

#[derive(Clone)]
pub struct MinDefault;

impl SdfMin for MinDefault {
    fn function_body(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> String {
        let min_exps = children
            .iter()
            .map(|c| format!("m = min(m, {});", c.expression(p, shared_code)))
            .collect::<Vec<_>>()
            .join("\n    ");
        format!(
            "float m = 1000000.0;
    {min_exps}
    return m;",
            min_exps = min_exps
        )
    }
}
