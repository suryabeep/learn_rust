use rand::seq::SliceRandom;
use std::io::{self, BufRead};

fn main() -> Result<(), String> { 

    println!("Welcome to hangman!\nA word is chosen at random and the user must guess the word letter by letter before running out of attempts\n Enter 'quit' to quit.");
    
    let words = vec!["hangman", "chairs", "backpack", "bodywash", "clothing",
                 "computer", "python", "program", "glasses", "sweatshirt",
                 "sweatpants", "mattress", "friends", "clocks", "biology",
                 "algebra", "suitcase", "knives", "ninjas", "shampoo"];
    
    let word = words.choose(&mut rand::thread_rng()).ok_or("Failed to select a word to start the game!")?;

    let stdin = io::stdin();
    
    let mut user_input;
    let mut should_quit = false;
    let mut guessed_so_far = "_".repeat(word.chars().count());
    let mut attempts_remaining = word.chars().count();
    
    while !should_quit {
        println!("The word so far: {}", &guessed_so_far);
        println!("Guess a single character, or type 'quit' to quit the game.");

        user_input = stdin.lock()
            .lines()
            .next()
            .expect("There was no next line!")
            .expect("The line could not be read!");

        println!("You entered: {}.", user_input);
        if user_input == "quit" {
            should_quit = true;
        }
        else if user_input.chars().count() > 1 {
            println!("Except the quit command, you should only enter one character at a time!");
            continue;
        }
        else {
            let indices: Vec<_> = word.match_indices(&user_input).map(|(i, _)| i).collect();
            if indices.len() > 0 {
                for index in indices {
                    guessed_so_far.replace_range(index..(index+1), &word.chars().nth(index).unwrap().to_string());
                    println!("You guessed correctly!");
                }
            }
            else {
                attempts_remaining -= 1;
                println!("That was not in the word! You have {} attempts remaining.", attempts_remaining);
            }
        }

        if attempts_remaining == 0 {
            should_quit = true;
            println!("You died! The word was {}",&word);
        }
        if guessed_so_far.matches("_").count() == 0 {
            println!("You won! The word was {}", &word);
            should_quit = true;
        }
    }

    return Ok(());
}
