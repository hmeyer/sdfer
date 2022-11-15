use crate::primitive::Primitive;
use std::collections::HashSet;
use anyhow::{Result, bail};

#[derive(Clone)]
pub enum BooleanKind {
    Default,
    Polynomial(f32),      // Supports only two primitives.
    CubicPolynomial(f32), // Supports only two primitives.
    Root(f32),            // Supports only two primitives.
    Exponential(f32),     // smoothness = 10
    Chamfer(f32),
    Stairs(f32, usize),
}

impl BooleanKind {
    fn function_name(&self, num_primitives: usize) -> String {
        match self {
            BooleanKind::Default => format!("opMin{}", num_primitives),
            BooleanKind::Polynomial(_) => format!("opSmoothMinPolynomial{}", num_primitives),
            BooleanKind::CubicPolynomial(_) => {
                format!("opSmoothMinCubicPolynomial{}", num_primitives)
            }
            BooleanKind::Root(_) => format!("opSmoothMinRoot{}", num_primitives),
            BooleanKind::Exponential(_) => format!("opSmoothMinExponential{}", num_primitives),
            BooleanKind::Chamfer(_) => format!("opChamferMin{}", num_primitives),
            BooleanKind::Stairs(_, _) => format!("opStairsMin{}", num_primitives),
        }
    }
    fn make_extra_params(&self) -> String {
        match self {
            BooleanKind::Default => String::new(),
            BooleanKind::Polynomial(s) => format!(", {:.8}", s),
            BooleanKind::CubicPolynomial(s) => format!(", {:.8}", s),
            BooleanKind::Root(s) => format!(", {:.8}", s),
            BooleanKind::Exponential(s) => format!(", {:.8}", s),
            BooleanKind::Chamfer(s) => format!(", {:.8}", s),
            BooleanKind::Stairs(s, n) => format!(", {:.8}, {:.1}", s, *n as f32),
        }
    }
    fn make_params(num_primitives: usize) -> Vec<String> {
        (0..num_primitives).map(|i| format!("d{}", i)).collect()
    }
    fn make_function(&self, num_primitives: usize) -> Result<String> {
        if num_primitives < 2 {
            bail!("Boolean function needs at least to primitives.");
        }
        match self {
            BooleanKind::Default => make_default_min_function(
                &self.function_name(num_primitives),
                &BooleanKind::make_params(num_primitives),
            ),
            BooleanKind::Polynomial(_) => {
                make_polynomial_min_function(&self.function_name(num_primitives), num_primitives)
            }
            BooleanKind::CubicPolynomial(_) => make_cubic_polynomial_min_function(
                &self.function_name(num_primitives),
                num_primitives,
            ),
            BooleanKind::Root(_) => {
                make_root_min_function(&self.function_name(num_primitives), num_primitives)
            }
            BooleanKind::Exponential(_) => make_exponential_min_function(
                &self.function_name(num_primitives),
                &BooleanKind::make_params(num_primitives),
            ),
            BooleanKind::Chamfer(_) => {
                make_chamfer_min_function(&self.function_name(num_primitives), num_primitives)
            }
            BooleanKind::Stairs(_, _) => {
                make_stairs_min_function(&self.function_name(num_primitives), num_primitives)
            }
        }
    }
}

fn make_default_min_function(function_name: &str, params: &[String]) -> Result<String> {
    let expr_begin = params[0..params.len() - 1]
        .iter()
        .map(|p| format!("min({}", p))
        .collect::<Vec<_>>()
        .join(", ");
    let expr_end = String::from_utf8(vec![b')'; params.len() - 1]).unwrap();
    Ok(format!(
        "
float {}(float {}) {{
return {expr_begin}, {last_param}{expr_end};
}}
",
        function_name,
        params.join(", float "),
        expr_begin = expr_begin,
        last_param = params[params.len() - 1],
        expr_end = expr_end
    ))
}

fn make_polynomial_min_function(
    function_name: &str,
    num_primitives: usize,
) -> Result<String> {
    if num_primitives != 2 {
        bail!("Polynomial min requires exactly two arguments (got {}).", num_primitives);
    }
    Ok(format!(
        "
float {name}(float d1, float d2, float k) {{
    float h = max(k-abs(d1-d2),0.0);
    return min(d1, d2) - h*h*0.25/k;
}}
",
        name = function_name
    ))
}

fn make_cubic_polynomial_min_function(
    function_name: &str,
    num_primitives: usize,
) -> Result<String> {
    if num_primitives != 2 {
        bail!("Cubic polynomial min requires exactly two arguments (got {}).", num_primitives);
    }
    Ok(format!(
        "
float {name}(float d1, float d2, float k) {{
    float h = max(k-abs(d1-d2),0.0) / k;
    return min(d1, d2) - h*h*h*k*(1./6.);
}}
",
        name = function_name
    ))
}

