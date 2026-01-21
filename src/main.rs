mod cell;
mod generator;
mod grid;
mod sodoku_output;
mod solvers;
mod tests;

use crate::grid::Grid;
use crate::tests::Test;
use clearscreen::clear;
use std::env::args;
use std::fmt::Debug;
use std::io::stdin;

#[allow(dead_code)]
enum GroupType {
    Rows,
    Columns,
    Regions,
}
#[derive(Copy, Clone, Debug)]
struct Position {
    row: usize,
    col: usize,
}
impl Position {
    const fn new(row: usize, col: usize) -> Position {
        if row > 8 || col > 8 {
            panic!("Position out of bounds");
        }
        Position { row, col }
    }
    const fn region(&self) -> (usize, usize) {
        (
            self.row / 3 * 3 + self.col / 3,
            (self.row % 3) * 3 + self.col % 3,
        )
    }
    const fn from_index(index: usize) -> Position {
        Position {
            col: index % 9,
            row: index / 9,
        }
    }
    const fn get_index(&self) -> usize {
        self.row * 9 + self.col
    }
}

const fn generate_groups() -> [[[usize; 9]; 9]; 3] {
    let mut rows: [[usize; 9]; 9] = [[0; 9]; 9];
    let mut cols: [[usize; 9]; 9] = [[0; 9]; 9];
    let mut regions: [[usize; 9]; 9] = [[0; 9]; 9];
    let mut i = 0;
    loop {
        if i == 9 {
            break;
        }
        let mut j = 0;
        loop {
            if j == 9 {
                break;
            }
            rows[i][j] = i * 9 + j;
            cols[i][j] = j * 9 + i;
            let (reg_x, reg_y) = Position { row: i, col: j }.region();
            regions[reg_x][reg_y] = i * 9 + j;
            j += 1;
        }
        i += 1;
    }
    [rows, cols, regions]
}
static COLLECTIONS: [[[usize; 9]; 9]; 3] = generate_groups();
static ROWS: &[[usize; 9]; 9] = &COLLECTIONS[0];
static COLS: &[[usize; 9]; 9] = &COLLECTIONS[1];
static REGS: &[[usize; 9]; 9] = &COLLECTIONS[2];

fn run_test(test: Test) {
    let mut grid = Grid::from_string(test.board, Some(*test.answer)).unwrap();
    solvers::solve(&mut grid);
    let percent = grid.get_percent();
    if percent < 1f32 {
        println!("Failed: {}%", percent * 100f32);
        grid.print_board();
        grid.print_possibilities();
    } else {
        println!("Passed");
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum RunType {
    Timing,
    Testing,
    Solving,
    Display,
    NYTimes,
    Generate,
}
impl RunType {
    const ITERATOR: [Self; 3] = [Self::Solving, Self::Generate, Self::Testing];
    fn parse(input: &str) -> Option<RunType> {
        let input_lower = input.to_lowercase();
        let mut starts: Vec<RunType> = Vec::new();
        let mut overlaps: Vec<RunType> = Vec::new();
        for run_type in RunType::ITERATOR {
            let run_string = format!("{:?}", run_type).to_lowercase();
            if input_lower.eq(&run_string) {
                return Some(run_type);
            }
            if run_string.starts_with(input_lower.as_str()) {
                starts.push(run_type.clone());
            }
            if run_string.contains(input_lower.as_str()) {
                overlaps.push(run_type);
            }
        }
        if starts.len() == 1 {
            return Some(starts[0].clone());
        } else if starts.len() == 0 && overlaps.len() == 1 {
            return Some(overlaps[0].clone());
        }
        None
    }
}
fn input_sodoku_board() -> Grid {
    loop {
        clear().expect("Failed to clear screen");
        println!("Please enter your board");
        println!("Use 1-9 for known digits, 0 or ' ' can be used for unknown cells");
        println!("You can use '|' to help space out digits, though they are not necessary");
        let mut board;
        loop {
            board = "".to_string();
            for i in 0..9 {
                let mut new_line: String = String::new();
                stdin()
                    .read_line(&mut new_line)
                    .expect("Failed to read line");

                new_line.retain(|c| c != '|');
                board += new_line.as_str();
                if i == 2 || i == 5 {
                    println!("-----------")
                }
            }
            break;
        }
        let grid = Grid::from_string(board.as_str(), None);
        if grid.is_some() {
            return grid.unwrap();
        }
        println!("Failed to parse board");
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
fn mode_solve() {
    let mut grid = input_sodoku_board();
    println!("Would you like to see it step by step? Yes/No");
    let is_async: bool = loop {
        let mut answer: String = String::new();
        stdin().read_line(&mut answer).expect("Failed to read line");
        let start = answer.chars().nth(0).unwrap();
        if start == 'n' || start == 'N' {
            break false;
        } else if start == 'y' || start == 'Y' {
            break true;
        }
    };
    if is_async {
        solvers::solve_async(&mut grid);
    } else {
        solvers::solve(&mut grid);
    }
}
fn main() {
    clear().expect("Failed to clear screen");
    let args = args();
    for arg in args.skip(1) {
        println!("{:?}", arg);
    }
    let run_type;
    loop {
        println!("Select Sodoku Mode: Solve, Generate, Testing");
        let mut mode: String = String::new();
        stdin().read_line(&mut mode).expect("Failed to read line");
        let mode = RunType::parse(mode.trim());
        if mode.is_some() {
            run_type = mode.unwrap();
            break;
        }
    }

    let test = tests::hard_tests::TEST_7;
    match run_type {
        RunType::Timing => {
            let mut grid: Grid;
            const ITERATIONS: usize = 10000;
            let start_time = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                grid = Grid::from_string(test.board, Some(*test.answer)).unwrap();
                solvers::solve(&mut grid);
            }
            println!("Solve Time: {:?}", start_time.elapsed() / ITERATIONS as u32);
        }
        RunType::Solving => {
            mode_solve();
        }
        RunType::Testing => {
            println!("Completed Tests:");
            for i in tests::all_tests::ALL_SOLVED_TESTS {
                run_test(i);
            }
            println!("Uncompleted Tests:");
            for i in tests::all_tests::ALL_UNSOLVED_TESTS {
                run_test(i);
            }
        }

        RunType::Display => {
            let mut grid = Grid::from_string(test.board, None).unwrap();
            solvers::solve_async(&mut grid);
        }

        RunType::NYTimes => {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            let mut grid = Grid::from_string(test.board, None).unwrap();
            solvers::solve(&mut grid);
            let start_time = std::time::Instant::now();
            sodoku_output::send_input(grid);
            println!("Solve Time: {:?}", start_time.elapsed());
        }
        RunType::Generate => {
            let start_time = std::time::Instant::now();
            let grid = generator::create_board();
            println!("Create Time: {:?}", start_time.elapsed());

            println!("{}", grid);
            //solvers::solve_async(&mut grid);
        }
    }
}
