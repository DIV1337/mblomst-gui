use std::sync::mpsc::{self, Sender, Receiver};
use std::net::{TcpStream, TcpListener};
use std::thread;
use std::io::{Read, Write};

use ggez::ContextBuilder;
use ggez::event;

use crate::MblomstGui;
use crate::ChessColor;
use crate::piece::Piece;
use crate::piece::Color;
use crate::helper as chess_helper;
use crate::ggezGameResult;

pub fn piece_to_code(piece: Piece) -> String { // translates pieces into their "code". 
    match piece {
        Piece::Pawn(Color::White)   => "wP".to_string(),
        Piece::Knight(Color::White) => "wN".to_string(),
        Piece::Bishop(Color::White) => "wB".to_string(),
        Piece::Rook(Color::White)   => "wR".to_string(),
        Piece::Queen(Color::White)  => "wQ".to_string(),
        Piece::King(Color::White)   => "wK".to_string(),

        Piece::Pawn(Color::Black)   => "bP".to_string(),
        Piece::Knight(Color::Black) => "bN".to_string(),
        Piece::Bishop(Color::Black) => "bB".to_string(),
        Piece::Rook(Color::Black)   => "bR".to_string(),
        Piece::Queen(Color::Black)  => "bQ".to_string(),
        Piece::King(Color::Black)   => "bK".to_string(),
    }
}
