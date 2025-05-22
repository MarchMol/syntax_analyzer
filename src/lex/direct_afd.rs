use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::lex::grammar_tree::{Tree, TreeNode};
use crate::lex::tokenizer::Token;


pub struct DirectAFD {
    syntax_tree: Rc<Tree>,
}

impl DirectAFD {
    pub fn new(tree: Rc<Tree>) -> Self {
        Self {
            syntax_tree: tree
        }
    }

    pub fn generate_afd(&mut self) {
        self.read_tree();
        self.find_nullable();
        self.find_first_last_pos();
        self.find_followpos();
        self.create_states();
    }

    // Lee el árbol y guarda sus labels
    pub fn read_tree(&self) -> (HashMap<String, String>, String, Vec<String>) {
        let mut labels = HashMap::new();
        let mut literal_count = 1;
        let mut union_count = 1;
        let mut kleene_count = 1;
        let mut concat_count = 1;
        let mut root_key = String::new();
        let mut token_list = Vec::new();

        // Función recursiva que recorre el árbol y asigna etiquetas
        fn traverse(
            node: &TreeNode,
            labels: &mut HashMap<String, String>,
            literal_count: &mut usize,
            union_count: &mut usize,
            kleene_count: &mut usize,
            concat_count: &mut usize,
            token_list: &mut Vec<String>,
        ) -> String {
            // println!("Visitando nodo: {:?}", node.get_value());

            // Obtener los identificadores de los hijos (si existen)
            let left_id = node.get_left().map(|left| {
                let id = traverse(
                    &left,
                    labels,
                    literal_count,
                    union_count,
                    kleene_count,
                    concat_count,
                    token_list,
                );
                // println!("Nodo izquierdo: {:?} -> ID: {:?}", left.get_value(), id);
                id
            });
            let right_id = node.get_right().map(|right| {
                let id = traverse(
                    &right,
                    labels,
                    literal_count,
                    union_count,
                    kleene_count,
                    concat_count,
                    token_list,
                );
                // println!("Nodo izquierdo: {:?} -> ID: {:?}", right.get_value(), id);
                id
            });

            // Asignar identificador al nodo actual
            let node_id = match node.get_value() {
                Token::Literal(c) => {
                    let id = literal_count.to_string();
                    labels.insert(id.clone(), format!("Literal('{}')", c));
                    *literal_count += 1;
                    id
                }
                Token::Range(c, d) => {
                    let id = literal_count.to_string();
                    labels.insert(id.clone(), format!("Range('{},{}')", c, d));
                    *literal_count += 1;
                    id
                }
                Token::Tokener(c,) => {
                    let id = literal_count.to_string();
                    labels.insert(id.clone(), format!("Token('{}')", c));
                    token_list.push(c.to_string());
                    *literal_count += 1;
                    id
                }
                Token::Union => {
                    let id = format!("alpha{}", *union_count);
                    *union_count += 1;
                    if let (Some(c1), Some(c2)) = (left_id.clone(), right_id.clone()) {
                        labels.insert(id.clone(), format!("({}, {})", c1, c2));
                    }
                    id
                }
                Token::Kleene => {
                    let id = format!("beta{}", *kleene_count);
                    *kleene_count += 1;
                    if let Some(c1) = left_id.clone() {
                        labels.insert(id.clone(), format!("({})", c1));
                    }
                    id
                }
                Token::Concat => {
                    let id = format!("gama{}", *concat_count);
                    *concat_count += 1;
                    if let (Some(c1), Some(c2)) = (left_id.clone(), right_id.clone()) {
                        labels.insert(id.clone(), format!("({}, {})", c1, c2));
                    }
                    id
                }
                Token::Sentinel => {
                    let id = literal_count.to_string();
                    labels.insert(id.clone(), "Sentinel".to_string());
                    *literal_count += 1;
                    id
                }
                Token::Empty => {
                    let id = "empty".to_string();
                    labels.insert(id.clone(), "Empty".to_string());
                    id
                }
                _ => unreachable!("Unexpected token type in syntax tree"),
            };

            // println!("Asignando etiquieta: {} -> {:?}", node_id, labels.get(&node_id));

            node_id
        }

        // Llamar a la función de recorrido desde la raíz
        if let Some(root_node) = self.syntax_tree.get_root() {
            // println!("Iniciando recorrido desde la raíz");
            // Realizamos el recorrido y asignamos las etiquetas
            root_key = traverse(
                &root_node,
                &mut labels,
                &mut literal_count,
                &mut union_count,
                &mut kleene_count,
                &mut concat_count,
                &mut token_list,
            );

            // println!("Árbol etiquetado: {:?}", labels);
            // println!("Clave raíz: {}", root_key);
        }

        (labels, root_key, token_list)
    }

