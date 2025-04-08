use crate::utility::reader::read_lines;

fn process_token(line: String)->(u8, u8, String){
    (0,0,line)
}

fn process_production(line: String)->String{
    let mut prod_iter = line.chars().into_iter();
    let mut head = String::new();
    let mut head_finished = false;
    while let Some(c) = prod_iter.next(){
        if !head_finished{
            print!("{}",c); 
            if c==':'{
                head_finished = true;
            } else{
                head.push(c);
            }
        }
    }
    println!("head: {}",head);
    line
}

// Called 
pub fn read_yalpar(){
    let filename = "./grammar/parser.yalp";
    let mut isProdSection= false;
    let mut tsec: Vec<(u8, u8, String)> = Vec::new();
    let mut psec: Vec<String> = Vec::new();
    if let Ok(lines) = read_lines(filename){
        for line in lines{
            if let Ok(content) = line{
                // No es comentario
                if !content.starts_with("/*"){
                    if content.starts_with("%%"){
                        isProdSection = !isProdSection;
                    } else{
                        if !isProdSection{
                            let tem = process_token(content);
                            tsec.push(tem);
                        } 
                        else{
                            let tem = process_production(content);
                            psec.push(tem);
                        }
                    }
                }
            }
        }
    }
    
    // for p in psec{
    //     println!("{}",p);
    // }
    // for t in tsec{
    //     println!("id: {:?} type: {:?} content: {}",t.0,t.1,t.2);
    // }
}