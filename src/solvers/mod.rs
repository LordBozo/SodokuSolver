mod hidden_pair;
mod hidden_single;
mod locked_candidates;
mod naked_pair;
mod naked_single;
mod x_wing;

use crate::grid::Grid;
use crate::solvers::hidden_pair::HIDDEN_PAIR;
use crate::solvers::hidden_single::HIDDEN_SINGLE;
use crate::solvers::locked_candidates::LOCKED_CANDIDATES;
use crate::solvers::naked_pair::{NAKED_PAIR, NAKED_QUAD, NAKED_TRIPLET};
use crate::solvers::naked_single::NAKED_SINGLE;
use crate::solvers::x_wing::X_WING;
use crate::{parse_yes_no, query_args_or_user, CommandArgs};
use clearscreen::clear;
use crossterm::{cursor, style, terminal, QueueableCommand};
use std::io;
use std::io::{stdin, Stdout, Write};

pub const SOLVERS: [&Solver; 8] = [
    &NAKED_SINGLE,
    &HIDDEN_SINGLE,
    &NAKED_PAIR,
    &HIDDEN_PAIR,
    &NAKED_TRIPLET,
    &NAKED_QUAD,
    &LOCKED_CANDIDATES,
    &X_WING,
];
pub fn get_solvers(filter: &str) -> Vec<&'static Solver> {
    /* Rule Codes:
     N1: Naked Single
     H1: Hidden Single
     N2: Naked Pair
     H2: Hidden Pair
     LC: Locked Candidates
    */
    if filter.len() == 0 {
        return SOLVERS.to_vec();
    }
    let mut solvers = Vec::new();
    for solver in SOLVERS {
        if filter.contains(solver.abbreviation) {
            solvers.push(solver);
        }
    }
    solvers
}
#[allow(unused)]
pub fn solve_subset(grid: &mut Grid, solvers: &Vec<&Solver>) {
    let mut dirty = true;
    while dirty {
        dirty = false;
        for step in solvers {
            let func = step.solve_function;
            dirty |= func(grid);
            if dirty {
                break;
            }
        }
    }
}
pub fn solve(grid: &mut Grid, _arguments: &CommandArgs) {
    let mut dirty = true;
    while dirty {
        dirty = false;

        for step in SOLVERS {
            let func = step.solve_function;
            dirty |= func(grid);
            if dirty {
                break;
            }
        }
    }
}
pub fn solve_async(grid: &mut Grid, arguments: &CommandArgs) {
    let (_, should_auto_advance) = query_args_or_user(
        "Auto Advance? Yes/No",
        "Invalid input",
        "-a",
        arguments,
        |x| parse_yes_no(x),
    );

    let mut stdout = io::stdout();
    let mut dirty = true;
    clear().expect("");
    grid.auto_promote = false;
    stdout.queue(cursor::DisableBlinking).unwrap();
    while dirty {
        dirty = false;
        grid.clear_dirty();

        for step in SOLVERS {
            let func = step.step_function;
            dirty |= func(grid);
            if dirty {
                print_and_flush_grid_changes(&mut stdout, grid, Some(step));
                if should_auto_advance {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                } else {
                    stdin()
                        .read_line(&mut Default::default())
                        .expect("Failed to read line");
                }
                break;
            }
        }
    }
}
pub fn print_and_flush_grid_changes(stdout: &mut Stdout, grid: &mut Grid, step: Option<&Solver>) {
    print!("{}", cursor::MoveTo(0, 0));
    let board = format!("{}\n", grid);
    stdout.queue(style::Print(board)).unwrap();
    stdout
        .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
        .unwrap();
    if step.is_some() {
        let step = step.unwrap();
        let step = format!("{}: {}", step.name, step.description);
        stdout.queue(style::Print(step)).unwrap();
    }
    stdout.flush().unwrap();
}
pub struct Solver {
    pub name: &'static str,
    pub description: &'static str,
    pub abbreviation: &'static str,
    pub solve_function: fn(&mut Grid) -> bool,
    pub step_function: fn(&mut Grid) -> bool,
}
impl Solver {
    const fn new(
        name: &'static str,
        abbreviation: &'static str,
        description: &'static str,
        solve_function: fn(&mut Grid) -> bool,
        step_function: fn(&mut Grid) -> bool,
    ) -> Solver {
        Solver {
            name,
            description,
            abbreviation,
            solve_function,
            step_function,
        }
    }
}
