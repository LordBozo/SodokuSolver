mod sodoku_output;
mod solvers;
mod tests;

use crate::tests::Test;
use colored::{Color, Colorize};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

#[derive(Copy, Clone, Debug)]
struct Position {
    row: usize,
    col: usize,
}
impl Position {
    const fn new(row: usize, col: usize) -> Position {
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
}

const fn group_fill() -> [[[usize; 9]; 9]; 3] {
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
static COLLECTIONS: [[[usize; 9]; 9]; 3] = group_fill();
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
    pub fn get_print_card(&self) -> String {
        if self.value == 0 {
            let mut base = format!("{:?}", self);
            base.insert(6, '\n');
            base.insert(3, '\n');
            base.replace('-', " ")
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
            NUMBERS[self.value as usize - 1].to_string()
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
    }
}

#[derive(Clone)]
struct Grid {
    cells: [Cell; 81],
    solution: Option<[[u8; 9]; 9]>,
    starting_cell_count: usize,
}
// region Getters
impl Grid {
    fn get_mut_cell(&mut self, location: Position) -> Option<&mut Cell> {
        if location.row > 8 || location.col > 8 {
            return None;
        }
        Some(&mut self.cells[location.row * 9 + location.col])
    }
    fn get_cell_unchecked(&self, location: Position) -> &Cell {
        &self.cells[location.row * 9 + location.col]
    }
    #[allow(dead_code)]
    fn get_mut_cell_unchecked(&mut self, location: Position) -> &mut Cell {
        &mut self.cells[location.row * 9 + location.col]
    }
    #[allow(dead_code)]
    fn get_percent(&self) -> f32 {
        let mut count_cells = 0;
        for cell in self.cells {
            if cell.value > 0 {
                count_cells += 1;
            }
        }
        let added = count_cells - self.starting_cell_count;
        let needed = 81 - self.starting_cell_count;
        (added as f32) / (needed as f32)
    }
}
// endregion Getters
// region Init
impl Grid {
    fn from_string(input: &str, answer: Option<[[u8; 9]; 9]>) -> Grid {
        let mut grid = Grid::new();
        let mut starting_cell_count = 0;
        for (row, line) in input.lines().enumerate() {
            for (col, cell_value) in line.chars().enumerate() {
                if cell_value == ' ' || cell_value == '0' {
                    continue;
                }
                starting_cell_count += 1;
                grid.set_cell(
                    Position { row, col },
                    cell_value.to_digit(10).unwrap() as u8,
                );
                grid.get_mut_cell_unchecked(Position { row, col }).is_given = true;
            }
        }
        if answer.is_some() {
            let answer = answer.unwrap();
            for row in 0..answer.len() {
                for col in 0..answer[row].len() {
                    grid.get_mut_cell_unchecked(Position { row, col }).answer =
                        Some(answer[row][col]);
                }
            }
        }
        grid.solution = answer;
        grid.starting_cell_count = starting_cell_count;
        grid
    }

    fn new() -> Grid {
        let cell = Cell {
            candidates: 0x1FF,
            value: 0,
            answer: None,
            is_given: false,
        };
        let cells = [cell; 81];

        Grid {
            cells,
            solution: None,
            starting_cell_count: 0,
        }
    }
}
// endregion Init
// region Filters
impl Grid {
    // Locked Candidates?
    // Might want to rewrite this to take in 2 "Groups" to compare against
    // Order would matter, so Region-Row is different from Row-Region (BlockWithinRow vs RowWithinBlock)
    fn solve_locked_candidates_old(&mut self) -> bool {
        let mut dirty = false;
        for i in 0..9 {
            dirty |= self.filter_region_by_lines(i);
        }
        dirty
    }
    fn filter_region_by_lines(&mut self, region_index: usize) -> bool {
        let mut dirty = false;
        for num in 1..=9 {
            let mut rows_found: HashSet<usize> = HashSet::new();
            let mut cols_found: HashSet<usize> = HashSet::new();
            let region = REGS[region_index];
            for index in region {
                if self.cells[index].contains_value(num) {
                    let pos = Position::from_index(index);
                    rows_found.insert(pos.row);
                    cols_found.insert(pos.col);
                }
            }
            if rows_found.len() == 1 {
                let row = rows_found.iter().last().unwrap();
                for i in 0..9 {
                    let region_found = Position::new(*row, i).region().0;
                    if region_found != region_index {
                        let cell = &mut self.cells[ROWS[*row][i]];
                        if cell.contains_value(num) {
                            dirty = true;
                            cell.remove_possibility(num);
                        }
                    }
                }
            }
            if cols_found.len() == 1 {
                let col = cols_found.iter().last().unwrap();
                for i in 0..9 {
                    let region_found = Position::new(i, *col).region().0;
                    if region_found != region_index {
                        let cell = &mut self.cells[COLS[*col][i]];
                        if cell.contains_value(num) {
                            dirty = true;
                            cell.remove_possibility(num);
                        }
                    }
                }
            }
        }
        dirty
    }
}
// endregion Filters

impl Grid {
    fn set_cell(&mut self, location: Position, value: u8) {
        let cell = self.get_mut_cell(location.clone());
        if cell.is_none() {
            return;
        }
        let cell = cell.unwrap();
        if cell.value != 0 {
            if cell.value != value {
                panic!("Overwriting existing cell!");
            }
            return;
        }
        if cell.candidates & (1 << (value - 1)) == 0 {
            // If this value isn't a possible value, panic
            panic!("INVALID SET CELL");
        }
        cell.set_value(value);
        self.remove_seen_candidates(location);
    }
    fn remove_seen_candidates(&mut self, location: Position) {
        let index = location.row * 9 + location.col;
        let value = self.cells[index].value;
        for cell_index in COLS[location.col] {
            if cell_index != index {
                self.cells[cell_index].remove_possibility(value);
                if self.cells[cell_index].promote_single_candidate() {
                    self.remove_seen_candidates(Position::from_index(cell_index));
                }
            }
        }
        for cell_index in ROWS[location.row] {
            if cell_index != index {
                self.cells[cell_index].remove_possibility(value);
                if self.cells[cell_index].promote_single_candidate() {
                    self.remove_seen_candidates(Position::from_index(cell_index));
                }
            }
        }
        let region = location.region();
        for cell_index in REGS[region.0] {
            if cell_index != index {
                self.cells[cell_index].remove_possibility(value);
                if self.cells[cell_index].promote_single_candidate() {
                    self.remove_seen_candidates(Position::from_index(cell_index));
                }
            }
        }
    }
}
// region Print
impl Grid {
    fn print_possibilities(&self) {
        println!(
            "{}-",
            "- 987654321 987654321 987654321  "
                .repeat(3)
                .color(Color::White)
        );
        for i in 0..9 {
            print!("| ");
            for j in 0..9 {
                print!(
                    "{:?} ",
                    self.get_cell_unchecked(Position { row: i, col: j })
                );
                if j % 3 == 2 {
                    print!(" | ");
                }
            }
            println!();
            if i % 3 == 2 {
                println!("{}", "-".repeat(100));
            }
        }
    }
    fn print_board(&self) {
        println!("{}", "-".repeat(25));
        for i in 0..9 {
            print!("| ");
            for j in 0..9 {
                print!(
                    "{:} ",
                    self.get_cell_unchecked(Position { row: i, col: j }).value
                );
                if j % 3 == 2 {
                    print!("| ");
                }
            }
            println!();
            if i % 3 == 2 {
                println!("{}", "-".repeat(25));
            }
        }
    }
}
impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut accumulate = "".to_string();
        accumulate += "";
        let mut lines: Vec<String> = Vec::with_capacity(40);
        lines.push("╔═══════════╦═══════════╦═══════════╗".to_string());
        for i in 0..9 {
            if i == 3 || i == 6 {
                lines.push("╠═══════════╬═══════════╬═══════════╣".to_string());
            }
            if i % 3 != 0 {
                lines.push("║┄┄┄ ┄┄┄ ┄┄┄║┄┄┄ ┄┄┄ ┄┄┄║┄┄┄ ┄┄┄ ┄┄┄║".to_string());
            }
            let mut cards: Vec<String> = Vec::with_capacity(9);
            for j in 0..9 {
                let cell = self.get_cell_unchecked(Position { row: i, col: j });
                cards.push(cell.get_print_card());
            }
            let card_rows = cards
                .iter()
                .map(|str| str.split('\n').collect::<Vec<&str>>())
                .collect::<Vec<Vec<&str>>>();
            let mut rows: Vec<String> = vec!["║".to_string(); 3];
            for (j, row) in card_rows.iter().enumerate() {
                rows[0] += row[0];
                rows[1] += row[1];
                rows[2] += row[2];
                if j % 3 == 2 {
                    rows[0] += "║";
                    rows[1] += "║";
                    rows[2] += "║";
                } else {
                    rows[0] += "┆";
                    rows[1] += "┆";
                    rows[2] += "┆";
                }
            }
            rows[0] = rows[0].trim_end().to_string();
            rows[1] = rows[1].trim_end().to_string();
            rows[2] = rows[2].trim_end().to_string();
            lines.push(rows[0].to_string());
            lines.push(rows[1].to_string());
            lines.push(rows[2].to_string());
        }
        lines.push("╚═══════════╩═══════════╩═══════════╝".to_string());
        for i in lines {
            accumulate += i.as_str();
            accumulate += "\n";
        }
        write!(f, "{:9}", accumulate)
    }
}
// endregion Print

