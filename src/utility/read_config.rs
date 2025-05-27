use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

/// Método de parsing posible
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParseMethod {
    SLR,
    LALR,
}

/// Configuración general de la aplicación, mapeada desde `config.json`
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub parse_method: ParseMethod,

    pub debug: DebugConfig,

    pub vis: VisConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugConfig {
    pub generation: bool,
    pub parsing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisConfig {
    pub slr_png: Option<String>,
    pub parse_table: Option<String>,
    pub parse_steps: Option<String>,
    pub symbol_table: Option<String>,
    pub grammar_tree: Option<String>,
    pub dfa: Option<String>,
}

impl Config {
    /// Lee y deserializa `config.json` desde la raíz del proyecto
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let cfg = serde_json::from_str(&contents)?;
        Ok(cfg)
    }
}

/// Conveniencia para el binario: carga “config.json” o panic si falla
pub fn fetch_config() -> Config {
    Config::from_file("config.json").expect("No pude leer `config.json` en la raíz del proyecto")
}
