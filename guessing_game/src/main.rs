use std::io;  // Importing the io module for input/output operations
use rand::Rng; // Importing the Rng trait for generating random numbers
use std::cmp::Ordering; // Importing the Ordering enum for comparing values

fn main() {
    println!("Guess the number!");
    let number = rand::thread_rng().gen_range(1..=100); // Generate a random number between 1 and 100
    println!("The Secret number is: {number}");
    loop{ // Infinite loop to keep the game running
        println!("Please input your guess.");
        let mut guess = String::new(); // Create a mutable String to store the user's guess

        io::stdin() // Read input from the standard input
            .read_line(&mut guess) // Read a line from the standard input and store it in the guess variable
            .expect("Failed to read line"); // Expect the read operation to be successful
        let guess: u32 = guess.trim().parse().expect("Please input valid number"); // Parse the guess as a u32 and expect it to be successful

        println!("You guessed: {guess}");
        match guess.cmp(&number){ // Compare the user's guess with the secret number
            Ordering::Less => println!("Too Small!"), 
            Ordering::Greater => println!("Too Big!"),
            Ordering::Equal => {println!("You win!");
                                break;
            }  
        }
    }
}
