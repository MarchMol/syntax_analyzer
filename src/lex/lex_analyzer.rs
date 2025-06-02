use crate::{
    utility::{read_config::Config, writer::write_to_file},
    view::{logging::print_log, render::render_dfa},
};
use console::Style;
use ron::error;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{direct_afd, grammar_tree, minimize, tokenizer, yl_reader::read_yalex};

#[derive(Serialize, Deserialize, Debug)]
pub struct LexAnalyzer {
    pub map: HashMap<char, HashMap<String, char>>,
    pub accept: HashSet<char>,
    pub start: char,
    pub token_list: Vec<String>,
    pub actions: HashMap<usize, String>,
    pub header: Vec<String>,
}

pub struct Symbol {
    pub id: usize,
    pub token: String,
    pub token_name: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub content: String,
}

impl LexAnalyzer {
    pub fn generate(filename: &str, config: &Config) -> LexAnalyzer {
        let blue = Style::new().blue().bold();
        let green = Style::new().green().bold();

        // 1. Read YALex
        let lexer_data = read_yalex(filename);

        if config.debug.generation {
            print!("\n");
            print_log("~ L: Tokenizing Regex", 1, 6, &blue);
        }
        // 2. Tokenize Merged Regex
        let tokenized_data = tokenizer::inf_to_pos(&lexer_data.merged);
        if config.debug.generation {
            print_log("~ L: Calculating Grammar Tree", 2, 6, &blue);
        }

        // 3. Generate Grammar Tree
        let mut gtree = grammar_tree::Tree::new();
        let root = gtree.generate(tokenized_data);
        let gtree_ref = Rc::new(gtree);
        if config.debug.generation {
            print_log("~ L: Generating DFA", 3, 6, &blue);
        }
        if let Some(path) = &config.vis.grammar_tree {
            let gt = (*root).clone().print_tree(0, "root\n");
            let _ = write_to_file(path, &gt);
        }

        // 4. Generate DFA with Direct Method
        let mut afd = direct_afd::DirectAFD::new(gtree_ref);
        afd.generate_afd();
        if config.debug.generation {
            print_log("~ L: Minimizing DFA", 4, 6, &blue);
        }
        let (state_map, acceptance_states, token_list) = afd.create_states();

        // 5. Minimza DFA
        let (minimized_map, minimized_accept_states, minimized_start) =
            minimize::minimize_dfa(&state_map, &acceptance_states);
        if config.debug.generation {
            print_log("~ L: Saving information", 5, 6, &blue);
        }
        if let Some(path) = &config.vis.dfa {
            render_dfa(&minimized_map, &minimized_accept_states, path);
        }

        // 6. Collect Relevant Info
        let la = LexAnalyzer {
            map: minimized_map,
            accept: minimized_accept_states,
            start: minimized_start,
            actions: lexer_data.actions,
            header: lexer_data.imports,
            token_list: token_list,
        };

        if config.debug.generation {
            print_log("~ L: Lexic Analysis - Succesful Generation", 6, 6, &green);
            println!("\n\n");
        }

        // 7. Return
        la
    }

    fn leer_cadena(&self, input: &str) -> HashSet<char> {
        let mut current_states = HashSet::new();
        let mut next_state = HashSet::new();
        current_states.insert(self.start);
        let mut chars = input.chars().peekable();
        let mut remaining = input;

        // println!("--- INICIANDO SIMULACIÓN ---");
        // println!("Cadena de entrada: \"{}\"", input);
        // println!("Estado inicial: '{:?}'\n", current_states);

        while let Some(symbol) = chars.peek().copied() {
            // println!("Símbolo a procesar: '{}'", symbol);
            // println!("Cadena restante: \"{}\"", remaining);

            for &current_state in &current_states {
                if let Some(transitions) = self.map.get(&current_state) {
                    // println!("Posibles transiciones desde '{}': {:?}", current_state, transitions);

                    for (key, &state) in transitions {
                        // println!("  Probando clave de transición: \"{}\"", key);

                        if key.len() == 1 && key.chars().next().unwrap() == symbol {
                            // Literales
                            // println!("  → Coincidencia exacta con literal '{}'", key);
                            next_state.insert(state);
                        } else if key.contains('-') {
                            // Rangos
                            let parts: Vec<char> = key.chars().collect();
                            if parts.len() == 3 && parts[1] == '-' {
                                let start = parts[0];
                                let end = parts[2];
                                if start <= symbol && symbol <= end {
                                    // println!("  → Coincidencia en rango '{}'", key);
                                    next_state.insert(state);
                                }
                            }
                        }
                    }
                }
            }

            if next_state.is_empty() {
                // println!("No se encontraron más transiciones.");
                return HashSet::new();
            }

            chars.next();
            remaining = &remaining[1..];
            current_states = next_state.clone();
            next_state.clear();
        }
        // println!("\n--- SIMULACIÓN FINALIZADA ---\n");
        // println!("Estados alcanzados: {:?}", current_states);

        current_states
    }