    pub fn find_nullable(&self) -> HashMap<String, bool> {
        let (tree_map, _key, _token_list) = self.read_tree();
        let mut nullable_map = HashMap::new();
        // println!("Entrada: {:?}", tree_map);

        // Primera pasada: inicializar literales y Sentinel
        for (key, value) in &tree_map {
            if value.starts_with("Literal") {
                nullable_map.insert(key.clone(), false);
                // println!("Inicializando {} como false (Literal)", key);
            } else if value == "Sentinel" {
                nullable_map.insert(key.clone(), false);
                // println!("Inicializando {} como false (Sentinel)", key);
            } else if value == "Empty" {
                nullable_map.insert(key.clone(), true);
                // println!("Inicializando {} como true (Empty)", key);
            } else if value.starts_with("Range") {
                nullable_map.insert(key.clone(), false);
                // println!("Inicializando {} como false (Range)", key);
            } else if value.starts_with("Token") {
                nullable_map.insert(key.clone(), false);
                // println!("Inicializando {} como false (Token)", key);
            }
        }

        // Fijación: Realizar múltiples pasadas hasta que no haya cambios
        let mut changes = true;
        while changes {
            changes = false;

            // Segunda pasada: calcular valores de Kleene, Concat y Union
            for (key, value) in &tree_map {
                let original_nullable = nullable_map.get(key).cloned();

                if key.starts_with("beta") {
                    // Kleene (beta): El valor siempre es true
                    nullable_map.insert(key.clone(), true);
                    // println!("{} es Kleene, siempre true", key);
                } else if key.starts_with("gama") {
                    if let Some((c1, c2)) = extract_children(value) {
                        let nullable_c1 = *nullable_map.get(&c1).unwrap_or(&false);
                        let nullable_c2 = *nullable_map.get(&c2).unwrap_or(&false);
                        nullable_map.insert(key.clone(), nullable_c1 && nullable_c2);
                        // println!("{} = {} AND {} → {}", key, c1, c2, nullable_c1 && nullable_c2);
                    }
                } else if key.starts_with("alpha") {
                    // Union, si un hijo es nullable
                    if let Some((c1, c2)) = extract_children(value) {
                        let nullable_c1 = *nullable_map.get(&c1).unwrap_or(&false);
                        let nullable_c2 = *nullable_map.get(&c2).unwrap_or(&false);
                        nullable_map.insert(key.clone(), nullable_c1 || nullable_c2);
                        // println!("{} = {} OR {} → {}", key, c1, c2, nullable_c1 || nullable_c2);
                    }
                }

                // Si hubo un cambio, marcamos que hay cambios
                if nullable_map.get(key) != original_nullable.as_ref() {
                    // println!("Cambio detectado en {}", key);
                    changes = true;
                }
            }

            // Ejemplo para la cadena (a|b)c*(d|e)*f?{ID}[0-5]l+
            // let expected_map = HashMap::from([
            //     ("4".to_string(), false), ("beta2".to_string(), true), ("gama4".to_string(), false), 
            //     ("alpha1".to_string(), false), ("gama7".to_string(), false), ("6".to_string(), false), 
            //     ("alpha3".to_string(), true), ("gama6".to_string(), false), ("7".to_string(), false), 
            //     ("gama1".to_string(), false), ("2".to_string(), false), ("gama5".to_string(), false), 
            //     ("beta3".to_string(), true), ("3".to_string(), false), ("beta1".to_string(), true), 
            //     ("gama2".to_string(), false), ("5".to_string(), false), ("1".to_string(), false), 
            //     ("9".to_string(), false), ("8".to_string(), false), ("alpha2".to_string(), false), 
            //     ("10".to_string(), false), ("gama8".to_string(), false), ("gama3".to_string(), false), 
            //     ("empty".to_string(), true), ("11".to_string(), false)
            // ]);
            
            // println!("Nullable Map Final: {:?}", nullable_map);
            // println!("Coincide con el mapa esperado: {}", nullable_map == expected_map);
        }

        nullable_map
    }

