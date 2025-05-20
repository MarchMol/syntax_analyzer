use std::collections::HashMap;
use crate::syn::slr_automata::Element;

pub struct ParsingStep {
    pub stack: String,
    pub input: String,
    pub action: String,
}

pub fn slr_parsing(
    action_table: &HashMap<(u8, String), String>,
    goto_table: &HashMap<(u8, String), u8>,
    productions: &HashMap<usize, Vec<Element>>,
    mut input: Vec<String>,
) -> Vec<ParsingStep> {
    let mut steps = Vec::new();
    let mut stack: Vec<u8> = vec![0]; // Comienza en estado 0
    let mut symbols: Vec<String> = vec![];

    // Agrega el sentinela "$"
    if input.last().map(|s| s != "$").unwrap_or(true) {
        input.push("$".to_string());
    }

    let mut input_buf = input;

    println!("\n--- SLR Parsing Start ---");
    println!("Initial Stack: {:?}, Symbols: {:?}, Input: {}\n", stack, symbols, input_buf.join(" "));

    loop {
        let state = *stack.last().unwrap();
        let current_token = input_buf.first().cloned().unwrap_or_else(|| "$".to_string());

        let key = (state, current_token.clone());
        let action = action_table.get(&key);

        let stack_str = format!("{:?} {:?}", stack, symbols);
        let input_str = input_buf.join(" ");

        println!("Current State: {}", state);
        println!("Next Input Token: {}", current_token);
        println!("Action Lookup: {:?}", action);

        match action {
            Some(act) if act.starts_with("s") => {
                // Shift
                let next_state: u8 = act[1..].parse().unwrap();
                println!("-> SHIFT to state {}", next_state);
                symbols.push(current_token.clone());
                stack.push(next_state);
                input_buf.remove(0);
                steps.push(ParsingStep {
                    stack: stack_str,
                    input: input_str,
                    action: format!("Shift {}", next_state),
                });
            }
            Some(act) if act.starts_with("r") => {
                // Reduce
                let prod_id: usize = act[1..].parse().unwrap();
                let production = &productions[&prod_id];
                let lhs = if let Element::NonTerminal(ref nt) = production[0] {
                    nt.clone()
                } else {
                    panic!("Invalid production");
                };

                let rhs_len = production.len() - 1; // omit head
                println!("-> REDUCE by r{}: {} -> {:?}", prod_id, lhs, &production[1..]);
                for _ in 0..rhs_len {
                    stack.pop();
                    symbols.pop();
                }

                let top_state = *stack.last().unwrap();
                let goto_state = goto_table
                    .get(&(top_state, lhs.clone()))
                    .expect("Missing goto");

                println!("Goto Lookup: ({}, {}) -> {}", top_state, lhs, goto_state);

                stack.push(*goto_state);
                symbols.push(lhs.clone());

                steps.push(ParsingStep {
                    stack: stack_str,
                    input: input_str,
                    action: format!("r{}: {} → {:?}", prod_id, lhs, &production[1..]),
                });
            }
            Some(act) if act == "acc" => {
                println!("-> ACCEPT");
                steps.push(ParsingStep {
                    stack: stack_str,
                    input: input_str,
                    action: "ACCEPTANCE".to_string(),
                });
                break;
            }
            _ => {
                println!("-> ERROR: no action found for ({}, {})", state, current_token);
                steps.push(ParsingStep {
                    stack: stack_str,
                    input: input_str,
                    action: "Error".to_string(),
                });
                break;
            }
        }
        println!("Updated Stack: {:?}, Symbols: {:?}, Remaining Input: {}\n", stack, symbols, input_buf.join(" "));
    }
    print!("\n--- SLR PARSING ACCEPTED ---\n");
    steps
}
