use std::{
    collections::{HashMap, HashSet},
    iter::Peekable,
};

use super::{
    first_follow,
    slr_automata::{self, Element},
    yp_reader::read_yalpar,
};
use crate::{
    lex::lex_analyzer::Symbol,
    utility::read_config::Config,
    view::{print_table, render},
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
pub struct SynAnalyzer {
    pub productions: HashMap<u8, Vec<Element>>,
    pub action: HashMap<(u8, String), String>,
    pub goto: HashMap<(u8, String), u8>,
    pub ignore: HashSet<String>,
}

pub struct ParsingStep {
    pub stack: String,
    pub input: String,
    pub action: String,
}

impl SynAnalyzer {
    pub fn generate(filename: &str, config: &Config) -> SynAnalyzer {
        // 1. Obtener gramatica
        let grammar = read_yalpar(filename);
        if config.debug.generation {
            println!("~ S: Grammar obtained");
        }

        // 2. Obtener firsts
        let firsts = first_follow::find_first(
            grammar.productions.clone(),
            grammar.terminals.clone(),
            grammar.non_terminals.clone(),
        );
        if config.debug.generation {
            println!("~ S: First calculated");
        }
        // 3. Obtener Follow
        let follows = first_follow::find_follow(
            &grammar.productions,
            &grammar.terminals,
            &grammar.non_terminals,
            &firsts,
            &grammar.init_symbol,
        );
        if config.debug.generation {
            println!("~ S: Follow calculated");
        }

        // 4. Generar SLR
        let mut slr = slr_automata::SLR::new(
            &grammar.productions,
            &grammar.terminals,
            &grammar.init_symbol,
        );

        if config.debug.generation {
            println!("~ S: SLR initialized");
        }

        slr.generate();

        if config.debug.generation {
            println!("~ S: SLR calculated!");
        }
        if let Some(render_path) = &config.vis.slr_png {
            render::render_png(&slr, &render_path);
        }

        // 5. Shift and reduces
        let (action, goto) = slr.build_parsing_table(&follows);

        if config.debug.generation {
            println!("~ S: Action Table Calculated");
        }
        if let Some(path) = &config.vis.parse_table {
            let _rslt = print_table::print_parse_table(
                slr.icount,
                grammar.terminals,
                grammar.non_terminals,
                &action,
                &goto,
                &path,
            );
        }

        // 6. Save result
        let sa = SynAnalyzer {
            productions: slr.productions,
            action: action,
            goto: goto,
            ignore: grammar.ignore,
        };

        if config.debug.generation {
            println!("~ S: Generation successfull");
        }
        sa
    }

    pub fn parse(&self, tokens: &[Symbol]) -> (Vec<ParsingStep>, Option<(String, String)>) {
        let start = Instant::now();
        let mut steps = Vec::new();
        let mut stack: Vec<u8> = vec![0];
        let mut symbols: Vec<String> = vec![];
        let mut input: Peekable<_> = tokens
            .iter()
            .map(|p| p.token_name.clone())
            .chain(std::iter::once("$".to_string()))
            .peekable();

        // Verificación inicial para tokens inválidos en estado 0
        if let Some(first_token) = input.peek().cloned() {
            let initial_key = (0, first_token.clone());
            if !self.action.contains_key(&initial_key) && !self.goto.contains_key(&initial_key) {
                let error_index = 0;
                let error_msg = highlight_error_token(tokens, error_index);
                let detailed_msg = format!("Invalid initial token '{}'", first_token);

                steps.push(ParsingStep {
                    stack: format!("{:?} {:?}", stack, symbols),
                    input: input.clone().collect::<Vec<_>>().join(" "),
                    action: detailed_msg.clone(),
                });

                return (
                    steps,
                    Some((
                        error_msg_with_arrow(error_msg, error_index, tokens),
                        detailed_msg,
                    )),
                );
            }
        }

        let mut tokens_consumed = 0;
        loop {
            let state = *stack.last().unwrap();
            let lookahead = input.peek().unwrap().clone();
            let key = (state, lookahead.clone());

            let stack_str = format!("{:?} {:?}", stack, symbols);
            let input_str = input.clone().collect::<Vec<_>>().join(" ");

            match self.action.get(&key).map(String::as_str) {
                Some("acc") => {
                    steps.push(ParsingStep {
                        stack: stack_str,
                        input: input_str,
                        action: "ACCEPTANCE".to_string(),
                    });
                    break;
                }
                Some(s) if s.starts_with('s') => {
                    let next_st: u8 = s[1..].parse().unwrap();
                    stack.push(next_st);
                    symbols.push(lookahead.clone());
                    input.next();
                    tokens_consumed += 1;
                    steps.push(ParsingStep {
                        stack: stack_str,
                        input: input_str,
                        action: format!("Shift {}", next_st),
                    });
                }
                Some(r) if r.starts_with('r') => {
                    let prod_id: u8 = r[1..].parse().unwrap();
                    let rhs_len = self.productions[&(prod_id)].len() - 1;
                    for _ in 0..rhs_len {
                        stack.pop();
                        symbols.pop();
                    }
                    let top = *stack.last().unwrap();
                    let lhs = if let Element::NonTerminal(nt) = &self.productions[&(prod_id)][0] {
                        nt.clone()
                    } else {
                        unreachable!()
                    };
                    let goto_key = (top, lhs.clone());
                    let goto_st = match self.goto.get(&goto_key) {
                        Some(&st) => st,
                        None => {
                            let detailed_msg = format!("Error: no GOTO for ({}, {})", top, lhs);
                            let error_index = tokens_consumed;
                            let error_msg = highlight_error_token(tokens, error_index);

                            steps.push(ParsingStep {
                                stack: stack_str,
                                input: input_str,
                                action: detailed_msg.clone(),
                            });

                            return (
                                steps,
                                Some((
                                    error_msg_with_arrow(error_msg, error_index, tokens),
                                    detailed_msg,
                                )),
                            );
                        }
                    };
                    stack.push(goto_st);
                    symbols.push(lhs.clone());

                    steps.push(ParsingStep {
                        stack: stack_str,
                        input: input_str,
                        action: format!(
                            "r{}: {} -> {:?}",
                            prod_id,
                            lhs,
                            &self.productions[&(prod_id)][1..]
                        ),
                    });
                }
                _ => {
                    let detailed_msg = format!("Syntax error at ({}, '{}')", state, lookahead);
                    let error_index = tokens_consumed;
                    let error_msg = highlight_error_token(tokens, error_index);

                    steps.push(ParsingStep {
                        stack: stack_str.clone(),
                        input: input_str.clone(),
                        action: detailed_msg.clone(),
                    });

                    return (
                        steps,
                        Some((
                            error_msg_with_arrow(error_msg, error_index, tokens),
                            detailed_msg,
                        )),
                    );
                }
            }
        }

        let duration = start.elapsed();
        let success_msg = format!("Parsing Completed in {:.2?}", duration);
        let message = format!("\n\x1b[1;32m{}:\x1b[0m\n", success_msg,);
        println!("{}", message);
        (steps, None)
    }
}

fn highlight_error_token(tokens: &[Symbol], error_index: usize) -> String {
    let mut result = String::new();
    for (i, token) in tokens.iter().enumerate() {
        if i == error_index {
            // ANSI rojo brillante
            result.push_str(&format!("\x1b[31m{}\x1b[0m ", token.content));
        } else {
            result.push_str(&format!("{} ", token.content));
        }
    }
    result.trim_end().to_string()
}
fn error_msg_with_arrow(error_msg: String, error_index: usize, tokens: &[Symbol]) -> String {
    format!(
        "\n\x1b[1;31mParsing Error:\x1b[0m\n{}\n{:>width$}↑ here",
        error_msg,
        "",
        width = compute_token_offset(tokens, error_index)
    )
}
fn compute_token_offset(tokens: &[Symbol], error_index: usize) -> usize {
    let mut offset = 0;
    for (i, token) in tokens.iter().enumerate() {
        if i == error_index {
            break;
        }
        offset += token.content.len() + 1;
    }
    offset
}
