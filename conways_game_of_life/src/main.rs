use array2d::{Array2D};
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;
use num_cpus;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file containing the game's start state
    #[arg(short, long)]
    file: String,

    /// Number of iterations to run the game for 
    #[arg(short, long)]
    iterations: i8
}

#[derive(Clone, PartialEq, Copy)]
enum CellState {
    Dead,
    Live,
}

#[derive(Clone)]
struct GameState {
    state: Array2D<CellState>,
}

fn update_single_cell(mut_self: &Mutex<&mut GameState>, prev_state: &GameState, row_idx: usize, col_idx: usize) {
    // Determine the bounds of the subarray to consider
    let top_left_row_idx = if row_idx == 0 { 0 } else { row_idx - 1};
    let top_left_col_idx = if col_idx == 0 { 0 } else { col_idx - 1};
    let bottom_right_row_idx = if row_idx + 1 > prev_state.state.num_rows() { row_idx } else { row_idx + 1};
    let bottom_right_col_idx = if col_idx + 1 > prev_state.state.num_columns() { col_idx } else { col_idx + 1};
    let num_rows = bottom_right_row_idx - top_left_row_idx + 1;
    let num_cols = bottom_right_col_idx - top_left_col_idx + 1;
    
    // Get the subarray.
    let subarray = prev_state.state.rows_iter()
        .skip(top_left_row_idx)
        .take(num_rows)
        .map(|row| row.skip(top_left_col_idx).take(num_cols).collect::<Vec<_>>())
        .flatten()
        .collect::<Vec<_>>();

    // Compute the number of live cells
    let num_live_cells = subarray.iter().map(|x| if **x == CellState::Live {1} else {0}).fold(0, |acc, x| acc + x);
    
    // The subarray includes the current cell, so we may need to exclude from the live count.
    let mut locked_self = mut_self.lock().unwrap();
    let num_live_neighbours = match locked_self.state[(row_idx, col_idx)] {
        CellState::Live => num_live_cells - 1,
        CellState::Dead => num_live_cells,
    };

    // apply the game rules to this cell
    locked_self.state[(row_idx, col_idx)] = match num_live_neighbours {
        0 | 1 => CellState::Dead,
        2     => locked_self.state[(row_idx, col_idx)], // Live cells stay alive, dead cells stay dead
        3     => CellState::Live,   // live cells survive, and dead cells revive
        _     => CellState::Dead,
    };
}

impl GameState {
    fn clone(&self) -> Self {
        GameState {
            state: self.state.clone(),
        }
    }

    fn update(&mut self, prev_state: &GameState) {
        assert_eq!(self.state.num_rows(), prev_state.state.num_rows());
        
        let num_threads = num_cpus::get();
        let rows_per_thread = (self.state.num_rows() as f64 / num_threads as f64).ceil() as usize;
        let num_rows = self.state.num_rows();
        let num_columns = self.state.num_columns();

        let arc_self = Arc::new(Mutex::new(self));

        thread::scope(|s| {
            for thread_index in 0..num_threads {
                let start_row = thread_index * rows_per_thread;
                let end_row = (thread_index + 1) * rows_per_thread;
                let thread_self = Arc::clone(&arc_self);
                s.spawn(move || {
                    for i in start_row..end_row.min(num_rows) {
                        for j in 0..num_columns {
                            update_single_cell(&thread_self, &prev_state, i, j);
                        }
                    }
                });
            }
        });
    }

    fn print(&self) {
        let dim = self.state.num_rows();
        for i in 0..dim {
            let row_iter = match self.state.row_iter(i) {
                Ok(iter) => iter,
                Err(e) => panic!("Encountered error: {:?}", e),
            };
            let row_str : String = row_iter.map(|x| match x {
                CellState::Dead => 'o',
                CellState::Live => 'x',
            }).collect();
            println!("{}", row_str);
        }
    }
}

fn construct_game(dim: &usize) -> GameState {
    let game = GameState {
        state: Array2D::filled_with(CellState::Dead, *dim, *dim),
    };
    return game;
}

fn setup_states_from_file(filename: String) -> (GameState, GameState) {
    // Try to open the file
    let file = match File::open(filename) {
        Ok(obj) => obj,
        Err(e) => panic!("Could not open the file: {:?}", e),
    };

    let mut lines = BufReader::new(file).lines();

    // parse the grid dimension
    let grid_dim: usize;
    match lines.next() {
        Some(Ok(val)) => { grid_dim = val.parse::<usize>().expect("Not a valid number"); },
        Some(Err(e)) => panic!("There was a problem with a line: {:?}", e),
        None => panic!("File did not contain any lines!"),
    };

    // instantiate the game with the default values.
    let mut state_a = construct_game(&grid_dim);

    // fill the game state from the file's contents
    for (i, line) in lines.enumerate() {
        match line {
            Ok(line_val) => {
                // make sure the line has the correct length
                if line_val.len() != grid_dim {
                    panic!("Encountered a line with length != the stated grid dimension: {}", line_val);
                }

                for (j, c) in line_val.chars().enumerate() {
                    state_a.state[(i, j)] = match c {
                        'x' => CellState::Live,
                        'o' => CellState::Dead,
                        _ => panic!("Encountered an unknown symbol in a line. The only valid symbols are x and o for Live and Dead: {}", c),
                    }
                }
            },
            Err(_e) => {},
        };
    }

    let state_b = state_a.clone();
    return (state_a, state_b);
}

fn main() {
    println!("Welcome to the Game of Life!");

    let args = Args::parse();

    let (mut state_a, mut state_b) = setup_states_from_file(args.file);

    println!("Initial state was:");
    state_a.print();

    let iterations = args.iterations;
    for i in 0..iterations {
        if i % 2 == 0 {
            state_b.update(&state_a);
        }
        else {
            state_a.update(&state_b);
        }
    }

    println!("Final state after {} iterations was:", iterations);
    if iterations % 2 == 0 {
        state_a.print();
    }
    else {
        state_b.print();
    }
}
