use chess::position::get_piece_at;
use chess::*;
use chess::game::GameResult;
use chess::piece::Color as ChessColor;

use ggez::{Context, ContextBuilder};
use ggez::GameResult as ggezGameResult;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, Canvas};
use ggez::input::mouse::MouseButton;
use ggez::graphics::Image;
use ggez::input::keyboard::KeyCode;

use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::env;

use connection_state::ConnectionState;

mod connection_state;
mod connection;
mod move_piece;
mod helper;

struct MblomstGui {
    game: Game,
    board_size: (f32, f32),
    square_x: f32,
    square_y: f32,
    selected_square: Option<String>,
    piece_images: HashMap<String, Image>,
    checkmate: bool,
    color_won: Option<String>,
    win_messages: HashMap<String, Image>,
    stalemate: bool,
    connection_state: Arc<Mutex<ConnectionState>>,
}

impl MblomstGui {
    pub fn new(ctx: &mut Context, connection_state: Arc<Mutex<ConnectionState>>) -> ggezGameResult<MblomstGui> {
        println!("Initializing GUI...");
        let position = initialize_board();
        let game = Game::new(position);

        let mut piece_images = HashMap::new();
        let piece_codes = ["wP", "wN", "wB", "wR", "wQ", "wK", "bP", "bN", "bB", "bR", "bQ", "bK"];
        for code in piece_codes {
            let path = format!("/pieces/{}.png", code);
            match Image::from_path(ctx, path) {
                Ok(image) => {
                    piece_images.insert(code.to_string(), image);
                }
                Err(e) => {
                    println!("Failed to load piece image '{}': {}", code, e);
                }
            }
        }

        let mut win_messages = HashMap::new();
        let messages_name = ["White_won", "Black_won", "Stalemate"];
        for message in messages_name {
            let path = format!("/messages/{}.png", message);
            match Image::from_path(ctx, path) {
                Ok(image) => {
                    win_messages.insert(message.to_string(), image);
                }
                Err(e) => {
                    println!("Failed to load win message '{}': {}", message, e);
                }
            }
        }

        Ok(MblomstGui {
            game,
            board_size: (0.0, 0.0),
            square_x: 0.0,
            square_y: 0.0,
            selected_square: None,
            piece_images,
            checkmate: false,
            color_won: None,
            win_messages,
            stalemate: false,
            connection_state,
        })
    }

    fn screen_to_square(&self, x: f32, y: f32) -> Option<String> {
        let col = (x / self.square_x) as usize;
        let row = (y / self.square_y) as usize;
        let file = (b'a' + col as u8) as char;
        let rank = 8 - row;
        Some(format!("{}{}", file, rank))
    }
}

