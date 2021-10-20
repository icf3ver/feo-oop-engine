use std::{io::{Stdout, Write}};
use colored::*;

pub async fn progress_bar(
        stdout: &mut Stdout, 
        file_name: &str, 
        current_part: &str,
        n: usize, 
        total: usize, 
        terminal_width: usize){
    let n_digits_total = f32::log10(total as f32) as usize;
    let n_digits_now = f32::log10(n as f32) as usize;
    
    if total != n {
        let remaining_space: i32 = terminal_width as i32 - (8 /* Loading: */ + 1 /* */ + file_name.len() + 1 /* */ + 3 /* here */ + 1 /* */ + current_part.len() + 1 /* */ + n_digits_total + 1 /* / */ + n_digits_total + 3 /* */) as i32;

        let sf = remaining_space as f32/total as f32;

        let done = n as f32 * sf;
        let not_done = (total - n) as f32 * sf;
        print!("\r{} {} [{}>{}] {} {}{}/{}", 
            "Loading:".cyan().bold(),
            file_name, 
            (0..done as usize).map(|_| '=').collect::<String>(), // file read
            (0..not_done as usize).map(|_| ' ').collect::<String>(), // file to read
            current_part,
            (0..(n_digits_total - n_digits_now)).map(|_| ' ').collect::<String>(), 
            n,
            total
        );
        stdout.flush().unwrap();
    } else {
        let remaining_space = terminal_width - (8 /* Finished */ + 1 /* */ + file_name.len() + 1 /* */ + n_digits_total + 1 /* / */ + n_digits_total + 3 /* */);
        println!("\r{} {} {size}/{size} {}", 
            "Finished".green().bold(),
            file_name,
            (0..remaining_space).map(|_| ' ').collect::<String>(), // clear
            size = total,  
        );
    }
}