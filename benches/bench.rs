#![feature(test)]

extern crate test;
extern crate automaton_engine;

#[bench]
fn universe_ticks(bch: &mut test::Bencher) {
    let mut uni = automaton_engine::Universe::new();

    let n = test::black_box(&mut uni);

    bch.iter(|| {
        return n.tick_life();
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
