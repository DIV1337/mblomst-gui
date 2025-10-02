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
use std::collections::HashMap; 

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
}

impl MblomstGui { 
    pub fn new(ctx: &mut Context) -> ggezGameResult<MblomstGui> { 
        let position = initialize_board(); // creates "the game" 
        let game = Game::new(position); 
        let mut piece_images = HashMap::new(); // initiates the decoding of pieces 
        let piece_codes = [
            "wP", "wN", "wB", "wR", "wQ", "wK",
            "bP", "bN", "bB", "bR", "bQ", "bK", 
            ]; 
            for code in piece_codes { // connecting the decoded pieces to their co-responding image 
                let path = format!("/pieces/{}.png", code); 
                let image = Image::from_path(ctx, path)?; 
                piece_images.insert(code.to_string(), image); 
            } 
            let mut win_messages = HashMap::new(); // initiating the win screen for both black and white 
            let messages_name = [ 
                "White_won", "Black_won", "Stalemate", 
            ]; 
            for message in messages_name { // connecting it to the winning image 
                let path = format!("/messages/{}.png", message); 
                let image = Image::from_path(ctx, path)?; 
                win_messages.insert(message.to_string(), image); 
            } 
            Ok(MblomstGui { 
                game, board_size: (0.0, 0.0), 
                square_x: 0.0, square_y: 0.0, 
                selected_square: None, piece_images, 
                checkmate: false, 
                color_won: None, 
                win_messages, 
                stalemate: false, 
            }
        ) 
    } 
    
    fn screen_to_square(&self, x: f32, y: f32) -> Option<String> { 
        let col = ( x / self.square_x) as usize; // using mouse input x and dividing it by the square width to get a number (for example 3.2) then using usize to convert it to 3 
        let row = (y / self.square_y) as usize; 
        let file = (b'a' + col as u8) as char; // using the number the code got from col and converts it to an 8-bit and adds it to the byte literal 
        let rank = 8 - row; // since y:0 is at the top 
        Some(format!("{}{}", file, rank)) 
    } 
}

impl EventHandler for MblomstGui { 
    fn update(&mut self, ctx: &mut Context) -> ggezGameResult { 
        let (width, height) = ctx.gfx.drawable_size(); // Set the board size in the update function to adapt to new application sizes 
        self.board_size = (width, height); // updates board size every frame to ensure the board stretches to the applications size 
        self.square_x = width / 8.0; // makes the "squares" one eights of the sqreen, and thus covering the entire screen 
        self.square_y = height / 8.0; 
        if ctx.keyboard.is_key_pressed(KeyCode::R) { // detects if the key "r" is pressed, and if so resets the board 
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

    fn mouse_button_down_event( // process mouse events 
        &mut self, 
        _ctx: &mut Context, 
        button: MouseButton, 
        x: f32, 
        y: f32, 
        ) -> ggezGameResult { 
        if button == MouseButton::Left { 
            if !self.game.is_over() { // checks if game is not done 
                if let Some(square) = self.screen_to_square(x, y) { // makes sure a "square is clicked" 
                    match &self.selected_square { None => { 
                        let position = &self.game.position; // a "square" only gets selected if there is a piece on that "square" as well as the selected piece has the same color as the players turn, i.e white moves white's pieces 
                        if let Some(piece) = position::get_piece_at(position, chess::helper::square_to_index(&square).unwrap()) { 
                            if piece.color() == self.game.player_tracker() { 
                                self.selected_square = Some(square); // a "square" is selected to be able to highligt in the draw function, and more 
                                } 
                            } 
                        } 
                        Some(from_square) => { // Second click: use stored square + new square 
                            if let (Some(from), Some(to)) = (square_to_index(from_square), square_to_index(&square)) { 
                                move_piece::execute_move(&mut self.game, from, to); // selects the second "square" and executes the move, then resets the selected squares 
                            } // game logic process if the move is valid or not, if not the turn is not skipped, and the selected piece logic works for the same player again 
                            self.selected_square = None; 
                        } 
                    } 
                } 
            } 
            match self.game.result { // processing game state 
                GameResult::Ongoing => {} 
                GameResult::Checkmate(color) => { self.checkmate = true; 
                    if color == ChessColor::White { 
                        self.color_won = Some("Black".to_string()); 
                    } 
                    else { 
                        self.color_won = Some("White".to_string()); 
                    } 
                } 
                GameResult::Stalemate => { self.stalemate = true; }, 
            } 
        } 
        Ok(()) 
        
    } 
}

fn main() -> ggezGameResult { 
    let (mut ctx, event_loop) = ContextBuilder::new("Chess_gui", "Martin") 
    .window_setup(ggez::conf::WindowSetup::default().title("Chess :)")) 
    .build() 
    .expect("Failed to build ggez context"); 
    let my_game = MblomstGui::new(&mut ctx)?; 
    event::run(ctx, event_loop, my_game) 
}