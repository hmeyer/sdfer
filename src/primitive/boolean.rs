use crate::primitive::Primitive;
use std::collections::HashSet;

enum UnionKind {
    Default,
    Polynomial,      // Supports only two primitives.
    CubicPolynomial, // Supports only two primitives.
    Root,            // Supports only two primitives.
    Exponential,     // smoothness = 10
    Chamfer,
    Stairs(usize),
}

impl UnionKind {
    fn function_name(&self, num_primitives: usize) -> String {
        match self {
            UnionKind::Default => format!("opMin{}", num_primitives),
            UnionKind::Polynomial => format!("opSmoothMinPolynomial{}", num_primitives),
            UnionKind::CubicPolynomial => format!("opSmoothMinCubicPolynomial{}", num_primitives),
            UnionKind::Root => format!("opSmoothMinRoot{}", num_primitives),
            UnionKind::Exponential => format!("opSmoothMinExponential{}", num_primitives),
            UnionKind::Chamfer => format!("opChamferMin{}", num_primitives),
            UnionKind::Stairs(n) => format!("op{}StairsMin{}", n, num_primitives),
        }
    }
    fn make_params(num_primitives: usize) -> Vec<String> {
        (0..num_primitives).map(|i| format!("d{}", i)).collect()
    }
    fn make_function(&self, num_primitives: usize) -> String {
        if num_primitives < 2 {
            panic!("Union function needs at least to primitives.")
        }
        match self {
            UnionKind::Default => make_default_min_function(
                &self.function_name(num_primitives),
                &UnionKind::make_params(num_primitives),
            ),
            UnionKind::Polynomial => {
                make_polynomial_min_function(&self.function_name(num_primitives), num_primitives)
            }
            UnionKind::CubicPolynomial => {
                make_cubic_polynomial_min_function(&self.function_name(num_primitives), num_primitives)
            }
            UnionKind::Root => {
                make_root_min_function(&self.function_name(num_primitives), num_primitives)
            }
            UnionKind::Exponential => make_exponential_min_function(
                &self.function_name(num_primitives),
                &UnionKind::make_params(num_primitives),
            ),
            UnionKind::Chamfer => {
                make_chamfer_min_function(&self.function_name(num_primitives), num_primitives)
            }
            UnionKind::Stairs(n) => {
                make_stairs_min_function(&self.function_name(num_primitives), num_primitives, *n)
            }
        }
    }
}

fn make_default_min_function(function_name: &str, params: &[String]) -> String {
    let expr_begin = params[0..params.len() - 1]
        .iter()
        .map(|p| format!("min({}", p))
        .collect::<Vec<_>>()
        .join(", ");
    let expr_end = String::from_utf8(vec![b')'; params.len() - 1]).unwrap();
    format!(
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
    )
}

fn make_polynomial_min_function(function_name: &str, num_primitives: usize) -> String {
    if num_primitives != 2 {
        panic!("Polynomial min requires exactly two arguments.");
    }
    format!(
        "
float {name}(float d1, float d2, float k) {{
    float h = max(k-abs(d1-d2),0.0);
    return min(d1, d2) - h*h*0.25/k;
}}
",
        name = function_name
    )
}

fn make_cubic_polynomial_min_function(function_name: &str, num_primitives: usize) -> String {
    if num_primitives != 2 {
        panic!("Cubic polynomial min requires exactly two arguments.");
    }
    format!(
        "
float {name}(float d1, float d2, float k) {{
    float h = max(k-abs(d1-d2),0.0) / k;
    return min(d1, d2) - h*h*h*k*(1./6.);
}}
",
        name = function_name
    )
}

fn make_root_min_function(function_name: &str, num_primitives: usize) -> String {
    if num_primitives != 2 {
        panic!("Cubic polynomial min requires exactly two arguments.");
    }
    format!(
        "
float {name}(float d0, float d1, float k) {{
    float h = d0 - d1;
    return 0.5 * ((d0 + d1) - sqrt(h *h + k));
}}
",
        name = function_name
    )
}

