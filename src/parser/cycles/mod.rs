use super::*;
pub use cycles::Cycles;

pub mod cycles;

impl Cycles for super::Parser {
    // i do not remember well what does this, but works
    fn push_current_cycle(&mut self, tokens_index_list: &mut Vec<usize>) {
        let mut cache_vec: Vec<Token> = vec![];
        for i in 0..tokens_index_list.len() {
            cache_vec.push(self.tokens[tokens_index_list[i]].clone());
        }
        self.raw_cycles.push(cache_vec);
    }

    // this splits all the operations into cycles,
    // it is called "raw" because does not parse,
    // just marks in a vector when a token finishes
    // the cycle and when another cycle starts
    fn parse_raw_cycles(&mut self) {
        // we need a cache between iterations
        let mut cache_tokens: Vec<usize> = vec![];

        for i in self.position..self.tokens.len() {
            // this means we've reached the end of
            // the tokens, we can push all the tokens
            if i == self.tokens.len() - 1 {
                self.push_current_cycle(&mut cache_tokens);
                cache_tokens = vec![];
            } else {
                // is it a semicolon? a cycle ended, push everything
                // is it another char? push it to the cache, next please.
                match Some(&*self.tokens[i].tok_type) {
                    Some("Semicolon") => {
                        self.push_current_cycle(&mut cache_tokens);
                        cache_tokens = vec![];
                    }
                    _ => {
                        cache_tokens.push(i);
                    }
                }
            }
        }
        println_on_debug!("  Raw cycles {:#?}", &self.raw_cycles);
    }
    fn parse_cycles(&mut self) {
        // for cycle in the raw cycles
        for cycle in &self.raw_cycles {
            // reset some stuff
            self.current_cycle_funcs = vec![];
            self.current_function = "NULL".to_string();

            // for token in cycle
            for i in 0..cycle.len() {
                // define useful variables
                let token = &cycle[i];
                let mut last_token = &cycle[i];

                // avoid panics
                if i != 0 && i != cycle.len() {
                    last_token = &cycle[i - 1];
                } else if i == 0 {
                    last_token = &cycle[i];
                } else if i == cycle.len() {
                    last_token = &cycle[i - 1];
                }

                // match the token type
                match token.tok_type.as_str() {
                    // is it a function? ok
                    // are we parsing arguments? not great
                    // if not? set current function to it
                    "Func" => {
                        if self.parsing_args {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!(
                                    "Expected a function argument like a Text, found {}",
                                    token.content
                                ),
                            );
                        }
                        self.current_function = token.content.clone();
                    }

                    // is it a '('? ok
                    // are we parsing arguments? not great
                    // was a function behind? great
                    // now we are parsing arguments
                    "Lparen" => {
                        if self.parsing_args {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!(
                                    "Expected a function argument like a Text, found {}",
                                    token.content
                                ),
                            );
                        } else {
                            if self.current_function != "NULL" {
                                self.parsing_args = true;
                            } else {
                                self.raise_parse_error(
                                    "UNEXPECTED TOKEN",
                                    format!("Expected a function, found {}", token.content),
                                );
                            }
                        }
                    }

                    // is it a ')'? ok
                    // are we parsing arguments? great
                    // was a function behind? great
                    // now function is unset
                    "Rparen" => {
                        if self.parsing_args && self.current_function != "NULL" {
                            self.parsing_args = false;
                            self.current_cycle_funcs.push(Function {
                                function: Func::from_name(self.current_function.clone()),
                                args: self.current_function_args.clone(),
                            });
                            self.current_function = "NULL".to_string();
                            self.current_function_args = vec![];
                        } else if !self.parsing_args {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!("Expected a function, found {}", token.content),
                            );
                        } else {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!("Expected a function argument, found {}", token.content),
                            );
                        }
                    }

                    // is it a literal? ok
                    // are we parsing arguments? not great
                    // was a comma behind? push a new vec of args
                    // if not? push to current function args
                    // why? because of multiple literal handling
                    "Txt" | "Var" | "Rgx" => {
                        if self.parsing_args && last_token.content == "," {
                            self.current_function_args
                                .push(vec![Literal::from_token(token)]);
                        } else if self.parsing_args && last_token.content != "," {
                            // most spaghetti part of the whole parsed imo,
                            // but gets the job done and i did not notice any bug

                            // get current last vector of arguments
                            let mut f_args: Vec<Literal> = self
                                .current_function_args
                                .last()
                                .cloned()
                                .unwrap_or_else(|| vec![]);

                            // push the token
                            f_args.push(Literal::from_token(token));

                            // get sure there is a value in self.current_function_args
                            match self.current_function_args.last().cloned() {
                                Some(_) => {}
                                None => self.current_function_args.push(vec![]),
                            }

                            // i need to create a variable for this or the compiler gets angry
                            let len: usize = self.current_function_args.len() - 1;

                            // change the vector with the new & old arguments with the one in
                            // self.current_function_args
                            mem::swap(&mut f_args, &mut self.current_function_args[len]);
                        } else if self.current_function == "NULL" {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!("Expected a function, found {}", token.content),
                            );
                        } else if !self.parsing_args {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!("Expected a (, found {}", token.content),
                            );
                        }
                    }

                    // is it a ','? ok
                    // was a function behind? great
                    // are we parsing arguments? great
                    "Comma" => {
                        if self.current_function == "NULL" {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!("Expected a function, found {}", token.content),
                            );
                        } else if !self.parsing_args {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!("Expected a (, found {}", token.content),
                            );
                        } else {
                            match Some(&*last_token.tok_type) {
                                Some("Txt") | Some("Var") | Some("Rgx") => {}
                                _ => {
                                    self.raise_parse_error(
                                        "UNEXPECTED TOKEN",
                                        format!("',' was not expected"),
                                    );
                                }
                            }
                        }
                    }

                    // is it a '{' or a '}'? not ok
                    "Lbrace" | "Rbrace" => {
                        if !self.parsing_args {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!("Expected a function, found {}", token.content),
                            );
                        } else {
                            self.raise_parse_error(
                                "UNEXPECTED TOKEN",
                                format!(
                                    "Expected a function argument like a Text, found {}",
                                    token.content
                                ),
                            );
                        }
                    }

                    // no one matched? not ok
                    _ => self.raise_parse_error(
                        "UNEXPECTED TOKEN",
                        format!("{} was not expected", token.content),
                    ),
                }
            }

            // push cycle
            self.cycles.push(Cycle {
                operations: self.current_cycle_funcs.clone(),
            });
        }
        println_on_debug!("  Parsed cycles {:#?}", &self.cycles);
    }
}
