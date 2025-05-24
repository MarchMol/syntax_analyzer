use std::{fs::File, io::BufReader};
use ron::de::from_reader;
use syntax_analyzer::{lex::lex_analyzer::{LexAnalyzer, Symbol}, syn::syn_analyzer::SynAnalyzer, view::print_table::{self, print_symbol_table}};
use std::fs;
fn actions(id: i32)-> &'static str{
    match id{
		16=>{return "ID";}
		3=>{return "SCINOT";}
		12=>{return "RBRACKET";}
		4=>{return "RETURN";}
		14=>{return "RPAREN";}
		11=>{return "LBRACKET";}
		9=>{return "ASSIGN";}
		6=>{return "WHILE";}
		13=>{return "LPAREN";}
		1=>{return "STRING";}
		10=>{return "EQUAL";}
		2=>{return "FLOAT";}
		15=>{return "SEMICOLON";}
		17=>{return "WS";}
		0=>{return "INT";}
		5=>{return "IF";}
		_=> {return "";}
    }
}

fn main()-> std::io::Result<()> {
    // Input Fetch
    let contents = fs::read_to_string("./grammar/input.txt")?;

    // Lexic Rules Fetch
    let l_file = File::open("./src/bin/lex_analyzer.ron").unwrap();
    let l_reader = BufReader::new(l_file);
    let lex: LexAnalyzer = from_reader(l_reader).unwrap();

    // Lexic Analysis
    let raw_symbol_table = lex.simulate(contents);

    // Action Implementation
    let mut symbol_table: Vec<Symbol> = Vec::new();
    for s in &raw_symbol_table{
        let tem = actions(s.token.parse::<i32>().unwrap());
        if !tem.is_empty(){
            symbol_table.push(
                Symbol { 
                    id: s.id, 
                    token: s.token.clone(), 
                    start: s.start, 
                    end: s.end,
                    content: s.content.clone(), 
                    token_name: tem.to_string(),
            });
        }
    }

    let _ = print_symbol_table(&symbol_table,"graph/symbol_table.txt");
    
    let s_file = File::open("./src/bin/syn_analyzer.ron").unwrap();
    let s_reader = BufReader::new(s_file);
    let syn: SynAnalyzer = from_reader(s_reader).unwrap();

    symbol_table.retain(|x| !syn.ignore.contains(&x.token_name));

    let (steps, error_msg) = syn.parse(
        &symbol_table
    );

    let _steps_rslt = print_table::print_parse_steps(
        &steps,
        "graph/parsing_steps.txt"
    );

    if let Some((visual_msg, detailed_msg)) = error_msg {
        println!("{}", visual_msg);
        println!("{}", detailed_msg);
    };
    Ok(())
}