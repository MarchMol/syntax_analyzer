use std::collections::{HashMap, HashSet};
use syntax_analyzer::syn::first_follow;

// use syntax_analyzer::syn::yp_reader;
use syntax_analyzer::syn::slr_automata;
use syntax_analyzer::view::render;

fn main() {
    // --- 1) Calcular FIRST/FOLLOW sobre la gramática ejemplo 0 ---
    let follows = first_and_follow(0);

    // --- 2) Construir y volcar el autómata SLR usando la misma gramática 0 ---
    let prod_slr = gen_prod(0);
    let term_slr = gen_term(0);
    let mut slr = slr_automata::SLR::new(prod_slr, term_slr);
    slr.generate();

    // --- 3) Generar ACTION y GOTO usando el FOLLOW de la gramática 0 ---
    let (action, goto) = slr.build_parsing_table(&follows);

    // --- 4) Imprimir ACTION (incluye shifts, reduces y acc) ---
    println!("\n== ACTION ==");
    for ((state, sym), act) in &action {
        println!("ACTION[{}, '{}'] = {}", state, sym, act);
    }

    // --- 5) Imprimir GOTO ---
    println!("\n== GOTO ==");
    for ((state, nt), dest) in &goto {
        println!("GOTO[{}, {}] = {}", state, nt, dest);
    }

    // --- 6) Renderizar PNG del autómata ---
    render::render_png(&slr);
}

/// Ahora toma `example` y devuelve el map de FOLLOW
fn first_and_follow(example: i32) -> HashMap<String, HashSet<String>> {
    let productions = gen_prod(example);
    let terminals = gen_term(example);
    let non_terminals = gen_not_term(example);
    let start_symbol = match example {
        0 => "S".to_string(), // cabeza de tu gramática 0
        1 => "E".to_string(),
        _ => panic!("Example no válido"),
    };

    println!("Producciones: {:?}", productions);
    println!("Terminales:   {:?}", terminals);
    println!("No-terminales:{:?}", non_terminals);

    let firsts = first_follow::find_first(
        productions.clone(),
        terminals.clone(),
        non_terminals.clone(),
    );
    println!("\n== FIRST ==");
    for (nt, set) in &firsts {
        println!("FIRST({}) = {:?}", nt, set);
    }

    let follows = first_follow::find_follow(
        &productions,
        &terminals,
        &non_terminals,
        &firsts,
        &start_symbol,
    );
    println!("\n== FOLLOW ==");
    for (nt, set) in &follows {
        println!("FOLLOW({}) = {:?}", nt, set);
    }

    follows // devolvemos el map de FOLLOW
}

fn gen_prod(example: i32) -> HashMap<String, Vec<Vec<String>>> {
    if example == 0 {
        let mut producciones: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        producciones.insert(
            "S".to_string(),
            vec![
                vec!["S", "^", "P"].into_iter().map(String::from).collect(),
                vec!["P"].into_iter().map(String::from).collect(),
            ],
        );
        producciones.insert(
            "P".to_string(),
            vec![
                vec!["P", "v", "Q"].into_iter().map(String::from).collect(),
                vec!["Q"].into_iter().map(String::from).collect(),
            ],
        );
        producciones.insert(
            "Q".to_string(),
            vec![
                vec!["[", "S", "]"].into_iter().map(String::from).collect(),
                vec!["sentence"].into_iter().map(String::from).collect(),
            ],
        );
        return producciones;
    } else if example == 1 {
        let mut producciones: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        producciones.insert(
            "E".to_string(),
            vec![vec!["T", "E'"].into_iter().map(String::from).collect()],
        );
        producciones.insert(
            "E'".to_string(),
            vec![
                vec!["+", "T", "E'"].into_iter().map(String::from).collect(),
                vec!["ε"].into_iter().map(String::from).collect(),
            ],
        );
        producciones.insert(
            "T".to_string(),
            vec![vec!["F", "T'"].into_iter().map(String::from).collect()],
        );
        producciones.insert(
            "T'".to_string(),
            vec![
                vec!["*", "F", "T'"].into_iter().map(String::from).collect(),
                vec!["ε"].into_iter().map(String::from).collect(),
            ],
        );
        producciones.insert(
            "F".to_string(),
            vec![
                vec!["(", "E", ")"].into_iter().map(String::from).collect(),
                vec!["id"].into_iter().map(String::from).collect(),
            ],
        );
        return producciones;
    } else {
        let producciones: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        return producciones;
    }
}

fn gen_not_term(example: i32) -> HashSet<String> {
    if example == 0 {
        let no_terminales: HashSet<String> =
            ["S", "P", "Q"].iter().map(|s| s.to_string()).collect();
        return no_terminales;
    } else if example == 1 {
        let no_terminales: HashSet<String> = ["E", "E'", "T", "T'", "F"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        return no_terminales;
    } else {
        let no_terminales: HashSet<String> = HashSet::new();
        return no_terminales;
    }
}

fn gen_term(example: i32) -> HashSet<String> {
    if example == 0 {
        let no_terminales: HashSet<String> = ["^", "v", "[", "]", "sentence"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        return no_terminales;
    } else if example == 1 {
        let terminales: HashSet<String> = ["+", "*", "(", ")", "id", "ε"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        return terminales;
    } else {
        let terminales: HashSet<String> = HashSet::new();
        return terminales;
    }
}
