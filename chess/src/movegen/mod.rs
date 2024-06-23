pub mod lookups;
pub mod castling;
pub mod legal_moves;
pub mod moves;
pub mod play_move;
pub mod move_array;

#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
pub mod magics;

#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
pub mod pext; 
