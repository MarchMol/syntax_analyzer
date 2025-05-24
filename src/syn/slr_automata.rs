use std::{
    collections::{HashMap, HashSet},
    hash::Hash
};

pub type ActionTable = HashMap<(u8, String), String>;
pub type GotoTable = HashMap<(u8, String), u8>;

use std::iter::Peekable;

use serde::{Deserialize, Serialize};

#[derive(Eq, Hash, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Element {
    Terminal(String),
    NonTerminal(String),
}

#[derive(Debug)]
pub struct SLR {
    // State id -> State name
    pub icount: u8,
    // State id + Element -> State id
    pub edges: HashMap<u8, HashMap<Element, u8>>,

    // State id -> Array of [ Production id + pointer ]
    contents: HashMap<u8, Vec<(u8, u8)>>,
    current_generation: HashSet<u8>,

    // Finish = State that ends production
    //      Structure: (State Id, Production Id)
    pub finish_states: HashSet<(u8, u8)>,

    // Acceptance = State that ends extended production
    //      Structure: (State Id)
    pub acceptance_states: HashSet<u8>,

    // Production Id -> Array of elements
    pub productions: HashMap<u8, Vec<Element>>,

    // Given an Element, what productions is it the head of?
    heads: HashMap<Element, HashSet<u8>>,
}

impl SLR {
    pub fn new(
        productions: &HashMap<String, Vec<Vec<String>>>, 
        terminals: &HashSet<String>, 
        init_symbol: &String
    ) -> Self {
        // Extend Grammar
        let mut heads: HashMap<Element, HashSet<u8>> = HashMap::new();
        let mut fprods: HashMap<u8, Vec<Element>> = HashMap::new();
        fprods.insert(
            0,
            Vec::from([
                Element::NonTerminal("S\'".to_string()),
                Element::NonTerminal(init_symbol.clone()),
            ]),
        );
        heads.insert(Element::NonTerminal("S\'".to_string()), HashSet::from([0]));

        //
        let mut counter = 1;
        let mut keys: Vec<_> = productions.keys().cloned().collect();
        keys.sort();

        for h in keys {
            let h_e = Element::NonTerminal(h.to_string());
            if let Some(opt) = productions.get(&h) {
                for p in opt {
                    let mut tem_prod: Vec<Element> = Vec::new();
                    tem_prod.push(h_e.clone());
                    for e in p {
                        if terminals.contains(e) {
                            tem_prod.push(Element::Terminal(e.to_string()));
                        } else {
                            tem_prod.push(Element::NonTerminal(e.to_string()))
                        }
                    }
                    fprods.insert(counter, tem_prod);
                    heads
                        .entry(h_e.clone())
                        .or_insert_with(HashSet::new)
                        .insert(counter);
                    counter += 1;
                }
            }
        }
        SLR {
            icount: 0,
            edges: HashMap::new(),
            contents: HashMap::new(),
            acceptance_states: HashSet::new(),
            finish_states: HashSet::new(),
            productions: fprods,
            current_generation: HashSet::new(),
            heads,
        }
    }

    pub fn is_finish(&mut self, id: u8) -> Option<u8> {
        let mut is_finish: Option<u8> = None;
        if let Some(prod_array) = self.contents.get(&id) {
            for prod in prod_array {
                if let Some(element_array) = self.productions.get(&prod.0) {
                    let prod_len = element_array.len();
                    if prod_len == (prod.1 + 1) as usize {
                        is_finish = Some(prod.0)
                    }
                }
            }
        }
        is_finish
    }
    pub fn create_state(&mut self, content: Vec<(u8, u8)>) -> u8 {
        let new_id = self.icount + 1;
        self.icount += 1;
        self.contents.insert(new_id, content);
        if let Some(finished_prod) = self.is_finish(new_id) {
            if finished_prod == 0 {
                self.acceptance_states.insert(new_id);
            } else {
                self.finish_states.insert((new_id, finished_prod));
            }
        }
        new_id
    }

    pub fn add_edge(&mut self, from: u8, to: u8, trans: Element) {
        self.edges
            .entry(from)
            .or_insert_with(HashMap::new)
            .insert(trans, to);
    }