    pub fn find_first_last_pos(&self,) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
        let (tree_map, _key, _token_list) = self.read_tree();
        let mut firstpos_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut lastpos_map: HashMap<String, Vec<String>> = HashMap::new();

        // Primera pasada: Inicializar Literales
        for (key, value) in &tree_map {
            if value.starts_with("Literal") || value.starts_with("Sentinel") || value.starts_with("Range") || value.starts_with("Token"){
                // Para Literals, firstpos y lastpos es solo su propia key
                firstpos_map.insert(key.clone(), vec![key.clone()]);
                lastpos_map.insert(key.clone(), vec![key.clone()]);
                // println!("Inicializando {}: firstpos = {:?}, lastpos = {:?}", key, firstpos_map.get(key), lastpos_map.get(key));
            }
        }

        // Fijación: Realizar múltiples pasadas hasta que no haya cambios
        let mut changes = true;
        while changes {
            changes = false;

            // Segunda pasada: Procesar los nodos no Literales
            for (key, value) in &tree_map {
                let original_firstpos = firstpos_map.get(key).cloned();
                let original_lastpos = lastpos_map.get(key).cloned();

                if key.starts_with("beta") {
                    // Kleene (beta): Igualar firstpos y lastpos al nodo al que está conectado
                    if let Some(c1) = extract_single_child(value) {
                        let firstpos = firstpos_map.get(&c1).cloned().unwrap_or_default();
                        let lastpos = lastpos_map.get(&c1).cloned().unwrap_or_default();
                        firstpos_map.insert(key.clone(), firstpos);
                        lastpos_map.insert(key.clone(), lastpos);

                        // println!("beta -> {} [Left: {}]: ", key, c1);
                        // println!("firstpos = {:?}, lastpos = {:?}", firstpos_map.get(key), lastpos_map.get(key));
                    }
                } else if key.starts_with("alpha") {
                    // Union (alpha): Unir firstpos y lastpos de los nodos izquierdo y derecho
                    if let Some((c1, c2)) = extract_children(value) {
                        let firstpos_c1 = firstpos_map.get(&c1).cloned().unwrap_or_default();
                        let firstpos_c2 = firstpos_map.get(&c2).cloned().unwrap_or_default();
                        let lastpos_c1 = lastpos_map.get(&c1).cloned().unwrap_or_default();
                        let lastpos_c2 = lastpos_map.get(&c2).cloned().unwrap_or_default();

                        // Unión de firstpos y lastpos
                        let firstpos = [&firstpos_c1[..], &firstpos_c2[..]].concat();
                        let lastpos = [&lastpos_c1[..], &lastpos_c2[..]].concat();

                        firstpos_map.insert(key.clone(), firstpos);
                        lastpos_map.insert(key.clone(), lastpos);

                        // println!("alpha -> {} [Left: {}, Right: {}]: ", key, c1, c2);
                        // println!("firstpos = {:?}, lastpos = {:?}", firstpos_map.get(key), lastpos_map.get(key));
                    }
                } else if key.starts_with("gama") {
                    // Concat (gama): Verificar nullabilidad para firstpos y lastpos
                    if let Some((c1, c2)) = extract_children(value) {
                        let firstpos_c1 = firstpos_map.get(&c1).cloned().unwrap_or_default();
                        let firstpos_c2 = firstpos_map.get(&c2).cloned().unwrap_or_default();
                        let lastpos_c1 = lastpos_map.get(&c1).cloned().unwrap_or_default();
                        let lastpos_c2 = lastpos_map.get(&c2).cloned().unwrap_or_default();

                        let nullable_c1 = *self.find_nullable().get(&c1).unwrap_or(&false);
                        let nullable_c2 = *self.find_nullable().get(&c2).unwrap_or(&false);

                        // Firstpos: Si el izquierdo es nullable, hacer unión con el derecho
                        let firstpos = if nullable_c1 {
                            [&firstpos_c1[..], &firstpos_c2[..]].concat()
                        } else {
                            firstpos_c1
                        };

                        // Lastpos: Si el derecho es nullable, hacer unión con el izquierdo
                        let lastpos = if nullable_c2 {
                            [&lastpos_c2[..], &lastpos_c1[..]].concat()
                        } else {
                            lastpos_c2
                        };

                        firstpos_map.insert(key.clone(), firstpos);
                        lastpos_map.insert(key.clone(), lastpos);

                        // println!("gama -> {} [Left: {}, Right: {}]: ", key, c1, c2);
                        // println!("firstpos = {:?}, lastpos = {:?}", firstpos_map.get(key), lastpos_map.get(key));
                    }
                }

                // Si hubo un cambio, marcamos que hay cambios
                if firstpos_map.get(key) != original_firstpos.as_ref()
                    || lastpos_map.get(key) != original_lastpos.as_ref()
                {
                    changes = true;
                }
            }
        }

