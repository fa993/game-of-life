mod utils;

// use wasm_bindgen::prelude::*;
use fixedbitset::FixedBitSet;
use getrandom::getrandom;

// extern crate web_sys;

// use web_sys::console;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }
//
// #[wasm_bindgen]
// pub fn greet(name: &str) {
//     alert(&format!("Hello, {}!", name));
// }

pub struct Timer<'a> {

    name: &'a str,

}

impl<'a> Timer<'a> {

    pub fn new(name: &'a str) -> Timer<'a> {
        // console::time_with_label(name);
        return Timer { name };
    }

}

impl<'a> Drop for Timer<'a> {

    fn drop(&mut self) {
        // console::time_end_with_label(self.name);
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferState {
    Primary,
    Secondary,
}

pub struct DoubleBuffer<T> {
    cells_primary: T,
    cells_secondary: T,
    active_buffer: BufferState,
}

impl<T> DoubleBuffer<T> {

    pub fn new(cells_primary: T, cells_secondary: T) -> DoubleBuffer<T> {
        return DoubleBuffer {
            cells_primary,
            cells_secondary,
            active_buffer: BufferState::Primary,
        };
    }

    pub fn read(&self) -> &T {
        return match self.active_buffer {
            BufferState::Primary => &self.cells_primary,
            BufferState::Secondary => &self.cells_secondary,
        };
    }

    pub fn read_mut(&mut self) -> &mut T {
        return match self.active_buffer {
            BufferState::Primary => &mut self.cells_primary,
            BufferState::Secondary => &mut self.cells_secondary,
        };
    }

    pub fn write(&mut self) -> &mut T {
        return match self.active_buffer {
            BufferState::Primary => &mut self.cells_secondary,
            BufferState::Secondary => &mut self.cells_primary,
        };
    }

    pub fn finish_write(&mut self) {
        self.active_buffer = match self.active_buffer {
            BufferState::Primary => BufferState::Secondary,
            BufferState::Secondary => BufferState::Primary,
        };
    }

}

#[inline(always)]
pub fn reflect_x(coords: (i32, i32)) -> (i32, i32) {
    return (coords.0, -coords.1);
}

#[inline(always)]
pub fn reflect_y(coords: (i32, i32)) -> (i32, i32) {
    return (-coords.0, coords.1);
}

#[inline(always)]
pub fn reflect_xy(coords: (i32, i32)) -> (i32, i32) {
    return (-coords.0, -coords.1);
}

#[inline(always)]
pub fn coordinate_shift_to(coords: (i32, i32), center: (u32, u32)) -> (i32, i32) {
    return (coords.0 + center.0 as i32, coords.1 + center.1 as i32);
}

#[inline(always)]
pub fn wrap_coordinates(coord: (i32, i32), limit: (u32, u32)) -> (u32, u32) {
    return (
        ( if coord.0 < 0 {limit.0 - (coord.0.abs() as u32)} else {(coord.0 as u32) % limit.0} ),
        ( if coord.1 < 0 {limit.1 - (coord.1.abs() as u32)} else {(coord.1 as u32) % limit.1} )
    );
}

// #[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: DoubleBuffer<FixedBitSet>,
    byte_store: Vec<u8>,
}

impl Universe {

    #[inline(always)]
    pub fn get_index(&self, row: u32, column: u32) -> usize {
        return (row * self.width + column) as usize;
    }

    #[inline(always)]
    pub fn get_index_tu(&self, coords: (u32, u32)) ->usize {
        return self.get_index(coords.1, coords.0);
    }

    pub fn get_enabled(&self, row: u32, column: u32) -> bool {
        return self.cells.read()[self.get_index(row, column)];
    }

    pub fn get_live_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let old_cell = self.cells.read();
        let mut count = 0;

        //hand check all possible 8 locations
        let minus_one_row;
        let minus_one_column;
        let plus_one_row;
        let plus_one_column;

        if row == 0 {
            minus_one_row = self.height - 1;
        } else {
            minus_one_row = row - 1;
        }

