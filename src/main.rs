use std::path::Path;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use rand::seq::SliceRandom;
use std::ops::Range;
use std::fs;
use std::collections::HashSet;

const NumberWords: u32 = 12;

fn main() {
    println!("Hello, world!");
    /// let diff = Difficulty::Easy;
    let diff = get_random_difficulty();

    let val = get_size(diff);
    let path = get_dict_path(val).expect("Should resolve dictionary path for difficulty");
    println!("{:?}",val);
    println!("{:?}",path);
   
    let words = fs::read_to_string(path).expect("Should be able to read file");
    let bank = words.split("\n").collect::<Vec<&str>>();
    let length = bank.len();
    
    let mut count = 0;
    let mut indices = HashSet::new();
    while count < NumberWords {
        let rand_idx = rand::thread_rng().gen_range(0..length);
        if !indices.contains(&rand_idx) {
            indices.insert(rand_idx);
            count += 1;
        }
    }
    
    let words: HashSet<_>  = indices.into_iter().map(|x| bank[x]).collect();
    

    run_simple_game(words);
}


struct GameState {
    tries: u16
}

const TotalTries: u16 = 4;
const Debug: bool = true;
fn run_simple_game(words: HashSet<&str>) {
    
    let word_list:Vec<_> = words.into_iter().collect();
    let word_to_guess = word_list.choose(&mut rand::thread_rng()).cloned(); 
    for word in word_list {
        println!("{word}")
    }

    if Debug {
        println!("word to guess is {:?}", &word_to_guess);
    }
    
   /// TODO: add game logic 


}

#[derive(Debug)]
enum Difficulty {
    VeryEasy,
    Easy,
    Average,
    Hard,
    VeryHard
}

impl Distribution<Difficulty> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Difficulty {
        match rng.gen_range(0..5) {
            1 => Difficulty::Easy,
            2 => Difficulty::Average,
            3 => Difficulty::Hard,
            4 => Difficulty::VeryHard,
            _ => Difficulty::VeryEasy,
        }
    }
}

fn get_size(difficulty: Difficulty) -> u32 {
    let range = match difficulty {
        Difficulty::VeryEasy => 4..5,
        Difficulty::Easy => 6..8,
        Difficulty::Average => 9..10,
        Difficulty::Hard => 11..12,
        Difficulty::VeryHard => 13..15
    };

    let size: u32 = rand::thread_rng().gen_range(range);
    size
}

fn get_random_difficulty() -> Difficulty {
    
    let diff: Difficulty = rand::random();
    println!("{:?}", diff);

    diff
}

fn get_dict_path(word_size: u32) -> Option<String> {
    match word_size {
        4 => Some(String::from("./data/ve-4.txt")),
        5 => Some(String::from("./data/ve-5.txt")),
        6 => Some(String::from("./data/e-6.txt")),
        7 => Some(String::from("./data/e-7.txt")),
        8 => Some(String::from("./data/e-8.txt")),
        9 => Some(String::from("./data/a-9.txt")),
        10 => Some(String::from("./data/a-10.txt")),
        11 => Some(String::from("./data/h-11.txt")),
        12 => Some(String::from("./data/h-12.txt")),
        13 => Some(String::from("./data/vh-13.txt")),
        14 => Some(String::from("./data/vh-14.txt")),
        15 => Some(String::from("./data/vh-15.txt")),
        _ => None
    }
}





