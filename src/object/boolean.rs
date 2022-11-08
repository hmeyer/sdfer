use crate::object::Object;
use std::collections::HashSet;

trait BinaryBoolean {
    fn get_children(&self) -> &[Box<dyn Object>];
    fn get_op(&self) -> &'static str;
    fn get_fn_name(&self) -> &'static str;
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
        if self.get_children().len() > 1 {
            let params = (0..self.get_children().len())
                .map(|i| format!("float d{}", i))
                .collect::<Vec<_>>()
                .join(", ");
            let expr_begin = (0..self.get_children().len() - 1)
                .map(|i| format!("{}(d{}", self.get_op(), i))
                .collect::<Vec<_>>()
                .join(", ");
            let expr_end = (0..self.get_children().len() - 1)
                .map(|_| ")")
                .collect::<Vec<_>>()
                .join("");
            let code = format!(
                "float {}{}({}) {{
                return {expr_begin}{expr_mid}{expr_end};
            }}",
                self.get_fn_name(),
                self.get_children().len(),
                params,
                expr_begin = expr_begin,
                expr_mid = format!(", d{}", self.get_children().len() - 1),
                expr_end = expr_end
            );
            code_set.insert(code);
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
        return format!(
            "{}{}({})",
            self.get_fn_name(),
            self.get_children().len(),
            child_exps
        );
    }
}

pub struct Union {
    children: Vec<Box<dyn Object>>,
}

impl Union {
    pub fn new(children: Vec<Box<dyn Object>>) -> Result<Union, String> {
        if children.len() == 0 {
            return Err("Empty children for Union.".to_string());
        }
        Ok(Union { children })
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
}

pub struct Intersection {
    children: Vec<Box<dyn Object>>,
}

impl Intersection {
    pub fn new(children: Vec<Box<dyn Object>>) -> Result<Intersection, String> {
        if children.len() == 0 {
            return Err("Empty children for Intersection.".to_string());
        }
        Ok(Intersection { children })
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
pub fn new(mut children: Vec<Box<dyn Object>>) -> Result<Intersection, String> {
    if children.len() == 0 {
        return Err("Empty children for Difference.".to_string());
    }
    let mut new_children = vec![children.swap_remove(0)];
    while !children.is_empty() {
        let object = children.pop().unwrap();
        new_children.push(Box::new(Negation { object }));
    }
    Intersection::new(new_children)
}
}
