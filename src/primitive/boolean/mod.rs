use super::Primitive;
use anyhow::{bail, Result};

pub mod min_function;
pub use min_function::{
    MinChamfer, MinCubicPolynomial, MinDefault, MinExponential, MinFunction, MinPolynomial,
    MinRoot, MinStairs,
};

#[derive(Clone)]
pub struct Boolean {
    children: Vec<Box<dyn Primitive>>,
    min_function: Box<dyn MinFunction>,
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
            min_function: Box::new(MinDefault {}),
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
    pub fn set_min_function(&mut self, f: Box<dyn MinFunction>) {
        self.min_function = f;
    }
}

impl Primitive for Boolean {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        let expression = self
            .min_function
            .expression(p, shared_code, &self.children)?;
        let negate = if self.negate { "-" } else { "" };
        Ok(format!("{}{}", negate, expression))
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        let min_d = self
            .children
            .iter()
            .map(|c| c.eval(p))
            .collect::<Result<Vec<_>>>()?;
        return self.min_function.eval(&min_d);
    }
}

#[derive(Clone)]
struct Negation {
    child: Box<dyn Primitive>,
}

impl Primitive for Negation {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        Ok(format!("-({})", self.child.expression(p, shared_code)?))
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        return self.child.eval(p).map(|d| -d);
    }
}