fn run_test(test: Test) {
    let mut grid = Grid::from_string(test.board, Some(*test.answer));
    grid.solve();
    let percent = grid.get_percent();
    if percent < 1f32 {
        println!("Failed: {}", percent * 100f32);
        grid.print_board();
        grid.print_possibilities();
    } else {
        println!("Passed");
    }
    grid.solve_hidden_pair();
}
enum RunType {
    Timing,
    Testing,
    Solving,
    Display,
    NYTimes,
}
fn main() {
    let run_type = RunType::Solving;
    //let test = tests::rule_tests::HIDDEN_PAIR;
    let test = tests::hard_tests::TEST_6;
    //let test = tests::medium_tests::TEST_1;
    //let test = tests::easy_tests::TEST_2;
    match run_type {
        RunType::Timing => {
            let mut grid: Grid = Grid::new();
            const ITERATIONS: usize = 1000;
            let start_time = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                grid = Grid::from_string(test.board, Some(*test.answer));
                grid.solve();
            }
            println!("Solve Time: {:?}", start_time.elapsed() / ITERATIONS as u32);
        }
        RunType::Solving => {
            let mut grid = Grid::from_string(test.board, None);
            grid.solve();
            println!("{:?}", grid);
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
            grid.solve_async();
        }

        RunType::NYTimes => {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            let mut grid = Grid::from_string(test.board, None);
            grid.solve();
            let start_time = std::time::Instant::now();
            sodoku_output::send_input(grid);
            println!("Solve Time: {:?}", start_time.elapsed());
        }
    }
}
