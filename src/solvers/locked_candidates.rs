use crate::grid::Grid;
use crate::solvers::Solver;
use crate::{Cell, GroupType, Position, COLS, REGS, ROWS};
use std::collections::HashSet;

pub const LOCKED_CANDIDATES: Solver = Solver::new(
    "Locked Candidates",
    "LC",
    "If the only place a value appears in one group, is contained within second group, remove all other occurrences of that value in the second group",
    solve_locked_candidates,
    step_locked_candidates,
);
pub fn step_locked_candidates(grid: &mut Grid) -> bool {
    let mut dirty = false;
    //self.print_board();
    //self.print_possibilities();
    dirty |= solved_locked_candidates_line_region(&mut grid.cells, ROWS);
    dirty |= solved_locked_candidates_line_region(&mut grid.cells, COLS);
    for i in 0..9 {
        dirty |= filter_region_by_lines(grid, i);
    }
    dirty
}
pub fn solve_locked_candidates(grid: &mut Grid) -> bool {
    let mut dirty = false;
    //self.print_board();
    //self.print_possibilities();
    dirty |= solved_locked_candidates_line_region(&mut grid.cells, ROWS);
    dirty |= solved_locked_candidates_line_region(&mut grid.cells, COLS);
    for i in 0..9 {
        dirty |= filter_region_by_lines(grid, i);
    }
    dirty
}
fn solved_locked_candidates_line_region(
    cells: &mut [Cell; 81],
    line_collection: &[[usize; 9]; 9],
) -> bool {
    let mut dirty = false;
    for group in line_collection {
        //gather each region
        let mut region_values: Vec<u16> = Vec::new();
        for i in 0..3 {
            //gather within a region
            let mut region_candidates = 0u16;
            for j in 0..3 {
                let cell = group[i * 3 + j];
                region_candidates |= cells[cell].candidates;
            }
            region_values.push(region_candidates);
        }
        let ab_overlap = region_values[0] & region_values[1];
        let ac_overlap = region_values[0] & region_values[2];
        let bc_overlap = region_values[1] & region_values[2];
        let a_unique = region_values[0] & !ab_overlap & !ac_overlap;
        let b_unique = region_values[1] & !ab_overlap & !bc_overlap;
        let c_unique = region_values[2] & !ac_overlap & !bc_overlap;
        let a_region = Position::from_index(group[0]).region().0;
        let b_region = Position::from_index(group[3]).region().0;
        let c_region = Position::from_index(group[6]).region().0;
        for i in REGS[a_region] {
            if group.contains(&i) {
                continue;
            }
            if cells[i].remove_possibilities(a_unique) {
                dirty = true;
                //println!("Removed!");
            }
        }
        for i in REGS[b_region] {
            if group.contains(&i) {
                continue;
            }
            if cells[i].remove_possibilities(b_unique) {
                dirty = true;
                //println!("Removed!");
            }
        }
        for i in REGS[c_region] {
            if group.contains(&i) {
                continue;
            }
            if cells[i].remove_possibilities(c_unique) {
                dirty = true;
                //println!("Removed!");
            }
        }
    }
    dirty
}
fn filter_region_by_lines(grid: &mut Grid, region_index: usize) -> bool {
    let mut dirty = false;
    for num in 1..=9 {
        let mut rows_found: HashSet<usize> = HashSet::new();
        let mut cols_found: HashSet<usize> = HashSet::new();
        let region = REGS[region_index];
        for index in region {
            if grid.cells[index].contains_value(num) {
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
                    let cell =
                        &mut grid.cells[grid.unsolved_groups[GroupType::Rows as usize][*row][i]];
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
                    let cell = &mut grid.cells[COLS[*col][i]];
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
