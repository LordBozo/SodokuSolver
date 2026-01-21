use colored::Colorize;
use std::fmt;
use std::fmt::Formatter;

//static GROUPS: [&[[usize; 9]; 9]; 3] = [&ROWS, &COLS, &REGS];
#[derive(Copy, Clone)]
pub struct Cell {
    pub(crate) candidates: u16,
    pub(crate) value: u8,
    pub(crate) answer: Option<u8>,
    pub(crate) is_given: bool,
    pub(crate) is_dirty: bool,
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut candidates = self.candidates;
        let mut val = 1i8;
        let mut accumulate = "".to_string();
        while candidates != 0 {
            accumulate += &*if (candidates & 1) == 1 {
                val.to_string()
            } else {
                "-".to_string()
            };
            candidates >>= 1;
            val += 1;
        }
        write!(f, "{:9}", accumulate)
    }
}

#[allow(dead_code)]
impl Cell {
    fn color_card(&self, card: String) -> String {
        let result: Vec<String>;
        if self.is_given {
            result = card
                .split("\n")
                .map(|x| x.green().to_string())
                .collect::<Vec<String>>();
        } else if self.is_dirty {
            result = card
                .split("\n")
                .map(|x| x.blue().to_string())
                .collect::<Vec<String>>();
        } else {
            return card;
        }
        return result.join("\n").to_string();
    }
    pub fn get_print_card(&self) -> String {
        if self.value == 0 {
            let mut base = format!("{:?}", self);
            base.insert(6, '\n');
            base.insert(3, '\n');
            base = base.replace('-', " ");
            base = self.color_card(base);
            base
        } else {
            const NUMBERS: [&str; 9] = [
                " ┓ \n ┃ \n ┻ ",
                "┏━┓\n┏━┛\n┗━━",
                "┏━┓\n ━┫\n┗━┛",
                "╻ ╻\n┗━╋\n  ╹",
                "┏━╸\n┗━┓\n┗━┛",
                "┏━┓\n┣━┓\n┗━┛",
                "╺━┓\n  ┃\n  ╹",
                "┏━┓\n┣━┫\n┗━┛",
                "┏━┓\n┗━┫\n┗━┛",
            ];
            let result = NUMBERS[self.value as usize - 1];
            self.color_card(result.to_string())
        }
    }
    pub fn contains_value(&self, value: u8) -> bool {
        if self.value <= 0 {
            return (self.candidates & (1 << value - 1)) > 0;
        }
        false
    }
    pub fn promote_single_candidate(&mut self) -> bool {
        let mut val = 0u8;
        let mut possibilities = self.candidates;
        let mut bits_set = 0;
        while possibilities != 0 {
            if (possibilities & 1) == 1 {
                bits_set += 1;
            }
            possibilities >>= 1;
            val += 1;
        }
        if bits_set == 1 {
            self.set_value(val);
            return true;
        }
        false
    }
    pub(crate) fn remove_possibilities(&mut self, bits: u16) -> bool {
        if bits & self.candidates != 0 {
            self.candidates &= !bits;
            self.is_answer_possible();
            self.is_dirty = true;
            return true;
        }
        false
    }
    pub(crate) fn remove_possibility(&mut self, value: u8) {
        self.candidates &= !(1 << (value - 1));
        self.is_answer_possible();
    }

    pub fn is_answer_possible(&self) {
        if self.answer.is_none() || self.value > 0 {
            return;
        }
        let ans = self.answer.unwrap();
        if !self.contains_value(ans) {
            panic!("REMOVED ANSWER AS POSSIBILITY")
        }
    }
    pub fn get_possibilities(&self) -> Vec<u16> {
        if self.value != 0 {
            return Vec::new();
        }
        let mut possibilities = self.candidates;
        let mut results = Vec::new();
        for i in 0..9 {
            if possibilities & 1 == 1 {
                results.push(i + 1);
            }
            possibilities >>= 1;
        }
        results
    }
    pub(crate) fn set_value(&mut self, value: u8) {
        if self.answer.is_some() {
            if value != self.answer.unwrap() {
                println!(
                    "INVALID ANSWER: should be {:?}, is {value}",
                    self.answer.unwrap()
                );
            }
        }
        self.value = value;
        self.candidates = 0;
        self.is_dirty = true;
    }
}
