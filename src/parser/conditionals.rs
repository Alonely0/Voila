use super::*;

pub trait Conditionals {
    fn parse_next_conditional(&mut self) -> super::Conditional;
    fn reset_conditionals(&mut self);
}

impl Conditionals for super::Parser {
    fn parse_next_conditional(&mut self) -> Conditional {
        // We reset from previous execution, if we do not do it, the next
        // for-loop will not know what to do
        self.reset_conditionals();

        // define a variable outside the for loop that we'll use as cache
        // for getting the last iteration number
        let mut j: usize = 0;
        for i in self.position..self.tokens.len() {
            j = i;

            // define useful variables
            let token: &Token = &self.tokens[i];
            let next_token: &Token = &self.tokens[i + (&i + 1 != self.tokens.len()) as usize];
            let last_token: &Token = &self.tokens[i - (i != 0usize) as usize];

            // this way we know whenever a value has been already parsed
            if let None = self.val1 {
                println_on_debug!("  parsing val1, token {token}, next token {next_token}");

                // convert it to a literal, if was not converted successfully,
                // we throw an error. Else, we create the value
                let literal_val1 = Literal::from_token(token);
                self.val1 = match literal_val1 {
                    Err(content) => {
                        self.raise_parse_error(
                            "UNEXPECTED TOKEN",
                            format!(
                                "Expected a Variable, an Identifier or a Regex, got \"{}\"",
                                content
                            ),
                        );
                    },
                    _ => Some(literal_val1.unwrap()),
                };
            } else if let None = self.oper {
                println_on_debug!("  parsing oper, token {token}, next token {next_token}");

                // get sure the correct operator is being used
                let op = CondOperator::from_name(&token.tok_type);
                self.oper = match op {
                    Err(_) => {
                        self.raise_parse_error(
                            "UNEXPECTED TOKEN",
                            format!("Expected an operator, got {}", token.content),
                        );
                    },
                    Ok(CondOperator::Re) | Ok(CondOperator::Rn) => {
                        if next_token.tok_type != "Rgx" && last_token.tok_type != "Rgx" {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                 format!(
                                     "Expected a different operator, {} is only for regular expressions. Consider using other operator, like == or !=", token.content))
                        }
                        Some(op.unwrap())
                    },
                    _ => {
                        if next_token.tok_type == "Rgx" || last_token.tok_type == "Rgx" {
                            self.raise_parse_error("UNEXPECTED TOKEN", format!(
                                "Expected a different operator, {} is not for regular expressions. Consider using other operator, like ~= or ~!", token.content
                            ))
                        }
                        Some(op.unwrap())
                    },
                }
            } else if let None = self.val2 {
                println_on_debug!("  parsing val2, token {token}, next token {next_token}");

                // convert it to a literal, if was not converted successfully,
                // we throw an error. Else, we create the value
                let literal_val2 = Literal::from_token(token);
                self.val2 = match literal_val2 {
                    Err(content) => {
                        self.raise_parse_error(
                            "UNEXPECTED TOKEN",
                            format!(
                                "Expected a Variable, an Identifier or a Regex, got \"{}\"",
                                content
                            ),
                        );
                    },
                    _ => Some(literal_val2.unwrap()),
                };
            } else if let None = self.rela {
                println_on_debug!("  parsing rela, token {token}, next token {next_token}");

                // if error, exit, else, continue. simple
                self.rela = match CondRelationship::from_name(&token.tok_type) {
                    CondRelationship::Err => {
                        self.raise_parse_error(
                            "UNEXPECTED SYMBOL",
                            format!("Expected &&, || or {{, got {}", token.content),
                        );
                    },
                    _ => Some(CondRelationship::from_name(&token.tok_type)),
                };

                // we reached the end of this conditional, so we stop the loop
                break;
            } else {
                // because of some reason, all values were parsed
                // (or some were no uninitialized), probably the
                // error was already triggered, so we can stop
                break;
            }
        }
        // set position the the start of the next conditional
        self.position = j + 1;
        println_on_debug!("  position, {}", self.position);

        // return conditional
        Conditional {
            val1: self.val1.clone().unwrap_or_else(|| {
                self.raise_parse_error("EXPECTED VALUE", String::from("Found nothing"));
            }),
            op: self.oper.clone().unwrap_or_else(|| {
                self.raise_parse_error("EXPECTED OPERATOR", String::from("Found nothing"));
            }),
            val2: self.val2.clone().unwrap_or_else(|| {
                self.raise_parse_error("EXPECTED VALUE", String::from("Found nothing"));
            }),
            next_conditional_relationship: self.rela.clone().unwrap_or_else(|| {
                self.raise_parse_error("EXPECTED OPERATIONS", String::from("Found nothing"));
            }),
            position: self.position.clone(),
        }
    }

    // this way the conditional's parser knows whenever it has to parse a new conditional
    fn reset_conditionals(&mut self) {
        self.val1 = None;
        self.oper = None;
        self.val2 = None;
        self.rela = None;
    }
}
