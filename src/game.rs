use std::{fs, fmt};
use crate::game::Player::{PlayerX, PlayerO};
use open_ttt_lib::board::Position;
use serde::{Deserialize, Serialize};
use serde::export::Formatter;
use open_ttt_lib::game::{Game, State};
use serde::export::fmt::Debug;
use open_ttt_lib::ai::{Opponent, Difficulty};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    PlayerX,
    PlayerO,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            PlayerX => { write!(f, "PlayerX") }
            PlayerO => { write!(f, "PlayerO") }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MatchState {
    start_player: Player,
    current_player: Player,
    player_x_score: u64,
    player_o_score: u64,
    tie_score: u64,
    board: [u32; 9],
}

impl MatchState {
    fn create_ttt_lib_game(&self) -> Game {
        let mut open_ttt_game = Game::new();
        // Depending on the state of this game, we might start a new one to change the current player
        // (state machine ensuring the player who went second last game goes first next game)
        let (mut x_index, mut o_index) = match self.start_player {
            Player::PlayerX => {
                if open_ttt_game.state() != State::PlayerXMove {
                    open_ttt_game.start_next_game();
                }
                (0, 1)
            }
            Player::PlayerO => {
                if open_ttt_game.state() != State::PlayerOMove {
                    open_ttt_game.start_next_game();
                }
                (1, 0)
            }
        };

        // Replay the game to come back to the state
        let mut positions = [Option::None; 9];
        for (index, u) in self.board.iter().enumerate() {
            let position = Position { row: (index / 3) as i32, column: (index % 3) as i32 };
            if *u == 2 {
                // X player
                positions[x_index] = Some(position);
                x_index = x_index + 2;
            } else if *u == 1 {
                // O player
                positions[o_index] = Some(position);
                o_index = o_index + 2;
            }
        }

        for pos in positions.iter() {
            match pos {
                None => {}
                Some(p) => { open_ttt_game.do_move(*p).unwrap(); }
            }
        }
        open_ttt_game
    }
}

pub struct Match {
    state: MatchState,
    ttt_lib_game: Option<Game>,
}

impl Match {
    fn inc_score(&mut self, player: Option<Player>) {
        match player {
            None => { self.state.tie_score += 1 }
            Some(p) => {
                match p {
                    PlayerX => { self.state.player_x_score += 1 }
                    PlayerO => { self.state.player_o_score += 1 }
                }
            }
        }
    }

    pub fn prepare_next_game(&mut self) {
        match self.state.start_player {
            PlayerX => {
                self.state.start_player = PlayerO;
                self.state.current_player = PlayerO;
            }
            PlayerO => {
                self.state.start_player = PlayerX;
                self.state.current_player = PlayerX;
            }
        };
        self.state.board = [0; 9];
    }

    pub fn do_move(&mut self, position: Position) {
        let index = position.row * 3 + position.column;
        let player = &self.state.current_player;
        match player {
            PlayerX => { self.state.board[index as usize] = 2 }
            PlayerO => { self.state.board[index as usize] = 1 }
        }
        self.ttt_lib_game.as_mut().unwrap().do_move(position).unwrap();
        println!("### {} takes row {} column {}", player, position.row, position.column);
    }

    pub fn ai_move(&mut self) {
        let ai = Opponent::new(Difficulty::Medium);
        if let Some(ai_position) = ai.get_move(self.ttt_lib_game.as_ref().unwrap()) {
            self.do_move(ai_position);
        };
    }

    pub fn evaluate_state(&mut self) {
        match self.ttt_lib_game.as_mut().unwrap().state() {
            State::PlayerXMove => self.state.current_player = PlayerX,
            State::PlayerOMove => self.state.current_player = PlayerO,
            State::PlayerXWin(_) => {
                println!("### Game Over: X wins!");
                self.inc_score(Some(PlayerX));
                self.prepare_next_game();
            }
            State::PlayerOWin(_) => {
                println!("### Game Over: O wins!");
                self.inc_score(Some(PlayerO));
                self.prepare_next_game();
            }
            State::CatsGame => {
                println!("### Game Over: cat's game.");
                self.inc_score(None);
                self.prepare_next_game();
            }
        };
    }

    pub fn is_new_game(&self) -> bool {
        let x: u32 = self.state.board.iter().sum();
        x == 0
    }

    pub fn save_to_file(&self, path: &str) {
        let result = serde_json::to_string(&self.state).unwrap();
        fs::write(path, result).expect("Something went wrong writing the file");
    }

    pub fn load_from_file(path: &str) -> Match {
        let content = fs::read_to_string(path).expect("Something went wrong reading the file");
        let state: MatchState = serde_json::from_str(content.as_str()).unwrap();
        let game = state.create_ttt_lib_game();
        return Match {
            state,
            ttt_lib_game: Some(game),
        };
    }

    pub fn write_to_markdown(&self, path: &str) {
        let intro = ":x::o: Tic-Tac-Toe played indefinitely by github action runners! \
         See [my workflow](.github/workflows/play.yaml).\n\n";
        let score_table = format!("|PlayerX wins|PlayerO wins|Ties|\n\
                                 |-|-|-|\n\
                                 |{}|{}|{}|\n\n\
                                 ", self.state.player_x_score, self.state.player_o_score, self.state.tie_score);

        let game_info = format!("{}'s turn.\n\n", self.state.current_player);
        let board = format!("<pre>\n{}</pre>", self.ttt_lib_game.as_ref().unwrap().board());

        let content = format!("{}{}{}{}", intro, score_table, game_info, board);
        fs::write(path, content).expect("Something went wrong writing the file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Result;

    fn get_game() -> MatchState {
        MatchState {
            start_player: PlayerX,
            current_player: PlayerX,
            player_x_score: 123,
            player_o_score: 321,
            tie_score: 42,
            board: [0, 1, 0, 2, 1, 0, 0, 1, 0],
        }
    }

    #[test]
    fn write_game_state_from_json_file() -> Result<()> {
        let path = "resources/test/write_game_state_from_json_file.json";
        let m = Match { state: get_game(), ttt_lib_game: None };
        m.save_to_file(path);
        let read_game = Match::load_from_file(path);
        assert_eq!(read_game.state, m.state);
        Ok(())
    }
}
