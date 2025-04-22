use std::collections::{HashMap, HashSet};

pub fn find_first(
    grammar: HashMap<String, Vec<Vec<String>>>,
    terminales: HashSet<String>, 
    no_terminales: HashSet<String>,
) -> HashMap<String, HashSet<String>> {
    let mut firsts: HashMap<String, HashSet<String>> = HashMap::new();

    // Función recursiva para obtener FIRST de un símbolo
    fn compute_first(
        symbol: &String,
        grammar: &HashMap<String, Vec<Vec<String>>>,
        terminales: &HashSet<String>,
        no_terminales: &HashSet<String>,
        firsts: &mut HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
    ) -> HashSet<String> {
        // Si ya se calculó previamente
        if let Some(existing) = firsts.get(symbol) {
            return existing.clone();
        }

        // Si es terminal, su FIRST es él mismo
        if terminales.contains(symbol) {
            let mut set = HashSet::new();
            set.insert(symbol.clone());
            return set;
        }

        // Evitar recursión infinita
        if visited.contains(symbol) {
            return HashSet::new();
        }
        visited.insert(symbol.clone());

        let mut result = HashSet::new();

        // Obtener producciones del símbolo
        if let Some(productions) = grammar.get(symbol) {
            for prod in productions {
                if let Some(first_sym) = prod.first() {
                    if terminales.contains(first_sym) {
                        result.insert(first_sym.clone());
                    } else if no_terminales.contains(first_sym) {
                        let sub_first = compute_first(first_sym, grammar, terminales, no_terminales, firsts, visited);
                        result.extend(sub_first);
                    }
                }
            }
        }

        firsts.insert(symbol.clone(), result.clone());
        result
    }

    for nt in &no_terminales {
        let mut visited = HashSet::new();
        compute_first(nt, &grammar, &terminales, &no_terminales, &mut firsts, &mut visited);
    }

    firsts
}

pub fn find_follow(
    grammar: &HashMap<String, Vec<Vec<String>>>,
    terminales: &HashSet<String>,
    no_terminales: &HashSet<String>,
    firsts: &HashMap<String, HashSet<String>>,
    start_symbol: &String,
) -> HashMap<String, HashSet<String>> {
    let mut follows: HashMap<String, HashSet<String>> = HashMap::new();

    for nt in no_terminales {
        follows.insert(nt.clone(), HashSet::new());
    }

    // Regla 1: agregar símbolo de fin de cadena ($) al símbolo inicial
    if let Some(set) = follows.get_mut(start_symbol) {
        set.insert("$".to_string());
    }

    println!("\n== INICIO DEL CÁLCULO DE FOLLOW ==");
    println!("Símbolo inicial: {}", start_symbol);

    // Se repite hasta que no haya cambios
    let mut changed = true;
    while changed {
        changed = false;
        println!("\n--- Nueva iteración ---");

        for (prod_head, productions) in grammar {
            for production in productions {
                println!("Analizando producción: {} -> {:?}", prod_head, production);

                let prod_len = production.len();

                for i in 0..prod_len {
                    let current = &production[i];

                    // Nos interesan solo los no terminales
                    if !no_terminales.contains(current) {
                        continue;
                    }

                    let mut follow_to_add = HashSet::new();

                    // Regla 2: B -> alpha A beta
                    if i + 1 < prod_len {
                        let next = &production[i + 1];

                        if terminales.contains(next) {
                            println!("  Regla 2 (terminal después de {}): agregando {}", current, next);
                            follow_to_add.insert(next.clone());
                        } else if no_terminales.contains(next) {
                            println!("  Regla 2 (no terminal después de {}): FIRST({}) = {:?}", current, next, firsts.get(next).unwrap());
                            let first_of_next = firsts.get(next).unwrap();
                            for symbol in first_of_next {
                                if symbol != "ε" {
                                    follow_to_add.insert(symbol.clone());
                                }
                            }

                            // Si FIRST(beta) contiene ε, aplica también regla 3
                            if first_of_next.contains("ε") {
                                println!("  FIRST({}) contiene ε, aplicando también regla 3: FOLLOW({})", next, prod_head);
                                if let Some(follow_of_prod_head) = follows.get(prod_head) {
                                    follow_to_add.extend(follow_of_prod_head.clone());
                                }
                            }
                        }
                    } 
                    // Regla 3: B -> alpha A
                    else {
                        println!("  Regla 3 ({} es el último en la producción): agregando FOLLOW({})", current, prod_head);
                        if let Some(follow_of_prod_head) = follows.get(prod_head) {
                            follow_to_add.extend(follow_of_prod_head.clone());
                        }
                    }

                    // Añadimos los follow encontrados si no están
                    let follow_set = follows.get_mut(current).unwrap();
                    let initial_len = follow_set.len();
                    follow_set.extend(follow_to_add);
                    if follow_set.len() > initial_len {
                        println!("  FOLLOW({}) actualizado: {:?}", current, follow_set);
                        changed = true;
                    }
                }
            }
        }
    }

    follows
}


// {"S": [["S", "^", "P"], ["P"]]}
// {"P": [["P", "v", "Q"], ["Q"]]}
// {"Q": [["[", "Q", "]"], ["sentence"]]}

// {"S": [["S", "^", "P"], ["P"]], "P": [["P", "v", "Q"], ["Q"]], "Q": [["[", "Q", "]"], ["sentence"]]}