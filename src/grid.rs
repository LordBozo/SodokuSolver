use crate::cell::Cell;
use crate::{Position, COLS, REGS, ROWS};
use colored::{Color, Colorize};
use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Formatter;
#[allow(unused)]
#[derive(PartialEq)]
enum BoardState {
    Invalid,
    Constructing,
    Solving,
    Solved,
}

pub struct Grid {
    pub cells: [Cell; 81],
    pub starting_cell_count: usize,
    pub unsolved_groups: [Vec<Vec<usize>>; 3],
    pub auto_promote: bool,
    current_state: BoardState,
}
impl Clone for Grid {
    fn clone(&self) -> Grid {
        let mut new_grid = Self::new();
        for r in 0..9 {
            for c in 0..9 {
                let pos = Position::new(r, c);
                let value = self.cells[pos.get_index()].value;
                if value > 0 {
                    new_grid.set_cell(pos, value);
                }
            }
        }
        new_grid
    }
}

// region Getters
#[allow(unused)]
impl Grid {
    pub fn get_cell(&self, pos: Position) -> Option<&Cell> {
        if pos.row > 8 || pos.col > 8 {
            return None;
        }
        Some(&self.cells[pos.get_index()])
    }
    pub fn get_mut_cell(&mut self, pos: Position) -> Option<&mut Cell> {
        if pos.row > 8 || pos.col > 8 {
            return None;
        }
        Some(&mut self.cells[pos.get_index()])
    }
    pub fn get_cell_unchecked(&self, pos: Position) -> &Cell {
        &self.cells[pos.get_index()]
    }

