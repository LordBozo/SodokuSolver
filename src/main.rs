mod cell;
mod generator;
mod grid;
mod sodoku_output;
mod solvers;
mod tests;

use crate::grid::Grid;
use crate::solvers::{get_solvers, Solver, SOLVERS};
use crate::tests::Test;
use clearscreen::clear;
use std::collections::HashMap;
use std::env::args;
use std::fmt::Debug;
use std::io::stdin;
use std::ops::Add;

struct CommandArgs {
    arg_map: HashMap<String, String>,
}
impl CommandArgs {
    fn new() -> CommandArgs {
        let mut arg_map: HashMap<String, String> = Default::default();
        for arg in args().skip(1) {
            let sides = arg.split_once("=");
            if sides.is_none() {
                arg_map.insert(arg.clone(), "".to_string());
            } else {
                let (key, value) = sides.unwrap();
                arg_map.insert(key.to_string(), value.to_string());
            }
        }
        CommandArgs { arg_map }
    }
    fn get_arg(&self, key: &str) -> Option<&String> {
        self.arg_map.get(key)
    }
    fn has_arg(&self, key: &str) -> bool {
        self.arg_map.contains_key(key)
    }
}
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
fn parse_yes_no(input: &str) -> Option<bool> {
    let mut start = input.chars().nth(0).unwrap();
    start = start.to_ascii_lowercase();
    if start == 'n' || start == 'f' {
        return Some(false);
    } else if start == 'y' || start == 't' {
        return Some(true);
    }
    None
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

fn run_test(test: Test, arguments: &CommandArgs) {
    let mut grid = Grid::from_string(test.board, Some(*test.answer)).unwrap();
    solvers::solve(&mut grid, &arguments);
    let percent = grid.get_percent();
    if percent < 1f32 {
        println!("Failed: {}%", percent * 100f32);
        grid.print_board();
        grid.print_possibilities();
    } else {
        println!("Passed");
    }
}
fn query_args_or_user<P, T>(
    prompt: &str,
    failure_message: &str,
    arg_flag: &str,
    arguments: &CommandArgs,
    mut validity_test: P,
) -> (String, T)
where
    P: FnMut(&str) -> Option<T>,
{
    let arg = arguments.get_arg(arg_flag);
    if arg.is_some() {
        let validity = validity_test(arg.unwrap());
        if validity.is_some() {
            return (arg.unwrap().to_string(), validity.unwrap());
        }
        println!("{}", failure_message);
    }
    loop {
        println!("{}", prompt);
        let mut result = String::new();
        stdin().read_line(&mut result).expect("Failed to read line");
        let validity = validity_test(&*result);
        if validity.is_some() {
            return (result, validity.unwrap());
        }
        println!("{}", failure_message);
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum RunType {
    Solve,
    Generate,
    Test,
    Time,
    Display,
    NYTimes,
}
impl RunType {
    const ITERATOR: [Self; 3] = [Self::Solve, Self::Generate, Self::Test];
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

fn print_help() {
    println!("-h: Prints this help section");
    println!(
        "-m: Selects what mode to run in, valid inputs are any abbreviated version of Solve, Generate, or Test"
    );
    println!(
        "-b: The board to use in Solve mode, spaces or 0s can be used for unknown cells, use \\n for line breaks, surround in quotes"
    );
    println!("-t: Choose whether or not to show how to solve a board in solve mode, yes/no ");
    println!(
        "-a: If using -t, this determines whether to auto-advance, or wait for using input, yes/no "
    );
}
fn input_sodoku_board(arguments: &CommandArgs) -> Grid {
    clear().expect("Failed to clear screen");
    let arg_board = arguments.get_arg("-b");
    if arg_board.is_some() {
        let arg_board = arg_board.unwrap().replace("\\n", "\n");
        let grid = Grid::from_string(arg_board.as_str(), None);
        if grid.is_some() {
            return grid.unwrap();
        }
        println!("Failed to parse passed in board");
    }
    loop {
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
        clear().expect("Failed to clear screen");
    }
}
fn select_mode(arguments: &CommandArgs) -> RunType {
    let (_, run_type) = query_args_or_user(
        "Select Sodoku Mode: Solve, Generate, Test",
        "Invalid Mode",
        "-m",
        arguments,
        |x| RunType::parse(x.trim()),
    );
    run_type
}
fn mode_solve(arguments: &CommandArgs) {
    let mut grid = input_sodoku_board(arguments);
    let (_, is_async) = query_args_or_user(
        "Would you like to see it step by step? Yes/No",
        "Invalid input",
        "-t",
        arguments,
        |x| parse_yes_no(x),
    );
    if is_async {
        solvers::solve_async(&mut grid, arguments);
    } else {
        solvers::solve(&mut grid, arguments);
    }
}
fn construct_codes() -> String {
    let mut string: String = Default::default();
    for solver in SOLVERS {
        if solver.abbreviation == "N1" {
            continue;
        }
        let format = format!("{}: {}\n", solver.abbreviation, solver.name);
        string = string.add(&format);
    }
    string.trim_end().to_string()
}
fn try_get_solvers(string: String) -> Option<Vec<&'static Solver>> {
    let input = if !string.is_empty() && !string.contains("N1") {
        String::from("N1").add(string.as_str())
    } else {
        string
    };
    let solvers = get_solvers(input.as_str());
    // Maybe add check later to ensure it has Naked Single?
    Some(solvers)
}
fn mode_generate(arguments: &CommandArgs) {
    let start_time = std::time::Instant::now();
    let codes = construct_codes();
    let prompt = format!(
        "Which Rules would you like to enable? Empty means all rules are allowed\n{}\nExample: N1H1N2",
        codes
    );
    let (_args, solvers) =
        query_args_or_user(prompt.as_str(), "Invalid input", "-g", arguments, |x| {
            try_get_solvers(x.to_string())
        });
    let grid = generator::create_board(solvers);
    println!("Create Time: {:?}", start_time.elapsed());

    println!("{}", grid);
}
fn main() {
    let arguments: CommandArgs = CommandArgs::new();
    clear().expect("Failed to clear screen");
    if arguments.has_arg("-h") || arguments.has_arg("-help") {
        print_help();
        return;
    }

    let run_type = select_mode(&arguments);
    let test = tests::hard_tests::TEST_7;
    match run_type {
        RunType::Solve => {
            mode_solve(&arguments);
        }
        RunType::Generate => {
            mode_generate(&arguments)
            //solvers::solve_async(&mut grid);
        }
        RunType::Test => {
            println!("Completed Tests:");
            for i in tests::all_tests::ALL_SOLVED_TESTS {
                run_test(i, &arguments);
            }
            println!("Uncompleted Tests:");
            for i in tests::all_tests::ALL_UNSOLVED_TESTS {
                run_test(i, &arguments);
            }
        }

        RunType::Display => {
            let mut grid = Grid::from_string(test.board, None).unwrap();
            solvers::solve_async(&mut grid, &arguments);
        }
        RunType::Time => {
            let mut grid: Grid;
            const ITERATIONS: usize = 10000;
            let start_time = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                grid = Grid::from_string(test.board, Some(*test.answer)).unwrap();
                solvers::solve(&mut grid, &arguments);
            }
            println!("Solve Time: {:?}", start_time.elapsed() / ITERATIONS as u32);
        }
        RunType::NYTimes => {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            let mut grid = Grid::from_string(test.board, None).unwrap();
            solvers::solve(&mut grid, &arguments);
            let start_time = std::time::Instant::now();
            sodoku_output::send_input(grid);
            println!("Solve Time: {:?}", start_time.elapsed());
        }
    }
}
