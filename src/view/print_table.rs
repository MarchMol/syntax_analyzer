use std::collections::{HashMap, HashSet};
use crate::lex::lex_analyzer::Symbol;
use crate::syn::syn_analyzer::ParsingStep;

use prettytable::{row, Cell, Table};
use std::fs::File;
use std::path::Path;

pub fn print_symbol_table(
    symbols: &Vec<Symbol>,
    filename: &str
)->std::io::Result<()>{
    let mut table = Table::new();
    let header = row!["Lexem Id", "Content", "Token Id", "Token Action", "line","Start", "End"];
    table.add_row(header);
    for s in symbols{
        let row = row![
            s.id,
            s.content,
            s.token,
            s.token_name,
            s.line,
            s.start,
            s.end
        ];
        table.add_row(row);
    }

    let path = Path::new(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    table.print(&mut file)?;
    // println!("Table written to {:?}", path);
    Ok(())
}

pub fn print_parse_table(
    icount: u8, 
    terminals: HashSet<String>,
    non_terminals: HashSet<String>,
    action: &HashMap<(u8, String), String>,
    goto: &HashMap<(u8, String), u8>,
    filename: &str
)->std::io::Result<()>{
    let mut sorted_vec: Vec<String> = terminals.iter().cloned().collect();
    sorted_vec.sort();
    sorted_vec.push("$".to_string());

    let mut non_t_vec: Vec<String> = non_terminals.iter().cloned().collect();
    non_t_vec.sort();
    sorted_vec.extend(non_t_vec);

    let mut table = Table::new();
    let mut header = row![""];
    for tk in sorted_vec.clone(){
        header.add_cell(Cell::new(&tk));
    }
    table.add_row(header);
    for i in 0..icount+1{
        let mut row = row![i.to_string()];
        for _ in sorted_vec.clone().into_iter(){
            row.insert_cell(i as usize +1, Cell::new(""));
        }
        table.add_row(row);
    }

    for key  in action.keys(){
        if let Some(j) = sorted_vec.iter().position(|s| s == &key.1) {
            let i = key.0+1;
            if let Some(ac) = action.get(key){
                let rslt = table.set_element(ac, j+1 as usize, i as usize);
                if !rslt.is_ok(){
                    panic!("Error generating table text")
                }
            }
        }
    }

    for key  in goto.keys(){
        if let Some(j) = sorted_vec.iter().position(|s| s == &key.1) {
            let i = key.0+1;
            if let Some(ac) = goto.get(key){
                let rslt = table.set_element(&ac.to_string(), j+1 as usize, i as usize);
                if !rslt.is_ok(){
                    panic!("Error generating table text")
                }
            }
        }
    }

    let path = Path::new(filename);
    
    // Create the directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 3. Create and open the file
    let mut file = File::create(path)?;

    // 4. Write table output to the file
    table.print(&mut file)?;

    // println!("Table written to {:?}", path);

    Ok(())

}

pub fn print_parse_steps(
    steps: &[ParsingStep],
    filename: &str
) -> std::io::Result<()> {
    let mut table = Table::new();

    // Encabezado
    table.add_row(row!["STACK", "INPUT", "ACTION"]);

    for step in steps {
        table.add_row(row![
            step.stack.clone(),
            step.input.clone(),
            step.action.clone()
        ]);
    }

    let path = Path::new(filename);

    // Crear carpeta si no existe
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Crear archivo
    let mut file = File::create(path)?;

    // Imprimir la tabla en el archivo
    table.print(&mut file)?;

    // println!("Parsing steps written to {:?}", path);
    Ok(())
}