        if column == 0 {
            minus_one_column = self.width - 1;
        } else {
            minus_one_column = column - 1;
        }

        if row == self.height - 1 {
            plus_one_row = 0;
        } else {
            plus_one_row = row + 1
        }

        if column == self.width - 1 {
            plus_one_column = 0;
        } else {
            plus_one_column = column + 1
        }

        //upper checks
        count += old_cell[self.get_index(minus_one_row, column)] as u8;
        count += old_cell[self.get_index(minus_one_row, minus_one_column)] as u8;
        count += old_cell[self.get_index(minus_one_row, plus_one_column)] as u8;

        //bottom checks
        count += old_cell[self.get_index(plus_one_row, column)] as u8;
        count += old_cell[self.get_index(plus_one_row, minus_one_column)] as u8;
        count += old_cell[self.get_index(plus_one_row, plus_one_column)] as u8;

        //side checks
        count += old_cell[self.get_index(row, minus_one_column)] as u8;
        count += old_cell[self.get_index(row, plus_one_column)] as u8;

        return count;
    }

}

// #[wasm_bindgen]
impl Universe {

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width: u32 = 128;
        let height: u32 = 128;

        let size = (width * height) as usize;

        let mut cells_primary = FixedBitSet::with_capacity(size);
        let mut cells_secondary = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells_primary.set(i, i % 2 == 0 || i % 7 == 0);
            cells_secondary.set(i, i % 2 == 0 || i % 7 == 0);
        }

        let szb = size / 8 + (((size % 8) != 0) as usize);

        let mut byte_store = Vec::with_capacity(szb);

        for _ in 0..szb {
            byte_store.push(0);
        }

