use syntax_analyzer::syn::first_follow;

// use syntax_analyzer::syn::yp_reader;
use syntax_analyzer::syn::slr_automata;
use syntax_analyzer::view::render;
use syntax_analyzer::syn::yp_reader::read_yalpar;
use syntax_analyzer::view::print_table;
use syntax_analyzer::syn::parsing_slr::slr_parsing;
use syntax_analyzer::utility::prod_transform::flatten_productions;

fn main(){
    // 1. Source Grammar
    let filename = "./grammar/parser.yalp";
    let grammar = read_yalpar(filename);

    // 2. First
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

    println!("\n== FOLLOW ==");
    for (nt, set) in &follows {
        println!("FOLLOW({}) = {:?}", nt, set);
    }

    // 4. SLR
    let mut slr = slr_automata::SLR::new(
        &grammar.productions, 
        &grammar.terminals);
    slr.generate();

    render::render_png(&slr);

    // 5. Parsing
    let (
        action, 
        goto
    ) = slr.build_parsing_table(&follows);

    // println!("Action: {:?}", action);
    // println!("Goto: {:?}", goto);

    let _rslt = print_table::print_parse_table(
        slr.icount, 
        grammar.terminals, 
        grammar.non_terminals,
        &action,
        &goto,
    "graph/parse_table.txt");
    // if rslt.is_ok(){
    //     panic!("Error generating table")
    // } 

    // 6. Input para analizar
    let input: Vec<String> = vec![
        "TOKEN_SENTENCE".to_string(),
        "TOKEN_AND".to_string(),
        "TOKEN_SENTENCE".to_string()
    ];

    // 7. Análisis SLR
    let flat_productions = flatten_productions(&grammar.productions);
    let parsing_steps = slr_parsing(
        &action,
        &goto,
        &flat_productions,
        input
    );

    // 8. Mostrar resultados
    for step in &parsing_steps {
        println!(
            "{:<30} {:<30} {}",
            step.stack, step.input, step.action
        );
    }
}


// fn run_tests(
//     slr: &slr_automata::SLR,
//     action: &HashMap<(u8, String), String>,
//     goto: &HashMap<(u8, String), u8>,
// ) {
//     let tests = vec![
//         (
//             vec!["[".to_string(), "sentence".to_string(), "]".to_string()],
//             true,
//         ),
//         (vec!["[".to_string(), "]".to_string()], false),
//         (vec!["sentence".to_string(), "]".to_string()], false),
//         (vec!["v".to_string()], false),
//     ];

//     for (input, expected) in tests {
//         let result = slr.parse(&input, action, goto);
//         println!(
//             "Test {:?}: {}",
//             input,
//             if result == expected {
//                 " Passed"
//             } else {
//                 "Failed"
//             }
//         );
//     }
// }

// Ahora toma `example` y devuelve el map de FOLLOW
// fn first_and_follow(example: i32) -> HashMap<String, HashSet<String>> {
//     let productions = gen_prod(example);
//     let terminals = gen_term(example);
//     let non_terminals = gen_not_term(example);
//     let start_symbol = match example {
//         0 => "S".to_string(), // cabeza de tu gramática 0
//         1 => "E".to_string(),
//         _ => panic!("Example no válido"),
//     };

//     println!("Producciones: {:?}", productions);
//     println!("Terminales:   {:?}", terminals);
//     println!("No-terminales:{:?}", non_terminals);

//     let firsts = first_follow::find_first(
//         productions.clone(),
//         terminals.clone(),
//         non_terminals.clone(),
//     );
//     println!("\n== FIRST ==");
//     for (nt, set) in &firsts {
//         println!("FIRST({}) = {:?}", nt, set);
//     }

//     let follows = first_follow::find_follow(
//         &productions,
//         &terminals,
//         &non_terminals,
//         &firsts,
//         &start_symbol,
//     );
//     println!("\n== FOLLOW ==");
//     for (nt, set) in &follows {
//         println!("FOLLOW({}) = {:?}", nt, set);
//     }

//     follows // devolvemos el map de FOLLOW
// }

// fn gen_prod(example: i32) -> HashMap<String, Vec<Vec<String>>> {
//     if example == 0 {
//         let mut producciones: HashMap<String, Vec<Vec<String>>> = HashMap::new();
//         producciones.insert(
//             "S".to_string(),
//             vec![
//                 vec!["S", "^", "P"].into_iter().map(String::from).collect(),
//                 vec!["P"].into_iter().map(String::from).collect(),
//             ],
//         );
//         producciones.insert(
//             "P".to_string(),
//             vec![
//                 vec!["P", "v", "Q"].into_iter().map(String::from).collect(),
//                 vec!["Q"].into_iter().map(String::from).collect(),
//             ],
//         );
//         producciones.insert(
//             "Q".to_string(),
//             vec![
//                 vec!["[", "S", "]"].into_iter().map(String::from).collect(),
//                 vec!["sentence"].into_iter().map(String::from).collect(),
//             ],
//         );
//         return producciones;
//     } else if example == 1 {
//         let mut producciones: HashMap<String, Vec<Vec<String>>> = HashMap::new();
//         producciones.insert(
//             "E".to_string(),
//             vec![vec!["T", "E'"].into_iter().map(String::from).collect()],
//         );
//         producciones.insert(
//             "E'".to_string(),
//             vec![
//                 vec!["+", "T", "E'"].into_iter().map(String::from).collect(),
//                 vec!["ε"].into_iter().map(String::from).collect(),
//             ],
//         );
//         producciones.insert(
//             "T".to_string(),
//             vec![vec!["F", "T'"].into_iter().map(String::from).collect()],
//         );
//         producciones.insert(
//             "T'".to_string(),
//             vec![
//                 vec!["*", "F", "T'"].into_iter().map(String::from).collect(),
//                 vec!["ε"].into_iter().map(String::from).collect(),
//             ],
//         );
//         producciones.insert(
//             "F".to_string(),
//             vec![
//                 vec!["(", "E", ")"].into_iter().map(String::from).collect(),
//                 vec!["id"].into_iter().map(String::from).collect(),
//             ],
//         );
//         return producciones;
//     } else {
//         let producciones: HashMap<String, Vec<Vec<String>>> = HashMap::new();
//         return producciones;
//     }
// }

// fn gen_not_term(example: i32) -> HashSet<String> {
//     if example == 0 {
//         let no_terminales: HashSet<String> =
//             ["S", "P", "Q"].iter().map(|s| s.to_string()).collect();
//         return no_terminales;
//     } else if example == 1 {
//         let no_terminales: HashSet<String> = ["E", "E'", "T", "T'", "F"]
//             .iter()
//             .map(|s| s.to_string())
//             .collect();
//         return no_terminales;
//     } else {
//         let no_terminales: HashSet<String> = HashSet::new();
//         return no_terminales;
//     }
// }

// fn gen_term(example: i32) -> HashSet<String> {
//     if example == 0 {
//         let no_terminales: HashSet<String> = ["^", "v", "[", "]", "sentence"]
//             .iter()
//             .map(|s| s.to_string())
//             .collect();
//         return no_terminales;
//     } else if example == 1 {
//         let terminales: HashSet<String> = ["+", "*", "(", ")", "id", "ε"]
//             .iter()
//             .map(|s| s.to_string())
//             .collect();
//         return terminales;
//     } else {
//         let terminales: HashSet<String> = HashSet::new();
//         return terminales;
//     }
// }
