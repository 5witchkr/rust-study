use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
    thread::spawn,
};

use rand::seq::SliceRandom;

fn main() {
    let (ps, pr) = mpsc::channel();
    let (ms, mr) = mpsc::channel();
    BaseBallPlayer::new(pr, ms).run();
    BaseBallMachine::new(mr, ps).run();
}

trait Player {
    fn new(pr: Receiver<String>, ms: Sender<String>) -> Self;
    fn run(self);
    fn input(&self);
    fn output(&self, str: &str);
}

trait Machine {
    fn new(mr: Receiver<String>, ps: Sender<String>) -> Self;
    fn run(self);
    fn regist_game(&mut self) -> &mut Self;
}

trait Game {
    fn new() -> Self;
    fn check_answer(&self, input: &str) -> bool;
}

struct BaseBallPlayer {
    receiver: Receiver<String>,
    sender: Sender<String>,
}
impl Player for BaseBallPlayer {
    fn new(pr: Receiver<String>, ms: Sender<String>) -> Self {
        BaseBallPlayer {
            receiver: pr,
            sender: ms,
        }
    }
    fn run(self) {
        spawn(move || {
            while let rev = self.receiver.recv().unwrap().trim().to_uppercase().as_str() {
                match rev {
                    "INPUT" => self.input(),
                    "EXIT" => {
                        break;
                    }
                    _ => self.output(rev),
                }
            }
        });
    }
    fn input(&self) {
        let mut input: String = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let _ = self.sender.send(input.trim().to_owned().to_uppercase());
    }
    fn output(&self, str: &str) {
        println!("Info : {}", str);
    }
}

struct BaseBallMachine {
    receiver: Receiver<String>,
    sender: Sender<String>,
    current_game: Option<BaseBallGame>,
}
impl Machine for BaseBallMachine {
    fn new(mr: Receiver<String>, ps: Sender<String>) -> Self {
        BaseBallMachine {
            receiver: mr,
            sender: ps,
            current_game: None,
        }
    }
    fn run(mut self) {
        let _ = self
            .sender
            .send("게임을 시작 하시겠습니까? (YES / NO)".to_string());
        let _ = self.sender.send("INPUT".to_string());
        while let rev = self.receiver.recv().unwrap().trim().to_uppercase().as_str() {
            match self.current_game {
                None => {
                    match self.regist_game_service(rev) {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    };
                }
                Some(game) => {
                    let input = match game.validate(rev) {
                        Ok(ok) => ok,
                        Err(_) => {
                            let _ = self.sender.send("INPUT".to_string());
                            continue;
                        }
                    };
                    self.play_game_service(input);
                }
            }
        }
    }
    fn regist_game(&mut self) -> &mut Self {
        self.current_game = Some(BaseBallGame::new());
        self
    }
}

impl BaseBallMachine {
    fn regist_game_service(&mut self, rev: &str) -> Result<(), ()> {
        match rev {
            "YES" | "Y" => {
                self.regist_game();
                println!("게임을 시작합니다. 예상 숫자 세자리를 입력해주세요.");
                let _ = self.sender.send("INPUT".to_string());
                Ok(())
            }
            "NO" | "N" => {
                let _ = self.sender.send("EXIT".to_string());
                println!("게임을 종료합니다.");
                Err(())
            }
            _ => {
                println!("알수없는 입력입니다. 다시 입력해주세요. (YES / NO)");
                let _ = self.sender.send("INPUT".to_string());
                Ok(())
            }
        }
    }

    fn play_game_service(&mut self, input: &str) {
        if self.current_game.unwrap().check_answer(input) {
            self.current_game = None;
            println!("정답입니다. 게임을 다시하시겠습니까? (Yes / No)");
            let _ = self.sender.send("INPUT".to_string());
        } else {
            let _ = self.sender.send("INPUT".to_string());
        }
    }
}

#[derive(Clone, Copy)]
struct BaseBallGame {
    answer: [u32; 3],
}
impl Game for BaseBallGame {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let numbers: Vec<u32> = (0..10).collect();
        let random_numbers: Vec<u32> = numbers.choose_multiple(&mut rng, 3).cloned().collect();
        let answer = [random_numbers[0], random_numbers[1], random_numbers[2]];
        BaseBallGame { answer }
    }
    fn check_answer(&self, input: &str) -> bool {
        let mut arr: [u32; 3] = [0; 3];
        for (i, ch) in input.chars().enumerate() {
            arr[i] = ch.to_digit(10).unwrap();
        }
        if arr == self.answer {
            true
        } else {
            BaseBallGame::check_hint(arr, self.answer);
            false
        }
    }
}
impl BaseBallGame {
    fn validate(self, input: &str) -> Result<&str, ()> {
        if input.len() != 3 {
            println!("입력값은 세자리 숫자입니다. 다시 입력해주세요.");
            return Err(());
        }
        match input.parse::<u32>() {
            Ok(ok) => ok,
            Err(_) => {
                println!("입력값은 세자리 숫자입니다. 다시 입력해주세요.");
                return Err(());
            }
        };
        Ok(input)
    }
    fn check_hint(input1: [u32; 3], input2: [u32; 3]) {
        let mut strikes = 0;
        let mut balls = 0;
        for i in 0..3 {
            if input1[i] == input2[i] {
                strikes += 1;
            } else if input1.contains(&input2[i]) {
                balls += 1;
            }
        }
        println!("스트라이크: {}, 볼: {}", strikes, balls);
    }
}
