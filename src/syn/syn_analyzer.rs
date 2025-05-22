use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use crate::view::{print_table, render};

use super::{yp_reader::read_yalpar, first_follow, slr_automata};

#[derive(Serialize, Deserialize)]
pub struct SynAnalyzer{
    pub action: HashMap<(u8, String), String>,
    pub goto: HashMap<(u8, String), u8>,
    pub ignore: HashSet<String>
}

impl SynAnalyzer{
    pub fn generate(filename: &str, do_vis: bool)->SynAnalyzer{
        // 1. Obtener gramatica
        let grammar = read_yalpar(filename);
        println!("~ S: Grammar obtained");

        // 2. Obtener firsts
        let firsts = first_follow::find_first(
            grammar.productions.clone(),
            grammar.terminals.clone(),
            grammar.non_terminals.clone(),
        );
        println!("~ S: First calculated");

        // 3. Obtener Follow
        let follows = first_follow::find_follow(
            & grammar.productions,
            &grammar.terminals,
            &grammar.non_terminals,
            &firsts,
            &grammar.init_symbol,
        );
        println!("~ S: Follow calculated");

        // 4. Generar SLR
        let mut slr = slr_automata::SLR::new(
            &grammar.productions, 
            &grammar.terminals,
            &grammar.init_symbol);
        println!("~ S: SLR initialized");
        slr.generate();

        println!("~ S: SLR calculated!");
        if do_vis{
            render::render_png(&slr);
        }
    
        // 5. Shift and reduces
        let (
            action, 
            goto
        ) = slr.build_parsing_table(&follows);

        println!("~ S: Action Table Calculated");
        if do_vis{
            let _rslt = print_table::print_parse_table(
                slr.icount, 
                grammar.terminals, 
                grammar.non_terminals,
                &action,
                &goto,
            "graph/parse_table.txt"
            );
        }

        // 6. Save result
        let sa = SynAnalyzer{
            action: action,
            goto: goto,
            ignore: grammar.ignore
        };
        println!("~ S: Generation successfull");
        sa
    }
}