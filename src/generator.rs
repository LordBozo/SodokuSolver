use crate::grid::Grid;
use crate::solvers::{get_solvers, Solver};
use crate::{Position, COLS, ROWS};
use rand::seq::SliceRandom;
use rand::Rng;

struct SolveDifficulty {
    difficulty: usize,
    solver_counts: Vec<usize>,
}
impl SolveDifficulty {
    fn new(solver_counts: Vec<usize>) -> SolveDifficulty {
        let mut difficulty = 0;
        let mut scalar = 1;
        let exp = 5;
        //Each rule is 'exp' times harder than the previous
        for i in 0..solver_counts.len() {
            difficulty += solver_counts[i] * scalar;
            scalar *= exp;
        }
        SolveDifficulty {
            difficulty,
            solver_counts,
        }
    }
}

///
///
///
pub fn create_board() -> Grid {
    // First, fill in the board randomly until its complete
    let mut grid: Grid = Grid::new();
    fill_board(&mut grid);

    let solvers = get_solvers("");
    prune_hard(&mut grid, solvers);
    let mut new_grid = grid.copy_grid(true, false);
    for i in 0..81 {
        new_grid.cells[i].is_given = new_grid.cells[i].value != 0;
    }
    new_grid
}
// Fill the board completely, to ensure our board has a solved state
fn fill_board(grid: &mut Grid) {
    grid.auto_promote = false;
    let mut replacement = (1..=9).collect::<Vec<u8>>();
    replacement.shuffle(&mut rand::rng());
    for i in 0..9 {
        for j in 0..9 {
            let value = (j + i * 3 + i / 3) % 9;
            grid.set_cell(Position::new(i, j), replacement[value]);
        }
    }
    // 123;132;213;231;312;321
    //    ; --;-- ;-- ;- -;- -
    //              --; --
    for group in [ROWS, COLS] {
        for i in 0..3 {
            let swap = rand::rng().random_range(0..=5);
            match swap {
                1 => swap_group(grid, group, i * 3 + 0, i * 3 + 1),
                2 => swap_group(grid, group, i * 3 + 0, i * 3 + 2),
                3 => swap_group(grid, group, i * 3 + 1, i * 3 + 2),
                4 => {
                    swap_group(grid, group, i * 3 + 0, i * 3 + 1);
                    swap_group(grid, group, i * 3 + 1, i * 3 + 2);
                }
                5 => {
                    swap_group(grid, group, i * 3 + 0, i * 3 + 2);
                    swap_group(grid, group, i * 3 + 1, i * 3 + 2);
                }
                _ => {}
            }
        }
    }
    for row in 0..9 {
        for col in 0..9 {
            let cell = grid.get_mut_cell_unchecked(Position { row, col });
            cell.answer = Some(cell.value);
        }
    }
    /*
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
    }*/
}
fn swap_group(
    grid: &mut Grid,
    group: &[[usize; 9]; 9],
    group_index_1: usize,
    group_index_2: usize,
) {
    for i in 0..9 {
        let was = grid.cells[group[group_index_1][i]].value;
        grid.cells[group[group_index_1][i]].value = grid.cells[group[group_index_2][i]].value;
        grid.cells[group[group_index_2][i]].value = was;
    }
}
// Randomly tries removing cells, and then checking to make sure the board is still solvable,
// until no more cells can be removed
#[allow(unused)]
fn prune_board(grid: &mut Grid, solvers: Vec<&Solver>) {
    let mut set_cells = (0..81).collect::<Vec<usize>>();
    set_cells.shuffle(&mut rand::rng());
    grid.auto_promote = false;
    while set_cells.len() > 0 {
        let cell_index = set_cells.pop().unwrap();
        let pos = Position::from_index(cell_index);
        let old_value = grid.cells[cell_index].value;
        //grid.unset_cell(pos);
        let solve = try_solve(grid, &solvers, pos);
        if solve.is_none() {
            //println!("Failed to remove at: {},{}", pos.row, pos.col);
            grid.cells[cell_index].value = old_value;
        } else if solve.unwrap() != old_value {
            // This will only be hit if by removing this cell's value, the only value it found was
            // different from what it is now, which shouldn't be possible
            println!("{}", grid);
            println!(
                "{:?}: should be {}, found {}",
                pos,
                old_value,
                solve.unwrap()
            );
            panic!("PUZZLE BROKE WHILE DESTRUCTING")
        } else {
            grid.unset_cell(pos);
            //println!("Removed {} at: {},{}", old_value, pos.row, pos.col);
            //println!("{}", grid);
        }
    }
}
// Removes whichever cell will make the board the hardest, given the rules it is allowed to use
// Solvers array is treated as ordered from easiest to hardest
fn prune_hard(grid: &mut Grid, solvers: Vec<&Solver>) {
    let mut set_cells = (0..81).collect::<Vec<usize>>();
    grid.auto_promote = false;
    let mut rule_counts = vec![0; solvers.len()];
    while set_cells.len() > 0 {
        let mut best: Vec<usize> = Vec::new();
        let mut best_difficulty = 0usize;
        let mut to_be_removed_indices: Vec<usize> = Vec::new();
        for cell_index in set_cells.iter() {
            let result = ranked_solve_removal(grid, &solvers, Position::from_index(*cell_index));
            if result.is_none() {
                to_be_removed_indices.push(*cell_index);
                continue;
            }
            let difficulty = result.as_ref().unwrap().difficulty;
            rule_counts = result.unwrap().solver_counts;
            if difficulty > best_difficulty {
                best_difficulty = difficulty;
                best.clear();
                best.push(*cell_index);
            } else if difficulty == best_difficulty {
                best.push(*cell_index);
            }
        }
        if best.len() == 0 {
            break;
        }
        let remove_index = best[rand::rng().random_range(0..best.len())];
        grid.unset_cell(Position::from_index(remove_index));
        to_be_removed_indices.push(remove_index);
        set_cells.retain(|&x| !to_be_removed_indices.contains(&x));
    }
    println!("Rules Used:");
    for i in 0..rule_counts.len() {
        println!("\t{} {}", rule_counts[i], solvers[i].name);
    }
}
// Copies the board, and then solves the copy with the given position being unset
// if the board is solvable, returns the value at position
// only solves as far as necessary to recover the removed cell
fn try_solve(grid: &mut Grid, solvers: &Vec<&Solver>, pos: Position) -> Option<u8> {
    // Duplicate the grid, with the given Position being unset
    let index = pos.get_index();
    grid.cells[index].value = 0;
    let mut new_grid = grid.copy_grid(false, false);
    let answer = grid.cells[index].answer?;
    grid.cells[index].value = answer;

    // run solver until given position is found
    let mut dirty = true;
    while dirty {
        dirty = false;
        for step in solvers {
            let func = step.solve_function;
            dirty |= func(&mut new_grid);
            if dirty {
                break;
            }
        }
        if new_grid.cells[index].value != 0 {
            if new_grid.cells[index].value != answer {
                return None;
            }
            return Some(new_grid.cells[index].value);
        }
    }
    None
}
// Same as try_solve, except it returns a solve difficulty
fn ranked_solve_removal(
    grid: &mut Grid,
    solvers: &Vec<&Solver>,
    pos_to_remove: Position,
) -> Option<SolveDifficulty> {
    // Duplicate the grid, with the given Position being unset
    let index = pos_to_remove.get_index();
    grid.cells[index].value = 0;
    let mut new_grid = grid.copy_grid(false, false);
    let answer = grid.cells[index].answer.unwrap();
    grid.cells[index].value = answer;

    let mut rule_counts = vec![0usize; solvers.len()];
    // solve the entire puzzle, storing how many of each solver was used
    let mut dirty = true;
    while dirty {
        dirty = false;
        for (i, step) in solvers.iter().enumerate() {
            let func = step.solve_function;
            dirty |= func(&mut new_grid);
            if dirty {
                rule_counts[i] += 1;
                break;
            }
        }
    }
    if new_grid.cells[index].value != answer {
        return None;
    }

    Some(SolveDifficulty::new(rule_counts))
}
