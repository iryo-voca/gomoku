use macroquad::prelude::*;
use ::rand::Rng;
use ::rand::thread_rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbPiece {
    Black90,
    Black70,
    Black30,
    Black10,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DefinitePiece {
    Black,
    White,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Player {
    Black,
    White,
}

#[derive(Default)]
struct WinningPieces {
    black: Vec<(usize, usize)>,
    white: Vec<(usize, usize)>,
}

const BOARD_SIZE: usize = 15;
const SCALE: f32 = 1.5;
const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 1200.0;

const BASE_CELL_SIZE: f32 = 30.0;
const CELL_SIZE: f32 = BASE_CELL_SIZE * SCALE;
const PIECE_RADIUS: f32 = CELL_SIZE / 2.0;

const BOARD_ACTUAL_WIDTH: f32 = CELL_SIZE * (BOARD_SIZE - 1) as f32;
const BOARD_ACTUAL_HEIGHT: f32 = CELL_SIZE * (BOARD_SIZE - 1) as f32;

const BOARD_OFFSET_X: f32 = (WINDOW_WIDTH - BOARD_ACTUAL_WIDTH) / 2.0;
const BOARD_OFFSET_Y: f32 = (WINDOW_HEIGHT - BOARD_ACTUAL_HEIGHT) / 2.0 + 100.0;

const END_TURN_BUTTON_WIDTH: f32 = 160.0;
const END_TURN_BUTTON_HEIGHT: f32 = 50.0;

const GAME_OVER_BUTTON_SCALE: f32 = 1.5;
const HOVER_SCALE: f32 = 1.05;
const PREVIEW_ALPHA: f32 = 0.4;

struct GameState {
    board: Vec<Vec<ProbPiece>>,
    show_observation: bool,
    observation_board: Vec<Vec<DefinitePiece>>,
    observation_winner: Option<&'static str>,
    observe_remaining: u8,
    current_player: Player,
    black_prob_index: usize,
    white_prob_index: usize,
    winning_pieces: WinningPieces,
    game_over: bool,
    current_turn_move_count: u8,
    show_prob_hint: bool,
}

impl Default for GameState {
    fn default() -> Self {
        let board = vec![vec![ProbPiece::Empty; BOARD_SIZE]; BOARD_SIZE];
        GameState {
            board,
            show_observation: false,
            observation_board: vec![vec![DefinitePiece::Empty; BOARD_SIZE]; BOARD_SIZE],
            observation_winner: None,
            observe_remaining: 1,
            current_player: Player::Black,
            black_prob_index: 0,
            white_prob_index: 0,
            winning_pieces: WinningPieces::default(),
            game_over: false,
            current_turn_move_count: 0,
            show_prob_hint: true,
        }
    }
}

fn mouse_to_grid(x: f32, y: f32) -> Option<(usize, usize)> {
    let grid_x = x - BOARD_OFFSET_X;
    let grid_y = y - BOARD_OFFSET_Y;
    let tolerance = 15.0 * SCALE;
    
    if grid_x < -tolerance || grid_y < -tolerance || 
       grid_x > CELL_SIZE * BOARD_SIZE as f32 + tolerance || 
       grid_y > CELL_SIZE * BOARD_SIZE as f32 + tolerance {
        return None;
    }
    
    let col = (grid_x / CELL_SIZE).round() as usize;
    let row = (grid_y / CELL_SIZE).round() as usize;
    
    if row < BOARD_SIZE && col < BOARD_SIZE {
        Some((row, col))
    } else {
        None
    }
}

fn prob_to_definite(piece: ProbPiece) -> DefinitePiece {
    let mut rng = thread_rng();
    match piece {
        ProbPiece::Black90 => if rng.gen_range(0..100) < 90 { DefinitePiece::Black } else { DefinitePiece::White },
        ProbPiece::Black70 => if rng.gen_range(0..100) < 70 { DefinitePiece::Black } else { DefinitePiece::White },
        ProbPiece::Black30 => if rng.gen_range(0..100) < 30 { DefinitePiece::Black } else { DefinitePiece::White },
        ProbPiece::Black10 => if rng.gen_range(0..100) < 10 { DefinitePiece::Black } else { DefinitePiece::White },
        ProbPiece::Empty => DefinitePiece::Empty,
    }
}

fn get_current_prob_piece(state: &GameState) -> ProbPiece {
    match state.current_player {
        Player::Black => match state.black_prob_index {
            0 => ProbPiece::Black90,
            1 => ProbPiece::Black70,
            _ => ProbPiece::Black90,
        },
        Player::White => match state.white_prob_index {
            0 => ProbPiece::Black10,
            1 => ProbPiece::Black30,
            _ => ProbPiece::Black10,
        },
    }
}

fn switch_player_prob(state: &mut GameState) {
    match state.current_player {
        Player::Black => state.black_prob_index = (state.black_prob_index + 1) % 2,
        Player::White => state.white_prob_index = (state.white_prob_index + 1) % 2,
    }
}

fn check_winner(board: &[Vec<DefinitePiece>]) -> (Option<&'static str>, WinningPieces) {
    let directions = [(0, 1), (1, 0), (1, 1), (1, -1)];
    let mut black_has_win = false;
    let mut white_has_win = false;
    let mut winning_pieces = WinningPieces::default();
    
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let current = board[row][col];
            if current == DefinitePiece::Empty {
                continue;
            }
            
            for (dr, dc) in directions {
                let mut count = 1;
                let mut current_win_pieces = vec![(row, col)];
                
                for step in 1..5 {
                    let r = row as i32 + dr * step;
                    let c = col as i32 + dc * step;
                    
                    if r < 0 || r >= BOARD_SIZE as i32 || c < 0 || c >= BOARD_SIZE as i32 {
                        break;
                    }
                    
                    let (r, c) = (r as usize, c as usize);
                    if board[r][c] == current {
                        count += 1;
                        current_win_pieces.push((r, c));
                    } else {
                        break;
                    }
                }
                
                if count >= 5 {
                    if current == DefinitePiece::Black && !black_has_win {
                        black_has_win = true;
                        winning_pieces.black = current_win_pieces;
                    } else if current == DefinitePiece::White && !white_has_win {
                        white_has_win = true;
                        winning_pieces.white = current_win_pieces;
                    }
                }
            }
        }
    }
    
    let result = if black_has_win && white_has_win {
        Some("Draw! Both Players Win!")
    } else if black_has_win {
        Some("Black Wins!")
    } else if white_has_win {
        Some("White Wins!")
    } else {
        let is_full = board.iter().all(|row| row.iter().all(|&p| p != DefinitePiece::Empty));
        if is_full {
            Some("Draw! Board Full!")
        } else {
            None
        }
    };
    
    (result, winning_pieces)
}

