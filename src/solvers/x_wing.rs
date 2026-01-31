use crate::cell::Cell;
use crate::grid::Grid;
use crate::solvers::Solver;
use crate::{COLS, ROWS};

pub const X_WING: Solver = Solver::new(
    "X Wing",
    "XW",
    "If a value occurs only twice in a line, and occurs in the same index in a second line, then it can't be anywhere else in that index",
    solve_x_wing,
    step_x_wing,
);
pub fn step_x_wing(grid: &mut Grid) -> bool {
    let mut dirty = false;
    if x_wing_group(&mut grid.cells, ROWS) {
        return true;
    }
    if x_wing_group(&mut grid.cells, COLS) {
        return true;
    }
    dirty
}
pub fn solve_x_wing(grid: &mut Grid) -> bool {
    let mut dirty = false;
    x_wing_group(&mut grid.cells, ROWS);
    x_wing_group(&mut grid.cells, COLS);
    dirty
}
fn x_wing_group(cells: &mut [Cell; 81], line_collection: &[[usize; 9]; 9]) -> bool {
    let mut dirty = true;

    // Iterate over outer index
    for r in 0..8 {
        // Iterate over numbers
        for n in 1..=9 {
            let found_indices = find_occurences(cells, &line_collection[r], n);
            if found_indices.len() != 2 {
                continue;
            }

            // Iterate over outer index for match
            for r_2 in r + 1..9 {
                let found_2 = find_occurences(cells, &line_collection[r_2], n);
                if found_2.len() != 2 {
                    continue;
                }
                // if match found, use X-Wing to remove candidates
                if found_indices[0] == found_2[0] && found_indices[1] == found_2[1] {
                    let ci1 = found_indices[0];
                    let ci2 = found_indices[1];
                    // We have an X-Wing in rows r, and r_2, and columns f[0] and f[1]
                    for i in 0..9 {
                        if i == r || i == r_2 {
                            continue;
                        }
                        dirty |= cells[line_collection[i][ci1]].remove_possibility(n);
                        dirty |= cells[line_collection[i][ci2]].remove_possibility(n);
                    }
                }
            }
        }
    }

    dirty
}
fn find_occurences(cells: &mut [Cell; 81], line_collection: &[usize; 9], value: u8) -> Vec<usize> {
    let mut found_indices = Vec::new();
    for c in 0..9 {
        if cells[line_collection[c]].contains_value(value) {
            found_indices.push(c);
        }
    }
    found_indices
}
