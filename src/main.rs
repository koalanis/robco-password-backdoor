use std::path::Path;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use rand::seq::SliceRandom;
use std::ops::Range;
use std::fs;
use std::collections::{HashSet, HashMap};
use std::mem;
use std::char;


extern crate pancurses;
use pancurses::{initscr, Window, endwin, Input, noecho};


const NumberWords: u32 = 12;

fn main() {
    println!("Hello, world!");
    // let diff = Difficulty::VeryHard;
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
    
    let words: HashSet<String>  = indices.into_iter().map(|x| bank[x].to_string()).collect();
    

    hacker_ui(&words);
}


struct GameState {
    won: bool,
    tries: u16,
    word_list: HashSet<String>,
    target_word: String,
    used_words: HashSet<String>
}

enum TryResult {
    Correct,
    Invalid,
    Incorrect
}

impl GameState {

    fn do_try(&mut self, attempt: String) -> TryResult {
        let resp = if self.target_word == attempt {
            self.won = true;
            TryResult::Correct
        } else if self.word_list.contains(&attempt) && !self.used_words.contains(&attempt){
            TryResult::Incorrect
        } else {
            self.reset_attempts();
            TryResult::Invalid
        };

        match resp {
            TryResult::Invalid => (),
            _ => self.tries += 1 
        }

        resp
    }

    fn game_on(&self) -> bool {
        return self.tries < TOTAL_TRIES && !self.won;
    }

    fn get_available_choices(&self) -> HashSet<String> {
        let diff: HashSet<String> =  self.word_list.difference(&self.used_words).cloned().collect();
        diff
    }

    fn remove_choice(&mut self) {
        let choices = self.get_available_choices();
        let target = HashSet::from([self.target_word.clone()]);
        let removable: Vec<String> = choices.difference(&target).cloned().collect();

        let size = removable.len();
        let rand_idx = rand::thread_rng().gen_range(0..size);
        let to_remove = removable.get(rand_idx).unwrap();
        self.used_words.insert(to_remove.clone());
    }


    fn reset_attempts(&mut self) {
        self.tries = 0;
    }

    fn get_attempt_score(&self, attempt: String) -> u32 {
        let mut score = 0;
        for (i,c) in attempt.chars().enumerate() { 
            if c == self.target_word.chars().nth(i).unwrap() {
                score += 1;
            }
        }
        score
    }
}

const LEDGER_SIZE: usize  = 17;
const LEDGER_HEIGHT: usize  = LEDGER_SIZE * 2;
const LEDGER_WIDTH: usize  = 12;
const CURSES_HEIGHT: usize  = LEDGER_HEIGHT + 12;
const CURSES_WIDTH: usize  = LEDGER_WIDTH  + 12;
const FILLER_CHARS: &'static str ="(){}[]<>?:;^&$";

struct UiState<'slice> {
    ledger: &'slice mut [char],
    word_placement: HashMap<usize, String>,
}

fn abs_diff(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}

impl UiState<'_> {

    fn init(&mut self, words: Vec<&String>) {
        let mut i = 0;
        
        // create random ledger
        while i < self.ledger.len() {
            let rand_idx = rand::thread_rng().gen_range(0..FILLER_CHARS.len());
            let rand_ch = FILLER_CHARS.chars().nth(rand_idx).unwrap_or('*');
            self.ledger[i] = rand_ch;
            i += 1;
        }

        // add words into ledger
        // let mut set: HashSet<usize> = HashSet::new();
        // for word in words {
        //     let array_len = self.ledger.len();
        //     let mut row = rand::thread_rng().gen_range(0..LEDGER_HEIGHT);
        //     while set.contains(&row) {
        //         row = rand::thread_rng().gen_range(0..LEDGER_HEIGHT);
        //     }
        //     set.insert(row);
        //     let col_max = LEDGER_WIDTH-word.len();
        //     let col = rand::thread_rng().gen_range(0..col_max);
        //     self.word_placement.insert((row, col), word.clone());
        // }

        // for ((row,col), word) in &self.word_placement {
        //     let offset = row*LEDGER_WIDTH + col;
        //     let mut i = 0;
        //     while i <  word.len() {
        //         let ch = word.chars().nth(i).unwrap_or('*');
        //         self.ledger[offset + i] = ch;
        //         i += 1;
        //     }
        // }

        let mut set: HashSet<usize> = HashSet::new();
        for word in words {
            let array_len = self.ledger.len();
            let mut index = rand::thread_rng().gen_range(0..array_len-word.len());
            while set.contains(&index) && set.iter().any(|&x| abs_diff(index, x) < word.len() + 1) {
                index = rand::thread_rng().gen_range(0..array_len-word.len());
            }
            set.insert(index);
            self.word_placement.insert(index, word.clone());
        }

        for (index, word) in &self.word_placement {
            let offset = index;
            let mut i = 0;
            while i <  word.len() {
                let ch = word.chars().nth(i).unwrap_or('*');
                self.ledger[offset + i] = ch;
                i += 1;
            }
        }

    }

    fn get_ledger(&self) -> String {
        let mut row = 0;
        let mut agg = String::new();
        
        while row < LEDGER_SIZE {
            let mut col = 0;
            while col < LEDGER_WIDTH {
                let i = (row*LEDGER_WIDTH + col);
                let ch = self.ledger[i];
                agg.push(ch);
                col += 1;
            }
            // add space
            agg.push(' ');
            col = 0;
            while col < LEDGER_WIDTH {
                let i = ((LEDGER_SIZE+row)*LEDGER_WIDTH + col);
                let ch = self.ledger[i];
                agg.push(ch);
                col += 1;
            }
            agg.push('\n');
            row += 1;
        }

        agg
    }
}