    pub fn print_state(&self, state_index: u8) -> String {
        let mut state_content = String::new();
        if let Some(contents) = self.contents.get(&state_index) {
            state_content += &format!("I{}\n", state_index);

            for prod_id in contents {
                // println!("{:?}, {:?}",self.productions.get(&prod.0),prod.1);
                let mut line = String::new();
                if let Some(prod) = self.productions.get(&prod_id.0) {
                    for (i, e) in prod.iter().enumerate() {
                        if let Element::Terminal(str) = e {
                            line.push('"');
                            line += str;
                            line.push('"');
                            line += " ";
                        } else {
                            if let Element::NonTerminal(str) = e {
                                line += str;
                                line += " ";
                            }
                        }
                        if i == 0 {
                            line += "-> "
                        }
                        if i == prod_id.1 as usize {
                            line += ". "
                        }
                    }
                    state_content += &format!("~ {}\n", line);
                }
            }
        } else {
            panic!(
                "~ ~ Warning: no productions for state with index '{:?}'",
                state_index
            )
        }
        state_content
    }

    pub fn requires_closure(&mut self, content: &Vec<(u8, u8)>) -> Vec<Element> {
        let mut closures: Vec<Element> = Vec::new();
        for pointed_prod in content {
            if let Some(prod) = self.productions.get(&pointed_prod.0) {
                if let Some(next_elem) = prod.get(pointed_prod.1 as usize + 1) {
                    match next_elem {
                        Element::NonTerminal(_) => {
                            closures.push(next_elem.clone());
                        }
                        Element::Terminal(_) => {}
                    }
                }
            }
        }
        closures
    }

    pub fn generate(&mut self) {
        // State 0
        let mut i0_content: Vec<(u8, u8)> = Vec::new();
        i0_content.push((0,0));
        let mut is_closed: Vec<Element> = Vec::new();
        let mut counter = 0;
        loop {

            let mut to_close = self.requires_closure(&i0_content);
            to_close.retain(|x| !is_closed.contains(x));
            if counter>self.heads.len(){
                panic!("~ Error SLR: Stuck in infinite loop trying to apply closure to: {:?}",to_close);
            }

                    // No closure necessary
            if to_close.is_empty() {
                        // println!("Finished closures!!");
                break;
            }
                    // Close all
            for tc in to_close {
                if let Some(to_add) = self.heads.get(&tc) {
                            // println!("{:?} leads to => {:?}",tc,to_add);
                    for ta in to_add {
                        if !i0_content.contains(&(*ta, 0)) {
                            i0_content.push((*ta, 0));
                        }
                    }
                    is_closed.push(tc);
                            // println!(" ** now its {:?}",state_const);
                }
            }
            counter+=1;
        }

        self.contents.insert(0, i0_content);

        // first layer
        self.calculate_w_generation(Vec::from([0]));
        while !self.current_generation.is_empty() {
            // print!("\rGeneration num: {}", &self.current_generation.len());
            let mut sorted_vec: Vec<u8> = self.current_generation.iter().cloned().collect();
            sorted_vec.sort_by(|a, b| a.cmp(b));
            self.current_generation.clear();
            self.calculate_w_generation(sorted_vec);
        }
        println!("");

        // show finish & acceptance
        // println!("Finish states: {:?}", self.finish_states);
        // println!("Acceptance states: {:?}", self.acceptance_states);

        // <<< NUEVA SECCIÓN: imprimir todos los estados >>>
        for state_id in 0..=self.icount {
            let _dump = self.print_state(state_id);
            // println!("{}", dump);
        }
    }

