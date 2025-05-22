use std::collections::{HashMap, HashSet};

/// Minimiza un DFA usando el algoritmo de Hopcroft.
/// Devuelve (minimized_dfa, minimized_accept_states, minimized_start_state).
/// Se asume que el estado inicial original es `'A'`.
pub fn minimize_dfa(
    dfa: &HashMap<char, HashMap<String, char>>,
    accept_states: &HashSet<char>,
) -> (HashMap<char, HashMap<String, char>>, HashSet<char>, char) {
    // Construir alfabeto
    let mut alphabet = HashSet::new();
    for trans in dfa.values() {
        for sym in trans.keys() {
            alphabet.insert(sym.clone());
        }
    }

    // Completar DFA con estado sink '?'
    let sink = '?';
    let mut complete = dfa.clone();
    complete.entry(sink).or_default();
    for (&state, _) in dfa {
        let row = complete.entry(state).or_default();
        for sym in &alphabet {
            row.entry(sym.clone()).or_insert(sink);
        }
    }
    for sym in &alphabet {
        complete.get_mut(&sink).unwrap().insert(sym.clone(), sink);
    }

    // Partición inicial P = {F, Q\\F}
    let all_states: HashSet<char> = complete.keys().cloned().collect();
    let f = accept_states.clone();
    let non_f: HashSet<char> = all_states.difference(&f).cloned().collect();
    let mut p = Vec::new();
    if !f.is_empty() {
        p.push(f.clone());
    }
    if !non_f.is_empty() {
        p.push(non_f.clone());
    }

    // Conjunto de trabajo W
    let mut w = vec![p[0].clone()];

    // Hopcroft refinement
    while let Some(a) = w.pop() {
        for sym in &alphabet {
            let mut x = HashSet::new();
            for &s in &all_states {
                if complete
                    .get(&s)
                    .and_then(|m| m.get(sym))
                    .copied()
                    .filter(|t| a.contains(t))
                    .is_some()
                {
                    x.insert(s);
                }
            }

            let mut new_p = Vec::new();
            for y in p.drain(..) {
                let intersection: HashSet<char> = y.intersection(&x).cloned().collect();
                let difference: HashSet<char> = y.difference(&x).cloned().collect();
                if !intersection.is_empty() && !difference.is_empty() {
                    new_p.push(intersection.clone());
                    new_p.push(difference.clone());
                    if let Some(pos) = w.iter().position(|w| *w == y) {
                        w.remove(pos);
                        w.push(intersection);
                        w.push(difference);
                    } else if intersection.len() <= difference.len() {
                        w.push(intersection);
                    } else {
                        w.push(difference);
                    }
                } else {
                    new_p.push(y);
                }
            }
            p = new_p;
        }
    }

    // Mapear cada clase a un nuevo char
    let mut mapping = HashMap::new();
    let mut next_name = 'A';
    for block in &p {
        for &st in block {
            mapping.insert(st, next_name);
        }
        next_name = ((next_name as u8) + 1) as char;
    }

    // Construir DFA minimizado
    let mut minimized = HashMap::new();
    let mut minimized_accepts = HashSet::new();
    for block in &p {
        let repr = *block.iter().next().unwrap();
        let new_state = mapping[&repr];
        let mut row = HashMap::new();
        for sym in &alphabet {
            let target = complete
                .get(&repr)
                .and_then(|m| m.get(sym))
                .copied()
                .unwrap_or(sink);
            row.insert(sym.clone(), mapping[&target]);
        }
        minimized.insert(new_state, row);
        if block.iter().any(|s| accept_states.contains(s)) {
            minimized_accepts.insert(new_state);
        }
    }

    // Eliminar sink
    let sink_name = mapping[&sink];
    minimized.remove(&sink_name);
    minimized_accepts.remove(&sink_name);

    // Nuevo estado inicial = mapping de 'A'
    let minimized_start = mapping[&'A'];

    // println!("Mapa de estados: {:?}", minimized);
    (minimized, minimized_accepts, minimized_start)
}
