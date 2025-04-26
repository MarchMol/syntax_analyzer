
use syntax_analyzer::syn::first_follow;
use std::collections::{HashMap, HashSet};

// use syntax_analyzer::syn::yp_reader;
use syntax_analyzer::syn::slr_automata;
use syntax_analyzer::view::render;
fn main() {
    first_and_follow();
    let producciones = gen_prod(0);
    let terminales = gen_term(0);
    let mut _slr = slr_automata::SLR::new(producciones, terminales);
    _slr.generate();
    render::render_png(&_slr);
}

pub fn first_and_follow(){
    let producciones = gen_prod(1);
    let terminales = gen_term(1);
    let no_terminales = gen_not_term(1);
    let first_term = "E".to_string();

    // Aqui empiezan las pruebas
    println!("Producciones: {:?}", producciones);
    println!("Terminales: {:?}", terminales);
    println!("No terminales: {:?}", no_terminales);

    let firsts = first_follow::find_first(producciones.clone(), terminales.clone(), no_terminales.clone());
    println!("\n== FIRST ==");
    for (nt, first_set) in &firsts {
        println!("FIRST({}) = {:?}", nt, first_set);
    }

    let follows = first_follow::find_follow(&producciones, &terminales, &no_terminales, &firsts, &first_term);
    println!("\n== FOLLOW ==");
    for (nt, follow_set) in &follows {
        println!("FOLLOW({}) = {:?}", nt, follow_set);
    }
}

fn gen_prod(example: i32) -> HashMap<String, Vec<Vec<String>>>{
    if example == 0{
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
                vec!["[", "S", "]"].into_iter().map(String::from).collect(),
                vec!["sentence"].into_iter().map(String::from).collect(),
            ],
        );
        return producciones;
    }
    else if example == 1 {
        let mut producciones:HashMap<String, Vec<Vec<String>>> = HashMap::new();
        producciones.insert(
            "E".to_string(),
            vec![
                vec!["T", "E'"].into_iter().map(String::from).collect(),
            ],
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
            vec![
                vec!["F", "T'"].into_iter().map(String::from).collect(),
            ],
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
        let producciones:HashMap<String, Vec<Vec<String>>> = HashMap::new();
        return producciones;
    }
}

fn gen_not_term(example: i32)->HashSet<String>{
    if example == 0{
        let no_terminales:HashSet<String> = ["S", "P", "Q"]
        .iter()
        .map(|s| s.to_string())
        .collect();
        return no_terminales;
    } else if example == 1 {
        let no_terminales:HashSet<String> = ["E", "E'", "T", "T'", "F"]
        .iter()
        .map(|s| s.to_string())
        .collect();
        return no_terminales;
    } else {
        let no_terminales:HashSet<String> = HashSet::new();
        return no_terminales;
    }
}

fn gen_term(example: i32)->HashSet<String>{
    if example == 0{
        let no_terminales:HashSet<String> = ["^", "v", "[","]","sentence"]
        .iter()
        .map(|s| s.to_string())
        .collect();
        return no_terminales;
    } else if example == 1 {
        let terminales:HashSet<String> = ["+", "*", "(", ")", "id", "ε"]
        .iter()
        .map(|s| s.to_string())
        .collect();
        return terminales;
    } else {
        let terminales:HashSet<String> = HashSet::new();
        return terminales;
    }
}