use chess::{board::Board, piece::PieceType};

// NormalizeToPawnValue = 58
pub const WDL_MODEL: WdlModel = WdlModel {
    a: [-351.41899961, 1000.88793727, -955.31355420, 363.55852806],
    b: [-19.98922558, 70.04806399, -75.15487571, 69.08411233],
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct WdlModel {
    pub a: [f64; 4],
    pub b: [f64; 4],
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct WdlParams {
    a: f64,
    b: f64,
}

impl WdlModel {
    pub fn params(&self, board: &Board) -> WdlParams {
        use PieceType::*;

        let mat = board.piece_bbs[Pawn].count()
            + 3 * board.piece_bbs[Knight].count()
            + 3 * board.piece_bbs[Bishop].count()
            + 5 * board.piece_bbs[Rook].count()
            + 9 * board.piece_bbs[Queen].count();

        let mat = mat.clamp(10, 78) as f64 / 58.0;

        WdlParams {
            a: self.a[0]
                .mul_add(mat, self.a[1])
                .mul_add(mat, self.a[2])
                .mul_add(mat, self.a[3]),
            b: self.b[0]
                .mul_add(mat, self.b[1])
                .mul_add(mat, self.b[2])
                .mul_add(mat, self.b[3]),
        }
    }
}

impl WdlParams {
    pub fn get_wdl(&self, eval: i32) -> (u64, u64, u64) {
        let win_rate = 1000.0 / (1.0 + f64::exp((-eval as f64 + self.a) / self.b));
        let loss_rate = 1000.0 / (1.0 + f64::exp((eval as f64 + self.a) / self.b));
        let draw_rate = 1000.0 - win_rate - loss_rate;

        (win_rate as u64, draw_rate as u64, loss_rate as u64)
    }

    pub fn wdl_normalized(&self, score: i32) -> i32 {
        (100.0 * score as f64 / self.a) as i32
    }
}
