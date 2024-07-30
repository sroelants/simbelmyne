/// Setup LMR tables which need float math.
/// Straight up copied from Carp
use std::{error::Error, fs::File, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    // Build LMR table
    const QUIET_LMR_BASE: f32 = 0.75;
    const QUIET_LMR_FACTOR: f32 = 2.25;

    const TACTICAL_LMR_BASE: f32 = -0.15;
    const TACTICAL_LMR_FACTOR: f32 = 2.55;

    let mut reductions = [[[0; 64]; 64]; 2];
    for (tactical, table) in reductions.iter_mut().enumerate() {
        for (depth, table) in table.iter_mut().enumerate().skip(1) {
            for (move_count, reduction) in table.iter_mut().enumerate().skip(1) {
                *reduction = if tactical != 0 {
                    (TACTICAL_LMR_BASE + (depth as f32).ln() * (move_count as f32).ln() / TACTICAL_LMR_FACTOR) as usize
                } else {
                    (QUIET_LMR_BASE + (depth as f32).ln() * (move_count as f32).ln() / QUIET_LMR_FACTOR) as usize
                };
            }
        }
    }

    let lmr = unsafe {
        std::slice::from_raw_parts::<u8>(
            reductions.as_ptr().cast::<u8>(),
            64 * 64 * 2 * std::mem::size_of::<usize>(),
        )
    };
    File::create(PathBuf::new().join("..").join("bins").join("lmr.bin"))?.write_all(lmr)?;

    Ok(())
}