        // let expected_fp_map: HashMap<String, Vec<String>> = HashMap::from([
        //     ("alpha1".to_string(), vec!["1".to_string(), "2".to_string()]), ("5".to_string(), vec!["5".to_string()]), ("gama7".to_string(), vec!["1".to_string(), "2".to_string()]),
        //     ("1".to_string(), vec!["1".to_string()]), ("beta3".to_string(), vec!["10".to_string()]), ("gama6".to_string(), vec!["3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string()]),
        //     ("8".to_string(), vec!["8".to_string()]), ("gama3".to_string(), vec!["7".to_string()]), ("gama1".to_string(), vec!["9".to_string()]),
        //     ("gama4".to_string(), vec!["6".to_string(), "7".to_string()]), ("10".to_string(), vec!["10".to_string()]), ("beta1".to_string(), vec!["3".to_string()]),
        //     ("7".to_string(), vec!["7".to_string()]), ("gama8".to_string(), vec!["1".to_string(), "2".to_string()]), ("11".to_string(), vec!["11".to_string()]),
        //     ("4".to_string(), vec!["4".to_string()]), ("9".to_string(), vec!["9".to_string()]), ("2".to_string(), vec!["2".to_string()]), ("gama5".to_string(), vec!["4".to_string(), "5".to_string(), "6".to_string(), "7".to_string()]),
        //     ("alpha3".to_string(), vec!["6".to_string()]), ("beta2".to_string(), vec!["4".to_string(), "5".to_string()]), ("3".to_string(), vec!["3".to_string()]),
        //     ("alpha2".to_string(), vec!["4".to_string(), "5".to_string()]), ("6".to_string(), vec!["6".to_string()]), ("gama2".to_string(), vec!["8".to_string()]),
        // ]);
    