        return Universe {
            width,
            height,
            cells: DoubleBuffer::new(cells_primary, cells_secondary),
            byte_store,
        };
    }

    pub fn get_width(&self) -> u32 {
        return self.width;
    }

    pub fn get_height(&self) -> u32 {
        return self.height;
    }

    pub fn tick_life(&mut self) {
        let _timer = Timer::new("Universe::tick_life");
        for i in 0..self.width {
            for j in 0..self.height {
                let live_neighbour = self.get_live_neighbour_count(i , j);
                let enabled = self.get_enabled(i, j);
                let current_index = self.get_index(i, j);
                self.cells.write().set(
                    current_index,
                    match (enabled, live_neighbour) {
                        (true, x) if x < 2 || x > 3 => false,
                        (false, 3) => true,
                        (same_state, _) => same_state
                    }
                );
            }
        }

        self.cells.finish_write();
    }

    pub fn get_pointer(&self) -> *const u32 {
        return self.cells.read().as_slice().as_ptr();
    }

    pub fn toggle(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        let old_state = self.cells.read()[idx];
        self.cells.read_mut().set(idx, !old_state);
    }

    pub fn randomize_state(&mut self) {
        let size = (self.width * self.height) as usize;
        getrandom(&mut self.byte_store).expect("Something went wrong with rand");
        for i in 0..self.byte_store.len() {
            for j in 0..8 {
                //here endian-ness doesn't really matter since we're initializing random universes anyways
                if i * 8 + j >= size {
                    self.cells.finish_write();
                    break;
                }
                self.cells.write().set(i * 8 + j, (self.byte_store[i] & (1 << j)) == (1 << j));
            }
        }
        self.cells.finish_write();
    }

    pub fn insert_pulsar(&mut self, row: u32, col: u32) {
        //I give up... not unrolling the mod opertion for this
        //coordinates to blacken -  (1, 2), (1, 3), (1, 4),
        //                          (2, 6), (3, 6), (4, 6),
        //                          (6, 4), (6, 3), (6, 2),
        //                          (2, 1), (3, 1), (4, 1),
        //reflect in other 3 quadrants, and co-ordinate shift

        let coords = [
                    (1, 2), (1, 3), (1, 4),
                    (2, 6), (3, 6), (4, 6),
                    (6, 4), (6, 3), (6, 2),
                    (2, 1), (3, 1), (4, 1),
                    ];

        IntoIterator::into_iter(coords).for_each(|t| {
            let r = self.get_index_tu(wrap_coordinates(coordinate_shift_to(t , (col, row)), (self.width, self.height)));
            self.cells.read_mut().set(r, true);
        });

        IntoIterator::into_iter(coords).map(reflect_x).for_each(|t| {
            let r = self.get_index_tu(wrap_coordinates(coordinate_shift_to(t , (col, row)), (self.width, self.height)));
            self.cells.read_mut().set(r, true);
        });

        IntoIterator::into_iter(coords).map(reflect_y).for_each(|t| {
            let r = self.get_index_tu(wrap_coordinates(coordinate_shift_to(t , (col, row)), (self.width, self.height)));
            self.cells.read_mut().set(r, true);
        });

        IntoIterator::into_iter(coords).map(reflect_xy).for_each(|t| {
            let r = self.get_index_tu(wrap_coordinates(coordinate_shift_to(t , (col, row)), (self.width, self.height)));
            self.cells.read_mut().set(r, true);
        });

    }

    pub fn insert_glider(&mut self, row: u32, col: u32) {

        let minus_one_row;
        let minus_one_column;
        let plus_one_row;
        let plus_one_column;

        if row == 0 {
            minus_one_row = self.height - 1;
        } else {
            minus_one_row = row - 1;
        }

        if col == 0 {
            minus_one_column = self.width - 1;
        } else {
            minus_one_column = col - 1;
        }

        if row == self.height - 1 {
            plus_one_row = 0;
        } else {
            plus_one_row = row + 1
        }

        if col == self.width - 1 {
            plus_one_column = 0;
        } else {
            plus_one_column = col + 1
        }

        let left_index = self.get_index(row, minus_one_column);
        let bottom_index = self.get_index(plus_one_row, col);

        let right_up_index = self.get_index(row, plus_one_column);
        let right_middle_index = self.get_index(plus_one_row, plus_one_column);
        let right_bottom_index = self.get_index(minus_one_row, plus_one_column);

        let read_cells = self.cells.read_mut();

        //left inserts
        read_cells.set(left_index, true);


        //bottom inserts
        read_cells.set(bottom_index, true);

        //right inserts
        read_cells.set(right_up_index, true);
        read_cells.set(right_middle_index, true);
        read_cells.set(right_bottom_index, true);
    }

}

// #[wasm_bindgen]
impl Universe {

    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        let szb = size / 8 + (((size % 8) != 0) as usize);

        let mut byte_store = Vec::with_capacity(szb);

        for _ in 0..szb {
            byte_store.push(0);
        }

        self.byte_store = byte_store;
        self.cells = DoubleBuffer::new(FixedBitSet::with_capacity(size), FixedBitSet::with_capacity(size));
    }

}

impl Universe {

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter() {
            let idx = self.get_index(*row, *col);
            self.cells.read_mut().set(idx, true);
        }
    }

    pub fn get_cells(&self) -> &[u32] {
        return self.cells.read().as_slice();
    }

}

// #[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {

    Dead = 0,
    Alive = 1,

}

pub struct Universe_v2 {
    width: u32,
    height: u32,
    cells: DoubleBuffer<Vec<CellState>>,
    byte_store: Vec<u8>,
}

impl Universe_v2 {

    #[inline(always)]
    pub fn get_index(&self, row: u32, column: u32) -> usize {
        return (row * self.width + column) as usize;
    }

    #[inline(always)]
    pub fn get_index_tu(&self, coords: (u32, u32)) ->usize {
        return self.get_index(coords.1, coords.0);
    }

    pub fn get_enabled(&self, row: u32, column: u32) -> CellState {
        return self.cells.read()[self.get_index(row, column)];
    }

    pub fn get_live_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let old_cell = self.cells.read();
        let mut count = 0;

