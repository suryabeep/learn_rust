use array2d::{Array2D};
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, BufRead};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file containing the game's start state
    #[arg(short, long)]
    file: String,
}

#[derive(Clone, PartialEq, Copy)]
enum CellState {
    Dead,
    Live,
}

struct GameState {
    state: Array2D<CellState>,
}

impl GameState {
    fn update(&mut self, prev_state: &GameState) {
        assert_eq!(self.state.num_rows(), prev_state.state.num_rows());
        for i in 0..self.state.num_rows() {
            for j in 0..self.state.num_columns() {
                self.update_single_cell(prev_state, i, j);
            }
        }
    }

    fn update_single_cell(&mut self, prev_state: &GameState, row_idx: usize, col_idx: usize) {
        // count the number of live neighbours
        let mut num_live_neighbours = 0;
        // iterate over all neighbouring rows
        for row in (row_idx as i32) - 1 .. (row_idx as i32) + 1 {
            if row < 0 || row >= prev_state.state.num_rows() as i32 {
                continue;
            }
            // and iterate over all neighbouring columns
            for col in (col_idx as i32) - 1 .. (col_idx as i32) + 1 {
                if col < 0 || col >= prev_state.state.num_columns() as i32
                {
                    continue;
                }
                
                if prev_state.state[(row as usize, col as usize)] == CellState::Live {
                    num_live_neighbours += 1;
                }
            }
        }

        // apply the rules to this cell
        self.state[(row_idx, col_idx)] = match num_live_neighbours {
            0 | 1 => CellState::Dead,
            2     => self.state[(row_idx, col_idx)], // Live cells stay alive, dead cells stay dead
            3     => CellState::Live,   // live cells survive, and dead cells revive
            _     => CellState::Dead,
        };
    }
}

fn construct_game(dim: &usize) -> GameState {
    let game = GameState {
        state: Array2D::filled_with(CellState::Dead, *dim, *dim),
    };
    return game;
}

fn setup_states_from_file(filename: String) -> (GameState, GameState) {

    let file = match File::open(filename) {
        Ok(obj) => obj,
        Err(e) => panic!("Could not open the file: {:?}", e),
    };

    let mut lines = BufReader::new(file).lines();

    let mut grid_dim = 8;
    match lines.next() {
        Some(Ok(val)) => { grid_dim = val.parse::<usize>().expect("Not a valid number"); },
        Some(Err(e)) => panic!("There was a problem with a line: {:?}", e),
        None => panic!("File did not contain any lines!"),
    };

    println!("Grid dim was parsed as: {}", &grid_dim);

    let state_a = construct_game(&grid_dim);
    let state_b = construct_game(&grid_dim);
    return (state_a, state_b);
}

fn main() {
    println!("Welcome to the Game of Life!");

    let args = Args::parse();

    let (mut state_a, mut state_b) = setup_states_from_file(args.file);

    let iterations = 5;
    for i in 0..iterations {
        if i % 2 == 0 {
            state_b.update(&state_a);
        }
        else {
            state_a.update(&state_b);
        }
    }

}
