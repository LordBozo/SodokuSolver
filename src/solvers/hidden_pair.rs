use crate::grid::Grid;
use crate::solvers::Solver;
use crate::{Cell, Position};

pub const HIDDEN_PAIR: Solver = Solver::new(
    "Hidden Pair",
    "H2",
    "Two values only showed up in the same 2 cells, removed all other possibilites from those cells",
    solve_hidden_pair,
    step_hidden_pair,
    solve_hidden_pair_cell,
);
pub fn step_hidden_pair(grid: &mut Grid) -> bool {
    for collection in &grid.unsolved_groups {
        if solve_hidden_pair_collection_set(&mut grid.cells, collection) {
            return true;
        }
    }
    false
}
pub fn solve_hidden_pair(grid: &mut Grid) -> bool {
    let mut dirty = false;
    for collection in &grid.unsolved_groups {
        dirty |= solve_hidden_pair_collection_set(&mut grid.cells, collection);
    }
    dirty
}
fn solve_hidden_pair_collection_set(cells: &mut [Cell; 81], collection: &Vec<Vec<usize>>) -> bool {
    let mut dirty = false;
    'groups: for group in collection {
        let mut counts: Vec<Vec<usize>> = vec![Vec::new(); 9];
        for index in 0..group.len() {
            let possibilities = cells[group[index]].get_possibilities();
            for possibility in possibilities {
                counts[possibility as usize - 1].push(index);
            }
        }
        for i in 0..group.len() - 1 {
            if counts[i].len() != 2 {
                continue;
            }
            for j in i + 1..group.len() {
                if counts[j].len() != 2 {
                    continue;
                }
                let new_candidates = (1 << i) | (1 << j);
                if counts[i].eq(&counts[j]) {
                    for &index in counts[i].iter() {
                        if cells[group[index]].candidates != new_candidates {
                            cells[group[index]].candidates = new_candidates;
                            dirty = true;
                        }
                    }
                    continue 'groups;
                }
            }
        }
    }
    dirty
}
fn solve_hidden_pair_cell(grid: &mut Grid, pos: Position) -> bool {
    false
}