impl EventHandler for MblomstGui {
    fn update(&mut self, ctx: &mut Context) -> ggezGameResult {
        let rx = {
            let state = self.connection_state.lock().unwrap();
            state.incoming_rx.clone()
        };

        self.connection_state.lock().unwrap().turn = self.game.turn as usize;



        while let Ok(package) = rx.try_recv() {
            println!("Received package: {}", package);
            let parts: Vec<&str> = package.trim().split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(from), Ok(to)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                    move_piece::execute_move(&mut self.game, from, to);
                }
            }
        }

        let (width, height) = ctx.gfx.drawable_size();
        self.board_size = (width, height);
        self.square_x = width / 8.0;
        self.square_y = height / 8.0;

        if ctx.keyboard.is_key_pressed(KeyCode::R) {
            self.color_won = None;
            self.checkmate = false;
            self.stalemate = false;
            let position = initialize_board();
            self.game = Game::new(position);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> ggezGameResult { 
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb(255,192,203)); // makes background pink ;) 
        let (board_size_x, board_size_y) = self.board_size; 
        for row in 0..8 { 
            for col in 0..8 { // cycles thrue all rows and columns on the board and sets the apropriate color for each square 
                let x = col as f32 * self.square_x; // takes every column and multiples it by the square width, i.e taking the coordinate for the square 
                let y = row as f32 * self.square_y; 
                let color = if (row + col) % 2 == 0 { // every other "square" 
                Color::from_rgb(255,228,196) // "white" "square" 
                } 
                else { 
                    Color::from_rgb(105,47,15) // "black" "square" (had to make brown so black pieces are visable) 
                }; 
                let rectangle = graphics::Mesh::new_rectangle ( // the settings for the "square" 
                    ctx, 
                    graphics::DrawMode::fill(), 
                    graphics::Rect::new(x, y, self.square_x, self.square_y), 
                    color, 
                )?; 
                canvas.draw(&rectangle, graphics::DrawParam::default()); // draws it 
            } 
        } 

        let position = &self.game.position; // gets the state for the board 
        for row in 0..8 { 
            for col in 0..8 { // goes thrue every "square" on the board 
                let square_index = row * 8 + col; 
                if let Some(piece) = position::get_piece_at(&position, square_index as u8) { // to determine what piece should be drawn 
                    let code = helper::piece_to_code(piece); // converts the peice to its "name" 
                    if let Some(image) = self.piece_images.get(&code) { // selects the image with the same "name" 
                        let dest_x = col as f32 * self.square_x; 
                        let dest_y = (7 - row) as f32 * self.square_y; 
                        let image_scale = if self.selected_square.is_some()
                        && chess::square_to_index(&self.selected_square.clone().unwrap()).unwrap() == square_index
                            {
                                1.2
                            } else {
                                1.0
                            };
                        let dest = if image_scale == 1.2 { // adjust the position of the piece because of the image scaling making the piece overflow downward and to the right 
                            (dest_x - self.square_x * ((image_scale - 1.0 ) / 2.0), 
                            dest_y - self.square_y * ((image_scale - 1.0 ) / 2.0))     
                        }
                        else { 
                            (dest_x, dest_y) 
                        }; 
                        let (dest_x,dest_y) = dest; 
                        let param = graphics::DrawParam::default() 
                        .dest([dest_x, dest_y]) // draws the piece 
                        .scale([ image_scale * self.square_x / image.width() as f32, image_scale * self.square_y / image.height() as f32, ]); 
                        canvas.draw(image, param); 
                    }
                } 
            }
        } 
     
        let possible_moves = if let Some(selected_square) = &self.selected_square { 
            if let Some(from) = chess::square_to_index(selected_square) { // adds all valid moves for the selected "square" if there is one 
                if let Some(piece) = position::get_piece_at(&self.game.position, from) { 
                    moves::valid_moves(from, piece, &self.game.position) 
                } 
                else { 
                    Vec::new() // No piece on square 
                } 
            } 
            else { 
                Vec::new() // Invalid square string 
            } 
        } 
        else { 
            Vec::new() // No square selected 
        }; 
        if !possible_moves.is_empty() { // iterates thrue all the valid moves and makes the open spaces dotted while the possible takes are marked with a "scope" 
            for possible_move in possible_moves { 
                let dest_index = possible_move.to; 
                let dest_col = dest_index % 8; 
                let dest_row = dest_index / 8; 
                let dest_x = dest_col as f32 * self.square_x; 
                let dest_y = (7 - dest_row) as f32 * self.square_y; 
                if get_piece_at(position, dest_index).is_some() { 
                    let take_space = graphics::Mesh::new_circle( //settings for the "scope" 
                    ctx, 
                    graphics::DrawMode::stroke(8.0), 
                    [dest_x + self.square_x / 2.0, dest_y + self.square_y / 2.0], 
                    self.square_x.min(self.square_y) * 0.6, 0.1, 
                    Color::from_rgba(20, 20, 20, 200), 
                    )?; 
                    canvas.draw(&take_space, graphics::DrawParam::default()); 
                } 
                else { 
                    let open_space = graphics::Mesh::new_circle( // settings for the dots 
                        ctx, 
                        graphics::DrawMode::fill(), 
                        [dest_x + self.square_x / 2.0, dest_y + self.square_y / 2.0], 
                        self.square_x.min(self.square_y) * 0.15, 0.1, 
                        Color::from_rgba(20, 20, 20, 200), // made slightly grey and opaque for better visability 
                    )?; 
                    canvas.draw(&open_space, graphics::DrawParam::default()); 
                }
            } 
        } 

        if self.checkmate { 
            let code: String = format!("{}_won", self.color_won.clone().unwrap_or_default()); // checks if someone has won 
            if let Some(image) = self.win_messages.get(&code) { 
                let param = graphics::DrawParam::default() 
                .dest([0.0, 0.0]) // selects the corner of the board 
                .scale([
                    board_size_x / image.width() as f32, // selects the entire application screen (is not fully stretched to the sides due to the png's structure) 
                    board_size_y / image.height() as f32, 
                ]); 
                canvas.draw(image, param); 
            } 
        } 
        if self.stalemate { 
            if let Some(image) = self.win_messages.get("Stalemate") { 
                let param = graphics::DrawParam::default() 
                .dest([0.0, 0.0]) // selects the corner of the board 
                .scale([ board_size_x / image.width() as f32, // selects the entire application screen (is not fully stretched to the sides due to the png's structure) 
                board_size_y / image.height() as f32, 
                ]); 
                canvas.draw(image, param); 
            } 
        } 
        canvas.finish(ctx)?; // closes the draw 
        Ok(()) 
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> ggezGameResult {
        if button == MouseButton::Left {
            if !self.game.is_over() {
                if let Some(square) = self.screen_to_square(x, y) {
                    match &self.selected_square {
                        None => {
                            let position = &self.game.position;
                            if let Some(piece) = position::get_piece_at(position, chess::helper::square_to_index(&square).unwrap()) {
                                if piece.color() == self.game.player_tracker()
                                    && ((self.game.turn % 2 == 1) == self.connection_state.lock().unwrap().is_host)
                                {
                                    self.selected_square = Some(square);
                                }
                            }
                        }
                        Some(from_square) => {
                            if let (Some(from), Some(to)) = (square_to_index(from_square), square_to_index(&square)) {
                                move_piece::execute_move(&mut self.game, from, to);
                                let msg = format!("{} {}", from, to);
                                println!("Pushed to outgoing: {}", msg);
                                let tx = self.connection_state.lock().unwrap().outgoing_tx.clone();
                                if let Err(e) = tx.send(msg.to_string()) {
                                    println!("[Host] Failed to send test message: {}", e);
                                }


                            }
                            self.selected_square = None;
                        }
                    }
                }
            }

            match self.game.result {
                GameResult::Ongoing => {}
                GameResult::Checkmate(color) => {
                    self.checkmate = true;
                    self.color_won = Some(if color == ChessColor::White { "Black" } else { "White" }.to_string());
                }
                GameResult::Stalemate => {
                    self.stalemate = true;
                }
            }
        }

        Ok(())
    }
}

