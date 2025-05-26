pub mod utility{
    pub mod reader;
    pub mod writer;
    pub mod read_config;
}

pub mod lex{
    pub mod yl_reader;
    pub mod direct_afd;
    pub mod grammar_tree;
    pub mod tokenizer;
    pub mod minimize;
    pub mod lex_analyzer;
}

pub mod syn{
    pub mod yp_reader;
    pub mod slr_automata;
    pub mod lalr_automata;
    pub mod first_follow;
    pub mod syn_analyzer;
}

pub mod view{
    pub mod render;
    pub mod print_table;
    pub mod logging;
}
