use crate::cell::Cell;
use crate::grid::Grid;
use crate::solvers::Solver;
use crate::Position;

pub const HIDDEN_SINGLE: Solver = Solver::new(
    "Hidden Single",
    "H1",
    "Fills in a cell if that is the only place a digit can go in a Row/Column/Region",
    solve_hidden_single,
    step_hidden_single,
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
        let mut count_and_positions = [(0, 0); 10];
        for i in 0..group.len() {
            let possibilities = cells[group[i]].get_possibilities();
            for &possibility in possibilities.iter() {
                count_and_positions[possibility as usize].0 += 1;
                count_and_positions[possibility as usize].1 = i;
            }
        }
        for (i, (count, index)) in count_and_positions.iter().enumerate() {
            if *count == 1 {
                return Some((Position::from_index(group[*index]), i as u8));
            }
        }
    }
    None
}
#[allow(unused)]
fn solve_hidden_single_cell(grid: &Grid, pos: Position) -> Option<u8> {
    let cell_index = pos.get_index();
    let possibilities = grid.cells[pos.get_index()].get_possibilities();
    let groups = Grid::get_cell_groups(pos);
    for group in groups {
        let mut candidate_clone = possibilities.clone();
        for cell in group {
            if *cell == cell_index {
                continue;
            }
            let other_possibilities = grid.cells[*cell].get_possibilities();
            candidate_clone.retain(|x| !other_possibilities.contains(x));
        }
        // the candidate vec will only contain values that only show up once in the row
        // this should either be empty (rule cant discern), or have 1 entry (rule applied)
        if candidate_clone.len() == 1 {
            return Some(candidate_clone[0] as u8);
        }
    }
    None
}
