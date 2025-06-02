//! LALR(1) automaton – complete implementation
//! -------------------------------------------------------------

use std::collections::{hash_map::Entry, HashMap, HashSet};

use super::slr_automata::Element;

/*──────────────────────────────────────────────────────────────*/
/* LR(1) ITEM                                                   */
/*──────────────────────────────────────────────────────────────*/
#[derive(Clone, Debug, Eq)]
pub struct ItemLR1 {
    pub prod_id: u8,
    pub dot: usize,
    pub lookahead: HashSet<String>,
}

impl std::hash::Hash for ItemLR1 {
    fn hash<H: std::hash::Hasher>(&self, s: &mut H) {
        self.prod_id.hash(s);
        self.dot.hash(s);
    }
}
impl PartialEq for ItemLR1 {
    fn eq(&self, other: &Self) -> bool {
        self.prod_id == other.prod_id && self.dot == other.dot
    }
}

/*──────────────────────────────────────────────────────────────*/
/* STATE                                                        */
/*──────────────────────────────────────────────────────────────*/
#[derive(Clone, Debug)]
pub struct State {
    pub id: u8,
    pub items: HashSet<ItemLR1>,
    pub transitions: HashMap<String, u8>,
}

/*──────────────────────────────────────────────────────────────*/
/* LALR AUTOMATON                                               */
/*──────────────────────────────────────────────────────────────*/
pub struct LALR {
    pub productions: HashMap<u8, Vec<Element>>,
    pub terminals: HashSet<String>,
    pub init_symbol: String,

    pub states: Vec<State>,
}

impl LALR {
    /*--------------------------------------------------------*/
    pub fn new(
        productions: &HashMap<u8, Vec<Element>>,
        terminals: &HashSet<String>,
        init_symbol: &String,
    ) -> Self {
        Self {
            productions: productions.clone(),
            terminals: terminals.clone(),
            init_symbol: init_symbol.clone(),
            states: Vec::new(),
        }
    }

    /*--------------------------------------------------------*/
    /* helper: FIRST(β a)                                     */
    /*--------------------------------------------------------*/
    fn first_sequence(
        seq: &[Element],
        first: &HashMap<String, HashSet<String>>,
        mut lks: HashSet<String>,
    ) -> HashSet<String> {
        if seq.is_empty() {
            return lks;
        }
        match &seq[0] {
            Element::Terminal(t) => {
                lks.insert(t.clone());
                lks
            }
            Element::NonTerminal(nt) => {
                lks.extend(first.get(nt).cloned().unwrap_or_default());
                lks
            }
        }
    }