        //hand check all possible 8 locations
        let minus_one_row;
        let minus_one_column;
        let plus_one_row;
        let plus_one_column;

        if row == 0 {
            minus_one_row = self.height - 1;
        } else {
            minus_one_row = row - 1;
        }

        if column == 0 {
            minus_one_column = self.width - 1;
        } else {
            minus_one_column = column - 1;
        }

        if row == self.height - 1 {
            plus_one_row = 0;
        } else {
            plus_one_row = row + 1
        }

        if column == self.width - 1 {
            plus_one_column = 0;
        } else {
            plus_one_column = column + 1
        }

        //upper checks
        count += old_cell[self.get_index(minus_one_row, column)] as u8;
        count += old_cell[self.get_index(minus_one_row, minus_one_column)] as u8;
        count += old_cell[self.get_index(minus_one_row, plus_one_column)] as u8;

        //bottom checks
        count += old_cell[self.get_index(plus_one_row, column)] as u8;
        count += old_cell[self.get_index(plus_one_row, minus_one_column)] as u8;
        count += old_cell[self.get_index(plus_one_row, plus_one_column)] as u8;

        //side checks
        count += old_cell[self.get_index(row, minus_one_column)] as u8;
        count += old_cell[self.get_index(row, plus_one_column)] as u8;

        return count;
    }

}

// #[wasm_bindgen]
impl Universe_v2 {

    pub fn new() -> Universe_v2 {
        utils::set_panic_hook();
        let width: u32 = 128;
        let height: u32 = 128;

        let size = (width * height) as usize;

        let mut cells_primary = Vec::with_capacity(size);
        let mut cells_secondary = Vec::with_capacity(size);

        for i in 0..size {
            let tpm = if i % 2 == 0 || i % 7 == 0 {CellState::Alive} else {CellState::Dead};
            cells_primary.push(tpm);
            cells_secondary.push(tpm);
        }

        let szb = size / 8 + (((size % 8) != 0) as usize);

        let mut byte_store = Vec::with_capacity(szb);

        for _ in 0..szb {
            byte_store.push(0);
        }

        return Universe_v2 {
            width,
            height,
            cells: DoubleBuffer::new(cells_primary, cells_secondary),
            byte_store,
        };
    }

    pub fn get_width(&self) -> u32 {
        return self.width;
    }

    pub fn get_height(&self) -> u32 {
        return self.height;
    }

    pub fn tick_life(&mut self) {
        let _timer = Timer::new("Universe::tick_life");
        for i in 0..self.width {
            for j in 0..self.height {
                let live_neighbour = self.get_live_neighbour_count(i , j);
                let enabled = self.get_enabled(i, j);
                let current_index = self.get_index(i, j);
                self.cells.write()[current_index] =
                    (match (enabled, live_neighbour) {
                        (CellState::Alive, x) if x < 2 || x > 3 => CellState::Dead,
                        (CellState::Dead, 3) => CellState::Alive,
                        (same_state, _) => same_state
                    }
                );
            }
        }

        self.cells.finish_write();
    }

    pub fn get_pointer(&self) -> *const CellState {
        return self.cells.read().as_ptr();
    }

    pub fn toggle(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        // let old_state = self.cells.read()[idx];
        // self.cells.read_mut()[idx] = if old_state == 0 {CellState::Alive} else {CellState::Dead};
        self.cells.read_mut()[idx] = match self.cells.read()[idx] {
            CellState::Alive => CellState::Dead,
            CellState::Dead => CellState::Alive
        }
    }

    pub fn randomize_state(&mut self) {
        let size = (self.width * self.height) as usize;
        getrandom(&mut self.byte_store).expect("Something went wrong with rand");
        for i in 0..self.byte_store.len() {
            for j in 0..8 {
                //here endian-ness doesn't really matter since we're initializing random universes anyways
                if i * 8 + j >= size {
                    self.cells.finish_write();
                    break;
                }
                self.cells.write()[i * 8 + j] = if (self.byte_store[i] & (1 << j)) == (1 << j) {CellState::Alive} else {CellState::Dead};
            }
        }
        self.cells.finish_write();
    }

