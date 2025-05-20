use crate::syn::slr_automata::Element;
use std::collections::HashMap;

pub fn flatten_productions(
    raw: &HashMap<String, Vec<Vec<String>>>
) -> HashMap<usize, Vec<Element>> {
    let mut result = HashMap::new();
    let mut counter = 1;

    // println!("Productions: {:?}", raw);

    for (head, bodies) in raw {
        for body in bodies {
            let mut production = vec![Element::NonTerminal(head.clone())];
            for symbol in body {
                if symbol.chars().next().unwrap_or('_').is_uppercase() {
                    production.push(Element::NonTerminal(symbol.clone()));
                } else {
                    production.push(Element::Terminal(symbol.clone()));
                }
            }
            result.insert(counter, production);
            counter += 1;
        }
    }

    println!("\nFlattened Productions:");
    for (id, prod) in &result {
        if let Some(Element::NonTerminal(lhs)) = prod.first() {
            let rhs: Vec<String> = prod[1..].iter().map(|e| match e {
                Element::Terminal(t) => t.clone(),
                Element::NonTerminal(nt) => nt.clone(),
            }).collect();
            println!("r{}: {} -> {}", id, lhs, rhs.join(" "));
        }
    }

    result
}
