const START_POS: [[char; 8]; 8] =  [['r', 'p', '.', '.', '.', '.', 'P', 'R'], 
                                    ['n', 'p', '.', '.', '.', '.', 'P', 'N'], 
                                    ['b', 'p', '.', '.', '.', '.', 'P', 'B'], 
                                    ['q', 'p', '.', '.', '.', '.', 'P', 'Q'], 
                                    ['k', 'p', '.', '.', '.', '.', 'P', 'K'], 
                                    ['b', 'p', '.', '.', '.', '.', 'P', 'B'], 
                                    ['n', 'p', '.', '.', '.', '.', 'P', 'N'], 
                                    ['r', 'p', '.', '.', '.', '.', 'P', 'R']];

const KING: [RelMov; 8] = [RelMov{x: -1, y: -1}, RelMov{x: -1, y:   0}, 
                           RelMov{x: -1, y:  1}, RelMov{x:  0, y:  -1}, 
                           RelMov{x: 0,  y:  1}, RelMov{x:  1, y:  -1}, 
                           RelMov{x: 1,  y:  0}, RelMov{x:  1, y:   1}];

const KNIGHT: [RelMov; 8] = [RelMov{x: -1, y:  2}, RelMov{x:  2, y:   1}, 
                             RelMov{x: -1, y: -2}, RelMov{x:  2, y:  -1}, 
                             RelMov{x:  1, y:  2}, RelMov{x: -2, y:   1}, 
                             RelMov{x:  1, y: -2}, RelMov{x: -2, y:  -1}];

const PAWN_B: [RelMov; 2] = [RelMov{x: -1, y:  1},RelMov{x: 1, y:  1}];
const PAWN_W: [RelMov; 2] = [RelMov{x: -1, y: -1},RelMov{x: 1, y: -1}];

#[derive(Debug)]
pub enum Color{
    Black,
    White,
}

//TODO replace with something smarter
#[derive(Debug)]
struct CastlingRights{
    K: bool,
    Q: bool,
    k: bool,
    q: bool,
}

pub enum Castling{
    K,
    Q,
    k,
    q,
}

#[derive(Debug)]
pub struct Square(u8, u8);

pub enum Move{
    Move(Square, Square),
    Castling(Castling),
    Surrender(Color),
}

pub enum MoveResult{
    Valid,
    Invalid,
    WhiteWin,
    BlackWin
}


pub struct BoardState {
    board: [[char; 8]; 8], //where (0, 0) is top left corner or A8
    turn: Color,
    castling_rights: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    fullmoves: u32,
}

pub struct RelMov{
    x: i8,
    y: i8,
}

pub enum ConvertStrToU8Error{
    LetterOutOfRange,
    ArgumentTooShort,
    ArgumentIsNotConvertibleToNumber,
}

//Converts string representation like "e4" to coords; Coords (0, 0) is a8 
pub fn convert_str_to_u8(square: &str) -> Result<Square, ConvertStrToU8Error>{
    let mut arg = square.chars();

    let letter = match arg.next(){
        Some(x) => x,
        None => return Err(ConvertStrToU8Error::ArgumentTooShort)
    };

    let number: u8 = match arg.next(){
        Some(x) => match x.to_digit(10){
            Some(y) => y as u8,
            None => return Err(ConvertStrToU8Error::ArgumentIsNotConvertibleToNumber)
        },
        None => return Err(ConvertStrToU8Error::ArgumentTooShort),
    };

    let vertical = match letter{
        'a' | 'A' => 0,
        'b' | 'B' => 1,
        'c' | 'C' => 2,
        'd' | 'D' => 3,
        'e' | 'E' => 4,
        'f' | 'F' => 5,
        'g' | 'G' => 6,
        'h' | 'H' => 7,
        _ => return Err(ConvertStrToU8Error::LetterOutOfRange)
    };
    Ok(Square(8 - number as u8, vertical))
}

enum ConvertSquareToStrError{
    IncorrectHorizontalInput,
    IncorrectVerticalInput,
}

//Converts coords like (3, 5) (Coords (0, 0) is a8 ) to string like "d3"
fn convert_to_text_notation(square: &Square) -> Result<String, ConvertSquareToStrError>{
    let square_x = square.0;
    let square_y = square.1;
    let letter = match square_x{
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => return Err(ConvertSquareToStrError::IncorrectHorizontalInput)
    };
    if square_y > 7 || square_y < 0 {
        return Err(ConvertSquareToStrError::IncorrectVerticalInput)
    }
    Ok(format!("{}{}", letter, 8 - square_y))
}

//Basically adding coords with range check
fn get_relative_coords(square: Square, offset: RelMov) -> Option<Square>{
    let square_x: i8 = square.0 as i8;
    let square_y: i8 = square.1 as i8;

    if square_x + offset.x < 0 || square_x + offset.x > 7 ||  square_y + offset.y < 0 || square_y + offset.y > 7{
        None
    }else{
        Some(Square(offset.x as u8, offset.y as u8))
    }
}

