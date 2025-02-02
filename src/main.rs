/*

MIT License

Copyright (c) 2024 Kaleb Alanis

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


                 _
               _(_)_                          wWWWw   _
   @@@@       (_)@(_)   vVVVv     _     @@@@  (___) _(_)_
  @@()@@ wWWWw  (_)\    (___)   _(_)_  @@()@@   Y  (_)@(_)
   @@@@  (___)     `|/    Y    (_)@(_)  @@@@   \|/   (_)\
    /      Y       \|    \|/    /(_)    \|      |/      |
 \ |     \ |/       | / \ | /  \|/       |/    \|      \|/
jgs|//   \\|///  \\\|//\\\|/// \|///  \\\|//  \\|//  \\\|//
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
robco-password-backdoor - a Fallout inspired terminal minigame written in Rust

*/

use rand::distributions::{Distribution, Standard};
use rand::Rng;
use rand::seq::SliceRandom;
use std::fs;
use std::collections::{HashSet, HashMap};
use std::char;
use std::ops::Range;
use std::process::ExitCode;


extern crate pancurses;
use pancurses::{initscr, has_colors, start_color, set_blink, curs_set, A_COLOR,COLOR_PAIR,COLOR_CYAN,COLOR_BLACK, COLOR_YELLOW, init_pair, use_default_colors, Window, A_DIM,  A_OVERLINE, A_UNDERLINE, A_STANDOUT, A_REVERSE, A_BOLD, endwin, Input, noecho};

// The number of words in the hacking minigame
const NUMBER_WORDS: u32 = 12;
// The number of ncurse cells that the hacking game is offset by from the top of the terminal
const LEDGER_OFFSET_Y:usize = 5;
/*
The size of a single ledger in the hacking minigame. A ledger is one side of the gaming minigame.
The game has two ledgers but it is represented in an single array of characters

:>?&pepsinog nterceptable
enous<^:&$^: }?>(&;perime
nondefensibl dullary{&${;
y<}:?]>[>>[] {??(];&;{]ma
{>]<?}^(^;>> gnetisation}
}$[$^$)(()${ ^protectioni
^^(>[??{<<:: sm)$&$dichot
)((:[):)&}}) omizing&]{&;
)(noncommitm {&<semipecti
ent?&<]&>^$( nate})$<{&<^
}]^][]]<:^]] ))?:)]{]$()^
&::[&}&[rhad ?(][?$:?${;)
amanthine?(: nonmysticall
})(}<{$&]>); y[];(^>;}{?<
}}]$])[&(}:^ ^<>&&(>^<}[p
}>;:]:{<[<&) igmentophage
:{}(:??>([;i (&((<$)[[<:(

 */
const LEDGER_SIZE: usize  = 17;
// The total ledger height, in terms of the array representation in memory
const LEDGER_HEIGHT: usize  = LEDGER_SIZE * 2;
// The ledger's width
const LEDGER_WIDTH: usize  = 12;
// total number of characters in the ledger board
const LEDGER_CAPACITY: usize = LEDGER_WIDTH * LEDGER_HEIGHT;
// the sample of chars used for the "filler" chars in the ledger
const FILLER_CHARS: &'static str ="(){}[]<>?:;^&$";
// the sample of chars that are the opening char of a clickable token
const CLICKABLE_CHARS: &'static str = "({[<";
// max number of attempts in a game session
const TOTAL_TRIES: u16 = 4;

// the program entry point
fn main() -> ExitCode {
    let diff = get_random_difficulty();

    let val = get_size(diff);
    let path = get_dict_path(val).expect("Should resolve dictionary path for difficulty");
   
    let words = fs::read_to_string(path).expect("Should be able to read file");
    let bank = words.split("\n").collect::<Vec<&str>>();
    let length = bank.len();
    
    let mut count = 0;
    let mut indices = HashSet::new();
    while count < NUMBER_WORDS {
        let rand_idx = rand::thread_rng().gen_range(0..length);
        if !indices.contains(&rand_idx) {
            indices.insert(rand_idx);
            count += 1;
        }
    }
    
    let words: HashSet<String>  = indices.into_iter().map(|x| bank[x].to_string()).collect();
    

    if(hacker_ui(&words)) {
        ExitCode::SUCCESS        
    } else {
        ExitCode::FAILURE
    }
}

/*
    Struct used to represent the guessing game aspect of the hacking minigame
*/
struct GameState {
    won: bool,
    exit: bool,
    tries: u16,
    word_list: HashSet<String>,
    target_word: String,
    used_words: HashSet<String>
}

