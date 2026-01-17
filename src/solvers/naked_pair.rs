use crate::grid::Grid;
use crate::solvers::Solver;
use crate::{Cell, Position};

pub const NAKED_PAIR: Solver = Solver::new(
    "Naked Pair",
    "N2",
    "Two cells in a group have the same pair of numbers, so those numbers were removed from everywhere else in the group",
    solve_naked_pair,
    step_naked_pair,
    solve_naked_pair_cell,
);
pub fn step_naked_pair(grid: &mut Grid) -> bool {
    for collection in &grid.unsolved_groups {
        if solve_naked_pair_collection(&mut grid.cells, collection) {
            return true;
        }
    }
    false
}
pub fn solve_naked_pair(grid: &mut Grid) -> bool {
    let mut dirty = false;
    for collection in &grid.unsolved_groups {
        dirty |= solve_naked_pair_collection(&mut grid.cells, collection);
    }
    dirty
}
fn solve_naked_pair_collection(cells: &mut [Cell; 81], collection: &Vec<Vec<usize>>) -> bool {
    let mut dirty = false;
    for i in 0..collection.len() {
        let nine_cell = &collection[i];
        let mut matched = 0u16;
        'search: for j in 0..nine_cell.len() - 1 {
            let cell_index = nine_cell[j];
            let cell = cells[cell_index];
            if cell.candidates.count_ones() == 2 {
                for k in (j + 1)..nine_cell.len() {
                    let cell_2 = cells[nine_cell[k]];
                    if cell.candidates == cell_2.candidates {
                        matched = cell.candidates;
                        break 'search;
                    }
                }
            }
        }
        if matched != 0 {
            for j in 0..nine_cell.len() {
                let cell = &mut cells[nine_cell[j]];
                if cell.candidates != matched {
                    dirty |= cell.remove_possibilities(matched);
                }
            }
        }
    }
    dirty
}

fn solve_naked_pair_cell(grid: &mut Grid, pos: Position) -> bool {
    false
}
