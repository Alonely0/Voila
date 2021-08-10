use evalexpr::eval;

use crate::parser::ast::*;

use super::operators::Operators;
use super::variables::Variables;
use super::{println_on_debug, Literal, Str};

pub trait Conditionals {
    fn eval_conditionals(&mut self) -> bool;
    fn eval_relationships(&self, cond: &String) -> bool;
}

impl Conditionals for super::Interpreter {
    fn eval_conditionals(&mut self) -> bool {
        // init string containing the conditional
        let mut full_conditional: String = "".to_string();

        // go through all conditionals
        for i in 0..self.__ast__.conditionals.len() {
            // get conditional
            let conditional: &Conditional = &self.__ast__.conditionals[i];
            println_on_debug!("    {}", &conditional);

            // is there a var? ok, give the value
            // no? return me the original object
            let val1 = Literal {
                kind: self.get_var_if_any(&conditional.val1).unwrap().kind,
                content: self.trim_spaces(&self.get_var_if_any(&conditional.val1).unwrap().content),
            };
            let val2 = Literal {
                kind: self.get_var_if_any(&conditional.val2).unwrap().kind,
                content: self.trim_spaces(&self.get_var_if_any(&conditional.val2).unwrap().content),
            };

            // evaluate operators
            let cond_result: bool = match conditional.op {
                CondOperator::Eq => self.eq(&val1, &val2),
                CondOperator::Ne => self.ne(&val1, &val2),
                CondOperator::Gt => self.gt(&val1, &val2),
                CondOperator::Ge => self.ge(&val1, &val2),
                CondOperator::Lt => self.lt(&val1, &val2),
                CondOperator::Le => self.le(&val1, &val2),
                CondOperator::Re => self.re(&val1, &val2),
                CondOperator::Rn => self.rn(&val1, &val2),
                _ => false,
            };

            full_conditional = if i == 0 {
                // if is the first conditional, the format is `VAL RELA`
                match conditional.next_conditional_relationship {
                    CondRelationship::And => format!("{} &&", cond_result),
                    CondRelationship::Any => format!("{} ||", cond_result),
                    _ => format!("{}", cond_result),
                }
            } else {
                // if it has another conditional behind, the format is
                // `COND VAL RELA?`
                match conditional.next_conditional_relationship {
                    CondRelationship::And => format!("{} {} &&", &full_conditional, cond_result),
                    CondRelationship::Any => format!("{} {} ||", &full_conditional, cond_result),
                    _ => format!("{} {}", &full_conditional, cond_result),
                }
            };
        }
        // get final result
        let conditional_result = self.eval_relationships(&full_conditional);
        println_on_debug!("    Conditional Evaluated [ {} ]", &full_conditional);
        println_on_debug!("    Result Evaluated [ {} ]", &conditional_result);

        conditional_result
    }

    fn eval_relationships(&self, cond: &String) -> bool {
        // im not doing another lexer & another parser just for evaluating relationships,
        // and "evalexpr" library is pretty nice and gets the job done, so I wont change it
        eval(&cond).unwrap().as_boolean().unwrap()
    }
}
