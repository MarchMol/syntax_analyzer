use std::{collections::{HashMap, HashSet}, rc::Rc};
use serde::{Deserialize, Serialize};
use super::{direct_afd, grammar_tree, minimize, tokenizer, yl_reader::read_yalex};

#[derive(Serialize, Deserialize, Debug)]
pub struct LexAnalyzer{
    pub map: HashMap<char, HashMap<String, char>>,
    pub accept: HashSet<char>,
    pub start: char,
    pub token_list: Vec<String>,
    pub actions: HashMap<usize, String>,
    pub header: Vec<String>,
}


impl LexAnalyzer{
    pub fn generate(filename: &str)->LexAnalyzer{
        let lexer_data = read_yalex(filename);
        println!("~ L: YAL obtained"); 
        let tokenized_data = tokenizer::inf_to_pos(&lexer_data.merged);
        println!("~ L: Regex tokenized");
        let mut gtree = grammar_tree::Tree::new();
        let _ = gtree.generate(tokenized_data);
        let gtree_ref = Rc::new(gtree);
        println!("~ L: Grammar tree generated");
        let mut afd = direct_afd::DirectAFD::new(gtree_ref);
        afd.generate_afd();
        println!("~ L: AFD Generated");
        let (state_map, acceptance_states, token_list) = afd.create_states();
        let (minimized_map, minimized_accept_states, minimized_start) =
            minimize::minimize_dfa(&state_map, &acceptance_states);
        println!("~ L: AFD minimized");
        let la = LexAnalyzer{
            map: minimized_map,
            accept: minimized_accept_states,
            start: minimized_start,
            actions: lexer_data.actions,
            header: lexer_data.imports,
            token_list: token_list
        };
        println!("~ L: Finished Successfully");
        la
    }


    fn leer_cadena(
        &self,
        input: &str
    ) -> HashSet<char> {
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

    fn asignar_token(
        &self,
        input: &str,
    ) -> String {
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


    pub fn simulate(
        &self,
        input: String,
    ) -> Vec<String> {
    let mut tk_list: Vec<String> = Vec::new();
    let len = input.len();
    let mut last_start = 0;
    let mut condition = false;

    while !condition {
        let mut lexem = String::new();
        let mut greedy_match = String::new();
        let mut greedy_end = 0;
        for i in last_start..len {
            let c = input.char_indices().nth(i).map(|(_, c)| c).unwrap();
            lexem.push(c);
            let cmatch = self.get_token_type(
                lexem.to_string()
            );
            if cmatch != "UNKNOWN" {
                greedy_match = cmatch;
                greedy_end = i
            }
            // println!("({}-{}) Lex: \"{}\", match: {:?}\n", last_start, greedy_end,lexem, greedy_match);
        }
        greedy_end += 1;
        if last_start >= greedy_end {
            panic!("Token no identificado {}", lexem);
        } 
        // else {
        //     let biggest_lex: String = input
        //         .chars()
        //         .skip(last_start)
        //         .take(greedy_end - last_start)
        //         .collect();
        //     // println!("FINAL ({}-{}) Lex: \"{}\", match: {:?}", last_start, greedy_end,biggest_lex, greedy_match);
        // }
        // println!("GREEDY MATCH '{}' = {}",&input[last_start..greedy_end],greedy_match);
        if greedy_match == "UNKNOWN" {
            last_start += 1;
        } else {
            tk_list.push(greedy_match);
            last_start = greedy_end;
        }
        if greedy_end == len {
            condition = true;
        }
    }
    tk_list
}

fn get_token_type(
    &self,
    input: String,
) -> String {
    let mini = self.asignar_token(
        &input,
    );
    mini
}

}