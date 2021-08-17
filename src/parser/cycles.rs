use super::*;
pub trait Cycles {
    fn get_cycles(&mut self) -> Vec<Cycle>;
}

impl Cycles for super::Parser {
    // this splits all the operations into cycles,
    // it is called "raw" because does not parse,
    // just marks in a vector when a token finishes
    // the cycle and when another cycle starts
    fn get_cycles(&mut self) -> Vec<Cycle> {
        // we need a cache between iterations
        for i in self.position..self.tokens.len() {
            // this means we've reached the end of
            // the tokens, we can push 'em all
            if i == self.tokens.len() - 1 {
                self.cycles.push(Cycle {
                    operations: self.funcs.to_owned(),
                });
                self.funcs.clear();
            } else {
                // is it a semicolon? a cycle ended, push everything
                // is it another char? push it to the cache, next please.
                match Some(&*self.tokens[i].tok_type) {
                    Some("Semicolon") => {
                        self.cycles.push(Cycle {
                            operations: self.funcs.to_owned(),
                        });
                        self.funcs.clear();
                    },
                    _ => {
                        match *&self.tokens[i].tok_type.as_str() {
                            "Func" => self
                                .funcs
                                .push(self.tokens[i].content.function.clone().unwrap()),
                            _ => self.raise_parse_error(
                                "Invalid function",
                                format!("{:?} is not a valid function", &self.tokens[i].content.string.as_ref().unwrap().trim()),
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