fn make_root_min_function(function_name: &str, num_primitives: usize) -> Result<String> {
    if num_primitives != 2 {
        bail!("Root min requires exactly two arguments (got {}).", num_primitives);
    }
    Ok(format!(
        "
float {name}(float d0, float d1, float k) {{
    float h = d0 - d1;
    return 0.5 * ((d0 + d1) - sqrt(h *h + k));
}}
",
        name = function_name
    ))
}

fn make_chamfer_min_function(function_name: &str, num_primitives: usize) -> Result<String> {
    if num_primitives != 2 {
        bail!("Chamfer min requires exactly two arguments (got {}).", num_primitives);
    }
    Ok(format!(
        "
float {name}(float d0, float d1, float k) {{
    return min(min(d0, d1), (d0 - k + d1) * sqrt(0.5));
}}
",
        name = function_name
    ))
}

fn make_stairs_min_function(function_name: &str, num_primitives: usize) -> Result<String> {
    if num_primitives != 2 {
        bail!("Stairs min requires exactly two arguments (got {}).", num_primitives);
    }
    Ok(format!(
        "
float {name}(float d0, float d1, float k, float n) {{
    float s = k / n;
    float u = d1 - k;
    return min(min(d0, d1), 0.5 * (u + d0 + abs((mod(u - d0 + s, 2. * s)) - s)));
}}
",
        name = function_name,
    ))
}

fn make_exponential_min_function(function_name: &str, params: &[String]) -> Result<String> {
    if params.len() < 2 {
        bail!("Exponential min requires at least two arguments (got {}).", params.len());
    }
    Ok(format!(
        "
float {name}(float {params}, float k) {{
    float res = {expr};
    return -log2(res) / k;
}}
",
        name = function_name,
        expr = params
            .iter()
            .map(|p| format!("exp2(-k * {})", p))
            .collect::<Vec<_>>()
            .join(" + "),
        params = params.join(", float "),
    ))
}

#[derive(Clone)]
pub struct Boolean {
    children: Vec<Box<dyn Primitive>>,
    kind: BooleanKind,
    negate: bool,
}

impl Boolean {
    fn new_maybe_negate(
        children: Vec<Box<dyn Primitive>>,
        negate: bool,
    ) -> Result<Box<Boolean>> {
        if children.len() < 2 {
            bail!("Boolean requires at least 2 children (got only {}).", children.len());
        }
        Ok(Box::new(Boolean {
            children,
            kind: BooleanKind::Default,
            negate,
        }))
    }
    pub fn new_union(children: Vec<Box<dyn Primitive>>) -> Result<Box<Boolean>> {
        Boolean::new_maybe_negate(children, false)
    }
    pub fn new_intersection(children: Vec<Box<dyn Primitive>>) -> Result<Box<Boolean>> {
        let neg_children = children
            .into_iter()
            .map(|child| Box::new(Negation { child: child }) as Box<dyn Primitive>)
            .collect();
        Boolean::new_maybe_negate(neg_children, true)
    }
    pub fn new_difference(mut children: Vec<Box<dyn Primitive>>) -> Result<Box<Boolean>> {
        if children.len() == 0 {
            bail!("Difference requires at least one child (got none).");
        }
        let mut new_children = vec![children.swap_remove(0)];
        while !children.is_empty() {
            let child = children.pop().unwrap();
            new_children.push(Box::new(Negation { child }));
        }
        Boolean::new_intersection(new_children)
    }
    pub fn set_kind(&mut self, kind: BooleanKind) -> Result<()> {
        // Try to make a function, for implicit error checking.
        _ = kind.make_function(self.children.len())?;
        self.kind = kind;
        Ok(())
    }
}

impl Primitive for Boolean {
    fn static_code(&self) -> HashSet<String> {
        let mut code_set = HashSet::new();
        for child in &self.children {
            code_set.extend(child.static_code());
        }
        code_set.insert(self.kind.make_function(self.children.len()).unwrap());
        code_set
    }
    fn expression(&self, p: &str) -> String {
        let child_exps = (self.children.iter())
            .map(|c| c.expression(p))
            .collect::<Vec<_>>()
            .join(", ");
        return format!(
            "{negate}{fn_name}({child_exps}{extra})",
            negate = if self.negate { "-" } else { "" },
            fn_name = self.kind.function_name(self.children.len()),
            child_exps = child_exps,
            extra = self.kind.make_extra_params()
        );
    }
}

#[derive(Clone)]
struct Negation {
    child: Box<dyn Primitive>,
}

impl Primitive for Negation {
    fn static_code(&self) -> HashSet<String> {
        self.child.static_code()
    }
    fn expression(&self, p: &str) -> String {
        format!("-({})", self.child.expression(p))
    }
}
