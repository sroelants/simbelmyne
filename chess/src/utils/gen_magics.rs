use simbelmyne_chess::magics::gen_magics;

const BISHOP: bool = true;
const ROOK: bool = false;

fn main() {
    let bishop_magics = gen_magics::<BISHOP>();
    let rook_magics = gen_magics::<ROOK>();

    println!("Bishop magics:\n{bishop_magics:#?}");
    println!("Rook magics:\n{rook_magics:#?}");
}
