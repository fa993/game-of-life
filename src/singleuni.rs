pub use crate::*;
use wasm_bindgen::prelude::*;

// extern crate web_sys;
//
// use web_sys::console;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }
//
// #[wasm_bindgen]
// pub fn greet(name: &str) {
//     alert(&format!("Hello, {}!", name));
// }

// pub struct Timer<'a> {
//
//     name: &'a str,
//
// }
//
// impl<'a> Timer<'a> {
//
//     pub fn new(name: &'a str) -> Timer<'a> {
//         console::time_with_label(name);
//         return Timer { name };
//     }
//
// }
//
// impl<'a> Drop for Timer<'a> {
//
//     fn drop(&mut self) {
//         console::time_end_with_label(self.name);
//     }
//
// }

pub static mut U: Universe = Universe {
    width: WIDTH,
    height: HEIGHT,
    cells: DoubleBuffer {
        cells_primary: [CellState::Dead; SIZE],
        cells_secondary: [CellState::Dead; SIZE],
        active_buffer: BufferState::Primary,
    },
    byte_store: [0; SIZE / 8 + (SIZE % 8 != 0) as usize],
};

#[inline(always)]
pub unsafe fn get_index(row: u32, column: u32) -> usize {
    return (row * U.width + column) as usize;
}

#[inline(always)]
pub unsafe fn get_index_tu(coords: (u32, u32)) -> usize {
    return get_index(coords.1, coords.0);
}

pub unsafe fn get_enabled(row: u32, column: u32) -> CellState {
    return U.cells.read()[get_index(row, column)];
}

pub unsafe fn get_live_neighbour_count(row: u32, column: u32) -> u8 {
    let old_cell = U.cells.read();

    //hand check all possible 8 locations
    let minus_one_row;
    let minus_one_column;
    let plus_one_row;
    let plus_one_column;

    if row == 0 {
        minus_one_row = HEIGHT - 1;
    } else {
        minus_one_row = row - 1;
    }

    if column == 0 {
        minus_one_column = WIDTH - 1;
    } else {
        minus_one_column = column - 1;
    }

    if row == HEIGHT - 1 {
        plus_one_row = 0;
    } else {
        plus_one_row = row + 1
    }

    if column == WIDTH - 1 {
        plus_one_column = 0;
    } else {
        plus_one_column = column + 1
    }

    //upper checks
    return old_cell[get_index(minus_one_row, column)] as u8
        + old_cell[get_index(minus_one_row, minus_one_column)] as u8
        + old_cell[get_index(minus_one_row, plus_one_column)] as u8

        //bottom checks
        + old_cell[get_index(plus_one_row, column)] as u8
        + old_cell[get_index(plus_one_row, minus_one_column)] as u8
        + old_cell[get_index(plus_one_row, plus_one_column)] as u8

        //side checks
        + old_cell[get_index(row, minus_one_column)] as u8
        + old_cell[get_index(row, plus_one_column)] as u8;
}

#[wasm_bindgen]
pub fn get_width() -> u32 {
    return unsafe { U.width };
}

#[wasm_bindgen]
pub fn get_height() -> u32 {
    return unsafe { U.height };
}

#[wasm_bindgen]
pub fn init() {
    unsafe {
        for i in 0..SIZE {
            U.cells.write()[i] = if i % 2 == 0 || i % 7 == 0 {
                CellState::Alive
            } else {
                CellState::Dead
            };
        }
        U.cells.finish_write();
    }
}

#[wasm_bindgen]
pub fn tick_life() {
    // let _timer = Timer::new("Universe::tick_life");
    // let r = self.cells.write();
    unsafe {
        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let live_neighbour = get_live_neighbour_count(i, j);
                let enabled = get_enabled(i, j);
                let current_index = get_index(i, j);
                U.cells.write()[current_index] = match (enabled, live_neighbour) {
                    (CellState::Alive, x) if x < 2 || x > 3 => CellState::Dead,
                    (CellState::Dead, 3) => CellState::Alive,
                    (same_state, _) => same_state,
                };
            }
        }

        U.cells.finish_write();
    }
}

#[wasm_bindgen]
pub fn get_pointer() -> *const CellState {
    return unsafe { U.cells.read().as_ptr() };
}

#[wasm_bindgen]
pub fn toggle(row: u32, column: u32) {
    unsafe {
        let idx = get_index(row, column);
        U.cells.read_mut()[idx] = match U.cells.read()[idx] {
            CellState::Alive => CellState::Dead,
            CellState::Dead => CellState::Alive,
        }
    }
}