        // let expected_lp_map: HashMap<String, Vec<String>> = HashMap::from([
        //     ("2".to_string(), vec!["2".to_string()]), ("10".to_string(), vec!["10".to_string()]), ("1".to_string(), vec!["1".to_string()]),
        //     ("8".to_string(), vec!["8".to_string()]), ("9".to_string(), vec!["9".to_string()]), ("gama3".to_string(), vec!["10".to_string(), "9".to_string()]),
        //     ("gama6".to_string(), vec!["10".to_string(), "9".to_string()]), ("alpha3".to_string(), vec!["6".to_string()]), ("gama1".to_string(), vec!["10".to_string(), "9".to_string()]),
        //     ("gama7".to_string(), vec!["10".to_string(), "9".to_string()]), ("5".to_string(), vec!["5".to_string()]), ("11".to_string(), vec!["11".to_string()]),
        //     ("beta3".to_string(), vec!["10".to_string()]), ("4".to_string(), vec!["4".to_string()]), ("7".to_string(), vec!["7".to_string()]),
        //     ("gama8".to_string(), vec!["11".to_string()]), ("3".to_string(), vec!["3".to_string()]), ("6".to_string(), vec!["6".to_string()]),
        //     ("alpha1".to_string(), vec!["1".to_string(), "2".to_string()]), ("gama4".to_string(), vec!["10".to_string(), "9".to_string()]), ("gama2".to_string(), vec!["10".to_string(), "9".to_string()]),
        //     ("beta2".to_string(), vec!["4".to_string(), "5".to_string()]), ("gama5".to_string(), vec!["10".to_string(), "9".to_string()]), ("alpha2".to_string(), vec!["4".to_string(), "5".to_string()]),
        //     ("beta1".to_string(), vec!["3".to_string()]),
        // ]);
    
        // println!("Firstpos Map Final: {:?}", firstpos_map);
        // println!("Coincide con el mapa esperado: {}", firstpos_map == expected_fp_map);
    
        // println!("Lastpos Map Final: {:?}", lastpos_map);
        // println!("Coincide con el mapa esperado: {}", lastpos_map == expected_lp_map);

