use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main(){
     println!("Guess the number!");
     let number = rand::thread_rng().gen_range(1..=100);
     println!("The random number is: {number}");
     loop{
        println!("Enter yourr guess");
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("Failed to read line");
        let guess: u32 = guess.trim().parse().expect("Please input number");
        println!("you guessed: {guess}");
        match guess.cmp(&number){
            Ordering::Less => println!("Too Small!"),
            Ordering::Greater => println!("Too Big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}