fn draw_board() {
    for col in 0..BOARD_SIZE {
        let x = BOARD_OFFSET_X + col as f32 * CELL_SIZE;
        draw_line(
            x,
            BOARD_OFFSET_Y,
            x,
            BOARD_OFFSET_Y + CELL_SIZE * (BOARD_SIZE - 1) as f32,
            3.0 * SCALE,
            DARKGRAY,
        );
    }
    
    for row in 0..BOARD_SIZE {
        let y = BOARD_OFFSET_Y + row as f32 * CELL_SIZE;
        draw_line(
            BOARD_OFFSET_X,
            y,
            BOARD_OFFSET_X + CELL_SIZE * (BOARD_SIZE - 1) as f32,
            y,
            3.0 * SCALE,
            DARKGRAY,
        );
    }
    
    let star_positions = [(3, 3), (3, 11), (7, 7), (11, 3), (11, 11)];
    for (row, col) in star_positions {
        let x = BOARD_OFFSET_X + col as f32 * CELL_SIZE;
        let y = BOARD_OFFSET_Y + row as f32 * CELL_SIZE;
        draw_circle(
            x,
            y,
            6.0 * SCALE,
            BLACK,
        );
    }
}

fn draw_prob_pieces(board: &[Vec<ProbPiece>]) {
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let piece = board[row][col];
            if piece == ProbPiece::Empty {
                continue;
            }
            
            let x = BOARD_OFFSET_X + col as f32 * CELL_SIZE;
            let y = BOARD_OFFSET_Y + row as f32 * CELL_SIZE;

            let color = match piece {
                ProbPiece::Black90 => Color::new(0.1, 0.1, 0.1, 1.0),
                ProbPiece::Black70 => Color::new(0.3, 0.3, 0.3, 1.0),
                ProbPiece::Black30 => Color::new(0.6, 0.6, 0.6, 1.0),
                ProbPiece::Black10 => Color::new(0.8, 0.8, 0.8, 1.0),
                ProbPiece::Empty => continue,
            };

            draw_circle(
                x,
                y,
                PIECE_RADIUS,
                color,
            );

            let text = match piece {
                ProbPiece::Black90 => "90%",
                ProbPiece::Black70 => "70%",
                ProbPiece::Black30 => "30%",
                ProbPiece::Black10 => "10%",
                _ => "",
            };
            let text_size = 22.0 * SCALE;
            let text_width = measure_text(text, None, text_size as u16, 1.0).width;
            draw_text(
                text,
                x - text_width / 2.0,
                y + text_size / 3.0,
                text_size,
                WHITE,
            );
        }
    }
}

