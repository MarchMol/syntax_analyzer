use core::panic;
use std::{env, fs};
use std::{collections::HashMap, fs::File};
use std::io::Write;
use syntax_analyzer::lex::lex_analyzer::LexAnalyzer;
use syntax_analyzer::syn::syn_analyzer::SynAnalyzer;
use ron::ser::{to_string_pretty, PrettyConfig};
use syntax_analyzer::utility::read_config::fetch_config;

const LEX_RON_PATH: &str = "./src/bin/lex_analyzer.ron";
const SYN_RON_PATH: &str = "./src/bin/syn_analyzer.ron";
const PARSER_PATH: &str = "./src/bin/parser.rs";

fn main(){
    // 1. Fetch Arguments
    let args: Vec<String> = env::args().collect();
    if args.len()!=3{
        panic!("Arguments must be 'cargo run --bin syntax_analyzer -- ./path/to/lex.yal ./path/to/syn.yalp'")
    }
    let lex_path = &args[1];
    let syn_path = &args[2];

    // 2. fetch Config
    let config = fetch_config();

    // 3. Generate
    let la_raw = LexAnalyzer::generate(&lex_path, &config);
    let la_serialized = to_string_pretty(&la_raw, PrettyConfig::default()).unwrap();
    let mut l_file = File::create(LEX_RON_PATH).unwrap();
    l_file.write_all(la_serialized.as_bytes()).unwrap();

    let sa_raw = SynAnalyzer::generate(&syn_path, &config);
    let sa_serialized = to_string_pretty(&sa_raw, PrettyConfig::default()).unwrap();
    let mut s_file = File::create(SYN_RON_PATH).unwrap();
    s_file.write_all(sa_serialized.as_bytes()).unwrap();

    // Generate Parser
    let _ = write_to_main(PARSER_PATH, la_raw.header, la_raw.actions);
}

fn write_to_main(
    filename: &str,
    header: Vec<String>,
    actions: HashMap<usize, String>
)->std::io::Result<()>{
    let mut format_headers = String::new();
    for h in header{
        format_headers+=&h;
        format_headers+="\n";
    }
    format_headers+="\n";
    let constants = 
    "const LEX_RON_PATH: &str = \"./src/bin/lex_analyzer.ron\";
const SYN_RON_PATH: &str = \"./src/bin/syn_analyzer.ron\";\n\n";

    let mut format_actions = String::new();
    format_actions+="fn actions(id: i32)-> &'static str{
    match id{\n";
    for (id, act) in actions{
        if !act.is_empty(){
            format_actions+=&format!("\t\t{}=>{{",id);
            format_actions+=&act;
            format_actions+="}\n"
        }
    }
    format_actions+="\t\t_=> {return \"\";}
    }
}\n\n";
    let main_method = 
    "fn main() -> std::io::Result<()> {
    // 1. Fetch Arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!(\"Arguments must be 'cargo run --bin parser -- ./path/to/input.txt'\")
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
    // Action Implementation
    if let Some(raw) = lex.simulate(contents) {
        let mut symbol_table: Vec<Symbol> = Vec::new();
        for s in &raw {
            let tem = actions(s.token.parse::<i32>().unwrap());
            if !tem.is_empty() {
                symbol_table.push(Symbol {
                    id: s.id,
                    token: s.token.clone(),
                    start: s.start,
                    end: s.end,
                    content: s.content.clone(),
                    token_name: tem.to_string(),
                    line: s.line,
                });
            }
        }

        if let Some(path) = config.vis.symbol_table {
            let _ = print_symbol_table(&symbol_table, &path);
        }

        let s_file = File::open(SYN_RON_PATH).unwrap();
        let s_reader = BufReader::new(s_file);
        let syn: SynAnalyzer = from_reader(s_reader).unwrap();

        symbol_table.retain(|x| !syn.ignore.contains(&x.token_name));

        let (steps, error_msg) = syn.parse(&symbol_table);

        if let Some(path) = config.vis.parse_steps {
            let _steps_rslt = print_table::print_parse_steps(&steps, &path);
        }

        if let Some((visual_msg, detailed_msg)) = error_msg {
            println!(\"{}\", visual_msg);
            println!(\"{}\", detailed_msg);
        };
    }
    Ok(())
}
";


    let parsing_code = format_headers+constants+&format_actions+main_method;
    fs::write(filename, parsing_code)?;
    Ok(())
}