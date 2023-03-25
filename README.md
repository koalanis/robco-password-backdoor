# robco-password-backdoor

## Background
```
ROBCO INDUSTRIES (TM) TERMLINK PROTOCOL 
!!! WARNING: LOCKOUT IMMINENT !!!
```
Developed by Vault-Tek corporate spies to provide a backdoor access to RobCo terminals.

## Info
![screenshot](https://user-images.githubusercontent.com/5452212/227697672-51b09100-18b1-4bbe-9758-2f14771aba5b.png)

Try with cool-retro-term for maxixmum effect. 

#### Requires
- Rust
- Python3
- sh

Written in rust. 
Uses pancurses for terminal printing.

To run in local, you need to generate the `data/` folder. Running `bash get_data.sh` fetches a dictonrary from the web and parses it into word bank files in the data folder. 

Build using cargo. The program returns appropriate exit codes success/failure. Could be used as a terribly unsafe password lock. 

### TODO
- Add other stylings to hacking ledger
- Add animation on win
- Add animation on loss