    pub fn insert_pulsar(&mut self, row: u32, col: u32) {
        //I give up... not unrolling the mod opertion for this
        //coordinates to blacken -  (1, 2), (1, 3), (1, 4),
        //                          (2, 6), (3, 6), (4, 6),
        //                          (6, 4), (6, 3), (6, 2),
        //                          (2, 1), (3, 1), (4, 1),
        //reflect in other 3 quadrants, and co-ordinate shift

        let coords = [
                    (1, 2), (1, 3), (1, 4),
                    (2, 6), (3, 6), (4, 6),
                    (6, 4), (6, 3), (6, 2),
                    (2, 1), (3, 1), (4, 1),
                    ];

        IntoIterator::into_iter(coords).for_each(|t| {
            let r = self.get_index_tu(wrap_coordinates(coordinate_shift_to(t , (col, row)), (self.width, self.height)));
            self.cells.read_mut()[r] = CellState::Alive;
        });

        IntoIterator::into_iter(coords).map(reflect_x).for_each(|t| {
            let r = self.get_index_tu(wrap_coordinates(coordinate_shift_to(t , (col, row)), (self.width, self.height)));
            self.cells.read_mut()[r] = CellState::Alive;
        });

        IntoIterator::into_iter(coords).map(reflect_y).for_each(|t| {
            let r = self.get_index_tu(wrap_coordinates(coordinate_shift_to(t , (col, row)), (self.width, self.height)));
            self.cells.read_mut()[r] = CellState::Alive;
        });

        IntoIterator::into_iter(coords).map(reflect_xy).for_each(|t| {
            let r = self.get_index_tu(wrap_coordinates(coordinate_shift_to(t , (col, row)), (self.width, self.height)));
            self.cells.read_mut()[r] = CellState::Alive;
        });

    }

    pub fn insert_glider(&mut self, row: u32, col: u32) {

        let minus_one_row;
        let minus_one_column;
        let plus_one_row;
        let plus_one_column;

        if row == 0 {
            minus_one_row = self.height - 1;
        } else {
            minus_one_row = row - 1;
        }

        if col == 0 {
            minus_one_column = self.width - 1;
        } else {
            minus_one_column = col - 1;
        }

        if row == self.height - 1 {
            plus_one_row = 0;
        } else {
            plus_one_row = row + 1
        }

        if col == self.width - 1 {
            plus_one_column = 0;
        } else {
            plus_one_column = col + 1
        }

        let left_index = self.get_index(row, minus_one_column);
        let bottom_index = self.get_index(plus_one_row, col);

        let right_up_index = self.get_index(row, plus_one_column);
        let right_middle_index = self.get_index(plus_one_row, plus_one_column);
        let right_bottom_index = self.get_index(minus_one_row, plus_one_column);

        let read_cells = self.cells.read_mut();

        //left inserts
        read_cells[left_index] = CellState::Alive;


        //bottom inserts
        read_cells[bottom_index] = CellState::Alive;

        //right inserts
        read_cells[right_up_index] = CellState::Alive;
        read_cells[right_middle_index] = CellState::Alive;
        read_cells[right_bottom_index] = CellState::Alive;
    }

}

// #[wasm_bindgen]
impl Universe_v2 {

    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        let szb = size / 8 + (((size % 8) != 0) as usize);

        let mut byte_store = Vec::with_capacity(szb);

        for _ in 0..szb {
            byte_store.push(0);
        }

        self.byte_store = byte_store;
        self.cells = DoubleBuffer::new(Vec::with_capacity(size), Vec::with_capacity(size));
    }

}

impl Universe_v2 {

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter() {
            let idx = self.get_index(*row, *col);
            self.cells.read_mut()[idx] = CellState::Alive;
        }
    }

    pub fn get_cells(&self) -> &[CellState] {
        return self.cells.read().as_slice();
    }

}
