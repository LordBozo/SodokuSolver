mod generator;
mod grid;
mod sodoku_output;
mod solvers;
mod tests;
use crate::grid::Grid;
use crate::tests::Test;
use colored::Colorize;
use std::fmt;
use std::fmt::Formatter;

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
//static GROUPS: [&[[usize; 9]; 9]; 3] = [&ROWS, &COLS, &REGS];
#[derive(Copy, Clone)]
struct Cell {
    candidates: u16,
    value: u8,
    answer: Option<u8>,
    is_given: bool,
    is_dirty: bool,
}
impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut candidates = self.candidates;
        let mut val = 1i8;
        let mut accumulate = "".to_string();
        while candidates != 0 {
            accumulate += &*if (candidates & 1) == 1 {
                val.to_string()
            } else {
                "-".to_string()
            };
            candidates >>= 1;
            val += 1;
        }
        write!(f, "{:9}", accumulate)
    }
}
#[allow(dead_code)]
impl Cell {
    fn color_card(&self, card: String) -> String {
        let result: Vec<String>;
        if self.is_given {
            result = card
                .split("\n")
                .map(|x| x.green().to_string())
                .collect::<Vec<String>>();
        } else if self.is_dirty {
            result = card
                .split("\n")
                .map(|x| x.blue().to_string())
                .collect::<Vec<String>>();
        } else {
            return card;
        }
        return result.join("\n").to_string();
    }
    pub fn get_print_card(&self) -> String {
        if self.value == 0 {
            let mut base = format!("{:?}", self);
            base.insert(6, '\n');
            base.insert(3, '\n');
            base = base.replace('-', " ");
            base = self.color_card(base);
            base
        } else {
            const NUMBERS: [&str; 9] = [
                " ┓ \n ┃ \n ┻ ",
                "┏━┓\n┏━┛\n┗━━",
                "┏━┓\n ━┫\n┗━┛",
                "╻ ╻\n┗━╋\n  ╹",
                "┏━╸\n┗━┓\n┗━┛",
                "┏━┓\n┣━┓\n┗━┛",
                "╺━┓\n  ┃\n  ╹",
                "┏━┓\n┣━┫\n┗━┛",
                "┏━┓\n┗━┫\n┗━┛",
            ];
            let result = NUMBERS[self.value as usize - 1];
            self.color_card(result.to_string())
        }
    }
    pub fn contains_value(&self, value: u8) -> bool {
        if self.value <= 0 {
            return (self.candidates & (1 << value - 1)) > 0;
        }
        false
    }
    pub fn promote_single_candidate(&mut self) -> bool {
        let mut val = 0u8;
        let mut possibilities = self.candidates;
        let mut bits_set = 0;
        while possibilities != 0 {
            if (possibilities & 1) == 1 {
                bits_set += 1;
            }
            possibilities >>= 1;
            val += 1;
        }
        if bits_set == 1 {
            self.set_value(val);
            return true;
        }
        false
    }
    fn remove_possibilities(&mut self, bits: u16) -> bool {
        if bits & self.candidates != 0 {
            self.candidates &= !bits;
            self.is_answer_possible();
            self.is_dirty = true;
            return true;
        }
        false
    }
    fn remove_possibility(&mut self, value: u8) {
        self.candidates &= !(1 << (value - 1));
        self.is_answer_possible();
    }

    pub fn is_answer_possible(&self) {
        if self.answer.is_none() || self.value > 0 {
            return;
        }
        let ans = self.answer.unwrap();
        if !self.contains_value(ans) {
            panic!("REMOVED ANSWER AS POSSIBILITY")
        }
    }
    pub fn get_possibilities(&self) -> Vec<u16> {
        if self.value != 0 {
            return Vec::new();
        }
        let mut possibilities = self.candidates;
        let mut results = Vec::new();
        for i in 0..9 {
            if possibilities & 1 == 1 {
                results.push(i + 1);
            }
            possibilities >>= 1;
        }
        results
    }
    fn set_value(&mut self, value: u8) {
        if self.answer.is_some() {
            if value != self.answer.unwrap() {
                println!(
                    "INVALID ANSWER: should be {:?}, is {value}",
                    self.answer.unwrap()
                );
            }
        }
        self.value = value;
        self.candidates = 0;
        self.is_dirty = true;
    }
}

fn run_test(test: Test) {
    let mut grid = Grid::from_string(test.board, Some(*test.answer));
    solvers::solve(&mut grid);
    let percent = grid.get_percent();
    if percent < 1f32 {
        println!("Failed: {}", percent * 100f32);
        grid.print_board();
        grid.print_possibilities();
    } else {
        println!("Passed");
    }
    //grid.solve_hidden_pair();
}
#[allow(dead_code)]
enum RunType {
    Timing,
    Testing,
    Solving,
    Display,
    NYTimes,
    Generate,
}
fn main() {
    let run_type = RunType::Generate;
    //let test = tests::rule_tests::HIDDEN_PAIR;
    let test = tests::hard_tests::TEST_7;
    //let test = tests::medium_tests::TEST_1;
    //let test = tests::easy_tests::TEST_2;
    match run_type {
        RunType::Timing => {
            let mut grid: Grid;
            const ITERATIONS: usize = 10000;
            let start_time = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                grid = Grid::from_string(test.board, Some(*test.answer));
                solvers::solve(&mut grid);
            }
            println!("Solve Time: {:?}", start_time.elapsed() / ITERATIONS as u32);
        }
        RunType::Solving => {
            let mut grid = Grid::from_string(test.board, None);
            solvers::solve(&mut grid);
            println!("{}", grid);
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
            let mut grid = Grid::from_string(test.board, None);
            solvers::solve_async(&mut grid);
        }

        RunType::NYTimes => {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            let mut grid = Grid::from_string(test.board, None);
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