#[wasm_bindgen]
pub fn randomize_state() {
    unsafe {
        // let bit_size = SIZE / 8 + (SIZE % 8 != 0) as usize;
        // getrandom(&mut U.byte_store[..bit_size]).expect("Something went wrong with rand");
        for i in 0..SIZE {
            // let bucket_pos = i / 8;
            // let bit_pos = (i % 8) as u8;
            U.cells.write()[i] = if js_sys::Math::random() < 0.5 {
                CellState::Alive
            } else {
                CellState::Dead
            }
            //here endian-ness doesn't matter because it's random anyways
            // U.cells.write()[i] = if U.byte_store[bucket_pos] & (1u8 << bit_pos) == bit_pos {
            //     CellState::Alive
            // } else {
            //     CellState::Dead
            // };
        }
        U.cells.finish_write();
    }
}

#[wasm_bindgen]
pub fn insert_pulsar(row: u32, col: u32) {
    //I give up... not unrolling the mod opertion for this
    //coordinates to blacken -  (1, 2), (1, 3), (1, 4),
    //                          (2, 6), (3, 6), (4, 6),
    //                          (6, 4), (6, 3), (6, 2),
    //                          (2, 1), (3, 1), (4, 1),
    //reflect in other 3 quadrants, and co-ordinate shift
    unsafe {
        let coords = [
            (1, 2),
            (1, 3),
            (1, 4),
            (2, 6),
            (3, 6),
            (4, 6),
            (6, 4),
            (6, 3),
            (6, 2),
            (2, 1),
            (3, 1),
            (4, 1),
        ];

        IntoIterator::into_iter(coords).for_each(|t| {
            let r = get_index_tu(wrap_coordinates(
                coordinate_shift_to(t, (col, row)),
                (WIDTH, HEIGHT),
            ));
            U.cells.read_mut()[r] = CellState::Alive;
        });

        IntoIterator::into_iter(coords)
            .map(reflect_x)
            .for_each(|t| {
                let r = get_index_tu(wrap_coordinates(
                    coordinate_shift_to(t, (col, row)),
                    (WIDTH, HEIGHT),
                ));
                U.cells.read_mut()[r] = CellState::Alive;
            });

        IntoIterator::into_iter(coords)
            .map(reflect_y)
            .for_each(|t| {
                let r = get_index_tu(wrap_coordinates(
                    coordinate_shift_to(t, (col, row)),
                    (WIDTH, HEIGHT),
                ));
                U.cells.read_mut()[r] = CellState::Alive;
            });

        IntoIterator::into_iter(coords)
            .map(reflect_xy)
            .for_each(|t| {
                let r = get_index_tu(wrap_coordinates(
                    coordinate_shift_to(t, (col, row)),
                    (WIDTH, HEIGHT),
                ));
                U.cells.read_mut()[r] = CellState::Alive;
            });
    };
}

#[wasm_bindgen]
pub fn insert_glider(row: u32, col: u32) {
    unsafe {
        let minus_one_row;
        let minus_one_column;
        let plus_one_row;
        let plus_one_column;

        if row == 0 {
            minus_one_row = HEIGHT - 1;
        } else {
            minus_one_row = row - 1;
        }

        if col == 0 {
            minus_one_column = WIDTH - 1;
        } else {
            minus_one_column = col - 1;
        }

        if row == HEIGHT - 1 {
            plus_one_row = 0;
        } else {
            plus_one_row = row + 1
        }

        if col == WIDTH - 1 {
            plus_one_column = 0;
        } else {
            plus_one_column = col + 1
        }

        let left_index = get_index(row, minus_one_column);
        let bottom_index = get_index(plus_one_row, col);

        let right_up_index = get_index(row, plus_one_column);
        let right_middle_index = get_index(plus_one_row, plus_one_column);
        let right_bottom_index = get_index(minus_one_row, plus_one_column);

        let read_cells = U.cells.read_mut();

        //left inserts
        read_cells[left_index] = CellState::Alive;

        //bottom inserts
        read_cells[bottom_index] = CellState::Alive;

        //right inserts
        read_cells[right_up_index] = CellState::Alive;
        read_cells[right_middle_index] = CellState::Alive;
        read_cells[right_bottom_index] = CellState::Alive;
    };
}

#[wasm_bindgen]
pub fn kill_all() {
    unsafe {
        for i in 0..SIZE {
            U.cells.write()[i] = CellState::Dead;
        }
        U.cells.finish_write();
    }
}

// #[wasm_bindgen]
// impl Universe {
//
//     pub unsafe fn set_dimensions(&mut self, width: u32, height: u32) {
//         self.width = width;
//         self.height = height;
//         let size = (width * height) as usize;
//
//         self.cells.read_mut().clear();
//         self.cells.write().clear();
//         self.byte_store.clear();
//
//         for _ in 0..size {
//             self.byte_store.push(0);
//             self.cells.read_mut().push(CellState::Dead);
//             self.cells.write().push(CellState::Dead);
//         }
//
//     }
//
// }

pub unsafe fn set_cells(cells: &[(u32, u32)]) {
    for (row, col) in cells.iter() {
        let idx = get_index(*row, *col);
        U.cells.read_mut()[idx] = CellState::Alive;
    }
}

pub unsafe fn get_cells() -> &'static [CellState] {
    return U.cells.read().as_slice();
}
