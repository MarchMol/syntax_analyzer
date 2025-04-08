
use std::collections::{HashMap, HashSet};

// use syntax_analyzer::syn::yp_reader;
fn main() {
    let producciones = gen_prod();
    let terminales = gen_term();
    let terminales = gen_not_term();
}

fn gen_prod()->HashMap<String, Vec<Vec<String>>>{
    let mut producciones:HashMap<String, Vec<Vec<String>>> = HashMap::new();
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
            vec!["[", "Q", "]"].into_iter().map(String::from).collect(),
            vec!["sentence"].into_iter().map(String::from).collect(),
        ],
    );
    producciones
}

fn gen_term()->HashSet<String>{
    let terminales:HashSet<String> = ["S", "P", "Q"]
    .iter()
    .map(|s| s.to_string())
    .collect();
    terminales
}

fn gen_not_term()->HashSet<String>{
    let no_terminales:HashSet<String> = ["^", "v", "[","]","sentence"]
    .iter()
    .map(|s| s.to_string())
    .collect();
    no_terminales
}