// 重点修改：落子预判改为统一颜色（深灰色）
fn draw_piece_preview(state: &GameState) {
    if state.game_over || state.show_observation || state.current_turn_move_count > 0 {
        return;
    }
    
    let (mouse_x, mouse_y) = mouse_position();
    if let Some((row, col)) = mouse_to_grid(mouse_x, mouse_y) {
        if state.board[row][col] == ProbPiece::Empty {
            let x = BOARD_OFFSET_X + col as f32 * CELL_SIZE;
            let y = BOARD_OFFSET_Y + row as f32 * CELL_SIZE;
            
            // 统一使用深灰色（可根据喜好修改 r/g/b 值），保留原有的透明度
            let preview_color = Color::new(0.2, 0.2, 0.2, PREVIEW_ALPHA);
            
            draw_circle(
                x,
                y,
                PIECE_RADIUS - 4.0 * SCALE,
                preview_color
            );
        }
    }
}

fn draw_observation_board(
    board: &[Vec<DefinitePiece>],
    winner: Option<&str>,
    winning_pieces: &WinningPieces,
) {
    let bg_padding = 30.0 * SCALE;
    let bg_color = Color::new(0.0, 0.0, 0.0, 0.3);
    draw_rectangle(
        BOARD_OFFSET_X - bg_padding,
        BOARD_OFFSET_Y - bg_padding,
        CELL_SIZE * (BOARD_SIZE - 1) as f32 + bg_padding * 2.0,
        CELL_SIZE * (BOARD_SIZE - 1) as f32 + bg_padding * 2.0,
        bg_color,
    );

    let observe_piece_radius = PIECE_RADIUS;
    let win_border_width = 3.0 * SCALE;
    let win_border_color = Color::new(1.0, 0.0, 0.0, 1.0);

    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let piece = board[row][col];
            if piece == DefinitePiece::Empty {
                continue;
            }
            
            let x = BOARD_OFFSET_X + col as f32 * CELL_SIZE;
            let y = BOARD_OFFSET_Y + row as f32 * CELL_SIZE;

            match piece {
                DefinitePiece::Black => draw_circle(x, y, observe_piece_radius, BLACK),
                DefinitePiece::White => draw_circle(x, y, observe_piece_radius, WHITE),
                DefinitePiece::Empty => continue,
            }
        }
    }

    for (row, col) in &winning_pieces.black {
        let x = BOARD_OFFSET_X + (*col as f32) * CELL_SIZE;
        let y = BOARD_OFFSET_Y + (*row as f32) * CELL_SIZE;
        draw_circle_lines(
            x,
            y,
            observe_piece_radius + win_border_width / 2.0,
            win_border_width,
            win_border_color,
        );
    }
    for (row, col) in &winning_pieces.white {
        let x = BOARD_OFFSET_X + (*col as f32) * CELL_SIZE;
        let y = BOARD_OFFSET_Y + (*row as f32) * CELL_SIZE;
        draw_circle_lines(
            x,
            y,
            observe_piece_radius + win_border_width / 2.0,
            win_border_width,
            win_border_color,
        );
    }

    if let Some(winner_text) = winner {
        let text_size = 100.0 * SCALE;
        let text_width = measure_text(winner_text, None, text_size as u16, 1.0).width;
        let text_height = text_size * 1.2;

        let text_x = (WINDOW_WIDTH - text_width) / 2.0;
        let text_y = 50.0 * SCALE;

        let bg_padding = 15.0 * SCALE;
        draw_rectangle(
            text_x - bg_padding,
            text_y - bg_padding / 2.0,
            text_width + bg_padding * 2.0,
            text_height + bg_padding,
            Color::new(1.0, 1.0, 0.0, 0.95),
        );
        draw_rectangle_lines(
            text_x - bg_padding,
            text_y - bg_padding / 2.0,
            text_width + bg_padding * 2.0,
            text_height + bg_padding,
            4.0 * SCALE,
            BLACK,
        );

        draw_text(
            winner_text,
            text_x,
            text_y + text_size * 0.8,
            text_size,
            BLACK,
        );
    }
}