    /*--------------------------------------------------------*/
    /* CLOSURE LR(1)                                           */
    /*--------------------------------------------------------*/
    fn closure_lr1(
        mut items: HashSet<ItemLR1>,
        prods: &HashMap<u8, Vec<Element>>,
        first: &HashMap<String, HashSet<String>>,
    ) -> HashSet<ItemLR1> {
        let mut changed = true;
        while changed {
            changed = false;
            let snapshot: Vec<_> = items.clone().into_iter().collect();
            for it in snapshot {
                let rhs = &prods[&it.prod_id];
                if it.dot >= rhs.len() - 1 {
                    continue; // dot before EOF
                }
                let next_sym = &rhs[it.dot + 1];
                let beta = &rhs[it.dot + 2..]; // β
                if let Element::NonTerminal(B) = next_sym {
                    // FIRST(β a)
                    let look = Self::first_sequence(beta, first, it.lookahead.clone());
                    // For every production B → γ
                    for (p_id, rhs_b) in prods {
                        if let Element::NonTerminal(lhs) = &rhs_b[0] {
                            if lhs == B {
                                let kernel = ItemLR1 {
                                    prod_id: *p_id,
                                    dot: 0,
                                    lookahead: look.clone(),
                                };
                                match items.get(&kernel) {
                                    None => {
                                        items.insert(kernel);
                                        changed = true;
                                    }
                                    Some(existing) => {
                                        // merge lookaheads
                                        let mut merged = existing.lookahead.clone();
                                        let before = merged.len();
                                        merged.extend(look.clone());
                                        if merged.len() > before {
                                            items.replace(ItemLR1 {
                                                prod_id: existing.prod_id,
                                                dot: existing.dot,
                                                lookahead: merged,
                                            });
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        items
    }

    /*--------------------------------------------------------*/
    /* GOTO LR(1)                                              */
    /*--------------------------------------------------------*/
    fn goto_lr1(
        items: &HashSet<ItemLR1>,
        sym: &str,
        prods: &HashMap<u8, Vec<Element>>,
        first: &HashMap<String, HashSet<String>>,
    ) -> HashSet<ItemLR1> {
        let mut next = HashSet::new();
        for it in items {
            let rhs = &prods[&it.prod_id];
            if it.dot < rhs.len() - 1 {
                if let Element::Terminal(t) = &rhs[it.dot + 1] {
                    if t == sym {
                        next.insert(ItemLR1 {
                            prod_id: it.prod_id,
                            dot: it.dot + 1,
                            lookahead: it.lookahead.clone(),
                        });
                    }
                } else if let Element::NonTerminal(nt) = &rhs[it.dot + 1] {
                    if nt == sym {
                        next.insert(ItemLR1 {
                            prod_id: it.prod_id,
                            dot: it.dot + 1,
                            lookahead: it.lookahead.clone(),
                        });
                    }
                }
            }
        }
        Self::closure_lr1(next, prods, first)
    }

    /*--------------------------------------------------------*/
    /* GENERATE LR(1) CANONICAL COLLECTION + MERGE → LALR      */
    /*--------------------------------------------------------*/
    pub fn generate(&mut self, first: &HashMap<String, HashSet<String>>) {
        // ---------- I0 ----------
        let mut i0_la = HashSet::new();
        i0_la.insert("$".to_string());
        let mut items0 = HashSet::new();
        items0.insert(ItemLR1 {
            prod_id: 0,
            dot: 0,
            lookahead: i0_la,
        });
        let i0 = Self::closure_lr1(items0, &self.productions, first);

        // maps LR(0) kernel → state_id
        let mut canonical: HashMap<Vec<(u8, usize)>, u8> = HashMap::new();

        self.states.push(State {
            id: 0,
            items: i0.clone(),
            transitions: HashMap::new(),
        });
        canonical.insert(Self::kernel_key(&i0), 0);

        // BFS over states
        let mut queue: Vec<u8> = vec![0];
        while let Some(sid) = queue.pop() {
            let symbols: HashSet<String> = self.states[sid as usize]
                .items
                .iter()
                .filter_map(|it| {
                    let rhs = &self.productions[&it.prod_id];
                    if it.dot < rhs.len() - 1 {
                        match &rhs[it.dot + 1] {
                            Element::Terminal(t) => Some(t.clone()),
                            Element::NonTerminal(nt) => Some(nt.clone()),
                        }
                    } else {
                        None
                    }
                })
                .collect();

            for sym in symbols {
                let goto_set = Self::goto_lr1(
                    &self.states[sid as usize].items,
                    &sym,
                    &self.productions,
                    first,
                );
                if goto_set.is_empty() {
                    continue;
                }
                let kkey = Self::kernel_key(&goto_set);

                let tid = match canonical.entry(kkey) {
                    Entry::Occupied(o) => *o.get(),
                    Entry::Vacant(v) => {
                        let new_id = self.states.len() as u8;
                        self.states.push(State {
                            id: new_id,
                            items: goto_set.clone(),
                            transitions: HashMap::new(),
                        });
                        v.insert(new_id);
                        queue.push(new_id);
                        new_id
                    }
                };
                self.states[sid as usize]
                    .transitions
                    .insert(sym.clone(), tid);
            }
        }

        // ------------ MERGE lookaheads of identical kernels -------------
        let mut core_map: HashMap<Vec<(u8, usize)>, usize> = HashMap::new();
        for (idx, st) in self.states.clone().into_iter().enumerate() {
            let key = Self::kernel_key(&st.items);
            if let Some(&master) = core_map.get(&key) {
                // merge into master
                let mst_items = &mut self.states[master].items;
                for it in st.items {
                    if let Some(orig) = mst_items.take(&it) {
                        let mut merged = orig.lookahead.clone();
                        merged.extend(it.lookahead);
                        mst_items.insert(ItemLR1 {
                            prod_id: orig.prod_id,
                            dot: orig.dot,
                            lookahead: merged,
                        });
                    } else {
                        mst_items.insert(it);
                    }
                }
                // move transitions
                for (s, tgt) in st.transitions {
                    self.states[master].transitions.entry(s).or_insert(tgt);
                }
            } else {
                core_map.insert(key, idx);
            }
        }
    }

    // helper to get LR(0) core key
    fn kernel_key(items: &HashSet<ItemLR1>) -> Vec<(u8, usize)> {
        let mut v: Vec<(u8, usize)> = items.iter().map(|it| (it.prod_id, it.dot)).collect();
        v.sort();
        v
    }

    /*--------------------------------------------------------*/
    /* BUILD PARSING TABLE                                     */
    /*--------------------------------------------------------*/
    pub fn build_parsing_table(
        &self
    ) -> (HashMap<(u8, String), String>, HashMap<(u8, String), u8>) {
        let mut action = HashMap::new();
        let mut goto = HashMap::new();

        for st in &self.states {
            for it in &st.items {
                
                let rhs = &self.productions[&it.prod_id];
                
                // ---------------------------------
                // Shift
                if it.dot < rhs.len() - 1 {
                    let sym = &rhs[it.dot + 1];
                    if let Element::Terminal(t) = sym {
                        if let Some(&tgt) = st.transitions.get(t) {
                            let ac: Option<&String> =  action.get(&(st.id, t.clone()));
                            match ac {
                                Some(s)=>{
                                    panic!("LALR Error: Ambiguity in shifts\n state:{} token:{} already present with {}",st.id, t, s)
                                }
                                None=>{
                                    action.insert((st.id, t.clone()), format!("s{}", tgt));
                                }
                            }
                            
                        }
                    } else if let Element::NonTerminal(nt) = sym {
                        if let Some(&tgt) = st.transitions.get(nt) {
                            let gt=  action.get(&(st.id, nt.clone()));
                            match gt {
                                Some(s)=>{
                                    panic!("LALR Error: Ambiguity in GOTO\n state:{} token:{} already present with {}",st.id, nt, s)
                                }
                                None=>{
                                    goto.insert((st.id, nt.clone()), tgt);
                                }
                            }
                            
                        }
                    }
                }
                // ---------------------------------
                // Reduce  / Accept
                else {
                    
                    let lhs = if let Element::NonTerminal(lhs) = &rhs[0] {
                        lhs.clone()
                    } else {
                        continue;
                    };
                    if it.prod_id == 0 && it.lookahead.contains("$") {
                        action.insert((st.id, "$".to_string()), "acc".to_string());
                    } else {
                        for la in &it.lookahead {
                            let rd=  action.get(&(st.id, la.clone()));
                            match rd {
                                Some(s)=>{
                                    panic!("LALR Error: Ambiguity in GOTO\n state:{} token:{} already present with {}",st.id, la, s)
                                }
                                None=>{
                                    action.insert((st.id, la.clone()), format!("r{}", it.prod_id));
                                }
                            }
                            
                        }
                        // GOTO entries for <lhs> handled during Shift part
                    }
                }
            }
        }
        // Fill goto table for non-terminals already in transitions
        for st in &self.states {
            for (sym, &tgt) in &st.transitions {
                if !self.terminals.contains(sym) {
                    goto.insert((st.id, sym.clone()), tgt);
                }
            }
        }
        (action, goto)
    }
}
