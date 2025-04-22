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
        let first = compute_first(nt, &grammar, &terminales, &no_terminales, &mut firsts, &mut visited);
        firsts.insert(nt.clone(), first);
    }

    firsts
}

// ["S": [["S", "^", "P"], ["P"]]]