fn draw_ui(
    show_observation: bool,
    observe_remaining: u8,
    current_player: Player,
    current_prob_piece: ProbPiece,
    game_over: bool,
    current_turn_move_count: u8,
    show_prob_hint: bool,
) {
    if game_over {
        return;
    }

    let (mouse_x, mouse_y) = mouse_position();

    let (player_text, prob_text) = match current_player {
        Player::Black => (
            "Current Turn: Black",
            match current_prob_piece {
                ProbPiece::Black90 => "Next Piece: 90% Black",
                ProbPiece::Black70 => "Next Piece: 70% Black",
                _ => "Next Piece: 90% Black",
            },
        ),
        Player::White => (
            "Current Turn: White",
            match current_prob_piece {
                ProbPiece::Black10 => "Next Piece: 90% White (10% Black)",
                ProbPiece::Black30 => "Next Piece: 70% White (30% Black)",
                _ => "Next Piece: 90% White (10% Black)",
            },
        ),
    };

    let player_text_size = 28.0 * SCALE;
    let player_text_width = measure_text(player_text, None, player_text_size as u16, 1.0).width;
    let player_bg_x = BOARD_OFFSET_X + (BOARD_ACTUAL_WIDTH - player_text_width) / 2.0 - 15.0 * SCALE;
    draw_rectangle(
        player_bg_x,
        20.0 * SCALE,
        player_text_width + 30.0 * SCALE,
        40.0 * SCALE,
        Color::new(0.9, 0.9, 0.9, 0.8),
    );
    draw_text(
        player_text,
        player_bg_x + 15.0 * SCALE,
        20.0 * SCALE + 30.0 * SCALE,
        player_text_size,
        BLACK,
    );

    if show_prob_hint {
        let prob_text_size = 24.0 * SCALE;
        let prob_text_width = measure_text(prob_text, None, prob_text_size as u16, 1.0).width;
        let prob_bg_x = BOARD_OFFSET_X + (BOARD_ACTUAL_WIDTH - prob_text_width) / 2.0 - 15.0 * SCALE;
        draw_rectangle(
            prob_bg_x,
            70.0 * SCALE,
            prob_text_width + 30.0 * SCALE,
            35.0 * SCALE,
            Color::new(0.85, 0.85, 0.85, 0.8),
        );
        draw_text(
            prob_text,
            prob_bg_x + 15.0 * SCALE,
            70.0 * SCALE + 25.0 * SCALE,
            prob_text_size,
            BLACK,
        );
    }

    let button_y = 160.0 * SCALE;
    let button_width = 160.0 * SCALE;
    let button_height = 50.0 * SCALE;

    let observe_x = BOARD_OFFSET_X + (BOARD_ACTUAL_WIDTH - button_width - END_TURN_BUTTON_WIDTH * SCALE - 40.0 * SCALE) / 2.0;
    let observe_text = if show_observation { "Hide Preview" } else { "Preview Board" };
    let is_observe_hover = (observe_remaining > 0 || show_observation) &&
        mouse_x >= observe_x && mouse_x <= observe_x + button_width &&
        mouse_y >= button_y && mouse_y <= button_y + button_height;
    let observe_color = if observe_remaining > 0 || show_observation {
        if is_observe_hover { Color::new(0.1, 0.7, 0.1, 0.9) } else { GREEN }
    } else {
        GRAY
    };
    let observe_scale = if is_observe_hover { HOVER_SCALE } else { 1.0 };
    let observe_draw_x = observe_x - (button_width * (observe_scale - 1.0)) / 2.0;
    let observe_draw_y = button_y - (button_height * (observe_scale - 1.0)) / 2.0;

    draw_rectangle(
        observe_draw_x,
        observe_draw_y,
        button_width * observe_scale,
        button_height * observe_scale,
        observe_color,
    );
    let observe_text_size = 22.0 * SCALE;
    let observe_text_width = measure_text(observe_text, None, observe_text_size as u16, 1.0).width;
    draw_text(
        observe_text,
        observe_draw_x + (button_width * observe_scale - observe_text_width) / 2.0,
        observe_draw_y + 30.0 * SCALE,
        observe_text_size,
        WHITE,
    );

    let count_text = format!("Previews Left: {}", observe_remaining);
    let count_text_size = 22.0 * SCALE;
    let count_text_x = observe_x + 4.0 * SCALE;
    draw_text(
        &count_text,
        count_text_x,
        button_y - 10.0 * SCALE,
        count_text_size,
        BLACK,
    );

    let move_hint = if current_turn_move_count > 0 {
        "Click End Turn"
    } else {
        "Place a Piece"
    };
    let move_hint_color = if current_turn_move_count > 0 {
        Color::new(0.0, 0.7, 0.0, 1.0)
    } else {
        Color::new(0.8, 0.0, 0.0, 1.0)
    };
    let move_hint_size = 24.0 * SCALE;
    let move_hint_width = measure_text(move_hint, None, move_hint_size as u16, 1.0).width;
    let end_turn_button_x = observe_x + button_width + 40.0 * SCALE;
    let move_hint_x = end_turn_button_x + (END_TURN_BUTTON_WIDTH * SCALE - move_hint_width) / 2.0;
    draw_text(
        move_hint,
        move_hint_x,
        button_y - 10.0 * SCALE,
        move_hint_size,
        move_hint_color,
    );

    let end_turn_button_enabled = current_turn_move_count > 0;
    let is_end_turn_hover = end_turn_button_enabled &&
        mouse_x >= end_turn_button_x && mouse_x <= end_turn_button_x + END_TURN_BUTTON_WIDTH * SCALE &&
        mouse_y >= button_y && mouse_y <= button_y + END_TURN_BUTTON_HEIGHT * SCALE;
    let end_turn_button_color = if end_turn_button_enabled {
        if is_end_turn_hover { Color::new(0.1, 0.3, 0.7, 0.9) } else { Color::new(0.2, 0.4, 0.8, 0.9) }
    } else {
        Color::new(0.5, 0.5, 0.5, 0.7)
    };
    let end_turn_scale = if is_end_turn_hover { HOVER_SCALE } else { 1.0 };
    let end_turn_draw_x = end_turn_button_x - (END_TURN_BUTTON_WIDTH * SCALE * (end_turn_scale - 1.0)) / 2.0;
    let end_turn_draw_y = button_y - (END_TURN_BUTTON_HEIGHT * SCALE * (end_turn_scale - 1.0)) / 2.0;

    let end_turn_button_width = END_TURN_BUTTON_WIDTH * SCALE;
    let end_turn_button_height = END_TURN_BUTTON_HEIGHT * SCALE;
    let end_turn_text = "End Turn";
    let end_turn_text_size = 22.0 * SCALE;

    draw_rectangle(
        end_turn_draw_x,
        end_turn_draw_y,
        end_turn_button_width * end_turn_scale,
        end_turn_button_height * end_turn_scale,
        end_turn_button_color,
    );
    // 已删除结束回合按钮的边框绘制代码
    let end_turn_text_width = measure_text(end_turn_text, None, end_turn_text_size as u16, 1.0).width;
    draw_text(
        end_turn_text,
        end_turn_draw_x + (end_turn_button_width * end_turn_scale - end_turn_text_width) / 2.0,
        end_turn_draw_y + 30.0 * SCALE,
        end_turn_text_size,
        WHITE,
    );
}

