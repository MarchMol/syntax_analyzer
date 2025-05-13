use std::{collections::{HashMap, HashSet}, hash::Hash};

use crate::utility::reader::read_lines;

#[derive(Eq, Hash, Debug, PartialEq, Clone)]
pub struct TokenAction{
    id: i32,
    do_ignore: bool,
    name: String
}

pub struct GrammarInfo{
    pub productions: HashMap<String, Vec<Vec<String>>>,
    pub terminals: HashSet<String>,
    pub non_terminals: HashSet<String>,
    pub ignore: HashSet<String>,    
    pub init_symbol: String,
}

fn process_token(line: String, counter: i32)->Option<TokenAction>{
    if line.is_empty(){
        return None;
    }
    let mut action: bool = true;
    let mut name = String::new();
    let division:Vec<&str> = line.split_whitespace().collect();
    if division.len() != 2{
        panic!("Error in token definition. Couldnt determine the following: \n'{:?}'", line)
    }
    for (id, div) in division.into_iter().enumerate(){
        if id == 0{
            if div == "%token"{
                action = false;
            }
        }
        if id == 1{
            name = div.to_string();
        }
    }
    let ta = TokenAction{
        id: counter,
        do_ignore: action,
        name: name
    };
    Some(ta)
}

fn process_production(
    prod_string: String,
    non_terminals: &mut HashSet<String>,
    terminals: &HashSet<String>
)->(
    String, // Head
    Vec<Vec<String>> // Productions
){
    let mut prod_vec:Vec<Vec<String>> = Vec::new();
    let mut head = String::new();
    let mut cut = 0;
    // Head extraction
    for (id, ch) in prod_string.chars().into_iter().enumerate(){
        if ch !=':'{
            head.push(ch);
        } else{
            cut = id+1;
            break;
        }
    }
    println!("Head: {:?}",head);
    if !non_terminals.contains(&head){
        non_terminals.insert(head.clone());
    }
    // Head cut
    let byte_index = prod_string.char_indices().nth(cut)
        .map(|(i, _)| i)
        .unwrap_or_else(|| prod_string.len()); // handle out-of-bounds

    let sliced = prod_string[byte_index..].to_string();
    
    // Whitespace split
    let production_array: Vec<&str>= sliced.split('|').collect();
    for p in production_array{
        let tem_str: Vec<&str> = p.split_whitespace().collect();
        let tem_string:Vec<String> = tem_str.iter().map(|s| s.to_string()).collect();
        for t in tem_str{
            if !terminals.contains(t){
                if !non_terminals.contains(t){
                    non_terminals.insert(t.to_string());
                }
            }
        }
        prod_vec.push(tem_string);
    }
    println!("Prods: {:?}", prod_vec);
    // Return
    (head,prod_vec)
}

// Called 
pub fn read_yalpar(filename: &str)->GrammarInfo{
    // 1. Section division (ignoring comments)
    let mut counter = 0;
    let mut is_prod_section= false;
    let mut production_string = String::new();
    let mut tsec: Vec<TokenAction> = Vec::new();
    let mut psec: Vec<String> = Vec::new();
    if let Ok(lines) = read_lines(filename){
        for line in lines{
            if let Ok(content) = line{
                // No es comentario
                if !content.starts_with("/*"){
                    if content.starts_with("%%"){
                        is_prod_section = !is_prod_section;
                    } else{
                        if !is_prod_section{
                            if let Some(tem) = process_token(content, counter){
                                tsec.push(tem);
                            }
                        } 
                        else{
                            if content.contains(";"){
                                // Production ended
                                psec.push(production_string.clone());
                                production_string.drain(..);
                            } else{
                                production_string+=&content;
                            }
                        }
                    }
                }
            }
            counter+=1;
        }
    }

    // 2. Token section
    let mut terminals: HashSet<String> = HashSet::new();
    let mut ignore: HashSet<String> = HashSet::new();

    for t in tsec{
        if t.do_ignore{
            ignore.insert(t.name);
        } else{
            terminals.insert(t.name);
        }
    }
    // println!("Ignore: {:?}\n",ignore);
    // println!("Terminals: {:?}\n", terminals)

    // 3. Process production section
    let mut productions: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    let mut init_symbol = String::new();
    let mut non_terminals: HashSet<String> = HashSet::new();
    for (id, p) in psec.iter().enumerate(){
        if id == 0{

        }
        let (head, prods) = process_production(
            p.to_string(),
            &mut non_terminals,
            &terminals
        );
        if id == 0{
            init_symbol = head.clone();
        }
        productions.insert(head, prods);
    }

    // Return Result
    
    GrammarInfo{
        productions: productions,
        terminals: terminals,
        non_terminals: non_terminals,
        ignore: ignore,    
        init_symbol: init_symbol,
    }

}