use crate::{Grid, ROWS};
use rdev::{simulate, Button, EventType, Key};

const DELAY: std::time::Duration = std::time::Duration::from_millis(1);
fn send(number: usize) {
    const NUM_KEYS: [Key; 10] = [
        Key::Num0,
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
    ];
    simulate(&EventType::KeyPress(NUM_KEYS[number])).unwrap();
    std::thread::sleep(DELAY);
}
fn click() {
    simulate(&EventType::ButtonPress(Button::Left)).unwrap();
    std::thread::sleep(DELAY);
    simulate(&EventType::ButtonRelease(Button::Left)).unwrap();
}
pub fn send_input(grid: Grid) {
    click();
    let mut forward = true;
    for i in 0..9 {
        if forward {
            for j in 0..9 {
                if !grid.cells[ROWS[i][j]].is_given {
                    send(grid.cells[ROWS[i][j]].value as usize);
                }
                simulate(&EventType::KeyPress(Key::RightArrow)).unwrap();
                std::thread::sleep(DELAY);
            }
        } else {
            for j in (0..9).rev() {
                if !grid.cells[ROWS[i][j]].is_given {
                    send(grid.cells[ROWS[i][j]].value as usize)
                }
                simulate(&EventType::KeyPress(Key::LeftArrow)).unwrap();
                std::thread::sleep(DELAY);
            }
        }
        forward = !forward;

        simulate(&EventType::KeyPress(Key::DownArrow)).unwrap();
        std::thread::sleep(DELAY);
    }
}
