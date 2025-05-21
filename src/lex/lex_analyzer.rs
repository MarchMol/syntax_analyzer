use std::{collections::{HashMap, HashSet}, rc::Rc};

use serde::{Deserialize, Serialize};

use super::{direct_afd, grammar_tree, minimize, tokenizer, yl_reader::read_yalex};

#[derive(Serialize, Deserialize)]
pub struct LexAnalyzer{
    pub map: HashMap<char, HashMap<String, char>>,
    pub accept: HashSet<char>,
    pub start: char,
    pub actions: HashMap<usize, String>,
    pub header: Vec<String>,
}


impl LexAnalyzer{
    pub fn generate(filename: &str)->LexAnalyzer{
        let lexer_data = read_yalex(filename);
        let tokenized_data = tokenizer::inf_to_pos(&lexer_data.merged);
        let mut gtree = grammar_tree::Tree::new();
        let _ = gtree.generate(tokenized_data);
        let gtree_ref = Rc::new(gtree);
        let mut afd = direct_afd::DirectAFD::new(gtree_ref);
        afd.generate_afd();
        let (state_map, acceptance_states, token_list) = afd.create_states();
        let (minimized_map, minimized_accept_states, minimized_start) =
            minimize::minimize_dfa(&state_map, &acceptance_states);
            
        let la = LexAnalyzer{
            map: minimized_map,
            accept: minimized_accept_states,
            start: minimized_start,
            actions: lexer_data.actions,
            header: lexer_data.imports
        };
        la
    }
}