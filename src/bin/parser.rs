use std::{env, fs::File, io::BufReader};
use ron::de::from_reader;
use syntax_analyzer::{lex::lex_analyzer::{LexAnalyzer, Symbol}, syn::syn_analyzer::SynAnalyzer, utility::read_config::fetch_config, view::print_table::{self, print_symbol_table}};
use std::fs;

const LEX_RON_PATH: &str = "./src/bin/lex_analyzer.ron";
const SYN_RON_PATH: &str = "./src/bin/syn_analyzer.ron";

fn actions(id: i32)-> &'static str{
    match id{
		16=>{return "ID";}
		11=>{return "LBRACKET";}
		2=>{return "FLOAT";}
		13=>{return "LPAREN";}
		4=>{return "RETURN";}
		17=>{return "WS";}
		10=>{return "EQUAL";}
		6=>{return "WHILE";}
		9=>{return "ASSIGN";}
		15=>{return "SEMICOLON";}
		5=>{return "IF";}
		1=>{return "STRING";}
		14=>{return "RPAREN";}
		12=>{return "RBRACKET";}
		0=>{return "INT";}
		3=>{return "SCINOT";}
		_=> {return "";}
    }
}

fn main()-> std::io::Result<()> {
    // 1. Fetch Arguments
    let args: Vec<String> = env::args().collect();
    if args.len()!=2{
        panic!("Arguments must be 'cargo run --bin parser -- ./path/to/input.txt'")
    }
    let input_path = &args[1];

    // 2. Config
    let config = fetch_config();

    // Input Fetch
    let contents = fs::read_to_string(input_path)?;

    // Lexic Rules Fetch
    let l_file = File::open(LEX_RON_PATH).unwrap();
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
    if let Some(path) = config.vis.symbol_table{
         let _ = print_symbol_table(&symbol_table,&path);
    }
    
    let s_file = File::open(SYN_RON_PATH).unwrap();
    let s_reader = BufReader::new(s_file);
    let syn: SynAnalyzer = from_reader(s_reader).unwrap();

    symbol_table.retain(|x| !syn.ignore.contains(&x.token_name));

    let (steps, error_msg) = syn.parse(
        &symbol_table
    );

    if let Some(path) = config.vis.parse_steps{
        let _steps_rslt = print_table::print_parse_steps(
            &steps,
            &path
        );
    }

    if let Some((visual_msg, detailed_msg)) = error_msg {
        println!("{}", visual_msg);
        println!("{}", detailed_msg);
    };
    Ok(())
}