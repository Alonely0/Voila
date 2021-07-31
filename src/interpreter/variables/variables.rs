use super::Literal;

pub trait Variables {
    fn get_var_if_any(&self, var: &Literal) -> Result<Literal, String>;
}