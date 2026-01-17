use crate::grid::Grid;
use crate::solvers::Solver;
use crate::{Cell, Position};

pub const HIDDEN_SINGLE: Solver = Solver::new(
    "Hidden Single",
    "H1",
    "Fills in a cell if that is the only place a digit can go in a Row/Column/Region",
    solve_hidden_single,
    step_hidden_single,
    solve_hidden_single_cell,
);

pub fn step_hidden_single(grid: &mut Grid) -> bool {
    let mut result = None;
    for group in &grid.unsolved_groups {
        result = solve_hidden_single_collection(&mut grid.cells, group);
        if result.is_some() {
            break;
        }
    }
    if result.is_some() {
        let (pos, val) = result.unwrap();
        grid.set_cell(pos, val);
        return true;
    }
    false
}

pub fn solve_hidden_single(grid: &mut Grid) -> bool {
    let mut dirty = false;
    let mut results = Vec::new();
    for group in &grid.unsolved_groups {
        let result = solve_hidden_single_collection(&mut grid.cells, group);
        if result.is_some() {
            dirty = true;
            let result = result.unwrap();
            results.push(result);
        }
    }
    for result in results {
        grid.set_cell(result.0, result.1);
    }
    dirty
}
fn solve_hidden_single_collection(
    cells: &mut [Cell; 81],
    collection: &Vec<Vec<usize>>,
) -> Option<(Position, u8)> {
    for group in collection {
        // TODO: rather than using a counts arr, use a set, so you can get length to check if its only 1
        //  and if it is only 1, dont need to re-iterate to find where that 1 is
        let mut counts = [0; 10];
        for i in 0..group.len() {
            let possibilities = cells[group[i]].get_possibilities();
            for &possibility in possibilities.iter() {
                counts[possibility as usize] += 1;
            }
        }
        for i in 0..group.len() {
            let cell = group[i];
            let possibilities = cells[cell].get_possibilities();
            for possibility in possibilities {
                if counts[possibility as usize] == 1 {
                    //grid.set_cell(Position::from_index(cell), possibility as u8);
                    return Some((Position::from_index(cell), possibility as u8));
                }
            }
        }
    }
    None
}
fn solve_hidden_single_cell(grid: &mut Grid, pos: Position) -> bool {
    false
}