    #[allow(dead_code)]
    pub fn get_mut_cell_unchecked(&mut self, pos: Position) -> &mut Cell {
        &mut self.cells[pos.get_index()]
    }
    #[allow(dead_code)]
    pub fn get_percent(&self) -> f32 {
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
    pub fn from_string(
        input: &str,
        answer: Option<[[u8; 9]; 9]>,
        auto_promote: bool,
    ) -> Option<Grid> {
        let mut grid = Grid::new();
        grid.auto_promote = auto_promote;
        grid.current_state = BoardState::Constructing;
        let mut starting_cell_count = 0;
        'rowloop: for (row, line) in input.lines().enumerate() {
            for (col, cell_value) in line.chars().enumerate() {
                if cell_value == ' ' || cell_value == '0' {
                    continue;
                }
                starting_cell_count += 1;
                let digit = cell_value.to_digit(10);
                if digit.is_none() {
                    grid.current_state = BoardState::Invalid;
                    break 'rowloop;
                }
                let digit = digit.unwrap();
                grid.set_cell(Position { row, col }, digit as u8);
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
        grid.starting_cell_count = starting_cell_count;
        if grid.current_state == BoardState::Invalid {
            return None;
        }
        grid.current_state = BoardState::Solving;
        Some(grid)
    }
    pub fn copy_grid(&self, copy_answer: bool, auto_promote: bool) -> Grid {
        let mut new_grid = Self::new();
        new_grid.auto_promote = auto_promote;
        for r in 0..9 {
            for c in 0..9 {
                let pos = Position::new(r, c);
                let index = pos.get_index();
                let value = self.cells[index].value;
                if value > 0 {
                    new_grid.set_cell(pos, value);
                }
                if copy_answer {
                    new_grid.cells[index].answer = self.cells[index].answer;
                }
            }
        }
        new_grid
    }

    pub fn new() -> Grid {
        let cell = Cell {
            candidates: 0x1FF,
            value: 0,
            answer: None,
            is_given: false,
            is_dirty: false,
        };
        let cells = [cell; 81];
        let rows = ROWS.clone().map(|x| x.to_vec()).to_vec();
        let cols = COLS.clone().map(|x| x.to_vec()).to_vec();
        let regs = REGS.clone().map(|x| x.to_vec()).to_vec();
        let unsolved_groups = [rows, cols, regs];
        Grid {
            cells,
            starting_cell_count: 0,
            unsolved_groups,
            auto_promote: true,
            current_state: BoardState::Constructing,
        }
    }
}
// endregion Init

impl Grid {
    pub fn is_done(&self) -> bool {
        for cell in self.cells {
            if cell.value == 0 {
                return false;
            }
        }
        true
    }
    pub fn set_cell(&mut self, pos: Position, value: u8) {
        let cell = &mut self.cells[pos.get_index()];
        if cell.value != 0 {
            if cell.value != value {
                if self.current_state == BoardState::Solving {
                    panic!("Overwriting existing cell!");
                } else if self.current_state == BoardState::Constructing {
                    self.current_state = BoardState::Invalid;
                }
            }
            return;
        }
        if cell.candidates & (1 << (value - 1)) == 0 {
            if self.current_state == BoardState::Solving {
                // If this value isn't a possible value, panic
                panic!("INVALID SET CELL");
            } else if self.current_state == BoardState::Constructing {
                self.current_state = BoardState::Invalid;
            }
        }
        cell.set_value(value);
        self.remove_seen_candidates(pos);
        //self.remove_unsolved_cell(pos.row * 9 + pos.col)
    }
    pub fn get_cell_groups(pos: Position) -> Vec<&'static [usize; 9]> {
        let region = Position::region(&pos).0;
        vec![&COLS[pos.col], &ROWS[pos.row], &REGS[region]]
    }
    pub fn unset_cell(&mut self, pos: Position) {
        let cell = self.get_mut_cell(pos);
        if cell.is_none() {
            return;
        }
        let cell = cell.unwrap();
        cell.value = 0;
        cell.candidates = 0b111_111_111;
        cell.is_dirty = true;
        self.force_update_candidates(pos);

        let groups = Self::get_cell_groups(pos);
        for group in groups {
            for cell in *group {
                self.force_update_candidates(Position::from_index(cell));
            }
        }
    }
    fn force_update_candidates(&mut self, pos: Position) {
        let cell_index = pos.get_index();
        let mut candidates = 0b111_111_111;
        let groups = Self::get_cell_groups(pos);
        for group in groups {
            for other_index in *group {
                if other_index == cell_index {
                    continue;
                }
                let cell = self.cells[other_index];
                if cell.value != 0 {
                    candidates &= !(1 << (cell.value - 1));
                }
            }
        }
        self.cells[cell_index].candidates = candidates;
    }
    #[allow(dead_code)]
    fn remove_unsolved_cell(&mut self, index: usize) {
        // Removes all occurrences of the specified index in Unsolved Groups
        self.unsolved_groups.iter_mut().for_each(|group| {
            group
                .iter_mut()
                .for_each(|cells| cells.retain(|x| *x != index))
        });
    }
    pub fn remove_seen_candidates(&mut self, pos: Position) {
        let index = pos.row * 9 + pos.col;
        let value = self.cells[index].value;
        self.remove_seen_candidate_group(&COLS[pos.col], index, value);
        self.remove_seen_candidate_group(&ROWS[pos.row], index, value);
        let region = pos.region();
        self.remove_seen_candidate_group(&REGS[region.0], index, value);
    }
    fn remove_seen_candidate_group(&mut self, group: &[usize; 9], index: usize, value: u8) {
        for other_index in *group {
            if other_index != index {
                self.cells[other_index].remove_possibility(value);
                if self.auto_promote {
                    if self.cells[other_index].promote_single_candidate() {
                        self.remove_seen_candidates(Position::from_index(other_index));
                    }
                }
            }
        }
    }
    pub fn clear_dirty(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.is_dirty = false;
        }
    }
}
// region Print
impl Grid {
    pub fn print_possibilities(&self) {
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
    pub fn print_board(&self) {
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
impl fmt::Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut accumulate = "".to_string();
        accumulate += "";
        let mut lines: Vec<String> = Vec::with_capacity(40);
        lines.push("╔═══════════╦═══════════╦═══════════╗".normal().to_string());
        for i in 0..9 {
            if i == 3 || i == 6 {
                lines.push("╠═══════════╬═══════════╬═══════════╣".normal().to_string());
            }
            if i % 3 != 0 {
                lines.push("║┄┄┄ ┄┄┄ ┄┄┄║┄┄┄ ┄┄┄ ┄┄┄║┄┄┄ ┄┄┄ ┄┄┄║".normal().to_string());
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
            let mut rows: Vec<String> = vec!["║".normal().to_string(); 3];
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
        accumulate = accumulate.trim_end().to_string();
        write!(f, "{:9}", accumulate)
    }
}
// endregion Print
