// move_piece.rs is taken souly from https://github.com/INDA25PlusPlus/nhg-chess/blob/main/src/main.rs

use chess::*;

/// Find index of the move that goes to `to_square`.
pub fn find_move_to(moves: &Vec<Move>, to_square: u8) -> Option<usize> {
    moves.iter().position(|m| m.to == to_square)
}
/// Execute the move from `from_square` to `to_square` (searches the valid_moves and uses make_move).
pub fn execute_move(game: &mut Game, from_square: u8, to_square: u8) -> bool {
    print!("{:?}", game.player_tracker());
    print!("'s turn.");
    match game.select_piece(from_square) {
        Ok(piece) => {
            println!("You selected: {:?} on square {}", piece, index_to_square(from_square));
            let moves = valid_moves(from_square, piece, &game.position);
            if moves.is_empty() {
                println!("No valid moves for this piece!");
                return false;
            }
            println!("Valid moves:");
            for (i, m) in moves.iter().enumerate() {
                println!("{}. {:?}", i, m);
            }
            println!("");

            // NOTE, being able to pick move id is important for special moves like promotion so this step should NOT always be left to computer.
            if let Some(idx) = find_move_to(&moves, to_square) {
                let chosen_move = moves[idx];
                // just hnd over game and then let make_mvoe call position?
                match make_move(chosen_move, game) {
                    Ok(()) => {
                        if game.is_over() {
                            println!("Game has ended: {:?}", game.result);
                            return true;
                        }
                    }
                    Err(e) => println!("Move failed: {}", e),
                }
            } else {
                println!("No valid move from {:?} to {:?} found.", index_to_square(from_square), index_to_square(to_square));
            }
        }
        Err(msg) => println!("Selection failed: {}", msg),
    }
    false
}