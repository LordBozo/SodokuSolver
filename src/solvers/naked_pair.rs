use crate::cell::Cell;
use crate::grid::Grid;
use crate::solvers::Solver;
use itertools::Itertools;

pub const NAKED_PAIR: Solver = Solver::new(
    "Naked Pair",
    "N2",
    "Two cells in a group have the same pair of numbers, so those numbers were removed from everywhere else in the group",
    solve_naked_pair,
    step_naked_pair,
);
pub const NAKED_TRIPLET: Solver = Solver::new(
    "Naked Triple",
    "N3",
    "Three cells in a group have the same pair of numbers, so those numbers were removed from everywhere else in the group",
    solve_naked_triple,
    step_naked_triple,
);
pub const NAKED_QUAD: Solver = Solver::new(
    "Naked Quad",
    "N4",
    "Four cells in a group have the same pair of numbers, so those numbers were removed from everywhere else in the group",
    solve_naked_quad,
    step_naked_quad,
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
pub fn step_naked_triple(grid: &mut Grid) -> bool {
    for collection in &grid.unsolved_groups {
        if step_naked_group_collection(&mut grid.cells, collection, 3) {
            return true;
        }
    }
    false
}
pub fn solve_naked_triple(grid: &mut Grid) -> bool {
    let mut dirty = false;
    for collection in &grid.unsolved_groups {
        dirty |= solve_naked_group_collection(&mut grid.cells, collection, 3);
    }
    dirty
}
pub fn step_naked_quad(grid: &mut Grid) -> bool {
    for collection in &grid.unsolved_groups {
        if step_naked_group_collection(&mut grid.cells, collection, 4) {
            return true;
        }
    }
    false
}
pub fn solve_naked_quad(grid: &mut Grid) -> bool {
    let mut dirty = false;
    for collection in &grid.unsolved_groups {
        dirty |= solve_naked_group_collection(&mut grid.cells, collection, 4);
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
fn solve_naked_group_collection(
    cells: &mut [Cell; 81],
    collection: &Vec<Vec<usize>>,
    group_size: usize,
) -> bool {
    let mut dirty = false;
    let combinations = (0..9).combinations(group_size);
    for i in 0..collection.len() {
        let mut group_found: Option<(Vec<usize>, u16)> = None;
        let mut candidates = [0; 9];
        for j in 0..collection[i].len() {
            candidates[j] = cells[collection[i][j]].candidates
        }
        'combinatorics: for combination in combinations.clone() {
            let mut found_candidates = 0;
            for &index in combination.iter() {
                if candidates[index] == 0 {
                    continue 'combinatorics;
                }
                found_candidates |= candidates[index];
            }
            if found_candidates.count_ones() == group_size as u32 {
                group_found = Some((combination.clone(), found_candidates));
                break 'combinatorics;
            }
        }
        if group_found.is_some() {
            let (cell_indices, candidates) = group_found.unwrap();
            for j in 0..collection[i].len() {
                if cell_indices.contains(&j) {
                    continue;
                }
                dirty |= cells[collection[i][j]].remove_possibilities(candidates);
            }
        }
    }
    dirty
}
fn step_naked_group_collection(
    cells: &mut [Cell; 81],
    collection: &Vec<Vec<usize>>,
    group_size: usize,
) -> bool {
    let mut dirty = false;
    let combinations = (0..9).combinations(group_size);
    for i in 0..collection.len() {
        let mut group_found: Option<(Vec<usize>, u16)> = None;
        let mut candidates = [0; 9];
        for j in 0..collection[i].len() {
            candidates[j] = cells[collection[i][j]].candidates
        }
        'combinatorics: for combination in combinations.clone() {
            let mut found_candidates = 0;
            for &index in combination.iter() {
                if candidates[index] == 0 {
                    continue 'combinatorics;
                }
                found_candidates |= candidates[index];
            }
            if found_candidates.count_ones() == group_size as u32 {
                group_found = Some((combination.clone(), found_candidates));
                break 'combinatorics;
            }
        }
        if group_found.is_some() {
            let (cell_indices, candidates) = group_found.unwrap();
            for j in 0..collection[i].len() {
                if cell_indices.contains(&j) {
                    continue;
                }
                dirty |= cells[collection[i][j]].remove_possibilities(candidates);
            }
            if dirty {
                return true;
            }
        }
    }
    dirty
}
