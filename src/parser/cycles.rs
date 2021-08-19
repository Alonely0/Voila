use super::*;
pub trait Cycles {
    fn get_cycles(&mut self) -> Vec<Cycle>;
}

impl Cycles for super::Parser {
    fn get_cycles(&mut self) -> Vec<Cycle> {
        // we need a cache between iterations
        for i in 0..self.tokens.len() {
            // as we always remove the last element
            // for getting it instead of cloning
            // which is much worse in performance,
            // we must remove all tokens behind the
            // first function
            if i < self.position {
                self.tokens.remove(0);
                continue;
            }
            // this means we've reached the end of
            // the tokens, we can push 'em all
            if self.tokens.len() <= 1 {
                if self.tokens[0].tok_type != "Rbrace" {
                    self.raise_parse_error(
                        "UNCLOSED OPERATIONS BLOCK",
                        "Voila scripts must end with a '}' for delimiting the end of the operations. For more information refer to the documentation."
                            .to_string(),
                    );
                }
                self.cycles.push(Cycle {
                    operations: self.funcs.to_owned(),
                });
                self.funcs.clear();
            } else {
                // is it a semicolon? a cycle ended, push everything
                // is it another char? pus11h it to the cache, next please.
                match Some(&*self.tokens[0].tok_type) {
                    Some("Semicolon") => {
                        self.cycles.push(Cycle {
                            operations: self.funcs.to_owned(),
                        });
                        self.tokens.remove(0);
                        self.funcs.clear();
                    },
                    _ => {
                        match self.tokens[0].tok_type.as_str() {
                            "Func" => self
                                .funcs
                                .push(self.tokens.remove(0).content.function.unwrap()),
                            _ => self.raise_parse_error(
                                "Invalid function",
                                format!(
                                    "{:?} is not a valid function",
                                    &self.tokens[0].content.string.as_ref().unwrap().trim()
                                ),
                            ), // .string because only a valid function will be in .function
                        }
                    },
                }
            }
        }
        println_on_debug!("  Cycles {:#?}", &self.cycles);
        self.cycles.to_owned()
    }
}
