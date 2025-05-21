use std::{collections::HashMap, hash::Hash};

use crate::{utility::reader::read_lines};


pub struct LexerData{
    pub merged: String,
    pub actions: HashMap<usize, String>,
}

#[derive(Eq, Hash, Debug, PartialEq, Clone)]
pub struct LexemVar{
    pub id: u8,
    pub regex: String,
}

pub struct Lexem{
    pub id: usize,
    pub regex: String,
    pub action: String,
}

fn trim_ws(content: String)->String{
    let mut ret = String::new();
    let mut flag = true;
    for ch in content.chars(){
        if flag{
            if !ch.is_whitespace(){
                flag=false;
                ret.push(ch);
            }
        } else{
            ret.push(ch);
        }
    }
    ret.trim_end().to_string()
}

fn encode_vars(vars: Vec<String>)->HashMap<String, LexemVar>{
    let mut map: HashMap<String,LexemVar> = HashMap::new();
    for (i,mut v) in vars.into_iter().enumerate(){
        let mut name = String::new();
        let mut reg = String::new();
        let mut section: usize = 0;
        v.drain(..4);
        for ch in v.chars(){
            if section==0{
                if ch.is_whitespace(){
                    section = 1;
                } else{
                    name.push(ch);
                }
            }
            if section == 1{
                if ch=='='{
                    section=2;
                }
            }
            if section ==2 {
                if ch.is_whitespace(){
                    section = 3;
                    continue;
                }
            }
            if section == 3{
                reg.push(ch);
            }
        }  

        map.insert(
            name.to_string(), 
            LexemVar { id: i as u8, regex: reg.to_string()}
        );
    }
    map
}

fn contains_var(reg_og: String)->(bool,usize, usize){
    let mut reg = reg_og.clone();
    let operations: Vec<char> = Vec::from(['|','*','+','?','(',')']);
    let mut is_range = false;
    let mut is_literal = false;
    let mut has_var = false;
    let mut start: usize = 0;
    let mut end: usize = 0;
    for (i,ch) in reg.chars().enumerate(){
        if !operations.contains(&ch){
            if ch=='['{
                is_range = true;
            }
            else if ch==']'{
                is_range = false;
            }
            else if ch == '\"'{
                is_literal = !is_literal;
            }
            else{
                if !is_range && !is_literal{
                    has_var = true;
                    start = i;
                    break;
                }
            }
        }
    }
    reg.drain(..start);
    if !has_var{
        return (false, 0,0)
    }
    for (i,ch) in reg.chars().enumerate(){
        if operations.contains(&ch){
            end=i;
            break;
        }
        if Vec::from(['[',']','\"']).contains(&ch){
            end = i;
            break;
        }
    }

    (has_var, start, end+start)
}

fn replace_vars(og_reg: String, vars: &HashMap<String, LexemVar>)->String{
    println!("Starting: {}",og_reg);
    let mut new_reg = og_reg.clone();
    loop{
        let contains = contains_var(new_reg.clone());
        if contains.0{
            if contains.1!= contains.2{
                let sus = &new_reg[contains.1..contains.2];
                if let Some(sus_r) = vars.get(sus){
                    let mut tem= String::new();
                    tem+=  &new_reg[0..contains.1];
                    tem+= &sus_r.regex;
                    tem+= &new_reg[contains.2..];
                    new_reg = tem.clone();
                } else {
                    panic!("~ Error Lex: var not found {}",sus);
                }
            } else {
                let sus = &new_reg[contains.1..];
                if let Some(sus_r) = vars.get(sus){
                    let mut tem= String::new();
                    tem+=&new_reg[..contains.1];
                    tem+= &sus_r.regex;
                    new_reg = tem.clone();
                } else {
                    panic!("~ Error Lex: var not found {}",sus);
                }
            }
        } else {
            break
        }
    }
    new_reg
}

fn encode_rule(rule: Vec<String>, vars: &HashMap<String, LexemVar>)->Vec<Lexem>{
    let mut rule_vec: Vec<Lexem> = Vec::new();
    for (i, r_) in rule.iter().enumerate(){
        let mut r = r_.clone();
        if r.starts_with("|"){
            r.drain(..2);
        }
        let mut reg = String::new();
        let mut action = String::new();
        let mut section: usize = 0;
        let mut is_lit = false;
        let mut is_range = false;
        for ch in r.chars(){
            if section==0{
                if ch == '\"'{
                    is_lit = !is_lit;
                }
                if ch=='['{
                    is_range = true;
                }
                if ch==']'{
                    is_range = false;
                }
                if !is_range && !is_lit{
                    if !ch.is_whitespace(){
                        reg.push(ch);
                        continue;
                    } else {
                        section = 1;
                    }
                } else {
                    reg.push(ch);
                }
            }
            if section==1{
                if ch=='{'{
                    section = 2;
                    continue;
                }
            }
            if section == 2{
                action.push(ch);
            }
        }
        let mut cl_action = trim_ws(action);
        cl_action.pop();
        cl_action = trim_ws(cl_action);
        let cl_reg = replace_vars(reg, &vars);

        rule_vec.push(
            Lexem { id: i, regex: cl_reg, action: cl_action }
        )
    }
    rule_vec
}

fn genereate_action_table(rules: &Vec<Lexem>)->LexerData{
    let mut merged = String::new();
    let mut actions: HashMap<usize, String> = HashMap::new();
    
    for r in rules{
        merged+=&format!("(({}){{{}}})|",r.regex, r.id);
        actions.insert(r.id, r.action.clone());
    }
    merged.pop();
    println!("{}",merged);

    LexerData{
        merged,
        actions
    }
}

pub fn read_yalex(filename:&str)->LexerData{
    let mut section:usize= 0;
    // 1. Section yal content
    let mut header: Vec<String> = Vec::new();
    let mut vars: Vec<String> = Vec::new();
    let mut rule: Vec<String> = Vec::new();
    if let Ok(lines) = read_lines(filename){
        for line in lines{
            if let Ok(raw_content) = line{
                // Remove inital whitespace of line
                let content = trim_ws(raw_content);
                if !content.clone().starts_with("(*"){ // if its not a comment
                    if section == 0 { // Header section
                        if !content.starts_with("{"){
                            if content.starts_with("}"){
                                section = 1;
                            } else{
                                header.push(content.clone());
                            }
                        }
                    }
                    if section == 1{ // Var section
                        if content.starts_with("let"){
                            vars.push(content.clone());
                        } else {
                            if content.starts_with("rule"){
                                section = 2;
                                continue;
                            }
                        }
                    }
                    if section == 2{ // Rule section
                        rule.push(content.clone());
                    }
                }
            }
        }
    }
    // 2. Encode variable
    let var_vec = encode_vars(vars);
    // for (k,v) in &var_vec{
    //     println!("Name: {}, Reg: {}",k,v.regex);
    // }
    // 3. Encode Rules
    let rule_vec = encode_rule(rule, &var_vec);
    // for v in rule_vec{
    //     println!("ID: {}, Reg: {}, action: {},",v.id, v.regex, v.action);
    // }
    let lexdata = genereate_action_table(&rule_vec);
    lexdata

}