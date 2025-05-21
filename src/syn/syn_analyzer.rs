use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use super::{yp_reader::read_yalpar, first_follow, slr_automata};

#[derive(Serialize, Deserialize)]
pub struct SynAnalyzer{
    action: HashMap<(u8, String), String>,
    goto: HashMap<(u8, String), u8>,
    ignore: HashSet<String>
}

impl SynAnalyzer{
    pub fn generate(filename: &str)->SynAnalyzer{
        let grammar = read_yalpar(filename);

        let firsts = first_follow::find_first(
        grammar.productions.clone(),
        grammar.terminals.clone(),
        grammar.non_terminals.clone(),
        );

        // 3. Follow
        let follows = first_follow::find_follow(
            & grammar.productions,
            &grammar.terminals,
            &grammar.non_terminals,
            &firsts,
            &grammar.init_symbol,
        );
            // 4. SLR
        let mut slr = slr_automata::SLR::new(
            &grammar.productions, 
            &grammar.terminals);
        slr.generate();

        // render::render_png(&slr)

        let (
            action, 
            goto
        ) = slr.build_parsing_table(&follows);

        let sa = SynAnalyzer{
            action: action,
            goto: goto,
            ignore: grammar.ignore
        };
        sa
    }
}