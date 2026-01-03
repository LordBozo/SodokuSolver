use crate::{Cell, Grid, Position, COLLECTIONS, COLS, REGS, ROWS};
use clearscreen::clear;
/*
Terminology:
    Row: Horizontal line
    Column: Vertical line
    Region: 3x3 box
    Group: a single set, either Row, Column, or Region
    Collection: All of on Group type, e.g. all Rows
 */

impl Grid {
    /// TODO: Locked Candidates:
    ///     if the only places that a digit appears in one 'region', are also all in a different 'region',
    ///     then you can remove all other possibilities in the second region
    ///     IE, if the only place a 5 can show up in a row, are all in the same box,
    ///     then 5 can't show up anywhere else in the box
    pub fn solve(&mut self) {
        /// TODO: convert to struct containing func pointer and debug string for printing what the steps were
        const SOLVE_STEPS: [fn(&mut Grid) -> bool; 5] = [
            //Grid::solve_naked_single, // Removed because naked singles are handle automatically when removing a digit
            Grid::solve_hidden_single,
            Grid::solve_naked_pair,
            Grid::solve_hidden_pair,
            Grid::solve_locked_candidates,
            Grid::solve_locked_candidates_old,
        ];
        let mut dirty = true;
        while dirty {
            dirty = false;

            for step in SOLVE_STEPS {
                dirty |= step(self);
                if dirty {
                    break;
                }
            }
        }
    }
    pub fn solve_async(&mut self) {
        /// TODO: convert to struct containing func pointer and debug string for printing what the steps were

        const SOLVE_STEPS: [fn(&mut Grid) -> bool; 5] = [
            //Grid::solve_naked_single, // Removed because naked singles are handle automatically when removing a digit
            Grid::solve_hidden_single,
            Grid::solve_naked_pair,
            Grid::solve_hidden_pair,
            Grid::solve_locked_candidates,
            Grid::solve_locked_candidates_old,
        ];
        let mut dirty = true;
        while dirty {
            dirty = false;
            //print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            clear().expect("TODO: panic message");
            println!("{:?}", self);

            for step in SOLVE_STEPS {
                dirty |= step(self);
                if dirty {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    break;
                }
            }
        }
    }
}
//<editor-fold defaultstate="collapsed" desc="naked single">
impl Grid {
    /// If any cell has only one candidate, set it to that value
    #[allow(dead_code)]
    pub fn solve_naked_single(&mut self) -> bool {
        let mut dirty = false;
        for index in 0..self.cells.len() {
            let cell = &mut self.cells[index];
            if cell.value > 0 {
                continue;
            }
            let result = cell.promote_single_candidate();
            if result {
                self.remove_seen_candidates(Position::from_index(index));
                dirty = true;
            }
        }
        dirty
    }
}
//</editor-fold>

//<editor-fold defaultstate="collapsed" desc="hidden single">
impl Grid {
    fn solve_hidden_single(&mut self) -> bool {
        let mut dirty = false;
        for group in &COLLECTIONS {
            dirty |= self.solve_hidden_single_collection(group);
        }
        dirty
    }
    fn solve_hidden_single_collection(&mut self, collection: &[[usize; 9]; 9]) -> bool {
        for group in collection {
            // TODO: rather than using a counts arr, use a set, so you can get length to check if its only 1
            //  and if it is only 1, dont need to re-iterate to find where that 1 is
            let mut counts = [0; 10];
            for i in 0..9 {
                let possibilities = self.cells[group[i]].get_possibilities();
                for possibility in possibilities {
                    counts[possibility as usize] += 1;
                }
            }
            for i in 0..9 {
                let cell = group[i];
                let possibilities = self.cells[cell].get_possibilities();
                for possibility in possibilities {
                    if counts[possibility as usize] == 1 {
                        self.set_cell(Position::from_index(cell), possibility as u8);
                        return true;
                    }
                }
            }
        }
        false
    }
}
//</editor-fold>

