use std::collections::{HashMap, HashSet};

use prettytable::{row, Cell, Table};
use std::fs::File;
use std::path::Path;

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

    println!("Table written to {:?}", path);

    Ok(())

}