    fn asignar_token(&self, input: &str) -> String {
        // println!("input: {}",input);
        // println!("=== DEBUG: asignar_token ===");
        // println!("Input recibido: {}", input);
        // println!("Estado inicial: {}", first_state);
        // println!("Estados de aceptación: {:?}", acceptance_states);
        // println!("Token list: {:?}", token_list);

        let last_state_list = self.leer_cadena(input);
        // println!("Últimos estados alcanzados: {:?}", last_state_list);

        let mut valid_transitions = HashSet::new();

        for &state in &last_state_list {
            if let Some(transitions) = self.map.get(&state) {
                // println!("Estado {} tiene transiciones: {:?}", state, transitions);
                for (transition, &next_state) in transitions {
                    if self.accept.contains(&next_state) {
                        // println!(
                        //     "Transición válida encontrada: '{}' -> Estado {}",
                        //     transition, next_state
                        // );
                        valid_transitions.insert(transition.clone()); // Guardamos la transición
                    }
                }
            } else {
                // println!("Estado {} no tiene transiciones.", state);
            }
        }
        // println!("VALID {:?}",valid_transitions);
        // println!("Transiciones válidas detectadas: {:?}", valid_transitions);

        for token in &self.token_list {
            if valid_transitions.contains(token) {
                // println!("Token encontrado en la lista: {}", token);
                return token.clone();
            }
        }

        // Ningún token encontrado, devolviendo 'UNKNOWN'
        String::from("UNKNOWN")
    }

    fn get_error_bounds(&self, start: usize, input: String) -> usize {
        let mut end: usize = 0;
        end
    }

    pub fn simulate(&self, input: String) -> Option<Vec<Symbol>> {
        let line_breaks = get_line_breaks(&input);
        let mut tk_list: Vec<String> = Vec::new();
        let mut symbols: Vec<Symbol> = Vec::new();
        let len = input.len();
        let mut unknown: Vec<(usize, usize)> = Vec::new();
        let mut last_start: usize = 0;
        let mut condition = false;
        let mut counter = 0;
        let mut is_continuous = false;
        let mut tem_error: (usize, usize) = (0, 0);
        while !condition {
            let mut lexem = String::new();
            let mut greedy_match = String::new();
            let mut greedy_end = 0;
            let mut _biggest_lex = String::new();
            for i in last_start..len {
                let c = input.char_indices().nth(i).map(|(_, c)| c).unwrap();
                lexem.push(c);
                let cmatch = self.get_token_type(lexem.to_string());
                if cmatch != "UNKNOWN" {
                    greedy_match = cmatch;
                    greedy_end = i
                }
                // println!("({}-{}) Lex: \"{}\", match: {:?}\n", last_start, greedy_end,lexem, greedy_match);
            }
            if greedy_end == 0 && last_start>0{
                if !is_continuous {
                    is_continuous = true;
                    tem_error.0 = last_start;
                    tem_error.1 = last_start + 1;
                } else {
                    tem_error.1 = last_start + 1;
                }
                last_start += 1;
                continue;
            }
            // if last_start >= greedy_end {
            //     // panic!("I dont kno what error this is but it comes here: {}", lexem);
            //     // unknown.push(last_start);
            //     println!("{}, {}",last_start, greedy_end);
            // }
            else {
                if is_continuous {
                    is_continuous = false;
                    let line = get_line(&line_breaks, tem_error.1);
                    unknown.push(tem_error);
                    symbols.push(Symbol {
                        id: counter,
                        token: "!error".to_string(),
                        start: tem_error.0,
                        end: tem_error.1,
                        content: input[tem_error.0..tem_error.1].to_string(),
                        token_name: String::new(),
                        line: line,
                    });
                }
                greedy_end += 1;
                _biggest_lex = input
                    .chars()
                    .skip(last_start)
                    .take(greedy_end - last_start)
                    .collect();
                //     // println!("FINAL ({}-{}) Lex: \"{}\", match: {:?}", last_start, greedy_end,biggest_lex, greedy_match);
            }
            // println!("GREEDY MATCH '{}' = {}",&input[last_start..greedy_end],greedy_match);
            if greedy_match == "UNKNOWN" {
                if !is_continuous {
                    tem_error.0 = last_start;
                    tem_error.1 = last_start + 1;
                } else {
                    unknown.push(tem_error);
                }
                last_start += 1;
            } else {
                let line = get_line(&line_breaks, greedy_end);

                symbols.push(Symbol {
                    id: counter,
                    token: greedy_match.clone(),
                    start: last_start,
                    end: greedy_end,
                    content: _biggest_lex,
                    token_name: String::new(),
                    line: line,
                });
                tk_list.push(greedy_match);
                last_start = greedy_end;
            }
            if greedy_end == len {
                condition = true;
            }
            counter += 1;
        }
        if unknown.is_empty() {
            return Some(symbols);
        } else {
            print_lexic_errors(&symbols, &unknown);
            return None
        }
    }
    fn get_token_type(&self, input: String) -> String {
        let mini = self.asignar_token(&input);
        mini
    }
}

