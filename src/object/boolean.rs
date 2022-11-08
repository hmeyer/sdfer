use crate::object::Object;
use std::collections::HashSet;

trait BinaryBoolean {
    fn get_children(&self) -> &[Box<dyn Object>];
    fn get_op(&self) -> &'static str;
    fn get_fn_name(&self) -> &'static str;
    fn get_full_fn_name(&self) -> String {
        format!(
            "{}{}{}",
            self.get_fn_name(),
            if self.get_smoothness() > 0. {
                "Smooth"
            } else {
                ""
            },
            self.get_children().len()
        )
    }
    fn get_smoothness(&self) -> f32;
}

impl<T> Object for T
where
    T: BinaryBoolean,
{
    fn static_code(&self) -> HashSet<String> {
        let mut code_set = HashSet::new();
        for c in self.get_children() {
            code_set.extend(c.static_code());
        }
        if self.get_smoothness() > 0. {
            code_set.insert(smooth_binary_function(
                self.get_children().len(),
                &self.get_full_fn_name(),
                self.get_op(),
            ));
        } else {
            code_set.insert(binary_function(
                self.get_children().len(),
                &self.get_full_fn_name(),
                self.get_op(),
            ));
        }
        code_set
    }
    fn expression(&self, p: &str) -> String {
        if self.get_children().len() == 1 {
            return self.get_children()[0].expression(p);
        }
        let child_exps = (self.get_children().iter())
            .map(|c| c.expression(p))
            .collect::<Vec<_>>()
            .join(", ");
        let smoothness_exp = if self.get_smoothness() > 0. {
            format!(", {:.8}", self.get_smoothness())
        } else {
            String::new()
        };
        return format!(
            "{}({}{})",
            self.get_full_fn_name(),
            child_exps,
            smoothness_exp
        );
    }
}

fn binary_function(num_objects: usize, full_fn_name: &str, op: &str) -> String {
    if num_objects == 0 {
        return String::new();
    }
    let params = (0..num_objects)
        .map(|i| format!("float d{}", i))
        .collect::<Vec<_>>()
        .join(", ");
    let expr_begin = (0..num_objects - 1)
        .map(|i| format!("{}(d{}", op, i))
        .collect::<Vec<_>>()
        .join(", ");
    let expr_end = (0..num_objects - 1)
        .map(|_| ")")
        .collect::<Vec<_>>()
        .join("");
    format!(
        "
    float {}({}) {{
    return {expr_begin}{expr_mid}{expr_end};
    }}
    ",
        full_fn_name,
        params,
        expr_begin = expr_begin,
        expr_mid = format!(", d{}", num_objects - 1),
        expr_end = expr_end
    )
}

fn smooth_binary_function(num_objects: usize, full_fn_name: &str, op: &str) -> String {
    if num_objects == 0 {
        return String::new();
    }
    if num_objects == 2 {
        let sgn = match op {
            "min" => '-',
            "max" => '+',
            _ => unimplemented!(),
        };
        format!(
            "
float {name}(float d1, float d2, float k) {{
    float h = max(k-abs(d1-d2),0.0);
    return {op}(d1, d2) {sgn} h*h*0.25/k;    
}}
        ",
            name = full_fn_name,
            op = op,
            sgn = sgn
        )
    } else {
        unimplemented!()
    }
}

pub struct Union {
    children: Vec<Box<dyn Object>>,
    smoothness: f32,
}

impl Union {
    pub fn new(children: Vec<Box<dyn Object>>) -> Result<Union, String> {
        Union::new_with_smoothness(children, 0.)
    }
    pub fn new_with_smoothness(
        children: Vec<Box<dyn Object>>,
        smoothness: f32,
    ) -> Result<Union, String> {
        if children.len() == 0 {
            return Err("Empty children for Union.".to_string());
        }
        Ok(Union {
            children,
            smoothness,
        })
    }
}

impl BinaryBoolean for Union {
    fn get_children(&self) -> &[Box<dyn Object>] {
        &self.children
    }
    fn get_op(&self) -> &'static str {
        "min"
    }
    fn get_fn_name(&self) -> &'static str {
        "opUnion"
    }
    fn get_smoothness(&self) -> f32 {
        self.smoothness
    }
}

pub struct Intersection {
    children: Vec<Box<dyn Object>>,
    smoothness: f32,
}

impl Intersection {
    pub fn new(children: Vec<Box<dyn Object>>) -> Result<Intersection, String> {
        Intersection::new_with_smoothness(children, 0.)
    }
    pub fn new_with_smoothness(
        children: Vec<Box<dyn Object>>,
        smoothness: f32,
    ) -> Result<Intersection, String> {
        if children.len() == 0 {
            return Err("Empty children for Intersection.".to_string());
        }
        Ok(Intersection {
            children,
            smoothness,
        })
    }
}

impl BinaryBoolean for Intersection {
    fn get_children(&self) -> &[Box<dyn Object>] {
        &self.children
    }
    fn get_op(&self) -> &'static str {
        "max"
    }
    fn get_fn_name(&self) -> &'static str {
        "opIntersection"
    }
    fn get_smoothness(&self) -> f32 {
        self.smoothness
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
            return Err("Empty children for Difference.".to_string());
        }
        let mut new_children = vec![children.swap_remove(0)];
        while !children.is_empty() {
            let object = children.pop().unwrap();
            new_children.push(Box::new(Negation { object }));
        }
        Intersection::new_with_smoothness(new_children, smoothness)
    }
}