fn main() -> ggez::GameResult {
    println!("Starting Chess GUI...");
    let args: Vec<String> = env::args().collect();

    let (mut ctx, event_loop) = ContextBuilder::new("Chess_gui", "Martin")
        .window_setup(ggez::conf::WindowSetup::default().title("Chess :)"))
        .add_resource_path("./resources")
        .build()
        .expect("Failed to build ggez context");

    let conn_state = Arc::new(Mutex::new(ConnectionState::new()));
    println!("ConnectionState Arc address: {:p}", &conn_state);

    if args.len() > 1 {
        match args[1].as_str() {
            "--host" => {
                conn_state.lock().unwrap().is_host = true;
                let port: u16 = args.get(2).expect("Port not specified").parse().unwrap();
                let conn_clone = Arc::clone(&conn_state);
                thread::spawn(move || {
                    connection::start_server(port, conn_clone);
                });
            }
            "--connect" => {
                conn_state.lock().unwrap().is_host = false;
                let addr = args.get(2).expect("Address not specified").clone();
                let conn_clone = Arc::clone(&conn_state);
                thread::spawn(move || {
                    connection::start_client(&addr, conn_clone);
                });
            }
            _ => println!("Unknown option: {}", args[1]),
        }
    }

    let my_game = match MblomstGui::new(&mut ctx, Arc::clone(&conn_state)) {
        Ok(gui) => {
            println!("GUI initialized successfully");
            gui
        }
        Err(e) => {
            println!("Failed to initialize GUI: {}", e);
            return Err(e);
        }
    };

    println!("Running event loop...");
    event::run(ctx, event_loop, my_game)
}