use crate::syn::lalr_automata::LALR;
use petgraph::dot::Dot;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Graph;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::process::Command;

use crate::syn::slr_automata::{Element, SLR};

pub fn render_png(slr: &SLR, filename: &str) {
    let finish: Vec<u8> = slr.finish_states.iter().map(|(b, _)| *b).collect();
    let acceptance: Vec<u8> = slr.acceptance_states.iter().map(|b| *b).collect();

    let mut purple: HashSet<String> = HashSet::new();
    let mut green: HashSet<String> = HashSet::new();

    let mut graph_data: HashMap<String, HashMap<String, String>> = HashMap::new();
    for outer in &slr.edges {
        let mut tem: HashMap<String, String> = HashMap::new();
        for inner in outer.1 {
            let mut trans = String::new();
            match inner.0 {
                Element::Terminal(str) => {
                    trans += &str;
                }
                Element::NonTerminal(str) => {
                    trans += &str;
                }
            }
            let inner_str = slr.print_state(*inner.1);
            tem.insert(inner_str.clone(), trans);
            if finish.contains(inner.1) {
                purple.insert(inner_str.clone());
            }
            if acceptance.contains(inner.1) {
                green.insert(inner_str.clone());
            }
        }
        let outer_str = slr.print_state(*outer.0);
        if finish.contains(outer.0) {
            purple.insert(outer_str.clone());
        }
        if acceptance.contains(outer.0) {
            green.insert(outer_str.clone());
        }
        graph_data.insert(outer_str, tem);
    }

    // Step 1: Create graph with node labels as Strings, and edge labels as Strings
    let mut graph: DiGraph<String, String> = DiGraph::new();

    // Mapping node names to their graph NodeIndex
    let mut node_map: BTreeMap<String, petgraph::prelude::NodeIndex> = BTreeMap::new();

    // Step 2: Add all nodes
    for node in graph_data
        .keys()
        .chain(graph_data.values().flat_map(|targets| targets.keys()))
    {
        node_map
            .entry(node.clone())
            .or_insert_with(|| graph.add_node(node.clone()));
    }

    // Step 3: Add edges with labels
    for (from, targets) in &graph_data {
        for (to, label) in targets {
            let from_idx = node_map[from];
            let to_idx = node_map[to];
            graph.add_edge(from_idx, to_idx, label.clone());
        }
    }

    // Step 4: Write DOT file with labels
    let dot_file = filename.to_string() + ".dot";
    let png_file = filename.to_string() + ".png";
    let node_attr = |_: &Graph<String, String>, (_node_idx, label): (NodeIndex, &String)| {
        if purple.contains(label) {
            r#"shape=box, color=purple, penwidth=5"#.to_string()
        } else {
            if green.contains(label) {
                r#"shape=box, color=green, penwidth=5"#.to_string()
            } else {
                r#"shape=box"#.to_string()
            }
        }
    };

    let dot = Dot::with_attr_getters(&graph, &[], &|_graph, _edge| String::new(), &node_attr);
    fs::write(&dot_file, dot.to_string()).expect("Unable to write file");

    // Write to a file

    let output = Command::new("dot")
        .arg("-Tpng") // Output format: PNG
        .arg(dot_file) // Input file
        .arg("-o") // Output file flag
        .arg(&png_file) // Output file name
        .arg("-Grandom_seed=42")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            // println!("Graph saved as '{}'", png_file);
        }
        Ok(output) => {
            // eprintln!(
            // "dot command failed: {}",
            // String::from_utf8_lossy(&output.stderr)
            // );
        }
        Err(err) => {
            // eprintln!("Can't render. Failed to execute dot: {}", err);
        }
    }
}

// DFA

