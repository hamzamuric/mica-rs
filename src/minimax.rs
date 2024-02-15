use std::mem;
use serde::Deserialize;

pub trait MinimaxPlayer {
    fn into_next_player(self) -> impl MinimaxPlayer;
    fn toggle(&mut self);
}

pub trait Minimax {
    type Value;
    type Player: MinimaxPlayer;
    type Move;
    fn is_end(&self) -> bool;
    fn eval(&self) -> Self::Value;
    fn get_moves(&self) -> Vec<Self::Move>;
    fn minimax(&mut self, depth: u8, a: i32, b: i32) -> (Self::Value, Option<Self::Move>);
}

#[derive(Deserialize, Debug)]
pub struct MicaRequest {
    difficulty: String,
    pub player: i8,
    white_remaining: u8,
    black_remaining: u8,
    white_count: u8,
    black_count: u8,
    stones: Box<[[[i8; 3]; 3]; 3]>,
}

#[allow(dead_code)]
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicaPlayer {
    None = 0,
    White = 1,
    Black = -1,
}

impl MinimaxPlayer for MicaPlayer {
    fn into_next_player(self) -> MicaPlayer {
        unsafe { mem::transmute(-(self as i8)) }
    }

    fn toggle(&mut self) {
        *self = self.into_next_player();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicaMove {
    Set {
        x: u8,
        y: u8,
        z: u8,
    },
    Move {
        from_x: u8,
        from_y: u8,
        from_z: u8,
        to_x: u8,
        to_y: u8,
        to_z: u8,
    },
    SetRemove {
        x: u8,
        y: u8,
        z: u8,
        remove_x: u8,
        remove_y: u8,
        remove_z: u8,
    },
    MoveRemove {
        from_x: u8,
        from_y: u8,
        from_z: u8,
        to_x: u8,
        to_y: u8,
        to_z: u8,
        remove_x: u8,
        remove_y: u8,
        remove_z: u8,
    }
}

#[derive(Debug, Clone)]
pub struct MicaState {
    pub current_player: MicaPlayer,
    white_remaining: u8,
    black_remaining: u8,
    white_to_set: u8,
    black_to_set: u8,
    stones: Box<[[[MicaPlayer; 3]; 3]; 3]>,
}

impl MicaState {
    pub fn new() -> Self {
        MicaState {
            white_remaining: 0,
            black_remaining: 0,
            white_to_set: 9,
            black_to_set: 9,
            current_player: MicaPlayer::White,
            stones: Box::new([[[MicaPlayer::None; 3]; 3]; 3]),
        }
    }

    pub fn from_request(request: MicaRequest) -> Self {
        MicaState {
            white_remaining: request.white_count,
            black_remaining: request.black_count,
            white_to_set: request.white_remaining,
            black_to_set: request.black_remaining,
            current_player: if request.player == 1 { MicaPlayer::White } else { MicaPlayer::Black },
            stones: unsafe { mem::transmute(request.stones) },
        }
    }

    fn increment_player(&mut self) {
        match self.current_player {
            MicaPlayer::White => {
                self.white_remaining += 1;
            },
            MicaPlayer::Black => {
                self.black_remaining += 1;
            },
            MicaPlayer::None => unreachable!(),
        }
    }

    fn increment_oponent(&mut self) {
        match self.current_player {
            MicaPlayer::White => {
                self.black_remaining += 1;
            },
            MicaPlayer::Black => {
                self.white_remaining += 1;
            },
            MicaPlayer::None => unreachable!(),
        }
    }

    fn decrement_player(&mut self) {
        match self.current_player {
            MicaPlayer::White => {
                self.white_remaining -= 1;
            },
            MicaPlayer::Black => {
                self.black_remaining -= 1;
            },
            MicaPlayer::None => unreachable!(),
        }
    }

    fn decrement_oponent(&mut self) {
        match self.current_player {
            MicaPlayer::White => {
                self.black_remaining -= 1;
            },
            MicaPlayer::Black => {
                self.white_remaining -= 1;
            },
            MicaPlayer::None => unreachable!(),
        }
    }

    fn increment_remaining_to_set(&mut self) {
        match self.current_player {
            MicaPlayer::White => {
                self.white_to_set += 1;
            },
            MicaPlayer::Black => {
                self.black_to_set += 1;
            },
            MicaPlayer::None => unreachable!(),
        }
    }

    fn decrement_remaining_to_set(&mut self) {
        match self.current_player {
            MicaPlayer::White => {
                self.white_to_set -= 1;
            },
            MicaPlayer::Black => {
                self.black_to_set -= 1;
            },
            MicaPlayer::None => unreachable!(),
        }
    }

    pub fn apply_move(&mut self, mica_move: MicaMove) {
        match mica_move {
            MicaMove::Set { x, y, z } => {
                self.stones[x as usize][y as usize][z as usize] = self.current_player;
                self.increment_player();
                self.decrement_remaining_to_set();
            },
            MicaMove::Move { from_x, from_y, from_z, to_x, to_y, to_z } => {
                self.stones[from_x as usize][from_y as usize][from_z as usize] = MicaPlayer::None;
                self.stones[to_x as usize][to_y as usize][to_z as usize] = self.current_player;
            },
            MicaMove::SetRemove { x, y, z, remove_x, remove_y, remove_z } => {
                self.stones[x as usize][y as usize][z as usize] = self.current_player;
                self.stones[remove_x as usize][remove_y as usize][remove_z as usize] = MicaPlayer::None;
                self.increment_player();
                self.decrement_oponent();
                self.decrement_remaining_to_set();
            },
            MicaMove::MoveRemove { from_x, from_y, from_z, to_x, to_y, to_z, remove_x, remove_y, remove_z } => {
                self.stones[from_x as usize][from_y as usize][from_z as usize] = MicaPlayer::None;
                self.stones[to_x as usize][to_y as usize][to_z as usize] = self.current_player;
                self.stones[remove_x as usize][remove_y as usize][remove_z as usize] = MicaPlayer::None;
                self.decrement_oponent();
            }
        };
    }

    fn undo_move(&mut self, mica_move: MicaMove) {
        match mica_move {
            MicaMove::Set { x, y, z } => {
                self.stones[x as usize][y as usize][z as usize] = MicaPlayer::None;
                self.decrement_player();
                self.increment_remaining_to_set();
            },
            MicaMove::Move { from_x, from_y, from_z, to_x, to_y, to_z } => {
                self.stones[from_x as usize][from_y as usize][from_z as usize] = self.current_player;
                self.stones[to_x as usize][to_y as usize][to_z as usize] = MicaPlayer::None;
            },
            MicaMove::SetRemove { x, y, z, remove_x, remove_y, remove_z } => {
                self.stones[x as usize][y as usize][z as usize] = MicaPlayer::None;
                self.stones[remove_x as usize][remove_y as usize][remove_z as usize] = self.current_player.into_next_player();
                self.decrement_player();
                self.increment_oponent();
                self.increment_remaining_to_set();
            },
            MicaMove::MoveRemove { from_x, from_y, from_z, to_x, to_y, to_z, remove_x, remove_y, remove_z } => {
                self.stones[from_x as usize][from_y as usize][from_z as usize] = self.current_player;
                self.stones[to_x as usize][to_y as usize][to_z as usize] = MicaPlayer::None;
                self.stones[remove_x as usize][remove_y as usize][remove_z as usize] = self.current_player.into_next_player();
                self.increment_oponent();
            }
        };
    }

    fn line_check(&self, x: u8, y: u8, z: u8, target_sum: i8) -> bool {
        let x = x as usize;
        let y = y as usize;
        let z = z as usize;

        // check horizontal line
        let mut sum = 0;
        for iz in 0..3 {
            sum += self.stones[x][y][iz] as i8
        }

        if sum.abs() == target_sum {
            return true;
        }

        // check vertical line
        sum = 0;
        for iy in 0..3 {
            sum += self.stones[x][iy][z] as i8;
        }

        if sum.abs() == target_sum {
            return true;
        }

        // check cross-square line
        sum = 0;
        for ix in 0..3 {
            sum  += self.stones[ix][y][z] as i8;
        }

        if sum.abs() == target_sum {
            return true;
        }

        false
    }

    fn is_in_line(&self, x: u8, y: u8, z: u8) -> bool {
        self.line_check(x, y, z, 3)
    }

    fn will_make_line(&self, x: u8, y: u8, z: u8) -> bool {
        self.line_check(x, y, z, 2)
    }

    fn is_setting_phase(&self) -> bool {
        self.white_to_set > 0 && self.black_to_set > 0
    }

    fn get_neighboaring_empty_spots(&self, x: u8, y: u8, z: u8) -> Vec<(u8, u8, u8)> {
        let mut spots = Vec::new();

        // check left spot
        if z > 0 && self.stones[x as usize][y as usize][z as usize - 1] == MicaPlayer::None {
            spots.push((x, y, z - 1));
        }

        // check right spot
        if z < 2 && self.stones[x as usize][y as usize][z as usize + 1] == MicaPlayer::None {
            spots.push((x, y, z + 1));
        }

        // check spot above
        if y > 0 && self.stones[x as usize][y as usize - 1][z as usize] == MicaPlayer::None {
            spots.push((x, y - 1, z));
        }

        // check spot below
        if y < 2 && self.stones[x as usize][y as usize + 1][z as usize] == MicaPlayer::None {
            spots.push((x, y + 1, z));
        }

        // check cross-square neighboaring spots
        if (y == 1 && (z == 0 || z == 2)) || (z == 1 && (y == 0 || y == 2)) {
            if x > 0 && self.stones[x as usize - 1][y as usize][z as usize] == MicaPlayer::None {
                spots.push((x - 1, y, z));
            }

            if x < 2 && self.stones[x as usize + 1][y as usize][z as usize] == MicaPlayer::None {
                spots.push((x + 1, y, z));
            }
        }
        
        spots.into_iter().filter(|(_, y, z)| !(*y == 1 && *z == 1)).collect()
    }

    fn get_oponent_stones(&self) -> Vec<(u8, u8, u8)> {
        let mut opponent_stones = Vec::new();
        let opponent = self.current_player.into_next_player() as MicaPlayer;
        for x in 0u8..3 {
            for y in 0u8..3 {
                for z in 0u8..3 {
                    if self.stones[x as usize][y as usize][z as usize] == opponent && !self.is_in_line(x, y, z) {
                        opponent_stones.push((x, y, z));
                    }
                }
            }
        }

        opponent_stones
    }
}

impl Minimax for MicaState {
    type Value = i32;
    type Move = MicaMove;
    type Player = MicaPlayer;

    fn is_end(&self) -> bool {
        (self.white_to_set == 0 && self.black_to_set == 0) &&
        (self.white_remaining == 2 || self.black_remaining == 2)
    }

    fn eval(&self) -> i32 {
        self.white_remaining as i32 - self.black_remaining as i32
    }

    fn get_moves(&self) -> Vec<Self::Move> {
        let mut moves = Vec::new();
        if self.is_setting_phase() {
            for x in 0u8..3 {
                for y in 0u8..3 {
                    for z in 0u8..3 {
                        if y == 1 && z == 1 {
                            continue;
                        }
                        if self.stones[x as usize][y as usize][z as usize] == MicaPlayer::None {
                            let next_move = MicaMove::Set { x, y, z};
                            if self.will_make_line(x, y, z) {
                                let empty_spots = self.get_oponent_stones();
                                for (remove_x, remove_y, remove_z) in empty_spots {
                                    moves.push(MicaMove::SetRemove { x, y, z, remove_x, remove_y, remove_z })
                                }
                            } else {
                                moves.push(next_move);
                            }
                        }
                    }
                }
            }
        } else {
            for from_x in 0u8..3 {
                for from_y in 0u8..3 {
                    for from_z in 0u8..3 {
                        if from_y == 1 && from_z == 1 {
                            continue;
                        }
                        if self.stones[from_x as usize][from_y as usize][from_z as usize] == MicaPlayer::None {
                            let neighboaring_empty_spots = self.get_neighboaring_empty_spots(from_x, from_y, from_z);
                            for (to_x, to_y, to_z) in neighboaring_empty_spots {
                                let next_move = MicaMove::Move { from_x, from_y, from_z, to_x, to_y, to_z };
                                if self.will_make_line(to_x, to_y, to_z) {
                                    let empty_spots = self.get_oponent_stones();
                                    for (remove_x, remove_y, remove_z) in empty_spots {
                                        moves.push(MicaMove::MoveRemove { from_x, from_y, from_z, to_x, to_y, to_z, remove_x, remove_y, remove_z })
                                    }
                                } else {
                                    moves.push(next_move);
                                }
                            }
                        }
                    }
                }
            }
        }

        moves
    }

    fn minimax(&mut self, depth: u8, mut a: i32, mut b: i32) -> (Self::Value, Option<Self::Move>) {
        if depth == 0 {
            return (self.eval(), None);
        }
        if self.is_end() {
            return (self.eval(), None);
        }

        match self.current_player {
            MicaPlayer::White => {
                let mut best_value = i32::MIN;
                let mut best_move = None;
                // TODO: zero iterations needs eval
                let moves = self.get_moves();
                for next_move in moves {
                    self.apply_move(next_move);
                    self.current_player.toggle();
                    let new_value = self.minimax(depth - 1, a, b).0;
                    self.current_player.toggle();
                    if best_move == None || new_value > best_value {
                        best_value = new_value;
                        best_move = Some(next_move);
                    }
                    self.undo_move(next_move);
                    if new_value > b {
                        break;
                    }
                    a = a.max(new_value);
                }

                (best_value, best_move)
            },
            MicaPlayer::Black => {
                let mut best_value = i32::MAX;
                let mut best_move = None;
                // TODO: zero iterations needs eval
                let moves = self.get_moves();
                for next_move in moves {
                    self.apply_move(next_move);
                    self.current_player.toggle();
                    let new_value = self.minimax(depth - 1, a, b).0;
                    self.current_player.toggle();
                    if best_move == None || new_value < best_value {
                        best_value = new_value;
                        best_move = Some(next_move);
                    }
                    self.undo_move(next_move);
                    if new_value < a {
                        break;
                    }
                    b = b.min(new_value);
                }

                (best_value, best_move)
            },
            MicaPlayer::None => panic!("Reached invalid state of None player"),
        }
    }
}