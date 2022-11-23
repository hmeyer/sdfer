use super::Primitive;
use anyhow::{bail, Result};

#[derive(Clone)]
pub enum BooleanKind {
    Default,
    Polynomial(f32),               // Supports only two primitives.
    CubicPolynomial(f32),          // Supports only two primitives.
    Root(f32),                     // Supports only two primitives.
    Exponential(f32),              // smoothness = 10
    Chamfer(f32),                  // Supports only two primitives.
    Stairs { d: f32, num: usize }, // Supports only two primitives.
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
            BooleanKind::Stairs { d: _, num: _ } => format!("opStairsMin{}", num_primitives),
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
            BooleanKind::Stairs { d: s, num: n } => format!(", {:.8}, {:.1}", s, *n as f32),
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
            BooleanKind::Stairs { d: _, num: _ } => {
                make_stairs_min_function(&self.function_name(num_primitives), num_primitives)
            }
        }
    }
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        negate: bool,
        children: &[Box<dyn Primitive>],
    ) -> String {
        let local_p = "p";
        shared_code.push(self.make_function(children.len()).unwrap());
        let child_exps = (children.iter())
            .map(|c| c.expression(local_p, shared_code))
            .collect::<Vec<_>>()
            .join(", ");
        let boolean_exp = format!(
            "{negate}{fn_name}({child_exps}{extra})",
            negate = if negate { "-" } else { "" },
            fn_name = self.function_name(children.len()),
            child_exps = child_exps,
            extra = self.make_extra_params()
        );
        let boolean_name = format!("Boolean{}", shared_code.len());
        shared_code.push(format!(
            "float {}(vec3 {}) {{
return {};
}}",
            boolean_name, local_p, boolean_exp
        ));
        format!("{}({})", boolean_name, p)
    }
}

fn make_default_min_function(function_name: &str, params: &[String]) -> Result<String> {
    let min_exps = params[1..]
        .iter()
        .map(|a| format!("{} = min({}, {});", params[0], params[0], a))
        .collect::<Vec<_>>()
        .join("\n    ");
    Ok(format!(
        "
float {}(float {}) {{
    {}
    return {};
}}
",
        function_name,
        params.join(", float "),
        min_exps,
        params[0]
    ))
}

fn make_polynomial_min_function(function_name: &str, num_primitives: usize) -> Result<String> {
    if num_primitives != 2 {
        bail!(
            "Polynomial min requires exactly two arguments (got {}).",
            num_primitives
        );
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
        bail!(
            "Cubic polynomial min requires exactly two arguments (got {}).",
            num_primitives
        );
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
        bail!(
            "Root min requires exactly two arguments (got {}).",
            num_primitives
        );
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
        bail!(
            "Chamfer min requires exactly two arguments (got {}).",
            num_primitives
        );
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
        bail!(
            "Stairs min requires exactly two arguments (got {}).",
            num_primitives
        );
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
        bail!(
            "Exponential min requires at least two arguments (got {}).",
            params.len()
        );
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
    fn new_maybe_negate(children: Vec<Box<dyn Primitive>>, negate: bool) -> Result<Box<Boolean>> {
        if children.len() < 2 {
            bail!(
                "Boolean requires at least 2 children (got only {}).",
                children.len()
            );
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
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> String {
        self.kind
            .expression(p, shared_code, self.negate, &self.children)
    }
}

#[derive(Clone)]
struct Negation {
    child: Box<dyn Primitive>,
}

impl Primitive for Negation {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> String {
        format!("-({})", self.child.expression(p, shared_code))
    }
}