enum ParseFenError{
    InFenStringTurnInvalid,
    InFenStringBoardInvalid,
    InFenStringEnPasssantInvalid,
    InFenHalfmoveClockInvalid,
    InFenMoveclockInvalid
}

impl BoardState{
    pub fn new() -> BoardState{
        BoardState{
            board : START_POS,
            turn: Color::White,
            castling_rights: CastlingRights{K:true, Q:true, k:true, q:true},
            en_passant: None,
            halfmove_clock: 0,
            fullmoves: 1
        }
    }


    //Same as get_relative_coords but returns figure instead of coords
    fn get_relative(&self, square: &Square, mov: RelMov) -> Option<char>{
        let square_x = square.0 as i8;
        let square_y = square.1 as i8;

        if square_x + mov.x < 0 || square_x + mov.x > 7 ||  square_y + mov.y < 0 || square_y + mov.y > 7{
            None
        }else{
            Some(self.board[(square_x + mov.x) as usize][(square_y + mov.y) as usize])
        }
    }

    //Deserialization: parses Forsyth–Edwards Notation into Boardstate
    fn parse_fen(&mut self, fen_str: &str) -> Result<(), ParseFenError>{
        let fen_parts: Vec<&str> = fen_str.split_ascii_whitespace().collect();
        for (y, row) in fen_parts[0].split("/").enumerate(){
            for (x, chr) in row.chars().enumerate(){
                if chr.is_digit(10){
                    //TODO find the best way to do this (unwrap, ?, match)?
                    let temp = match chr.to_digit(10){
                        Some(x) => x,
                        None => return Err(ParseFenError::InFenStringBoardInvalid)
                    };
                    if temp > 8 || temp < 0 {return Err(ParseFenError::InFenStringBoardInvalid)}
                    for _ in 0..temp{ 
                        self.board[x][y] = '.';
                    }
                }
                else{
                    self.board[x][y] = chr;
                }
            }
        }

        if fen_parts[1].len() > 1 {
            return Err(ParseFenError::InFenStringTurnInvalid);
        }

        self.turn = match fen_parts[1].chars().next(){
            Some('w') | Some('W') => Color::White,
            Some('b') | Some('B') => Color::Black,
            _  => return Err(ParseFenError::InFenStringTurnInvalid)
        };


        self.castling_rights = CastlingRights{
            K: fen_parts[2].contains('K'),
            k: fen_parts[2].contains('k'),
            Q: fen_parts[2].contains('Q'),
            q: fen_parts[2].contains('q'),
        };

        if fen_parts[3] == "-" {
            self.en_passant = None;
        }else{
            let en_passant = convert_str_to_u8(fen_parts[3]);
            if let Ok(x) = en_passant{
                self.en_passant = Some(x);
            }
            else{
                return Err(ParseFenError::InFenStringEnPasssantInvalid);
            }
        }


        if let Ok(x) = fen_parts[4].parse::<u32>(){
            self.halfmove_clock = x;
        }
        else{
            return Err(ParseFenError::InFenHalfmoveClockInvalid);
        }

        if let Ok(x) = fen_parts[5].parse::<u32>(){
            self.fullmoves = x;
        }
        else{
            return Err(ParseFenError::InFenMoveclockInvalid);
        }
        Ok(())
    }

    //TODO find out why std::chunks does not work
    fn chunks(&self) -> Vec<Vec<char>>{
        let mut result: Vec<Vec<char>> = std::vec::Vec::with_capacity(8);
        for i in 0..8{
            result[i] = std::vec::Vec::with_capacity(8);
            for j in 0..8{
                result[i][j] = self.board[i][j];
            }
        }
        result
    }

    pub fn export_to_fen(&self) -> String{
        let mut board_str = std::string::String::new();
        for row in &self.chunks(){
            let mut i = 0;
            while i <= 7 {
                let mut counter = 0;
                while row[i] == '.' && i <= 7 {
                    counter += 1;
                    i += 1
                }
                if counter > 0{
                    board_str.push_str(&*counter.to_string());
                }
                else{
                    board_str.push(row[i] as char);
                }
            }
            board_str.push('/');
        }

        board_str.push(' ');

        let color = match self.turn{
            Color::Black => 'b',
            Color::White => 'w',
        };
        board_str.push(color);

        board_str.push(' ');

        let mut any_castling = true;
        if self.castling_rights.K {board_str.push('K'); any_castling = false;}
        if self.castling_rights.k {board_str.push('k'); any_castling = false;}
        if self.castling_rights.Q {board_str.push('Q'); any_castling = false;}
        if self.castling_rights.q {board_str.push('q'); any_castling = false;}
        if any_castling {board_str.push('-');}

        board_str.push(' ');

        match &self.en_passant{
            Some(x) => board_str.push_str(&*convert_to_text_notation(&x).ok().unwrap()),// due to way it is created inner state can not be invalid, or it would have returned error before
            None => board_str.push('-'),
        }

        board_str.push(' ');

        board_str.push_str(&*self.halfmove_clock.to_string());

        board_str.push(' ');

        board_str.push_str(&*self.fullmoves.to_string());
        return board_str
    } 

