use std::{collections::{HashMap, HashSet}, option};

#[derive(Debug, PartialEq, Clone)]
pub enum Element{
    Terminal(String),
    NonTerminal(String)
}

pub struct SLR {
    // State id -> State name 
    indexes: HashMap<u8, String>,
    icount: u8,
    // State id + Element -> State id
    edges: HashMap<u8,HashMap<Element,u8>>,

    // State id -> Array of [ Production id + pointer ]
    contents: HashMap<u8,Vec<(u8,u8)>>,

    // Production Id -> Array of elements
    productions: HashMap<u8, Vec<Element>>,
    pcount: u8
}

impl SLR {
    pub fn new(
        productions: HashMap<String, Vec<Vec<String>>>, 
        terminals: HashSet<String>)->Self{

        let heads = productions.keys();
        let elements: HashSet<Element> = HashSet::new();
        
        let mut fprods:  HashMap<u8, Vec<Element>> = HashMap::new();
        let mut counter = 0;
        for h in productions.keys(){
            let h_e = Element::NonTerminal(h.to_string());
            if let Some(opt) = productions.get(h){
                for p in opt{
                    let mut tem_prod: Vec<Element> = Vec::new();
                    tem_prod.push(h_e.clone());
                    for e in p{
                        if terminals.contains(e){
                            tem_prod.push(Element::Terminal(e.to_string()));
                        } else{
                            tem_prod.push(Element::NonTerminal(e.to_string()))
                        }
                    }
                    fprods.insert(counter, tem_prod);
                    counter+=1;
                } 
            }
        }

        SLR { 
            indexes: HashMap::new(), 
            icount: 0,
            edges: HashMap::new(), 
            contents: HashMap::new(), 
            productions: fprods,
            pcount: counter
        }
    }

    pub fn print_state(&self, state_index: u8){
        if let Some(contents) = self.contents.get(&state_index){
            if let Some(name) = self.indexes.get(&state_index){
                println!("{:?}:",name);
            }
            for prod_id in contents{
                // println!("{:?}, {:?}",self.productions.get(&prod.0),prod.1);
                let mut line = String::new();
                if let Some(prod) = self.productions.get(&prod_id.0){
                    for (i,e) in prod.iter().enumerate(){
                        if let Element::Terminal(str) = e{
                            line.push('"');
                            line+=str;
                            line.push('"');
                            line+=" ";
                        } else{
                            if let Element::NonTerminal(str) = e{
                                line+=str;
                                line+=" ";
                            }
                        }
                        if i==0{
                            line+="-> "
                        }
                        if i==prod_id.1 as usize{
                            line+=". "
                        }
                    }
                    println!("~ {}",line);
                }

            }

        } else{
            println!("~ ~ Warning: no productions for '{:?}' state",self.indexes.get(&state_index))
        }
    }
    pub fn generate(&mut self){
        // State 0
        self.indexes.insert(0, "I0".to_string());
        let mut i0_content: Vec<(u8,u8)> = Vec::new();
        for i in 0..self.pcount{
            i0_content.push((i,0));
        }
        self.contents.insert(0, i0_content);
        self.print_state(0);
    }
}