fn make_chamfer_min_function(function_name: &str, num_primitives: usize) -> String {
    if num_primitives != 2 {
        panic!("Chamfer min requires exactly two arguments.");
    }
    format!(
        "
float {name}(float d0, float d1, float k) {{
    return min(min(d0, d1), (d0 - k + d1) * sqrt(0.5));
}}
",
        name = function_name
    )
}

fn make_stairs_min_function(function_name: &str, num_primitives: usize, n: usize) -> String {
    if num_primitives != 2 {
        panic!("Round min requires exactly two arguments.");
    }
    format!(
        "
float {name}(float d0, float d1, float k) {{
    float n = {n}.;
    float s = k / n;
    float u = d1 - k;
    return min(min(d0, d1), 0.5 * (u + d0 + abs((mod(u - d0 + s, 2. * s)) - s)));
}}
",
        name = function_name,
        n = n,
    )
}

fn make_exponential_min_function(function_name: &str, params: &[String]) -> String {
    if params.len() < 2 {
        panic!("Exponential min requires at least two arguments.");
    }
    format!(
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
    )
}

pub struct Union {
    children: Vec<Box<dyn Primitive>>,
    smoothness: f32,
    kind: UnionKind,
}

impl Union {
    pub fn new(children: Vec<Box<dyn Primitive>>) -> Result<Union, String> {
        Union::new_with_smoothness(children, 0.)
    }
    pub fn new_with_smoothness(
        children: Vec<Box<dyn Primitive>>,
        smoothness: f32,
    ) -> Result<Union, String> {
        if children.len() < 2 {
            return Err("Union requires at least 2 children.".to_string());
        }
        let kind = if smoothness == 0.0 {
            UnionKind::Default
        } else {
            UnionKind::Stairs(4)
        };
        Ok(Union {
            children,
            smoothness,
            kind,
        })
    }
}

impl Primitive for Union {
    fn static_code(&self) -> HashSet<String> {
        let mut code_set = HashSet::new();
        for child in &self.children {
            code_set.extend(child.static_code());
        }
        code_set.insert(self.kind.make_function(self.children.len()));
        code_set
    }
    fn expression(&self, p: &str) -> String {
        let child_exps = (self.children.iter())
            .map(|c| c.expression(p))
            .collect::<Vec<_>>()
            .join(", ");
        let smoothness_exp = if self.smoothness > 0. {
            format!(", {:.8}", self.smoothness)
        } else {
            String::new()
        };
        return format!(
            "{}({}{})",
            self.kind.function_name(self.children.len()),
            child_exps,
            smoothness_exp
        );
    }
}

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

pub struct Intersection {
    inner_union: Union,
}

impl Intersection {
    pub fn new(children: Vec<Box<dyn Primitive>>) -> Result<Intersection, String> {
        Intersection::new_with_smoothness(children, 0.)
    }
    pub fn new_with_smoothness(
        mut children: Vec<Box<dyn Primitive>>,
        smoothness: f32,
    ) -> Result<Intersection, String> {
        let neg_children = children
            .into_iter()
            .map(|child| Box::new(Negation { child: child }) as Box<dyn Primitive>)
            .collect();
        Ok(Intersection {
            inner_union: Union::new_with_smoothness(neg_children, smoothness)?,
        })
    }
}

impl Primitive for Intersection {
    fn static_code(&self) -> HashSet<String> {
        self.inner_union.static_code()
    }
    fn expression(&self, p: &str) -> String {
        format!("-({})", self.inner_union.expression(p))
    }
}

pub struct Difference {}

impl Difference {
    pub fn new(children: Vec<Box<dyn Primitive>>) -> Result<Intersection, String> {
        Difference::new_with_smoothness(children, 0.)
    }
    pub fn new_with_smoothness(
        mut children: Vec<Box<dyn Primitive>>,
        smoothness: f32,
    ) -> Result<Intersection, String> {
        if children.len() == 0 {
            return Err("Difference requires at least one child.".to_string());
        }
        let mut new_children = vec![children.swap_remove(0)];
        while !children.is_empty() {
            let child = children.pop().unwrap();
            new_children.push(Box::new(Negation { child }));
        }
        Intersection::new_with_smoothness(new_children, smoothness)
    }
}
