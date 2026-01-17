use crate::grid::Grid;
use crate::solvers::{solve, Solver};
use crate::Position;
use rand::Rng;

///
///
///
pub fn create_board() -> Grid {
    // First, fill in the board randomly until its complete
    let mut grid: Grid = Grid::new();
    fill_board(&mut grid);
    //let solvers = get_solvers("");
    //prune_board(&mut grid, solvers);
    grid
}
// Fill the board completely, to ensure our board has a solved state
fn fill_board(grid: &mut Grid) {
    let mut unset_cells = (0..81).collect::<Vec<usize>>();

    while unset_cells.len() > 0 {
        let index = rand::rng().random_range(0..unset_cells.len());
        let cell_index = unset_cells.swap_remove(index);
        let cell = &mut grid.cells[cell_index];
        if cell.value != 0 {
            continue;
        }
        let possibilities = grid.cells[cell_index].get_possibilities();
        assert!(
            !possibilities.is_empty(),
            "Failed to fill board, cell has no candidates"
        );
        let value_index = rand::rng().random_range(0..possibilities.len());
        grid.set_cell(
            Position::from_index(cell_index),
            possibilities[value_index] as u8,
        );
        solve(grid);
    }
}
fn prune_board(grid: &mut Grid, solvers: Vec<&Solver>) {
    let mut set_cells = (0..81).collect::<Vec<usize>>();
    grid.auto_promote = false;
    while set_cells.len() > 0 {
        let index = rand::rng().random_range(0..set_cells.len());
        let cell_index = set_cells.swap_remove(index);
    }
}
fn try_solve(grid: &mut Grid, solvers: &Vec<&Solver>) {}
