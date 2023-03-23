use rand::distributions::{Distribution, Standard};
use rand::Rng;
use rand::seq::SliceRandom;
use std::fs;
use std::collections::{HashSet, HashMap};
use std::char;
use std::ops::Range;

extern crate pancurses;
use pancurses::{initscr, has_colors, start_color, set_blink, curs_set, A_COLOR,COLOR_PAIR,COLOR_CYAN,COLOR_BLACK, COLOR_YELLOW, init_pair, use_default_colors, Window, A_DIM,  A_OVERLINE, A_UNDERLINE, A_STANDOUT, A_REVERSE, A_BOLD, endwin, Input, noecho};


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
    exit: bool,
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

    fn remove_choice(&mut self) -> String {
        let choices = self.get_available_choices();
        let target = HashSet::from([self.target_word.clone()]);
        let removable: Vec<String> = choices.difference(&target).cloned().collect();

        let size = removable.len();
        let rand_idx = rand::thread_rng().gen_range(0..size);
        let to_remove = removable.get(rand_idx).unwrap();
        self.used_words.insert(to_remove.clone());
        
        to_remove.to_string()
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
const LEDGER_CAPACITY: usize = LEDGER_WIDTH * LEDGER_HEIGHT;
const CURSES_HEIGHT: usize  = LEDGER_HEIGHT + 12;
const CURSES_WIDTH: usize  = LEDGER_WIDTH  + 12;
const FILLER_CHARS: &'static str ="(){}[]<>?:;^&$";
const CLICKABLE_CHARS: &'static str = "({[<";

struct UiState {
    ledger: String,
    word_placement: HashMap<usize, String>,
    cursor_seek: usize,
    word_size: usize
}

enum CursorScan {
    OnWord,
    OnClickable,
    OnRegular
}


fn abs_diff(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}

impl UiState {
    
    fn new() -> UiState {

        let mut con = String::with_capacity(LEDGER_CAPACITY);
        for _ in 0..con.capacity() {
            con.push('*');
        }

        UiState {
            ledger: con,
            word_placement: HashMap::new(),
            cursor_seek: 0,
            word_size: 0
        }
    }

    fn init(&mut self, words: Vec<&String>) {
        let mut i = 0;
        // create random ledger
        while i < LEDGER_CAPACITY {
            let rand_idx = rand::thread_rng().gen_range(0..FILLER_CHARS.len());
            let rand_ch = FILLER_CHARS.chars().nth(rand_idx).unwrap_or('*');
            self.ledger.replace_range(i..i+1, &rand_ch.to_string());
            i += 1;
        }


        let mut set: HashSet<usize> = HashSet::new();
        for word in words {
            let array_len = self.ledger.len();
            let mut index = rand::thread_rng().gen_range(0..array_len-word.len());
            while set.contains(&index) || 
                    set.iter().any(|&x| abs_diff(index, x) < word.len() + 2) {
                index = rand::thread_rng().gen_range(0..array_len-word.len());
            }
            set.insert(index);
            self.word_placement.insert(index, word.clone());
        }

        for (index, word) in &self.word_placement {
            let offset = index;
            self.ledger.replace_range(offset..&(offset+&word.len()), word.as_str());
            //let mut i = 0;
            //while i <  word.len() {
            //    let ch: u8 = word.chars().nth(i).unwrap_or('*') as u8;
            //    ledger_slice[offset + i] = ch ;
            //    i += 1;
            //}
        }
    }

    fn get_full_ledger(&self) -> String {
        let mut row = 0;
        let mut agg = String::new();
        while row < LEDGER_SIZE {
            let mut col = 0;
            while col < LEDGER_WIDTH {
                let i = (row*LEDGER_WIDTH + col);
                let ch = self.ledger.chars().nth(i).expect("get char from ledger");
                agg.push(ch as char);
                col += 1;
            }
            // add space
            agg.push(' ');
            col = 0;
            while col < LEDGER_WIDTH {
                let i = ((LEDGER_SIZE+row)*LEDGER_WIDTH + col);
                let ch = self.ledger.chars().nth(i).unwrap_or('_');
                agg.push(ch as char);
                col += 1;
            }
            agg.push('\n');
            row += 1;
        }
        
        agg
    }
    
    fn get_left_ledger_frame(&self) -> (usize, usize, usize, usize) {
        (0,0,LEDGER_WIDTH, LEDGER_SIZE)
    }

    fn get_right_ledger_frame(&self) -> (usize, usize, usize, usize) {
        (LEDGER_WIDTH+1, 0, LEDGER_WIDTH+1+LEDGER_WIDTH, LEDGER_SIZE)
    }

    fn get_cursor_ui_pos(&self) -> (i32, i32) {
        
        let mut col = self.cursor_seek % LEDGER_WIDTH;
        let mut row = self.cursor_seek / LEDGER_WIDTH;
        
        let offset  = if row < LEDGER_SIZE {
            self.get_left_ledger_frame()
        } else {
            row = row % LEDGER_SIZE;
            self.get_right_ledger_frame()
        };
        
        col = offset.0 + col;
        row = offset.1 + row;
        
        (i32::try_from(col).unwrap_or(0), i32::try_from(row).unwrap_or(0))
    }

    fn mv_cursor_up(&mut self) {
        let delta = self.cursor_seek.checked_sub(LEDGER_WIDTH);
        match delta {
            Some(val) => {self.cursor_seek = val;},
            None => ()
        }
    }

    fn mv_cursor_down(&mut self) {
        let delta = self.cursor_seek + LEDGER_WIDTH;
        if delta < self.ledger.len() {
            self.cursor_seek = delta;
        }
    }
    
    fn mv_cursor_left(&mut self) {
        let delta = self.cursor_seek.checked_sub(1);
        match delta {
            Some(val) => {self.cursor_seek = val;},
            None => ()
        }
    }
    
    fn mv_cursor_right(&mut self) {
        let delta = self.cursor_seek + 1;
        if delta < self.ledger.len() {
            self.cursor_seek = delta;
        }
    }

    fn get_char_at_cursor(&self) ->  char {
        return self.ledger.chars().nth(self.cursor_seek).expect("should get char from ledger");
    }

    fn get_word_at_cursor(&self) -> Option<usize> {
        if self.get_char_at_cursor().is_alphabetic() {
            let mut i = self.cursor_seek;
            while self.ledger.chars().nth(i).unwrap().is_alphabetic() && i.checked_sub(1).is_some() {
                i = i.checked_sub(1).unwrap();
            }
            let out = if self.ledger.chars().nth(i).unwrap().is_alphabetic() {
                i
            } else {
                i+1
            };
            return Some(out)
        }
        None
    }

    fn get_closing_char_at_cursor(&self) -> Option<usize> {
        let ch = self.get_char_at_cursor();
        if CLICKABLE_CHARS.contains(ch) {
            let looking_for = match ch {
                '<' => '>',
                '[' => ']',
                '(' => ')',
                '{' => '}',
                _ => ' '
            };

            if looking_for == ' ' {
                return None;
            }
            //FIXME: this logic isnt working correctly
            let end = self.cursor_seek + (LEDGER_WIDTH - (self.cursor_seek & LEDGER_WIDTH));
            let mut ith = self.cursor_seek;
            let mut found = false;
            while ith < end  && !found {
                let char_at = self.ledger.chars().nth(ith).unwrap();
                ith += 1;
                if char_at == looking_for {
                    found = true;
                    break;
                }
            }
            if found {
                return Some(ith);
            }
            return None

        }
        None
    }

    fn check_cursor(&self) -> (CursorScan, Option<Range<usize>>) {
        let idx = self.cursor_seek;
        let ch = self.get_char_at_cursor();
        
        let out = if ch.is_alphabetic() {
            let word = self.get_word_at_cursor().expect("Should get index of start of word");
            (CursorScan::OnWord, Some(word..word+self.word_size))
        } else if CLICKABLE_CHARS.contains(ch) {
            let closing = self.get_closing_char_at_cursor();
            let res = match closing {
                Some(end) => (CursorScan::OnClickable, Some(idx..end)),
                None => (CursorScan::OnRegular, None)
            };
            res
        } else {
            (CursorScan::OnRegular, None) 
        };

        out
    }

    fn draw_ledger(&self, window: &Window, highlight: &Option<Range<usize>>) { 
        let mut draw_offset = self.get_left_ledger_frame();
        window.mv(draw_offset.1 as i32, draw_offset.0 as i32);
        let should_highlight = highlight.as_ref().is_some();

        for (ith, ch) in self.ledger.chars().take(LEDGER_SIZE*LEDGER_WIDTH).enumerate() {
            if ith != 0 && ith % LEDGER_WIDTH == 0 {
                window.addch('\n');
            }
            if should_highlight {
                let range = highlight.as_ref().unwrap();
                let left = range.start;
                let right = range.end;
                if left <= ith && ith < right {
                    let mut attrs = A_REVERSE;
                    
                    if ith == self.cursor_seek {
                        attrs = attrs | COLOR_PAIR(1);
                    }

                    window.attron(attrs);
                    window.addch(ch);
                    window.attroff(attrs);
                } else {
                    window.addch(ch);
                }
            } else {
                window.addch(ch);
            }

        }
        
        draw_offset = self.get_right_ledger_frame();
        window.mv(draw_offset.1 as i32, draw_offset.0 as i32);
        for (jth, ch) in self.ledger.chars().skip(LEDGER_SIZE*LEDGER_WIDTH).enumerate() {
          let ith = jth + (LEDGER_SIZE*LEDGER_WIDTH);
          if jth != 0 && jth % LEDGER_WIDTH == 0 {
            window.addch('\n');
            window.mv(window.get_cur_y(), draw_offset.0 as i32);
          }
          if should_highlight {
                let range = highlight.as_ref().unwrap();
                let left = range.start;
                let right = range.end;
                if left <= ith && ith < right {
                    let mut attrs = A_REVERSE;
                    
                    if ith == self.cursor_seek {
                        attrs = attrs | COLOR_PAIR(1);
                    }

                    window.attron(attrs);
                    window.addch(ch);
                    window.attroff(attrs);
                } else {
                    window.addch(ch);
                }
            } else {
                window.addch(ch);
            }
        }


    }
    
    fn draw(&self, window: &Window) {
        let pos = self.get_cursor_ui_pos();
        let scan = self.check_cursor();
        
        self.draw_ledger(window, &scan.1);
        
        window.mv(0,40);
        let pos_str = format!("curses_pos:: ({},{})", pos.0, pos.1);
        window.addstr(pos_str);
        window.mv(10,40);
        let seek_str = format!("seek:: {}", self.cursor_seek);
        window.addstr(seek_str);

        window.mv(pos.0, pos.1);
    }

/*
    fn draw(&self, window: &Window) {
        let pos = self.get_cursor_ui_pos();
        let word_at = self.get_word_at_cursor();
        let at_word = word_at.is_some();
        let mut draw_offset = self.get_left_ledger_frame();
        window.refresh();
        window.mv(draw_offset.1 as i32, draw_offset.0 as i32);
        for (ith, ch) in self.ledger.chars().take(LEDGER_SIZE*LEDGER_WIDTH).enumerate() {
            if ith != 0 && ith % LEDGER_WIDTH == 0 {
                window.addch('\n');
            }
            if at_word {
                let left = word_at.unwrap();
                let word = self.word_placement.get(&left).unwrap();
                let right = left + word.len();
                if left <= ith && ith < right {
                    let mut attrs = A_REVERSE;
                    
                    if ith == self.cursor_seek {
                        attrs = attrs | A_OVERLINE | A_STANDOUT | A_UNDERLINE;
                    }

                    window.attron(attrs);
                    window.addch(ch);
                    window.attroff(attrs);
                } else {
                    window.addch(ch);
                }
            } else {
                window.addch(ch);
            }

        }
        
        draw_offset = self.get_right_ledger_frame();
        window.mv(draw_offset.1 as i32, draw_offset.0 as i32);
        for (jth, ch) in self.ledger.chars().skip(LEDGER_SIZE*LEDGER_WIDTH).enumerate() {
          let ith = jth + (LEDGER_SIZE*LEDGER_WIDTH);
          if jth != 0 && jth % LEDGER_WIDTH == 0 {
            window.addch('\n');
            window.mv(window.get_cur_y(), draw_offset.0 as i32);
          }
          if at_word {
                let left = word_at.unwrap();
                let word = self.word_placement.get(&left).unwrap();
                let right = left + word.len();
                if left <= ith && ith < right {
                    let mut attrs = A_REVERSE;
                    
                    if ith == self.cursor_seek {
                        attrs = A_OVERLINE | A_STANDOUT | A_UNDERLINE;
                    }
                    window.attron(attrs);
                    window.addch(ch);
                    window.attroff(attrs);
                } else {
                    window.addch(ch);
                }
            } else {
                window.addch(ch);
            }
        }

        window.mv(0,40);
        let pos_str = format!("curses_pos:: ({},{})", pos.0, pos.1);
        window.addstr(pos_str);
        window.mv(10,40);
        let seek_str = format!("seek:: {}", self.cursor_seek);
        window.addstr(seek_str);

        window.mv(pos.0, pos.1);
    }
*/
}

fn hacker_ui(words: &HashSet<String>) {

        
    let word_list:Vec<_> = words.into_iter().collect();
    let word_to_guess = word_list.choose(&mut rand::thread_rng())
                .cloned()
                .map(|x| x.to_string())
                .expect("Should have selected a word from list."); 
   let word_size = word_to_guess.len(); 
    if DEBUG {
        println!("word to guess is {:?}", &word_to_guess);
    }
   
    // init game state
    let mut game_state = GameState {
        won: false,
        exit: false,
        tries: 0,
        word_list: words.clone(),
        target_word: word_to_guess,
        used_words: HashSet::new()
    };
    // init ui state
    let mut ui_state = UiState::new();

    ui_state.init(word_list);
    ui_state.word_size = word_size;

    let mut agg = String::new();
    let mut choices: Vec<String> = game_state.get_available_choices()
                .into_iter()
                .collect();

    choices.sort();   
    
    for choice in choices {
        agg.push_str(&choice);
        agg.push_str(" ");
    }
        
    println!("{}", agg);
    println!();
    println!("{}",ui_state.get_full_ledger());


    // init ncurses
    let window = initscr();
    window.refresh();
    window.keypad(true);
    if has_colors() {
        start_color();
    }

    noecho();
    set_blink(true);
    curs_set(1);
    use_default_colors();
    init_pair(1,COLOR_BLACK, COLOR_CYAN);
    
    // window.printw(ui_state.get_full_ledger());
    window.mv(0,0);

    while game_state.game_on() && !game_state.exit {
        ui_state.draw(&window);
        let (col, row) = ui_state.get_cursor_ui_pos();
        window.mv(row ,col);
        handle_input(&window, &mut game_state, &mut ui_state); 
        if game_state.exit {
            break;
        }
    }
    endwin();

}

fn handle_input(window: &Window, game_state: &mut GameState, ui_state: &mut UiState) {
    match window.getch() {
            Some(Input::Character(' ')) => {println!("{:?}", window.get_max_yx());},
            Some(Input::KeyUp) => { ui_state.mv_cursor_up(); },
            Some(Input::KeyDown) => { ui_state.mv_cursor_down(); },
            Some(Input::KeyLeft) => { ui_state.mv_cursor_left(); },
            Some(Input::KeyRight) => { ui_state.mv_cursor_right();  },
            Some(Input::Character(c)) => { 
                // due to some weirdness with lib behavior, i have to catch escape and backspace
                // as u8s instead of using pancurses Input enum
                match c as u8 {
                    27 => {game_state.exit = true;},
                    8 =>  {game_state.exit = true;},
                    _ => () 
                }
            },

            Some(input) => { window.addstr(&format!("something{:?}", input)); },
            None => ()
        }
}

/*
fn curses_fn() {
    let window = initscr();
    window.printw("Type things, press delete to quit\n");
    window.refresh();
    window.keypad(true);
    noecho();
    set_blink(true);

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
*/

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
        exit: false,
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