fn print_lexic_errors(symbols: &Vec<Symbol>, errors: &Vec<(usize, usize)>) {
    // Detect symbols with errors and save lines
    let mut lines: Vec<usize> = Vec::new();
    for s in symbols.iter(){
        if s.token == "!error"{
            lines.push(s.line);
        }
    }

    // Make closure of all lines with errors
    let mut error_symbols: HashMap<usize, Vec<usize>> = HashMap::new();
    for l in &lines{
        let mut syms: Vec<usize> = Vec::new();
        for (i,s) in symbols.iter().enumerate(){
            if s.line == *l{
                syms.push(i);
            }
        }
        error_symbols.insert(*l, syms);
    }

    let mut messages: Vec<(usize, String)> = Vec::new();
    for l in &lines{
        if let Some(content) = error_symbols.get(l){
            let mut tem_message = String::new();

            for c in content{
                if let Some(symbol) = symbols.get(*c){
                    if symbol.token == "!error"{
                        tem_message+="`";
                        tem_message+=&symbol.content;
                        tem_message+="`";
                    } else{
                        tem_message+=&symbol.content;
                    }
                }
            }
            messages.push((*l,tem_message));
        }
    }
    let red = Style::new().red().bold();
    let error_title = &format!("{}", red.apply_to("LEXIC ERROR"));
    println!("{}",error_title);
    for m in messages{
        pretty_error(&m.1, m.0);
    }
}

fn get_line_breaks(input: &str) -> Vec<usize> {
    let mut line_breaks: Vec<usize> = Vec::new();
    for (i, c) in input.chars().enumerate() {
        if c == '\n' {
            line_breaks.push(i + 2);
        }
    }
    line_breaks
}

fn pretty_error(msg: &str, line: usize){
    let red = Style::new().red().bold();
    let blue = Style::new().blue().bold();
    let split :Vec<&str>= msg.split('`').collect();
    let mut final_msg = String::new();
    for (i,s) in split.iter().enumerate(){
        if i%2 == 0{
            final_msg+=&s;
        } else {
            final_msg+=&format!("{}", red.apply_to(s));
        }   
    }
    let line_msg = format!("Line {} --->\t",line+1);
    let prefix = format!("{}", blue.apply_to(line_msg));
    println!("{}{}\n",prefix,final_msg);
}

fn get_line(cuts: &[usize], index: usize) -> usize {
    for (i, &cut) in cuts.iter().enumerate() {
        if index < cut {
            return i;
        }
    }
    cuts.len()
}
