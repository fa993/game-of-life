#![feature(test)]

extern crate test;
extern crate automaton_engine;

#[bench]
fn static_universe_ticks(bch: &mut test::Bencher) {
    unsafe {
        automaton_engine::singleuni::init();

        // let n = test::black_box(&mut uni);

        bch.iter(|| {
            return automaton_engine::singleuni::tick_life();
        });
    }
}

#[bench]
fn universe_ticks(bch: &mut test::Bencher) {
    let mut uni = automaton_engine::Universe::new();

    // let n = test::black_box(&mut uni);

    bch.iter(|| {
        return uni.tick_life();
    });
}

// #[bench]
// fn universe_ticks_v2(bch: &mut test::Bencher) {
//     let mut uni = automaton_engine::Universe_v2::new();
//
//     // let n = test::black_box(&mut uni);
//
//     bch.iter(|| {
//         return uni.tick_life();
//     });
// }
