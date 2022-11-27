use super::Primitive;
use anyhow::{bail, Result};

pub trait MinFunction: MinFunctionClone {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String>;
    fn eval(&self, d: &[f32]) -> Result<f32>;
}

pub trait MinFunctionClone {
    /// Clone ```Box<MinFunction>```.
    fn clone_box(&self) -> Box<dyn MinFunction>;
}

impl<T> MinFunctionClone for T
where
    T: 'static + MinFunction + Clone,
{
    fn clone_box(&self) -> Box<dyn MinFunction> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn MinFunction> {
    fn clone(&self) -> Box<dyn MinFunction> {
        self.clone_box()
    }
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
    fn eval(&self, d: &[f32]) -> Result<f32> {
        Ok(d.iter().fold(1e10, |min, x| min.min(*x)))
    }
}

#[derive(Clone)]
pub struct MinPolynomial {
    k: f32,
}

impl MinPolynomial {
    pub fn new(k: f32) -> Result<Self> {
        if k < 0.0 {
            bail!("MinPolynomial requires k > 0.0, got k={}.", k);
        }
        Ok(MinPolynomial { k })
    }
}

impl MinFunction for MinPolynomial {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String> {
        if children.len() != 2 {
            bail!(
                "MinPolynomial requires exactly 2 children - got {}.",
                children.len()
            );
        }
        let local_p = "p";
        let children = children
            .iter()
            .map(|c| c.expression(local_p, shared_code))
            .collect::<Result<Vec<_>>>()?;
        let function_name = format!("MinPolynomial{}", shared_code.len());
        shared_code.push(format!(
            "
float {function_name}(vec3 {local_p}) {{
    float d0 = {child0};
    float d1 = {child1};
    float h = max(k - abs(d0 - d1), 0.0);
    return min(d0, d1) - h * h * 0.25 / {k:.8};
}}",
            function_name = function_name,
            local_p = local_p,
            child0 = children[0],
            child1 = children[1],
            k = self.k
        ));
        Ok(format!("{}({})", function_name, p))
    }
    fn eval(&self, d: &[f32]) -> Result<f32> {
        if d.len() != 2 {
            bail!(
                "MinPolynomial requires exactly 2 children - got {}.",
                d.len()
            );
        }
        let h = (self.k - (d[0] - d[1]).abs()).max(0.0);
        Ok(d[0].min(d[1]) - h * h * 0.25 / self.k)
    }
}

#[derive(Clone)]
pub struct MinCubicPolynomial {
    k: f32,
}

impl MinCubicPolynomial {
    pub fn new(k: f32) -> Result<Self> {
        if k < 0.0 {
            bail!("MinCubicPolynomial requires k > 0.0, got k={}.", k);
        }
        Ok(MinCubicPolynomial { k })
    }
}

impl MinFunction for MinCubicPolynomial {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String> {
        if children.len() != 2 {
            bail!(
                "MinCubicPolynomial requires exactly 2 children - got {}.",
                children.len()
            );
        }
        let local_p = "p";
        let children = children
            .iter()
            .map(|c| c.expression(local_p, shared_code))
            .collect::<Result<Vec<_>>>()?;
        let function_name = format!("MinCubicPolynomial{}", shared_code.len());
        shared_code.push(format!(
            "
float {function_name}(vec3 {local_p}) {{
    float d0 = {child0};
    float d1 = {child1};
    float k = {k:.8};
    float h = max(k - abs(d0 - d1), 0.0) / k;
    return min(d0, d1) - h * h * h * k * (1./6.);
}}",
            function_name = function_name,
            local_p = local_p,
            child0 = children[0],
            child1 = children[1],
            k = self.k
        ));
        Ok(format!("{}({})", function_name, p))
    }
    fn eval(&self, d: &[f32]) -> Result<f32> {
        if d.len() != 2 {
            bail!(
                "MinCubicPolynomial requires exactly 2 children - got {}.",
                d.len()
            );
        }
        let h = (self.k - (d[0] - d[1]).abs()).max(0.0) / self.k;
        Ok(d[0].min(d[1]) - h * h * h * self.k * (1. / 6.))
    }
}

#[derive(Clone)]
pub struct MinRoot {
    k: f32,
}

impl MinRoot {
    pub fn new(k: f32) -> Result<Self> {
        if k < 0.0 {
            bail!("MinRoot requires k > 0.0, got k={}.", k);
        }
        Ok(MinRoot { k })
    }
}

impl MinFunction for MinRoot {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String> {
        if children.len() != 2 {
            bail!(
                "MinRoot requires exactly 2 children - got {}.",
                children.len()
            );
        }
        let local_p = "p";
        let children = children
            .iter()
            .map(|c| c.expression(local_p, shared_code))
            .collect::<Result<Vec<_>>>()?;
        let function_name = format!("MinRoot{}", shared_code.len());
        shared_code.push(format!(
            "
float {function_name}(vec3 {local_p}) {{
    float d0 = {child0};
    float d1 = {child1};
    float h = d0 - d1;
    return 0.5 * ((d0 + d1) - sqrt(h *h + {k:.8}));
}}",
            function_name = function_name,
            local_p = local_p,
            child0 = children[0],
            child1 = children[1],
            k = self.k
        ));
        Ok(format!("{}({})", function_name, p))
    }
    fn eval(&self, d: &[f32]) -> Result<f32> {
        if d.len() != 2 {
            bail!("MinRoot requires exactly 2 children - got {}.", d.len());
        }
        let h = d[0] - d[1];
        Ok(0.5 * ((d[0] + d[1]) - (h * h + self.k).sqrt()))
    }
}

#[derive(Clone)]
pub struct MinChamfer {
    k: f32,
}

impl MinChamfer {
    pub fn new(k: f32) -> Result<Self> {
        if k < 0.0 {
            bail!("MinChamfer requires k > 0.0, got k={}.", k);
        }
        Ok(MinChamfer { k })
    }
}

impl MinFunction for MinChamfer {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String> {
        if children.len() != 2 {
            bail!(
                "MinChamfer requires exactly 2 children - got {}.",
                children.len()
            );
        }
        let local_p = "p";
        let children = children
            .iter()
            .map(|c| c.expression(local_p, shared_code))
            .collect::<Result<Vec<_>>>()?;
        let function_name = format!("MinChamfer{}", shared_code.len());
        shared_code.push(format!(
            "
float {function_name}(vec3 {local_p}) {{
    float d0 = {child0};
    float d1 = {child1};
    return min(min(d0, d1), (d0 - {k:.8} + d1) * sqrt(0.5));
}}",
            function_name = function_name,
            local_p = local_p,
            child0 = children[0],
            child1 = children[1],
            k = self.k
        ));
        Ok(format!("{}({})", function_name, p))
    }
    fn eval(&self, d: &[f32]) -> Result<f32> {
        if d.len() != 2 {
            bail!("MinChamfer requires exactly 2 children - got {}.", d.len());
        }
        Ok(d[0].min(d[1]).min((d[0] - self.k + d[1]) * 0.5_f32.sqrt()))
    }
}

#[derive(Clone)]
pub struct MinStairs {
    k: f32,
    n: i32,
}

impl MinStairs {
    pub fn new(k: f32, n: i32) -> Result<Self> {
        if k < 0.0 {
            bail!("MinStairs requires k > 0.0, got k={}.", k);
        }
        if n <= 0 {
            bail!("MinStairs requires n > 0, got n={}", n);
        }
        Ok(MinStairs { k, n })
    }
}

impl MinFunction for MinStairs {
    fn expression(
        &self,
        p: &str,
        shared_code: &mut Vec<String>,
        children: &[Box<dyn Primitive>],
    ) -> Result<String> {
        if children.len() != 2 {
            bail!(
                "MinStairs requires exactly 2 children - got {}.",
                children.len()
            );
        }
        let local_p = "p";
        let children = children
            .iter()
            .map(|c| c.expression(local_p, shared_code))
            .collect::<Result<Vec<_>>>()?;
        let function_name = format!("MinStairs{}", shared_code.len());
        shared_code.push(format!(
            "
float {function_name}(vec3 {local_p}) {{
    float d0 = {child0};
    float d1 = {child1};
    float k = {k:.8};
    float s = k / {n};
    float u = d1 - k;
    return min(min(d0, d1), 0.5 * (u + d0 + abs((mod(u - d0 + s, 2. * s)) - s)));
}}",
            function_name = function_name,
            local_p = local_p,
            child0 = children[0],
            child1 = children[1],
            k = self.k,
            n = self.n
        ));
        Ok(format!("{}({})", function_name, p))
    }
    fn eval(&self, d: &[f32]) -> Result<f32> {
        if d.len() != 2 {
            bail!("MinStairs requires exactly 2 children - got {}.", d.len());
        }
        let s = self.k / self.n as f32;
        let u = d[1] / self.k;
        Ok(d[0]
            .min(d[1])
            .min(0.5 * ((u + d[0] + s).rem_euclid(2. * s) - s).abs()))
    }
}

#[derive(Clone)]
pub struct MinExponential {
    k: f32,
}

impl MinExponential {
    pub fn new(k: f32) -> Result<Self> {
        if k < 0.0 {
            bail!("MinExponential requires k > 0.0, got k={}.", k);
        }
        Ok(MinExponential { k })
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
            .map(|c_expr| {
                format!(
                    "    t = {}; d = min(d, t); res += exp2(-{:.8} * t);",
                    c_expr, self.k
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let function_name = format!("MinExponential{}", shared_code.len());
        shared_code.push(format!(
            "
float {function_name}(vec3 {local_p}) {{
    float t = 0.0;
    float d = 1e10;
    float res = 0.0;
{min_exps}
    if (res < 10.0) {{
        return -log2(res) / {k:.8};
    }} else {{
        return d;
    }}
}}",
            function_name = function_name,
            local_p = local_p,
            min_exps = min_exps,
            k = self.k,
        ));
        Ok(format!("{}({})", function_name, p))
    }
    fn eval(&self, d: &[f32]) -> Result<f32> {
        if d.len() < 2 {
            bail!(
                "MinExponential requires at least 2 children - got {}.",
                d.len()
            );
        }
        let res: f32 = d.iter().map(|d| (d * -self.k).exp2()).sum();
        if res < 10.0 {
            Ok(-(res.log2()) / self.k)
        } else {
            Ok(d.iter().fold(1e10, |min, x| min.min(*x)))
        }
    }
}
