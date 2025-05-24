
use std::fs;
use std::{collections::HashMap, fs::File};
use std::io::Write;
use syntax_analyzer::lex::lex_analyzer::LexAnalyzer;
use syntax_analyzer::syn::syn_analyzer::SynAnalyzer;
use ron::ser::{to_string_pretty, PrettyConfig};

fn main(){
    _full_flow();
}

fn _full_flow(){ 
    // Lexer
    let l_filename = "./grammar/lexer.yal";
    let la_raw = LexAnalyzer::generate(l_filename);
    let la_serialized = to_string_pretty(&la_raw, PrettyConfig::default()).unwrap();
    let mut l_file = File::create("./src/bin/lex_analyzer.ron").unwrap();
    l_file.write_all(la_serialized.as_bytes()).unwrap();
    
    // Syntaxer
    let s_filename = "./grammar/test_grammar.yalp";
    let sa_raw = SynAnalyzer::generate(s_filename, true);
    let sa_serialized = to_string_pretty(&sa_raw, PrettyConfig::default()).unwrap();
    let mut s_file = File::create("./src/bin/syn_analyzer.ron").unwrap();
    s_file.write_all(sa_serialized.as_bytes()).unwrap();

    // Generate Parser
    let _ = write_to_main("./src/bin/parser.rs", la_raw.header, la_raw.actions);
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
    "fn main()-> std::io::Result<()> {
    // Input Fetch
    let contents = fs::read_to_string(\"./grammar/input.txt\")?;

    // Lexic Rules Fetch
    let l_file = File::open(\"./src/bin/lex_analyzer.ron\").unwrap();
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

    let _ = print_symbol_table(&symbol_table,\"graph/symbol_table.txt\");
    
    let s_file = File::open(\"./src/bin/syn_analyzer.ron\").unwrap();
    let s_reader = BufReader::new(s_file);
    let syn: SynAnalyzer = from_reader(s_reader).unwrap();

    symbol_table.retain(|x| !syn.ignore.contains(&x.token_name));

    let (steps, error_msg) = syn.parse(
        &symbol_table
    );

    let _steps_rslt = print_table::print_parse_steps(
        &steps,
        \"graph/parsing_steps.txt\"
    );

    if let Some((visual_msg, detailed_msg)) = error_msg {
        println!(\"{}\", visual_msg);
        println!(\"{}\", detailed_msg);
    };
    Ok(())
}";


    let parsing_code = format_headers+&format_actions+main_method;
    fs::write(filename, parsing_code)?;
    Ok(())
}
// fn _syn_flow(){
//     // 1. Source Grammar
//     let filename = "./grammar/parser.yalp";
//     let grammar = read_yalpar(filename);

//     // 2. First
//     let firsts = first_follow::find_first(
//         grammar.productions.clone(),
//         grammar.terminals.clone(),
//         grammar.non_terminals.clone(),
//     );

//     // 3. Follow
//     let follows = first_follow::find_follow(
//         & grammar.productions,
//         &grammar.terminals,
//         &grammar.non_terminals,
//         &firsts,
//         &grammar.init_symbol,
//     );

//     println!("\n== FOLLOW ==");
//     for (nt, set) in &follows {
//         println!("FOLLOW({}) = {:?}", nt, set);
//     }

//     // 4. SLR
//     let mut slr = slr_automata::SLR::new(
//         &grammar.productions, 
//         &grammar.terminals,
//         &grammar.init_symbol);
//     slr.generate();

//     render::render_png(&slr);

//     // 5. Parsing Table
//     let (
//         action, 
//         goto
//     ) = slr.build_parsing_table(&follows);

    

//     let _rslt = print_table::print_parse_table(
//         slr.icount, 
//         grammar.terminals, 
//         grammar.non_terminals,
//         &action,
//         &goto,
//     "graph/parse_table.txt");
//     // if rslt.is_ok(){
//     //     panic!("Error generating table")
//     // } 

//     // 6. Input para analizar
//     let tokens = vec![
//         "TOKEN_L_BRACE".to_string(),
//         "TOKEN_SENTENCE".to_string(),
//         "TOKEN_OR".to_string(),
//         "TOKEN_SENTENCE".to_string(),
//         "TOKEN_R_BRACE".to_string(),
//         "TOKEN_AND".to_string(),
//         "TOKEN_SENTENCE".to_string()
//     ];
// }
