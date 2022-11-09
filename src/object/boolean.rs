use crate::object::Object;
use std::collections::HashSet;

enum UnionKind {
    Default,
    Polynomial,      // Supports only two objects.
    CubicPolynomial, // Supports only two objects.
    Root,            // Supports only two objects.
    Exponential,     // smoothness = 10
}

impl UnionKind {
    fn function_name(&self, num_objects: usize) -> String {
        match self {
            UnionKind::Default => format!("opMin{}", num_objects),
            UnionKind::Polynomial => format!("opSmoothMinPolynomial{}", num_objects),
            UnionKind::CubicPolynomial => format!("opSmoothMinCubicPolynomial{}", num_objects),
            UnionKind::Root => format!("opSmoothMinRoot{}", num_objects),
            UnionKind::Exponential => format!("opSmoothMinExponential{}", num_objects),
        }
    }
    fn make_params(num_objects: usize) -> Vec<String> {
        (0..num_objects).map(|i| format!("d{}", i)).collect()
    }
    fn make_function(&self, num_objects: usize) -> String {
        if num_objects < 2 {
            panic!("Union function needs at least to objects.")
        }
        match self {
            UnionKind::Default => make_default_min_function(
                &self.function_name(num_objects),
                &UnionKind::make_params(num_objects),
            ),
            UnionKind::Polynomial => {
                make_polynomial_min_function(&self.function_name(num_objects), num_objects)
            }
            UnionKind::CubicPolynomial => {
                make_cubic_polynomial_min_function(&self.function_name(num_objects), num_objects)
            }
            UnionKind::Root => {
                make_root_min_function(&self.function_name(num_objects), num_objects)
            }
            UnionKind::Exponential => make_exponential_min_function(
                &self.function_name(num_objects),
                &UnionKind::make_params(num_objects),
            ),
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

fn make_polynomial_min_function(function_name: &str, num_objects: usize) -> String {
    if num_objects != 2 {
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

fn make_cubic_polynomial_min_function(function_name: &str, num_objects: usize) -> String {
    if num_objects != 2 {
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

fn make_root_min_function(function_name: &str, num_objects: usize) -> String {
    if num_objects != 2 {
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
    children: Vec<Box<dyn Object>>,
    smoothness: f32,
    kind: UnionKind,
}

impl Union {
    pub fn new(children: Vec<Box<dyn Object>>) -> Result<Union, String> {
        Union::new_with_smoothness(children, 0.)
    }
    pub fn new_with_smoothness(
        children: Vec<Box<dyn Object>>,
        smoothness: f32,
    ) -> Result<Union, String> {
        if children.len() < 2 {
            return Err("Union requires at least 2 children.".to_string());
        }
        let kind = if smoothness == 0.0 {
            UnionKind::Default
        } else {
            UnionKind::Root
        };
        Ok(Union {
            children,
            smoothness,
            kind,
        })
    }
}

impl Object for Union {
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
    object: Box<dyn Object>,
}

impl Object for Negation {
    fn static_code(&self) -> HashSet<String> {
        self.object.static_code()
    }
    fn expression(&self, p: &str) -> String {
        format!("-({})", self.object.expression(p))
    }
}

pub struct Intersection {
    object: Union,
}

impl Intersection {
    pub fn new(children: Vec<Box<dyn Object>>) -> Result<Intersection, String> {
        Intersection::new_with_smoothness(children, 0.)
    }
    pub fn new_with_smoothness(
        mut children: Vec<Box<dyn Object>>,
        smoothness: f32,
    ) -> Result<Intersection, String> {
        let neg_children = children
            .into_iter()
            .map(|object| Box::new(Negation { object: object }) as Box<dyn Object>)
            .collect();
        Ok(Intersection {
            object: Union::new_with_smoothness(neg_children, smoothness)?,
        })
    }
}

impl Object for Intersection {
    fn static_code(&self) -> HashSet<String> {
        self.object.static_code()
    }
    fn expression(&self, p: &str) -> String {
        format!("-({})", self.object.expression(p))
    }
}

pub struct Difference {}

impl Difference {
    pub fn new(children: Vec<Box<dyn Object>>) -> Result<Intersection, String> {
        Difference::new_with_smoothness(children, 0.)
    }
    pub fn new_with_smoothness(
        mut children: Vec<Box<dyn Object>>,
        smoothness: f32,
    ) -> Result<Intersection, String> {
        if children.len() == 0 {
            return Err("Difference requires at least one child.".to_string());
        }
        let mut new_children = vec![children.swap_remove(0)];
        while !children.is_empty() {
            let object = children.pop().unwrap();
            new_children.push(Box::new(Negation { object }));
        }
        Intersection::new_with_smoothness(new_children, smoothness)
    }
}
