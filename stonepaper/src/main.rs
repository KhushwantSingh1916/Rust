use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {
    println!("Enter stone=1, paper=2, scissor=3"); 
    let mut player_choice = String::new();
    let choices = rand::thread_rng().gen_range(1..=3);
    io::stdin()
	.read_line(&mut player_choice).expect("cant able to read line");

    let player_choice: u32= player_choice.trim().parse().expect("enter valid number");
    match player_choice.cmp(&choices){
	Ordering::Less => println!("You Lose"),
	Ordering::Greater => println!("you win"),
	Ordering::Equal => println!("Tie"),
   }
}