// Enum representing the outcome of a guessing / hacking attempt
enum AttemptResult {
    Correct,
    Invalid,
    Incorrect
}

// Enum representing the possible rewards/outcomes from pressing a clickable element in the game
enum ClickableReward {
    ResetAttempts,
    RemoveWord
}

impl GameState {

    fn do_try(&mut self, attempt: String) -> AttemptResult {
        let resp = if self.target_word == attempt {
            self.won = true;
            AttemptResult::Correct
        } else if self.word_list.contains(&attempt) && !self.used_words.contains(&attempt){
            AttemptResult::Incorrect
        } else {
            self.reset_attempts();
            AttemptResult::Invalid
        };

        match resp {
            AttemptResult::Invalid => (),
            _ => if self.tries > 0 {
                self.tries -= 1;
            }
        }

        resp
    }

    fn game_on(&self) -> bool {
        return self.tries > 0 && !self.won;
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
        self.tries = TOTAL_TRIES;
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

    fn random_clickable_reward() -> ClickableReward {
        let b = rand::random::<bool>();
        if b {
            ClickableReward::ResetAttempts
        } else {
            ClickableReward::RemoveWord
        }
    }
}
struct UiState {
    ledger: String,
    word_placement: HashMap<usize, String>,
    cursor_seek: usize,
    word_size: usize,
    side_log: Vec<String>
}

enum CursorScan {
    OnWord,
    OnClickable,
    OnRegular
}

// Returns the absolute value of the difference between a and b
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
            word_size: 0,
            side_log: Vec::new()
        }
    }

    fn add_log(&mut self, log_statement: String) {
        self.side_log.push(log_statement);
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
         (0,0+LEDGER_OFFSET_Y,LEDGER_WIDTH, LEDGER_SIZE+LEDGER_OFFSET_Y)
    }

    fn get_right_ledger_frame(&self) -> (usize, usize, usize, usize) {
        (LEDGER_WIDTH+1, 0+LEDGER_OFFSET_Y, LEDGER_WIDTH+1+LEDGER_WIDTH, LEDGER_SIZE+LEDGER_OFFSET_Y)
    }
    
    fn handle_enter(&mut self, game_state: &mut GameState) {
        let scan = self.check_cursor();
        match scan.0 {
            CursorScan::OnWord => {
                let word = self.get_word_at_cursor();
                if word.is_some() {
                    let beg = word.unwrap();
                    let at_word = self.word_placement.get(&beg).unwrap();
                    self.handle_enter_on_word(game_state, beg.clone(), at_word.to_string());
                }
            },
            CursorScan::OnClickable => {
                let closing = self.get_closing_char_at_cursor();
                if closing.is_some() {
                    let range = (self.cursor_seek, closing.unwrap());
                    self.handle_enter_on_clickable(game_state, range);
                } else {
                    self.add_log(format!("Invalid Token: {:?}", self.get_char_at_cursor()));
                }
            },
            _ => {
                self.add_log(format!("Invalid Token: {:?}", self.get_char_at_cursor()));
            }
        }
    }

    fn handle_enter_on_word(&mut self, game_state: &mut GameState, idx: usize, word: String) {
        let try_attempt = game_state.do_try(word.to_string());
        match try_attempt {
            AttemptResult::Incorrect => {
                self.add_log(format!("Password Incorrect: {:?}", word));
                let score = game_state.get_attempt_score(word.clone());
                self.add_log(format!("{}/{} correct", score, self.word_size));
            },
            _ => ()
        }
    }

    fn handle_enter_on_clickable(&mut self, game_state: &mut GameState, range: (usize, usize)) {
        let ch = self.get_char_at_cursor();
        let reward = GameState::random_clickable_reward();
        match reward {
            ClickableReward::RemoveWord => {
                let to_remove = game_state.remove_choice();
                self.add_log(String::from("Remove dud"));
                let mut to_remove_key = None;
                for (key, val) in &self.word_placement {
                    if val.clone() == to_remove {
                        to_remove_key = Some(key);
                    }
                }

                if to_remove_key.is_some() {
                    let key = to_remove_key.unwrap();
                    let range = key..&(key+self.word_size);
                    self.ledger.replace_range(range, &".".repeat(self.word_size));
                    
                    self.word_placement.remove(&key.clone());

                }
            },
            ClickableReward::ResetAttempts => {
                game_state.reset_attempts();
                self.add_log(String::from("Reset attempts"));
            }
        }

        self.ledger.replace_range(self.cursor_seek..self.cursor_seek+1, ".");
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

            let end = self.cursor_seek + (LEDGER_WIDTH - (self.cursor_seek %  LEDGER_WIDTH));
            let mut ith = self.cursor_seek;
            let mut found = false;
            while ith < end && ith < self.ledger.len() && !found {
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

    fn check_cursor(& self) -> (CursorScan, Option<Range<usize>>) {
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
    
    fn draw_heading(&self, window:&Window) {
        window.mv(0,0);
        window.addstr(String::from("ROBCO INDUSTRIES (TM) TERMLINK PROTOCOL\n"));
        window.addstr(String::from("!!! WARNING: LOCKOUT IMMINENT !!! (Press q to quit)\n"));
    }

    fn draw_attempts(&self, window:&Window, game_state: &GameState) {
        window.mv(3, 0);
        window.addstr(format!("{} ATTEMPT(S) LEFT: {}", game_state.tries, "# ".repeat(game_state.tries as usize)));
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

    fn get_side_log_frame(&self) -> (usize, usize, usize, usize) {
        let rf = self.get_right_ledger_frame();

        (rf.2+5, rf.1, rf.2 + 20, rf.3)
    }

    fn draw_side_log(&self, window: &Window) {
        let frame = self.get_side_log_frame();
        window.mv(30, 30);
        let take_n = frame.3;
        let mut pos = (frame.3 as i32, frame.0 as i32);
        pos = (pos.0-1, pos.1);
        for line in self.side_log.iter().rev().take(take_n) {
            window.mv(pos.0 , pos.1);
            window.addstr(line);
            pos = (pos.0-1,frame.0 as i32);
        }
    }
    
    fn draw(&self, window: &Window, game_state: &GameState) {
        let pos = self.get_cursor_ui_pos();
        let scan = self.check_cursor();
        window.clear();
        self.draw_heading(window);
        self.draw_attempts(window, game_state);
        self.draw_ledger(window, &scan.1);
        self.draw_side_log(window); 
        window.mv(pos.0, pos.1);
        window.refresh();
    }
}

// Procedure that starts the hacking mingame, given a set of words
fn hacker_ui(words: &HashSet<String>) -> bool {
    let word_list:Vec<_> = words.into_iter().collect();

    // select random word to be target word
    let word_to_guess = word_list.choose(&mut rand::thread_rng())
                .cloned()
                .map(|x| x.to_string())
                .expect("Should have selected a word from list."); 
  
    let word_size = word_to_guess.len(); 

    // init game state
    let mut game_state = GameState {
        won: false,
        exit: false,
        tries: TOTAL_TRIES,
        word_list: words.clone(),
        target_word: word_to_guess,
        used_words: HashSet::new()
    };
    // init ui state
    let mut ui_state = UiState::new();
    ui_state.init(word_list);
    ui_state.word_size = word_size;

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
        ui_state.draw(&window, &game_state);
        let (col, row) = ui_state.get_cursor_ui_pos();
        window.mv(row ,col);
        handle_input(&window, &mut game_state, &mut ui_state); 
        if game_state.exit {
            break;
        }
    }
    endwin();
    
    if game_state.won {
        //TODO: Could add animation on win here
        println!("Hacking complete...!\nAccess granted.");
        return true;
    } else {
        println!("Access denied");
        return false;
    }
}

fn handle_input(window: &Window, game_state: &mut GameState, ui_state: &mut UiState) {
    match window.getch() {
            Some(Input::KeyUp) => { ui_state.mv_cursor_up(); },
            Some(Input::KeyDown) => { ui_state.mv_cursor_down(); },
            Some(Input::KeyLeft) => { ui_state.mv_cursor_left(); },
            Some(Input::KeyRight) => { ui_state.mv_cursor_right();  },
            Some(Input::Character('\n')) => { ui_state.handle_enter(game_state);  },
            Some(Input::Character('q')) => {game_state.exit = true; },
            Some(Input::Character(c)) => { 
                // due to some weirdness with lib behavior, i have to catch escape and backspace
                // as u8s instead of using pancurses Input enum
                match c as u8 {
                    27 => {game_state.exit = true;},
                    8 =>  {game_state.exit = true;},
                    _ => () 
                }
            },
            _ | None => ()
        }
}

// Enum which represents the difficulty setting of the hacking minigame
#[derive(Debug)]
enum Difficulty {
    VeryEasy,
    Easy,
    Average,
    Hard,
    VeryHard
}

// Support for randomness by implementing distribution for Difficulty enum
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

// Returns a random word size given a difficulty enum
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

// Returns a random Difficulty enum
fn get_random_difficulty() -> Difficulty {
    let diff: Difficulty = rand::random();
    diff
}

// Returns the path of a dictionary file with a bunch of words with target word_size
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