fn draw_game_rules() {
    let base_y = BOARD_OFFSET_Y + CELL_SIZE * (BOARD_SIZE - 1) as f32 + 20.0 * SCALE;
    let window_center_x = WINDOW_WIDTH / 2.0;

    let title = "Game Rules";
    let title_size = 26.0 * SCALE;
    let rule_lines = [
        "1. Black goes first. Players take turns, 1 piece per turn.",
        "2. Black's pieces: 90% Black / 70% Black (rotates each turn)",
        "3. White's pieces: 90% White / 70% White (rotates each turn)",
        "4. Click 'Preview Board' to see final pieces once per turn.",
        "5. Win by getting 5 same pieces in a row after preview."
    ];
    let rule_size = 18.0 * SCALE;
    let line_spacing = 24.0 * SCALE;
    let padding = 25.0 * SCALE;
    let side_margin = 50.0 * SCALE;

    let title_width = measure_text(title, None, title_size as u16, 1.0).width;
    let mut max_rule_width = 0.0;
    for line in &rule_lines {
        let width = measure_text(line, None, rule_size as u16, 1.0).width;
        if width > max_rule_width {
            max_rule_width = width;
        }
    }
    let content_max_width = title_width.max(max_rule_width);
    let bg_width = (content_max_width + 2.0 * padding).min(WINDOW_WIDTH - 2.0 * side_margin);

    let title_height = title_size * 1.2;
    let rules_total_height = (rule_lines.len() as f32) * line_spacing;
    let bg_height = title_height + rules_total_height + 10.0 * SCALE;

    let bg_x = window_center_x - bg_width / 2.0;
    let bg_y = base_y - 5.0 * SCALE;

    draw_rectangle(
        bg_x,
        bg_y,
        bg_width,
        bg_height,
        Color::new(0.95, 0.95, 0.95, 0.9),
    );

    let title_x = window_center_x - title_width / 2.0;
    let title_draw_y = bg_y + padding;
    draw_text(
        title,
        title_x,
        title_draw_y,
        title_size,
        Color::new(0.8, 0.2, 0.2, 1.0),
    );

    let rules_start_y = title_draw_y + line_spacing * 1.0;
    let rule_x = window_center_x - max_rule_width / 2.0 + 5.0 * SCALE;

    for (i, line) in rule_lines.iter().enumerate() {
        let y = rules_start_y + (i as f32) * line_spacing;
        draw_text(
            line,
            rule_x,
            y,
            rule_size,
            BLACK,
        );
    }
}