    pub fn validate_move(&mut self, player_move: Move, player_color: Color) -> MoveResult{
        match player_move{
            Move::Castling(x) => self.handle_castling(x, player_color),
            Move::Surrender(x) => self.handle_surrender(x),
            Move::Move(sqr1, sqr2) => self.handle_move(sqr1, sqr2, player_color),
        }
    }

    fn handle_castling(&mut self, castling: Castling, player_color: Color) -> MoveResult{
        unimplemented!()
    }

    fn handle_move(&mut self, move_from: Square, move_to: Square, player_color: Color) -> MoveResult{
        let figure_from: char = self.board[move_from.0 as usize][move_from.1 as usize];
        let figure_to: char = self.board[move_from.0 as usize][move_from.1 as usize];

        if figure_from == '.' {return MoveResult::Invalid};
        match player_color{
            Color::White => {
                if figure_from.is_lowercase(){return MoveResult::Invalid};
            },
            Color::Black => {
                if figure_from.is_uppercase(){return MoveResult::Invalid};
            }
        };

        let follows_rule: bool = match figure_from.to_uppercase().next().unwrap(){
            'P' => self.pawn_rule(&move_from, move_to, player_color),
            'R' => self.rook_rule(&move_from, move_to, player_color),
            'N' => self.knight_rule(&move_from, move_to, player_color),
            'B' => self.bishop_rule(&move_from, move_to, player_color),
            'Q' => self.queen_rule(&move_from, move_to, player_color),
            'K' => self.king_rule(&move_from, move_to, player_color),
            _ => unreachable!()
        };
        return MoveResult::Invalid
    }

    fn handle_surrender(&mut self, player_color: Color) -> MoveResult{
        match player_color{
            Color::White => MoveResult::BlackWin,
            Color::Black => MoveResult::WhiteWin,
        }
    }

    fn pawn_rule(&mut self, move_from: &Square, move_to: Square, player_color: Color) -> bool{
        let pawn_dir: i8 = match player_color{Color::White => 1, Color::Black=> -1};
        if (move_from.1 as i8 - move_to.1 as i8).abs() == 1{
            if move_from.0 as i8 - move_to.0 as i8 == 0 && 
                self.get_relative(move_from, RelMov{x: 0,y: -pawn_dir}).unwrap() == '.'{return true}
            if move_from.0 as i8- move_to.0 as i8 == -1 && 
                self.get_relative(move_from, RelMov{x: 1,y: -pawn_dir}).unwrap() != '.' {return true}
            if move_from.0 as i8- move_to.0 as i8 == 1 && 
                self.get_relative(move_from, RelMov{x: -1,y: -pawn_dir}).unwrap() != '.'{return true}
        }
        if (move_from.1 as i8 - move_to.1 as i8).abs() == 2{
            if move_from.0 as i8 - move_to.0 as i8 == 0 && self.get_relative(move_from, RelMov{x: 0,y: -2}).unwrap() == '.' && 
            self.get_relative(move_from, RelMov{x: 0, y: -1}).unwrap() == '.'{return true};
        }
        false
    }

    fn rook_rule(&mut self, move_from: &Square, move_to: Square, player_color: Color) -> bool{
        unimplemented!()
    }

    fn knight_rule(&mut self, move_from: &Square, move_to: Square, player_color: Color) -> bool{
        unimplemented!()
    }

    fn bishop_rule(&mut self, move_from: &Square, move_to: Square, player_color: Color) -> bool{
        unimplemented!()
    }

    fn queen_rule(&mut self, move_from: &Square, move_to: Square, player_color: Color) -> bool{
        unimplemented!()
    }

    fn king_rule(&mut self, move_from: &Square, move_to: Square, player_color: Color) -> bool{
        unimplemented!()
    }
}

fn main(){

}

#[test]
    fn test_convert_str_to_u8() {
        let temp = convert_str_to_u8("a8").ok().unwrap();
        assert_eq!((temp.0, temp.1), (0, 0));
        let temp = convert_str_to_u8("h1").ok().unwrap();
        assert_eq!((temp.0, temp.1), (7, 7));
        let temp = convert_str_to_u8("e4").ok().unwrap();
        assert_eq!((temp.0, temp.1), (4, 4));
    }
#[test]
fn test_convert_to_text_notation(){
    assert_eq!(convert_to_text_notation(&Square(0, 0)).ok().unwrap(), "a8");
    assert_eq!(convert_to_text_notation(&Square(7, 7)).ok().unwrap(), "h1");
    assert_eq!(convert_to_text_notation(&Square(4, 4)).ok().unwrap(), "e4");
}