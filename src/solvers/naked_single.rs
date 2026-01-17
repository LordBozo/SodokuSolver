use crate::grid::Grid;
use crate::solvers::Solver;
use crate::Position;

pub const NAKED_SINGLE: Solver = Solver::new(
    "Naked Single",
    "N1",
    "Fills in a cell that only has one possibility",
    solve_naked_single,
    step_naked_single,
    solve_naked_single_cell,
);
pub fn step_naked_single(grid: &mut Grid) -> bool {
    if grid.auto_promote {
        return false;
    }
    for index in 0..grid.cells.len() {
        let cell = &mut grid.cells[index];
        if cell.value > 0 {
            continue;
        }
        let result = cell.promote_single_candidate();
        if result {
            grid.remove_seen_candidates(Position::from_index(index));
            return true;
        }
    }
    false
}
pub fn solve_naked_single(grid: &mut Grid) -> bool {
    if grid.auto_promote {
        return false;
    }
    let mut dirty = false;
    for index in 0..grid.cells.len() {
        let cell = &mut grid.cells[index];
        if cell.value > 0 {
            continue;
        }
        let result = cell.promote_single_candidate();
        if result {
            grid.remove_seen_candidates(Position::from_index(index));
            dirty = true;
        }
    }
    dirty
}
fn solve_naked_single_cell(grid: &mut Grid, pos: Position) -> bool {
    if grid.auto_promote {
        return false;
    }
    let result = grid.get_mut_cell_unchecked(pos).promote_single_candidate();
    if result {
        grid.remove_seen_candidates(pos);
    }
    result
}