pub fn render_lalr(lalr: &LALR, filename: &str) {
    // 1. Rutas de salida
    let dot_file = format!("{}.dot", filename);
    let png_file = format!("{}.png", filename);

    // 2. Detectar estados de finish y acceptance
    let mut finish_states: HashSet<usize> = HashSet::new();
    let mut acceptance_states: HashSet<usize> = HashSet::new();
    for (idx, state) in lalr.states.iter().enumerate() {
        for item in &state.items {
            let prod_len = lalr.productions[&item.prod_id].len();
            if item.dot + 1 == prod_len {
                if item.prod_id == 0 {
                    acceptance_states.insert(idx);
                } else {
                    finish_states.insert(idx);
                }
            }
        }
    }

    // 3. Construir etiquetas “al estilo SLR” para cada estado:
    let mut kernels: BTreeMap<usize, String> = BTreeMap::new();
    for (idx, state) in lalr.states.iter().enumerate() {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("I{}", idx));

        for item in &state.items {
            if let Some(prod_rhs) = lalr.productions.get(&item.prod_id) {
                let mut line = String::new();

                for (i, elem) in prod_rhs.iter().enumerate() {
                    match elem {
                        Element::Terminal(s) => {
                            line.push('"');
                            line.push_str(s);
                            line.push('"');
                            line.push(' ');
                        }
                        Element::NonTerminal(s) => {
                            line.push_str(s);
                            line.push(' ');
                        }
                    }
                    if i == 0{
                         line.push_str("->");
                    }
                    if i == item.dot {
                        line.push_str(". ");
                    }
                }

                
                
                let mut tem_line = String::new();
                let copy = item.lookahead.clone();
                let mut vec: Vec<String> = copy.into_iter().collect();
                vec.sort();
                for (i,la) in vec.iter().enumerate(){
                    if i == 0{
                        tem_line+=&format!("{}",la);
                    } else{
                        tem_line+=&format!(", {}",la);
                    }
                    
                }
                let full = format!("~ {} | {{{}}}", line.trim_end(), tem_line);
                lines.push(full);
            }
        }

        lines.sort();
        kernels.insert(idx, lines.join("\n"));
    }

    // 4. Construir el grafo
    let mut graph: DiGraph<String, String> = DiGraph::new();
    let mut node_map: BTreeMap<usize, NodeIndex> = BTreeMap::new();

    for (idx, label) in &kernels {
        let ni = graph.add_node(label.clone());
        node_map.insert(*idx, ni);
    }

    for (idx, state) in lalr.states.iter().enumerate() {
        let from_idx = node_map[&idx];
        for (sym, &to) in &state.transitions {
            let to_idx = node_map[&(to as usize)];
            graph.add_edge(from_idx, to_idx, sym.clone());
        }
    }

    // 5. Definir atributos de nodo
    let node_attr = |_: &DiGraph<String, String>, (ni, _): (NodeIndex, &String)| {
        let idx = ni.index();
        if acceptance_states.contains(&idx) {
            "shape=box, color=green, penwidth=5".to_string()
        } else if finish_states.contains(&idx) {
            "shape=box, color=purple, penwidth=5".to_string()
        } else {
            "shape=box".to_string()
        }
    };

    // 6. Escribir .dot y generar .png
    let dot = Dot::with_attr_getters(&graph, &[], &|_, _| String::new(), &node_attr);
    fs::write(&dot_file, dot.to_string()).expect("No pude escribir el .dot de LALR");
    let output = Command::new("dot")
        .arg("-Tpng")
        .arg(&dot_file)
        .arg("-o")
        .arg(&png_file)
        .arg("-Grandom_seed=42")
        .output();
    if let Err(err) = output {
        eprintln!("Error ejecutando dot para LALR: {}", err);
    }
}

pub fn get_all_states(ginfo: &HashMap<char, HashMap<String, char>>) -> Vec<String> {
    let states = ginfo.keys();
    let mut all_states: Vec<String> = Vec::new();
    for from in states {
        if !all_states.contains(&from.to_string()) {
            all_states.push(from.to_string());
        }
        match ginfo.get(from) {
            Some(ts) => {
                let dest = ts.values();
                for to in dest {
                    if !all_states.contains(&to.to_string()) {
                        all_states.push(to.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    all_states
}

pub fn generate_graph(
    ginfo: &HashMap<char, HashMap<String, char>>,
    states: &Vec<String>,
) -> Graph<String, String> {
    let mut graph = Graph::<String, String>::new();
    // Creating all nodes
    // println!("{:?}", ginfo);
    for st in states {
        graph.add_node(st.to_string());
    }
    // Creating edges
    for n_index in graph.node_indices() {
        let from_weight = &graph[n_index].chars().next().unwrap();
        match ginfo.get(from_weight) {
            Some(hash) => {
                let trans = hash.keys();
                for tr in trans {
                    match hash.get(tr) {
                        Some(dest_weight) => {
                            let to_node = graph
                                .node_indices()
                                .find(|&i| graph[i] == dest_weight.to_string())
                                .expect("Node not found");
                            graph.add_edge(n_index, to_node, tr.to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        // println!("nodes: {:?}",from_weight);
    }
    graph
}

pub fn render_dfa(
    ginfo: &HashMap<char, HashMap<String, char>>,
    accept: &HashSet<char>,
    dest: &str,
) {
    let all_states = get_all_states(&ginfo);
    let graph = generate_graph(ginfo, &all_states);
    let start_n = graph
        .node_indices()
        .find(|&i| graph[i] == "A")
        .expect("Node not found");
    let start = format!(
        "\"\" [style=invisible, width=0, height=0];\n\"\" -> {:?};\n",
        start_n.index()
    );

    let mut dot_output = format!("{}", Dot::with_config(&graph, &[]));
    dot_output.insert_str(dot_output.len() - 2, &start);

    for node in accept {
        let tem_node = graph
            .node_indices()
            .find(|&i| graph[i] == node.to_string())
            .expect("Node not found");
        let node_label = format!("{} [", tem_node.index());
        let peripheries_attr = format!("{} [peripheries=2, ", tem_node.index());
        if let Some(pos) = dot_output.find(&node_label) {
            dot_output.replace_range(pos..pos + node_label.len(), &peripheries_attr);
        }
    }

    let dot_file = &format!("{}.dot", dest);
    let png_file = &format!("{}.png", dest);
    let mut file = fs::File::create(dot_file).expect("Failed to create .dot file");
    file.write_all(dot_output.as_bytes())
        .expect("Failed to write to .dot file");

    let output = Command::new("dot")
        .arg("-Tpng") // Output format: PNG
        .arg(dot_file) // Input file
        .arg("-o") // Output file flag
        .arg(png_file) // Output file name
        .output();

    match output {
        Ok(output) if output.status.success() => {
            // println!("Graph saved as '{}'", png_file);
        }
        Ok(output) => {
            // eprintln!(
            //     "dot command failed: {}",
            //     String::from_utf8_lossy(&output.stderr)
            // );
        }
        Err(err) => {
            // eprintln!("Can't render. Failed to execute dot: {}", err);
        }
    }
}
