use console::Style;

pub fn print_log(log: &str, done: usize, total: usize, color: &Style){
    let n_log = &format!("{}", color.apply_to(log));
    let pbar = draw_progress(done, total);
    print!("\r{}\n",pbar);
    print!("\x1B[2A"); // Move cursor up 1 line
    print!("\r\x1B[2K{}\n",n_log);
}

fn draw_progress(done: usize, total: usize)->String{
    let percent = (done as f32 / total as f32 * 100.0).round() as u32;
    let bar_len = 20;
    let filled = (bar_len * percent / 100) as usize;
    let empty = bar_len as usize - filled ;

    let bar = format!(
        "{}{}",
        "▰".repeat(filled),
        "▱".repeat(empty)
    );
    format!(
        "Progress: [{}] {}%",
        bar,
        percent
    )
}