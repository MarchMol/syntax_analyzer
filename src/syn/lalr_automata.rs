//! Skeleton of a LALR(1) automaton.
//! The heavy logic (closure, goto, merge) will be completed next,
//! but this code already builds a single initial state so that the
//! whole project compiles and runs in LALR mode.

use super::slr_automata::Element;
use std::collections::{HashMap, HashSet};

/* ──────────────────────────────────────────────────────────
LR(1) ITEM
prod_id  ·  α  lookahead
---------------------------------------------------------*/

#[derive(Clone, Debug)]
pub struct ItemLR1 {
    pub prod_id: u8, // production number
    pub dot: usize,  // dot position
    pub lookahead: HashSet<String>,
}

// Hash / Eq **only by (prod_id, dot)** — lookahead is merged later
impl std::hash::Hash for ItemLR1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.prod_id.hash(state);
        self.dot.hash(state);
    }
}
impl PartialEq for ItemLR1 {
    fn eq(&self, other: &Self) -> bool {
        self.prod_id == other.prod_id && self.dot == other.dot
    }
}
impl Eq for ItemLR1 {}

/* ──────────────────────────────────────────────────────────
STATE
---------------------------------------------------------*/

#[derive(Clone, Debug)]
pub struct State {
    pub id: u8,
    pub items: HashSet<ItemLR1>,
    pub transitions: HashMap<String, u8>, // symbol → state_id
}

/* ──────────────────────────────────────────────────────────
LALR AUTOMATON
---------------------------------------------------------*/

pub struct LALR {
    pub productions: HashMap<u8, Vec<Element>>,
    pub terminals: HashSet<String>,
    pub init_symbol: String,

    /// Canonical collection of (merged) LALR states
    pub states: Vec<State>,
}

impl LALR {
    /// Build an empty automaton that owns a **clone** of the grammar.
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

    // --------------------------------------------------
    // FIRST(β a) helper (very simplified for now)
    // --------------------------------------------------
    fn first_sequence(
        seq: &[Element],
        first: &HashMap<String, HashSet<String>>,
        fallback: &HashSet<String>,
    ) -> HashSet<String> {
        if seq.is_empty() {
            return fallback.clone();
        }
        match &seq[0] {
            Element::Terminal(t) => [t.clone()].into(),
            Element::NonTerminal(nt) => first.get(nt).cloned().unwrap_or_default(),
            _ => HashSet::new(),
        }
    }

    // --------------------------------------------------
    // LR(1) CLOSURE — stub: returns the set unchanged
    // --------------------------------------------------
    fn closure_lr1(
        items: &HashSet<ItemLR1>,
        _prods: &HashMap<u8, Vec<Element>>,
        _first: &HashMap<String, HashSet<String>>,
    ) -> HashSet<ItemLR1> {
        items.clone() // TODO: real closure
    }

    // --------------------------------------------------
    // LR(1) GOTO — stub: returns empty set
    // --------------------------------------------------
    fn goto_lr1(
        _items: &HashSet<ItemLR1>,
        _sym: &str,
        _prods: &HashMap<u8, Vec<Element>>,
        _first: &HashMap<String, HashSet<String>>,
    ) -> HashSet<ItemLR1> {
        HashSet::new() // TODO: real goto
    }

    // --------------------------------------------------
    // GENERATE canonical LR(1) collection *and* merge ⇢ LALR.
    // For now we create only I0, so the rest of the code compiles.
    // --------------------------------------------------
    pub fn generate(&mut self, first: &HashMap<String, HashSet<String>>) {
        // Augmented production must be id = 0
        let mut look = HashSet::new();
        look.insert("$".to_string());
        let mut start_item = ItemLR1 {
            prod_id: 0,
            dot: 0,
            lookahead: look,
        };

        let mut kernel = HashSet::new();
        kernel.insert(start_item);
        let closure0 = Self::closure_lr1(&kernel, &self.productions, first);

        self.states.push(State {
            id: 0,
            items: closure0,
            transitions: HashMap::new(),
        });

        // TODO: expand with BFS over goto() + merge
    }

    // --------------------------------------------------
    // Build ACTION / GOTO tables — still empty.
    // --------------------------------------------------
    pub fn build_parsing_table(
        &self,
        _follows: &HashMap<String, HashSet<String>>,
    ) -> (HashMap<(u8, String), String>, HashMap<(u8, String), u8>) {
        (HashMap::new(), HashMap::new()) // will be filled next commit
    }
}