        // Retornar los dos diccionarios: firstpos_map y lastpos_map
        (firstpos_map, lastpos_map)
    }

    pub fn find_followpos(&self) -> HashMap<String, Vec<String>> {
        let (tree_map, _key, _token_list) = self.read_tree();
        let mut followpos_map: HashMap<String, Vec<String>> = HashMap::new();

        // Obtener firstpos y lastpos con la función existente
        let (firstpos_map, lastpos_map) = self.find_first_last_pos();

        // Inicializar followpos con listas vacías para todos los nodos
        for key in tree_map.keys() {
            followpos_map.insert(key.clone(), Vec::new());
        }

        // Iterar sobre los nodos para encontrar "gama" (concatenación) y "beta" (Kleene)
        for (key, value) in &tree_map {
            if key.starts_with("gama") {
                // Concat (gama): Followpos de lastpos del hijo izquierdo es firstpos del hijo derecho
                if let Some((c1, c2)) = extract_children(value) {
                    if let Some(lastpos_c1) = lastpos_map.get(&c1) {
                        if let Some(firstpos_c2) = firstpos_map.get(&c2) {
                            for num in lastpos_c1 {
                                followpos_map
                                    .entry(num.clone())
                                    .and_modify(|e| e.extend(firstpos_c2.clone()))
                                    .or_insert(firstpos_c2.clone());
                                // println!("gama -> {}: followpos = {:?}", num, followpos_map.get(num));
                            }
                        }
                    }
                }
            } else if key.starts_with("beta") {
                // Kleene (beta): Followpos de lastpos del nodo es firstpos del mismo nodo
                if let Some(c1) = extract_single_child(value) {
                    if let Some(lastpos_c1) = lastpos_map.get(&c1) {
                        if let Some(firstpos_c1) = firstpos_map.get(&c1) {
                            for num in lastpos_c1 {
                                followpos_map
                                    .entry(num.clone())
                                    .and_modify(|e| e.extend(firstpos_c1.clone()))
                                    .or_insert(firstpos_c1.clone());
                                // println!("beta -> {}: followpos = {:?}", num, followpos_map.get(num));
                            }
                        }
                    }
                }
            }
        }

        // Asegurar que todos los literales y sentinels tengan followpos, aunque sea vacío
        for (key, value) in &tree_map {
            if value.starts_with("Literal") || value.starts_with("Sentinel") || value.starts_with("Range") || value.starts_with("Token") {
                followpos_map.entry(key.clone()).or_insert(Vec::new());
            }
        }

        // Filtrar y eliminar los nodos que no sean Sentinel o Literal
        followpos_map.retain(|key, _| {
            if let Some(value) = tree_map.get(key) {
                value.starts_with("Literal") || value.starts_with("Sentinel") || value.starts_with("Range") || value.starts_with("Token")
            } else {
                false
            }
        });
        let followpos_map = normalize_map(&followpos_map);

        // println!("\n===== FOLLOWPOS FINAL =====");
        // let expected_followpos_map = HashMap::from([
        //     ("3".to_string(), vec!["3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string()]),
        //     ("9".to_string(), vec!["10".to_string(), "11".to_string()]),
        //     ("6".to_string(), vec!["7".to_string()]),
        //     ("11".to_string(), vec![]),
        //     ("10".to_string(), vec!["10".to_string(), "11".to_string()]),
        //     ("1".to_string(), vec!["3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string()]),
        //     ("8".to_string(), vec!["9".to_string()]),
        //     ("7".to_string(), vec!["8".to_string()]),
        //     ("5".to_string(), vec!["4".to_string(), "5".to_string(), "6".to_string(), "7".to_string()]),
        //     ("2".to_string(), vec!["3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string()]),
        //     ("4".to_string(), vec!["4".to_string(), "5".to_string(), "6".to_string(), "7".to_string()]),
        // ]);
    
        // println!("Followpos Map Final: {:?}", followpos_map);
        // println!("¿Coincide con el mapa esperado? {}", followpos_map == expected_followpos_map);

        followpos_map
    }

    pub fn create_states(&mut self) -> (HashMap<char, HashMap<String, char>>, HashSet<char>, Vec<String>) {
        let mut state_map: HashMap<char, HashMap<String, char>> = HashMap::new(); // Mapa de estados y sus transiciones
        let mut acceptance_states: HashSet<char> = HashSet::new(); // Lista de estados de aceptación
        let mut state_queue: HashMap<String, Vec<String>> = HashMap::new(); // Cola de estados por procesar
        let mut fake_state_queue: HashMap<String, Vec<String>> = HashMap::new();
        let mut visited_states: HashMap<String, Vec<String>> = HashMap::new(); // Para evitar procesar estados duplicados
        let mut state_letter = 'A';

        // Obtener el firstpos del nodo raíz
        let (mut labels_map, root_key, token_list) = self.read_tree();
        let root_firstpos = self
            .find_first_last_pos()
            .0
            .get(&root_key)
            .unwrap_or(&Vec::new())
            .clone();

        // println!("Root FirstPos: {:?}", root_firstpos);

        state_queue.insert(state_letter.to_string(), root_firstpos.clone());
        fake_state_queue.insert(state_letter.to_string(), root_firstpos.clone());

        let followpos_map = self.find_followpos();

        labels_map.retain(|key, _| followpos_map.contains_key(key));

        // println!("Labels Map after retaining: {:?}", labels_map);

        let mut columns: HashSet<String> = HashSet::new();
        for (_key, value) in &labels_map {
            if value.starts_with("Literal") || value.starts_with("Range") || value.starts_with("Token") {
                if let Some(start) = value.find('\'') {
                    if let Some(end) = value[start + 1..].find('\'') {
                        let extracted = &value[start + 1..start + 1 + end];
                        columns.insert(extracted.to_string()); // Insertar el valor extraído en el HashSet
                    }
                }
            }
        }

        // println!("Columns: {:?}", columns);

        while !state_queue.is_empty() {
            // Obtener el primer estado y removerlo de state_queue
            let (state_key, state_value) = fake_state_queue.drain().next().unwrap();
            state_queue.remove(&state_key.clone());
            // println!("Processing state: {} -> {:?}", state_key, state_value);

            // Agregar el estado a visited_states
            visited_states.insert(state_key.clone(), state_value.clone());

            // Crea los valores de cada columna
            for column in &columns {
                let mut column_vector: Vec<String> = Vec::new();
                // println!("Processing column: {}", column);

                // Verificar que números en state_value están asociados a la columna
                for number in &state_value {
                    if let Some(symbol) = labels_map.get(number) {
                        if let Some(start) = symbol.find('\'') {
                            if let Some(end) = symbol[start + 1..].find('\'') {
                                let extracted = &symbol[start + 1..start + 1 + end];
                                if extracted == column {
                                    if let Some(followpos_values) = followpos_map.get(number) {
                                        let mut set: HashSet<_> =
                                            column_vector.iter().cloned().collect();
                                        for val in followpos_values {
                                            if set.insert(val.clone()) {
                                                // insert() returns false if the value already exists
                                                column_vector.push(val.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // println!("Column vector: {:?}", column_vector);

                // Si el column_vector tiene elementos, guardamos en state_map
                if !column_vector.is_empty() {
                    // Verificar si el estado creado ya existe
                    let column_set: HashSet<_> = column_vector.iter().cloned().collect();

                    // println!("State Queue before adding new state: {:?}", state_queue);
                    // Verificar si ya existe en visited_states o en state_queue
                    let assigned_letter = visited_states.iter().chain(state_queue.iter()).find_map(|(key, value)| {
                        let v_set: HashSet<_> = value.iter().cloned().collect();
                        if v_set == column_set {
                            Some(key.clone())
                        } else {
                            None
                        }
                    }).or_else(|| {
                        // Si no existe, avanzar la letra y agregarlo a la queue
                        state_letter = (state_letter as u8 + 1) as char;
                        state_queue.insert(state_letter.to_string(), column_vector.clone());
                        // println!("New state added to queue: {}", state_letter);
                        Some(state_letter.to_string())
                    });
                    // println!("State Queue after adding new state: {:?}", state_queue);

                    // Insertar o actualizar el valor en state_map
                    if let Some(assigned_letter) = assigned_letter {
                        // Verificar si el estado es de aceptación
                        if column_vector.iter().any(|num| {
                            if let Some(symbol) = labels_map.get(num) {
                                // println!("Checking if symbol '{}' (from {}) starts with 'Sentinel' to accept {}", symbol, num, assigned_letter);
                                symbol.starts_with("Sentinel")
                            } else {
                                // println!("Number {} not found in labels_map", num);
                                false
                            }
                        }) {
                            acceptance_states.insert(assigned_letter.chars().next().unwrap());
                            // println!("State {} is acceptance state", state_letter);
                        }

                        state_map
                            .entry(state_key.chars().next().unwrap())
                            .or_insert_with(HashMap::new)
                            .insert(
                                column.replace(',' , "-").to_string(),
                                assigned_letter.chars().next().unwrap(),
                            );

                        
                        // println!("Inserted/Updated in state_map: {} -> {} -> {}", state_key, column, assigned_letter);
                    }
                }
            }
            fake_state_queue = state_queue.clone();
            // println!("State Queue after processing: {:?}", state_queue);

            // println!("Visited States after processing {}: {:?}", state_key, visited_states);
            // println!("-------------------------------------------");
        }

        // println!("Mapa de Estados: {:?}", state_map);
        // println!("Estados de aceptación: {:?}", acceptance_states);

        (state_map, acceptance_states, token_list)
    }

}

fn extract_children(value: &str) -> Option<(String, String)> {
    let content = value.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = content.split(", ").collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

fn extract_single_child(value: &str) -> Option<String> {
    let content = value.trim_start_matches('(').trim_end_matches(')');
    Some(content.to_string())
}

fn normalize_map(map: &HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    let mut normalized = HashMap::new();

    for (key, mut value) in map.clone() {
        value.sort(); // Ordena cada vector
        value.dedup(); // Elimina duplicados si existen
        normalized.insert(key, value);
    }

    normalized
}