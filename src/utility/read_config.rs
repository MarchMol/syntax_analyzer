use std::{fs::File, io::BufReader};

use serde::Deserialize;



#[derive(Deserialize)]
pub struct Config{
    pub parse_method: String,
    pub debug: Debug,
    pub vis: Vis
}
#[derive(Deserialize)]
pub struct Debug{
    pub generation: bool,
    pub parsing: bool
}
#[derive(Deserialize)]
pub struct Vis{
    pub slr_png: Option<String>,
    pub parse_table:Option<String>,
    pub parse_steps: Option<String>,
    pub symbol_table: Option<String>,
    pub grammar_tree: Option<String>,
    pub dfa: Option<String>
}

pub fn fetch_config()->Config{
    let file = File::open("config.json").unwrap();
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader).unwrap();
    config
}