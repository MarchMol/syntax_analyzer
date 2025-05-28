use super::slr_automata::Element;
use std::collections::{HashMap, HashSet};

/// Esqueleto del autómata LALR
pub struct LALR {
    pub productions: HashMap<u8, Vec<Element>>,
    pub terminals: HashSet<String>,
    pub init_symbol: String,
    // /// Aquí luego irán: estados, lookaheads, transiciones...
    // pub states: Vec<State>,
}

impl LALR {
    /// Clona la gramática y prepara la estructura
    pub fn new(
        productions: &HashMap<u8, Vec<Element>>,
        terminals: &HashSet<String>,
        init_symbol: &String,
    ) -> Self {
        Self {
            productions: productions.clone(),
            terminals: terminals.clone(),
            init_symbol: init_symbol.clone(),
            // states: Vec::new(),
        }
    }

    /// Genera los conjuntos de ítems y transiciones (todo por hacer)
    pub fn generate(&mut self) {
        // TODO: closure/goto de LR(1) + merge para LALR
    }

    /// Construye las tablas ACTION y GOTO a partir de los follows
    pub fn build_parsing_table(
        &self,
        follows: &HashMap<String, HashSet<String>>,
    ) -> (HashMap<(u8, String), String>, HashMap<(u8, String), u8>) {
        // TODO: recorrer self.states + lookaheads y llenar las tablas
        (HashMap::new(), HashMap::new())
    }
}
