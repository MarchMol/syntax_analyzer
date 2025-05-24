use std::collections::{BTreeMap, HashMap, HashSet};
use std::process::Command;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::dot::Dot;
use petgraph::Graph;
use std::fs;

use crate::syn::slr_automata::{Element, SLR};

pub fn render_png(slr: &SLR){
    let finish: Vec<u8> = slr.finish_states
    .iter()
    .map(|(b,_)| *b)
    .collect();
    let acceptance: Vec<u8> = slr.acceptance_states.iter().map(|b| *b).collect();
    
    let mut purple: HashSet<String> = HashSet::new();
    let mut green: HashSet<String> = HashSet::new();
    
    let mut graph_data: HashMap<String, HashMap<String, String>> = HashMap::new();
    for outer in &slr.edges{
        let mut tem: HashMap<String, String> = HashMap::new();
        for inner in outer.1{
            let mut trans = String::new();
            match inner.0{
                Element::Terminal(str)=>{
                    trans+=&str;
                },
                Element::NonTerminal(str)=>{
                    trans+=&str;
                }
            }
            let inner_str = slr.print_state(*inner.1);
            tem.insert(inner_str.clone(),trans);
            if finish.contains(inner.1){
                purple.insert(inner_str.clone());
            }
            if acceptance.contains(inner.1){
                green.insert(inner_str.clone());
            }
        }
        let outer_str = slr.print_state(*outer.0);
        if finish.contains(outer.0){
            purple.insert(outer_str.clone());
        }
        if acceptance.contains(outer.0){
            green.insert(outer_str.clone());
        }
        graph_data.insert(outer_str, tem);
    }

    // Step 1: Create graph with node labels as Strings, and edge labels as Strings
    let mut graph: DiGraph<String, String> = DiGraph::new();

    // Mapping node names to their graph NodeIndex
    let mut node_map: BTreeMap<String, petgraph::prelude::NodeIndex> = BTreeMap::new();

    // Step 2: Add all nodes
    for node in graph_data.keys().chain(graph_data.values().flat_map(|targets| targets.keys())) {
        node_map.entry(node.clone()).or_insert_with(|| graph.add_node(node.clone()));
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
    let dot_file = "./graph/slr.dot";
    let png_file = "./graph/slr.png";
    let node_attr = |_: &Graph<String, String>, (_node_idx, label): (NodeIndex, &String)| {
        if purple.contains(label) {
            r#"shape=box, color=purple, penwidth=5"#.to_string()
        } else {
            if green.contains(label){
                r#"shape=box, color=green, penwidth=5"#.to_string()
            } else{
                r#"shape=box"#.to_string()
            }
        }
    };

    let dot= Dot::with_attr_getters(
        &graph,
        &[], 
        &|_graph, _edge| String::new(), 
        &node_attr
    );
    fs::write(dot_file, dot.to_string()).expect("Unable to write file");

    // Write to a file


    let output = Command::new("dot")
        .arg("-Tpng") // Output format: PNG
        .arg(dot_file) // Input file
        .arg("-o") // Output file flag
        .arg(png_file) // Output file name
        .arg("-Grandom_seed=42")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("Graph saved as '{}'", png_file);
        }
        Ok(output) => {
            eprintln!(
                "dot command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(err) => {
            eprintln!("Can't render. Failed to execute dot: {}", err);
        }
    }
}