#[macroquad::main("Probability Gomoku")]
async fn main() {
    miniquad::window::set_window_size(1200, 1300);
    let mut game_state = GameState::default();

    loop {
        if !game_state.game_over && is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();

            let button_y = 160.0 * SCALE;
            let button_width = 160.0 * SCALE;
            let button_height = 50.0 * SCALE;
            let observe_x = BOARD_OFFSET_X + (BOARD_ACTUAL_WIDTH - button_width - END_TURN_BUTTON_WIDTH * SCALE - 40.0 * SCALE) / 2.0;
            let end_turn_button_x = observe_x + button_width + 40.0 * SCALE;

            if mouse_x >= observe_x && mouse_x <= observe_x + button_width &&
               mouse_y >= button_y && mouse_y <= button_y + button_height {
                if game_state.show_observation {
                    game_state.show_observation = false;
                } else if game_state.observe_remaining > 0 {
                    game_state.observe_remaining -= 1;
                    let mut observation_board = vec![vec![DefinitePiece::Empty; BOARD_SIZE]; BOARD_SIZE];
                    for row in 0..BOARD_SIZE {
                        for col in 0..BOARD_SIZE {
                            observation_board[row][col] = prob_to_definite(game_state.board[row][col]);
                        }
                    }
                    game_state.observation_board = observation_board;
                    let (winner, winning_pieces) = check_winner(&game_state.observation_board);
                    game_state.observation_winner = winner;
                    game_state.winning_pieces = winning_pieces;
                    game_state.show_observation = true;

                    if winner.is_some() {
                        game_state.game_over = true;
                    }
                }
            }

            if game_state.current_turn_move_count > 0 &&
               mouse_x >= end_turn_button_x && mouse_x <= end_turn_button_x + END_TURN_BUTTON_WIDTH * SCALE &&
               mouse_y >= button_y && mouse_y <= button_y + END_TURN_BUTTON_HEIGHT * SCALE {
                switch_player_prob(&mut game_state);
                game_state.current_player = match game_state.current_player {
                    Player::Black => Player::White,
                    Player::White => Player::Black,
                };
                game_state.observe_remaining = 1;
                game_state.show_observation = false;
                game_state.observation_winner = None;
                game_state.winning_pieces = WinningPieces::default();
                game_state.current_turn_move_count = 0;
                game_state.show_prob_hint = true;
            }

            if !game_state.show_observation && game_state.current_turn_move_count == 0 {
                if let Some((row, col)) = mouse_to_grid(mouse_x, mouse_y) {
                    if game_state.board[row][col] == ProbPiece::Empty {
                        let current_piece = get_current_prob_piece(&game_state);
                        game_state.board[row][col] = current_piece;

                        game_state.current_turn_move_count = 1;
                        game_state.show_prob_hint = false;
                    }
                }
            }
        }

        if game_state.game_over && is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();

            let button_y = BOARD_OFFSET_Y + CELL_SIZE * BOARD_SIZE as f32 + 40.0 * SCALE;
            let button_width = 180.0 * SCALE * GAME_OVER_BUTTON_SCALE;
            let button_height = 60.0 * SCALE * GAME_OVER_BUTTON_SCALE;
            let restart_x = BOARD_OFFSET_X + (BOARD_ACTUAL_WIDTH - button_width * 2.0 - 60.0 * SCALE) / 2.0;
            let exit_x = restart_x + button_width + 60.0 * SCALE;

            if mouse_x >= restart_x && mouse_x <= restart_x + button_width &&
               mouse_y >= button_y && mouse_y <= button_y + button_height {
                game_state = GameState::default();
            }

            if mouse_x >= exit_x && mouse_x <= exit_x + button_width &&
               mouse_y >= button_y && mouse_y <= button_y + button_height {
                std::process::exit(0);
            }
        }

        clear_background(WHITE);
        draw_board();
        draw_prob_pieces(&game_state.board);
        draw_piece_preview(&game_state);

        let current_prob_piece = get_current_prob_piece(&game_state);
        draw_ui(
            game_state.show_observation,
            game_state.observe_remaining,
            game_state.current_player,
            current_prob_piece,
            game_state.game_over,
            game_state.current_turn_move_count,
            game_state.show_prob_hint,
        );

        if game_state.show_observation {
            draw_observation_board(
                &game_state.observation_board,
                game_state.observation_winner,
                &game_state.winning_pieces,
            );
        }

        if !game_state.game_over {
            draw_game_rules();
        }

        if game_state.game_over {
            let (mouse_x, mouse_y) = mouse_position();
            let button_y = BOARD_OFFSET_Y + CELL_SIZE * BOARD_SIZE as f32 + 40.0 * SCALE;
            let button_width = 180.0 * SCALE * GAME_OVER_BUTTON_SCALE;
            let button_height = 60.0 * SCALE * GAME_OVER_BUTTON_SCALE;
            let restart_x = BOARD_OFFSET_X + (BOARD_ACTUAL_WIDTH - button_width * 2.0 - 60.0 * SCALE) / 2.0;
            let exit_x = restart_x + button_width + 60.0 * SCALE;

            let is_restart_hover = mouse_x >= restart_x && mouse_x <= restart_x + button_width &&
                mouse_y >= button_y && mouse_y <= button_y + button_height;
            let restart_color = if is_restart_hover { Color::new(0.1, 0.7, 0.1, 0.9) } else { Color::new(0.0, 0.8, 0.0, 0.9) };
            let restart_scale = if is_restart_hover { HOVER_SCALE } else { 1.0 };
            let restart_draw_x = restart_x - (button_width * (restart_scale - 1.0)) / 2.0;
            let restart_draw_y = button_y - (button_height * (restart_scale - 1.0)) / 2.0;

            draw_rectangle(
                restart_draw_x,
                restart_draw_y,
                button_width * restart_scale,
                button_height * restart_scale,
                restart_color,
            );
            draw_rectangle_lines(
                restart_draw_x,
                restart_draw_y,
                button_width * restart_scale,
                button_height * restart_scale,
                4.0 * SCALE,
                BLACK,
            );
            let restart_text = "Restart Game";
            let restart_text_size = 28.0 * SCALE * GAME_OVER_BUTTON_SCALE;
            let restart_text_width = measure_text(restart_text, None, restart_text_size as u16, 1.0).width;
            draw_text(
                restart_text,
                restart_draw_x + (button_width * restart_scale - restart_text_width) / 2.0,
                restart_draw_y + 35.0 * SCALE * GAME_OVER_BUTTON_SCALE,
                restart_text_size,
                WHITE,
            );

            let is_exit_hover = mouse_x >= exit_x && mouse_x <= exit_x + button_width &&
                mouse_y >= button_y && mouse_y <= button_y + button_height;
            let exit_color = if is_exit_hover { Color::new(0.7, 0.1, 0.1, 0.9) } else { Color::new(0.8, 0.0, 0.0, 0.9) };
            let exit_scale = if is_exit_hover { HOVER_SCALE } else { 1.0 };
            let exit_draw_x = exit_x - (button_width * (exit_scale - 1.0)) / 2.0;
            let exit_draw_y = button_y - (button_height * (exit_scale - 1.0)) / 2.0;

            draw_rectangle(
                exit_draw_x,
                exit_draw_y,
                button_width * exit_scale,
                button_height * exit_scale,
                exit_color,
            );
            draw_rectangle_lines(
                exit_draw_x,
                exit_draw_y,
                button_width * exit_scale,
                button_height * exit_scale,
                4.0 * SCALE,
                BLACK,
            );
            let exit_text = "Exit Game";
            let exit_text_size = 28.0 * SCALE * GAME_OVER_BUTTON_SCALE;
            let exit_text_width = measure_text(exit_text, None, exit_text_size as u16, 1.0).width;
            draw_text(
                exit_text,
                exit_draw_x + (button_width * exit_scale - exit_text_width) / 2.0,
                exit_draw_y + 35.0 * SCALE * GAME_OVER_BUTTON_SCALE,
                exit_text_size,
                WHITE,
            );
        }

        next_frame().await;
    }
}