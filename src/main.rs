use std::mem;

trait MinimaxPlayer {
    fn into_next_player(self) -> impl MinimaxPlayer;
    fn toggle(&mut self);
}

trait Minimax {
    type Value;
    type Player: MinimaxPlayer;
    type Move;
    fn is_end(&self) -> bool;
    fn eval(&self) -> Self::Value;
    fn get_moves(&self) -> Vec<Self::Move>;
    fn minimax(&mut self, depth: u8) -> (Self::Value, Option<Self::Move>);
}

#[repr(i8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum MicaPlayer {
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

#[derive(Debug)]
enum MicaMove {
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

struct MicaState {
    white_remaining: u8,
    black_remaining: u8,
    white_to_set: u8,
    black_to_set: u8,
    current_player: MicaPlayer,
    stones: Box<[[[MicaPlayer; 3]; 3]]>,
}

impl MicaState {
    fn new() -> Self {
        MicaState {
            white_remaining: 0,
            black_remaining: 0,
            white_to_set: 9,
            black_to_set: 9,
            current_player: MicaPlayer::White,
            stones: Box::new([[[MicaPlayer::None; 3]; 3]; 3]),
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

    fn apply_move(&mut self, mica_move: MicaMove) {
        match mica_move {
            MicaMove::Set { x, y, z } => {
                self.stones[x as usize][y as usize][z as usize] = self.current_player;
                self.increment_player();
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
            },
            MicaMove::MoveRemove { from_x, from_y, from_z, to_x, to_y, to_z, remove_x, remove_y, remove_z } => {
                self.stones[from_x as usize][from_y as usize][from_z as usize] = self.current_player;
                self.stones[to_x as usize][to_y as usize][to_z as usize] = MicaPlayer::None;
                self.stones[remove_x as usize][remove_y as usize][remove_z as usize] = self.current_player.into_next_player();
                self.increment_oponent();
            }
        };
    }

    fn is_in_line(&self, x: u8, y: u8, z: u8) -> bool {
        let x = x as usize;
        let y = y as usize;
        let z = z as usize;

        // check horizontal line
        let mut sum = 0;
        for iz in 0..3 {
            sum += self.stones[x][y][iz] as i8
        }

        if sum == 3 {
            return true;
        }

        // check vertical line
        sum = 0;
        for iy in 0..3 {
            sum += self.stones[x][iy][z] as i8;
        }

        if sum == 3 {
            return true;
        }

        // check cross-square line
        sum = 0;
        for ix in 0..3 {
            sum  += self.stones[ix][y][z] as i8;
        }

        if sum == 3 {
            return true;
        }

        false
    }

    fn is_setting_phase(&self) -> bool {
        self.white_to_set > 0 || self.black_to_set > 0
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
                spots.push((x, y, z + 1));
            }
        }
        
        spots
    }
}

impl Minimax for MicaState {
    type Value = i32;
    type Move = MicaMove;
    type Player = MicaPlayer;

    fn is_end(&self) -> bool {
        self.white_remaining == 2 || self.black_remaining == 2
    }

    fn eval(&self) -> i32 {
        (self.white_remaining - self.black_remaining) as i32
    }

    fn get_moves(&self) -> Vec<Self::Move> {
        let mut moves = Vec::new();
        if self.is_setting_phase() {
            for x in 0..3 {
                for y in 0..3 {
                    for z in 0..3 {
                        if self.stones[x][y][z] == MicaPlayer::None {
                            moves.push(MicaMove::Set { x: x as u8, y: y as u8, z: z as u8 })
                        }
                    }
                }
            }
        } else {
            for from_x in 0u8..3 {
                for from_y in 0u8..3 {
                    for from_z in 0u8..3 {
                        if self.stones[from_x as usize][from_y as usize][from_z as usize] == MicaPlayer::None {
                            let neighboaring_empty_spots = self.get_neighboaring_empty_spots(from_x, from_y, from_z);
                            for (to_x, to_y, to_z) in neighboaring_empty_spots {
                                moves.push(MicaMove::Move { from_x, from_y, from_z, to_x, to_y, to_z });
                            }
                        }
                    }
                }
            }
        }

        moves
    }

    fn minimax(&mut self, depth: u8) -> (Self::Value, Option<Self::Move>) {
        if depth == 0 || self.is_end() {
            return (self.eval(), None);
        }

        match self.current_player {
            MicaPlayer::White => {
                let mut best_value = i32::MIN;
                let mut best_move = None;
                // TODO: zero iterations needs eval
                let moves = self.get_moves();
                self.current_player.toggle();
                for next_move in moves {
                    let new_value = self.minimax(depth - 1).0;
                    if new_value > best_value {
                        best_value = new_value;
                        best_move = Some(next_move);
                    }
                }

                (best_value, best_move)
            },
            MicaPlayer::Black => {
                let mut best_value = i32::MAX;
                let mut best_move = None;
                // TODO: zero iterations needs eval
                let moves = self.get_moves();
                self.current_player.toggle();
                for next_move in moves {
                    let new_value = self.minimax(depth - 1).0;
                    if new_value < best_value {
                        best_value = new_value;
                        best_move = Some(next_move);
                    }
                }

                (best_value, best_move)
            },
            MicaPlayer::None => panic!("Reached invalid state of None player"),
        }
    }
}


fn main() {
    let mut mica_state = MicaState::new();
    let (value, best_move) = mica_state.minimax(4);
    println!("Value: {value}\nBest move: {best_move:?}");
}