//<editor-fold defaultstate="collapsed" desc="naked pair">
impl Grid {
    pub fn solve_naked_pair(&mut self) -> bool {
        let mut dirty = false;
        for collection in &COLLECTIONS {
            dirty |= Self::solve_naked_pair_collection(&mut self.cells, collection);
        }
        dirty
    }
    fn solve_naked_pair_collection(cells: &mut [Cell; 81], collection: &[[usize; 9]; 9]) -> bool {
        let mut dirty = false;
        for i in 0..9 {
            let nine_cell = collection[i];
            let mut matched = 0u16;
            'search: for j in 0..8 {
                let cell_index = nine_cell[j];
                let cell = cells[cell_index];
                if cell.candidates.count_ones() == 2 {
                    for k in (j + 1)..9 {
                        let cell_2 = cells[nine_cell[k]];
                        if cell.candidates == cell_2.candidates {
                            matched = cell.candidates;
                            break 'search;
                        }
                    }
                }
            }
            if matched != 0 {
                for j in 0..9 {
                    let cell = &mut cells[nine_cell[j]];
                    if cell.candidates != matched {
                        dirty |= cell.remove_possibilities(matched);
                    }
                }
            }
        }
        dirty
    }
}
//</editor-fold>

//<editor-fold defaultstate="collapsed" desc="hidden pair">
impl Grid {
    pub fn solve_hidden_pair(&mut self) -> bool {
        let mut dirty = false;
        for collection in &COLLECTIONS {
            dirty |= Self::solve_hidden_pair_collection_set(&mut self.cells, collection);
        }
        dirty
    }
    fn solve_hidden_pair_collection_set(
        cells: &mut [Cell; 81],
        collection: &[[usize; 9]; 9],
    ) -> bool {
        let mut dirty = false;
        'groups: for group in collection {
            let mut counts: Vec<Vec<usize>> = vec![Vec::new(); 9];
            for index in 0..9 {
                let possibilities = cells[group[index]].get_possibilities();
                for possibility in possibilities {
                    counts[possibility as usize - 1].push(index);
                }
            }
            for i in 0..8 {
                if counts[i].len() != 2 {
                    continue;
                }
                for j in i + 1..9 {
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
}
//</editor-fold>

//<editor-fold defaultstate="collapsed" desc="hidden pair">
impl Grid {
    pub fn solve_locked_candidates(&mut self) -> bool {
        let mut dirty = false;
        //self.print_board();
        //self.print_possibilities();
        dirty |= Self::solved_locked_candidates_line_region(&mut self.cells, ROWS);
        dirty |= Self::solved_locked_candidates_line_region(&mut self.cells, COLS);
        dirty
    }
    fn solved_locked_candidates_line_region(
        cells: &mut [Cell; 81],
        line_collection: &[[usize; 9]; 9],
    ) -> bool {
        let mut dirty = false;
        for group in line_collection {
            //gather each region
            let mut region_values: Vec<u16> = Vec::new();
            for i in 0..3 {
                //gather within a region
                let mut region_candidates = 0u16;
                for j in 0..3 {
                    let cell = group[i * 3 + j];
                    region_candidates |= cells[cell].candidates;
                }
                region_values.push(region_candidates);
            }
            let ab_overlap = region_values[0] & region_values[1];
            let ac_overlap = region_values[0] & region_values[2];
            let bc_overlap = region_values[1] & region_values[2];
            let a_unique = region_values[0] & !ab_overlap & !ac_overlap;
            let b_unique = region_values[1] & !ab_overlap & !bc_overlap;
            let c_unique = region_values[2] & !ac_overlap & !bc_overlap;
            let a_region = Position::from_index(group[0]).region().0;
            let b_region = Position::from_index(group[3]).region().0;
            let c_region = Position::from_index(group[6]).region().0;
            for i in REGS[a_region] {
                if group.contains(&i) {
                    continue;
                }
                if cells[i].remove_possibilities(a_unique) {
                    dirty = true;
                    //println!("Removed!");
                }
            }
            for i in REGS[b_region] {
                if group.contains(&i) {
                    continue;
                }
                if cells[i].remove_possibilities(b_unique) {
                    dirty = true;
                    //println!("Removed!");
                }
            }
            for i in REGS[c_region] {
                if group.contains(&i) {
                    continue;
                }
                if cells[i].remove_possibilities(c_unique) {
                    dirty = true;
                    //println!("Removed!");
                }
            }
        }
        dirty
    }
}
//</editor-fold>
