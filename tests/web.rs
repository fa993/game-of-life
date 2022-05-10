//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

extern crate automaton_engine;
use automaton_engine::Universe;
use automaton_engine::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1 + 1, 2);
}

#[cfg(test)]
pub fn input_spaceship() -> Universe {
    let mut universe = Universe::new();
    universe.set_dimensions(6, 6);
    universe.set_cells(&[(1,2), (2,3), (3,1), (3,2), (3,3)]);
    return universe;
}

#[cfg(test)]
pub fn expected_spaceship() -> Universe {
    let mut universe = Universe::new();
    universe.set_dimensions(6, 6);
    universe.set_cells(&[(2,1), (2,3), (3,2), (3,3), (4,2)]);
    return universe;
}

#[wasm_bindgen_test]
pub fn test_tick() {
    let mut input_universe = input_spaceship();
    let expected_universe = expected_spaceship();

    // Call `tick` and then see if the cells in the `Universe`s are the same.
    input_universe.tick_life();
    assert_eq!(&input_universe.get_cells(), &expected_universe.get_cells());
}

#[wasm_bindgen_test]
pub fn test_randomize() {
    let mut universe = Universe::new();
    universe.set_dimensions(6, 6);

    universe.randomize_state();
    assert_eq!(true, true);
}


#[wasm_bindgen_test]
pub fn test_reflects() {
    assert_eq!(reflect_y((4, 5)), (-4, 5));
    assert_eq!(reflect_x((4, 5)), (4, -5));
    assert_eq!(reflect_xy((4, 5)), (-4, -5));
}

#[wasm_bindgen_test]
pub fn test_coordinate_shift_to() {
    assert_eq!(coordinate_shift_to((4, 5), (2, 2)), (6, 7));
}

#[wasm_bindgen_test]
pub fn test_wrap() {
    assert_eq!(wrap_coordinates((65, -4), (64, 64)), (1, 60));
}