fn hacker_ui(words: &HashSet<String>) {

        
    let word_list:Vec<_> = words.into_iter().collect();
    let word_to_guess = word_list.choose(&mut rand::thread_rng())
                .cloned().map(|x| x.to_string())
                .expect("Should have selected a word from list."); 
    
    if DEBUG {
        println!("word to guess is {:?}", &word_to_guess);
    }
    
    // TODO: add game logic 
    let mut game_state = GameState {
        won: true,
        tries: 0,
        word_list: words.clone(),
        target_word: word_to_guess,
        used_words: HashSet::new()
    };
    let mut ledger =  [0u8 as char; LEDGER_WIDTH*LEDGER_HEIGHT];
    let mut ui_state = UiState {
        ledger: &mut ledger,
        word_placement: HashMap::new()
    };

    ui_state.init(word_list);

    println!();
    println!("{}",ui_state.get_ledger());



    while game_state.game_on() {
        let mut agg = String::new();
        let mut choices: Vec<String> = game_state.get_available_choices()
                .into_iter()
                .collect();

        choices.sort();
        
        for choice in choices {
            agg.push_str(&choice);
            agg.push_str(" ");
        }
        
        println!("Valid choices are :: {agg}");
        println!("You have {0} attempt(s) left", TOTAL_TRIES - game_state.tries);

        println!();
        let mut line = String::new();
        println!("Enter your guess :");
        std::io::stdin()
                .read_line(&mut line)
                .expect("Should get text from stdin");
        println!("Your guess is {}", line);
        println!();
        line = line.trim().to_string();

        if line == "q" {
            game_state.won = true;
        }

        let resp = game_state.do_try(line.clone());
        match resp {
            TryResult::Correct => handle_correct_guess(),
            TryResult::Incorrect => handle_incorrect_guess(game_state.get_attempt_score(line.clone())),
            TryResult::Invalid => handle_invalid_guess()
        }
    }

}

fn curses_fn() {
    let window = initscr();
    window.printw("Type things, press delete to quit\n");
    window.refresh();
    window.keypad(true);
    noecho();

    loop {
        match window.getch() {
            Some(Input::Character(' ')) => {println!("{:?}", window.get_max_yx());},
            Some(Input::KeyUp) => { window.mv(window.get_cur_y()-1, window.get_cur_x());},
            Some(Input::KeyDown) => { window.mv(window.get_cur_y()+1, window.get_cur_x());},
            Some(Input::KeyLeft) => { window.mv(window.get_cur_y(), window.get_cur_x()-1);},
            Some(Input::KeyRight) => { window.mv(window.get_cur_y(), window.get_cur_x()+1);},
            Some(Input::KeyResize) => break,
            Some(Input::Character(c)) => { 

                match c as u8 {
                    27 => break,
                    8 => break,
                    _ => { 
                        println!("{c}");
                        window.addch(c as char);
                     }
                }
            },

            Some(input) => { window.addstr(&format!("something{:?}", input)); },
            None => ()
        }
    }
    endwin();
}

const TOTAL_TRIES: u16 = 4;
const DEBUG: bool = true;
fn run_simple_game(words: &HashSet<String>) {
    
    let word_list:Vec<_> = words.into_iter().collect();
    let word_to_guess = word_list.choose(&mut rand::thread_rng())
                .cloned().map(|x| x.to_string())
                .expect("Should have selected a word from list."); 
    
    if DEBUG {
        println!("word to guess is {:?}", &word_to_guess);
    }
    
    // TODO: add game logic 
    let mut game_state = GameState {
        won: false,
        tries: 0,
        word_list: words.clone(),
        target_word: word_to_guess,
        used_words: HashSet::new()
    };

    while game_state.game_on() {
        let mut agg = String::new();
        let mut choices: Vec<String> = game_state.get_available_choices()
                .into_iter()
                .collect();

        choices.sort();
        
        for choice in choices {
            agg.push_str(&choice);
            agg.push_str(" ");
        }
        
        println!("Valid choices are :: {agg}");
        println!("You have {0} attempt(s) left", TOTAL_TRIES - game_state.tries);

        println!();
        let mut line = String::new();
        println!("Enter your guess :");
        std::io::stdin()
                .read_line(&mut line)
                .expect("Should get text from stdin");
        println!("Your guess is {}", line);
        println!();
        line = line.trim().to_string();

        if line == "q" {
            game_state.won = true;
        }

        let resp = game_state.do_try(line.clone());
        match resp {
            TryResult::Correct => handle_correct_guess(),
            TryResult::Incorrect => handle_incorrect_guess(game_state.get_attempt_score(line.clone())),
            TryResult::Invalid => handle_invalid_guess()
        }
    }

    // curses_fn();
}

fn handle_correct_guess() {
    println!("Hacking successful!");
}

fn handle_incorrect_guess(score: u32) {
    println!("You made an incorrect guess: Score={score}");
}

fn handle_invalid_guess() {
    println!("You made an invalid guess");
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





