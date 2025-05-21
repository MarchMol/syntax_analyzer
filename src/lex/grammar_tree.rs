use std::{cell::RefCell, rc::{Rc, Weak}, sync::mpsc::TryRecvError};
use super::tokenizer::Token;


#[derive(Debug, PartialEq, Clone)]
pub struct Tree{
    nodes: Vec<TreeNode>,
    root: Option<Rc<TreeNode>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct TreeNode{
    value: Token,
    left: Option<Rc<TreeNode>>,
    right: Option<Rc<TreeNode>>
}
impl TreeNode{
    pub fn print_tree(self, level: usize, prefix: &str)->String{
        let space = " ".repeat(level*4);
        let mut ret = format!("{}{}{:?}\n",space,prefix,self.value);
        match self.left{
            Some(left)=>{
                let lret = (*left).clone().print_tree(level+1, "L----");
                ret+=&lret;
            }
            _=>{
            }
        }
        match self.right {
            Some(right)=>{
                let rret = (*right).clone().print_tree(level+1,"R----");
                ret+=&rret;
            }
            _=>{
            }
        }
        ret
    }

    pub fn get_left(&self) -> Option<Rc<TreeNode>> {
        self.left.clone()
    }

    pub fn get_right(&self) -> Option<Rc<TreeNode>> {
        self.right.clone()
    }

    pub fn get_value(&self) -> &Token {
        &self.value
    }
}

impl Tree{
    pub fn new()->Self{
        Self { nodes: Vec::new(), root: None }
    }

    pub fn generate(&mut self, tokens: Vec<Token>)->Rc<TreeNode>{
        let mut stack : Vec<TreeNode> = Vec::new();

        for tk in tokens{
            match tk{
                Token::Literal(c) | Token::Range(c,_)=>{
                    let newnode = TreeNode{
                        value: tk, 
                        left: None,
                        right: None
                    };
                    stack.push(newnode);
                },
                Token::Concat | Token::Union=>{
                    match (stack.pop(), stack.pop()){
                        (Some(second), Some(first))=>{
                            let operator = TreeNode{
                                value: tk,
                                left: Some(Rc::new(first)),
                                right: Some(Rc::new(second))
                            };
                            stack.push(operator);
                        }
                        _=>{}
                    }

                },
                Token::Kleene=>{
                    match stack.pop(){
                        Some(first)=>{
                            let operator = TreeNode{
                                value: tk,
                                left: Some(Rc::new(first)),
                                right:None
                            };
                            stack.push(operator);
                        }
                        _=>{}
                    }
                },
                Token::Sentinel | Token::Empty=>{
                    let newnode = TreeNode{
                        value: tk, 
                        left: None,
                        right: None
                    };
                    stack.push(newnode);
                },
                Token::Tokener(_) => {
                    let newnode = TreeNode {
                        value: tk,
                        left: None,
                        right: None
                    };
                    stack.push(newnode);
                },
                _=>{}
            }
            
        }
        let root_node = Some(Rc::new(stack[0].clone()));
        self.root = Some(Rc::new(stack[0].clone()));
        root_node.unwrap()
    }

    pub fn get_root(&self) -> Option<Rc<TreeNode>> {
        self.root.clone()
    }
}