    pub fn calculate_w_generation(&mut self, generation: Vec<u8>) {
        // 0. Iterate through states in current generation
        for last_id in generation {
            // println!("~ I{}: ",last_id);
            // Detect where pointer is and possible transitions
            let mut outgoing_trans: Vec<(Element, HashSet<u8>)> = Vec::new();
            if let Some(last_contents) = self.contents.get(&last_id) {
                // Get contents of origin state
                for (i, pointed_prod) in last_contents.iter().enumerate() {
                    // For each pointed production
                    if let Some(prod) = self.productions.get(&pointed_prod.0) {
                        // ProdId->Production
                        if let Some(possible_trans) = prod.get(pointed_prod.1 as usize + 1) {
                            if let Some((_, set)) = outgoing_trans
                                .iter_mut()
                                .find(|(e, _)| *e == *possible_trans)
                            {
                                set.insert(i as u8);
                            } else {
                                let mut set = HashSet::new();
                                set.insert(i as u8);
                                outgoing_trans.push((possible_trans.clone(), set));
                            }
                        }
                    }
                }
            }
            // iterate through possibles
            for trans in outgoing_trans {
                // println!("Edge: {:?}",trans.0);

                let mut state_const: Vec<(u8, u8)> = Vec::new();

                let mut sorted_vec: Vec<u8> = trans.1.iter().cloned().collect();
                sorted_vec.sort_by(|a, b| b.cmp(a));
                if let Some(last_contents) = self.contents.get(&last_id) {
                    for prod_id in sorted_vec {
                        if let Some(pointed_prod) = last_contents.get(prod_id as usize) {
                            let pointer = pointed_prod.1 + 1;
                            state_const.push((pointed_prod.0, pointer));
                        }
                    }
                }

                // 2. Closure loop
                // println!("Before closure: {:?}",state_const);
                // println!("* Closures:");
                let mut is_closed: Vec<Element> = Vec::new();
                loop {
                    let mut to_close = self.requires_closure(&state_const);
                    to_close.retain(|x| !is_closed.contains(x));

                    // No closure necessary
                    if to_close.is_empty() {
                        // println!("Finished closures!!");
                        break;
                    }
                    // Close all
                    for tc in to_close {
                        if let Some(to_add) = self.heads.get(&tc) {
                            // println!("{:?} leads to => {:?}",tc,to_add);
                            for ta in to_add {
                                if !state_const.contains(&(*ta, 0)) {
                                    state_const.push((*ta, 0));
                                }
                            }
                        } else {
                            println!("WARNING: heads.get({:?}) returned None", tc);
                        }
                        is_closed.push(tc.clone());
                        // println!(" ** now its {:?}",state_const);
                    }
                }
                // println!("{:?}", state_const);
                // 3. Does it compare to other states?
                // Check if it exists
                let mut exists: Option<u8> = None;
                for existing_state in 0..self.icount {
                    if let Some(es_content) = self.contents.get(&existing_state) {
                        if are_equal(es_content, &state_const) {
                            // Already exists
                            exists = Some(existing_state);
                        }
                    }
                }
                // Add connections
                if let Some(existing) = exists {
                    self.add_edge(last_id, existing, trans.0.clone());
                } else {
                    let new_id = self.create_state(state_const.clone());
                    self.current_generation.insert(new_id);
                    self.add_edge(last_id, new_id, trans.0.clone());
                }
                // print!("\n");
            }
        }
    }

    /// Construye las tablas ACTION y GOTO usando los FOLLOW sets
    pub fn build_parsing_table(
        &self,
        follows: &HashMap<String, HashSet<String>>,
    ) -> (ActionTable, GotoTable) {
        let mut action: ActionTable = HashMap::new();
        let mut goto: GotoTable = HashMap::new();

        // 1) Shift y Goto desde las transiciones
        for (&state, trans_map) in &self.edges {
            for (sym, &dest) in trans_map {
                match sym {
                    Element::Terminal(t) => {
                        action.insert((state, t.clone()), format!("s{}", dest));
                    }
                    Element::NonTerminal(nt) => {
                        goto.insert((state, nt.clone()), dest);
                    }
                }
            }
        }

        // 2) Reduce: para cada estado final (no-accept) y cada a ∈ FOLLOW(head)
        for &(state, prod_id) in &self.finish_states {
            // cabeza de la producción
            let head = if let Element::NonTerminal(ref nt) = self.productions[&prod_id][0] {
                nt.clone()
            } else {
                continue;
            };

            if let Some(fset) = follows.get(&head) {
                for term in fset {
                    action.insert((state, term.clone()), format!("r{}", prod_id));
                }
            }
        }

        // 3) Accept
        for &state in &self.acceptance_states {
            action.insert((state, "$".to_string()), "acc".to_string());
        }

        (action, goto)
    }

}

pub fn are_equal(a: &Vec<(u8, u8)>, b: &Vec<(u8, u8)>) -> bool {
    let mut a_sorted = a.clone();
    let mut b_sorted = b.clone();
    a_sorted.sort_unstable(); // tuples implement Ord
    b_sorted.sort_unstable();
    a_sorted == b_sorted
}



fn compute_token_offset(tokens: &[String], error_index: usize) -> usize {
    let mut offset = 0;
    for (i, token) in tokens.iter().enumerate() {
        if i == error_index {
            break;
        }
        offset += token.len() + 1;
    }
    offset
}

