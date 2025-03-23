use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::piece::{Color::*, Piece, PieceType};
use chess::square::Square;
use crate::evaluate::lookups::CENTER_SQUARES;

use super::tuner::Trace;
use chess::constants::{DARK_SQUARES, LIGHT_SQUARES, RANKS};
use chess::movegen::lookups::BETWEEN;
use super::lookups::PASSED_PAWN_MASKS;
use super::{params::*, Eval, EvalContext, S};

pub const PIECE_SQUARE_TABLES: [[S; Square::COUNT]; PieceType::COUNT] = [
    PARAMS.pawn_psqt,
    PARAMS.knight_psqt, 
    PARAMS.bishop_psqt, 
    PARAMS.rook_psqt, 
    PARAMS.queen_psqt, 
    PARAMS.king_psqt, 
];

impl Eval {
    /// An evaluation score for having a specific piece on the board.
    ///
    /// This more or less corresponds to the classic heuristic values of
    /// 100 (Pawn), 300 (Knight), 300 (Bishop), 500 (Rook), 900 (Queen),
    /// but the values are tuned. 
    ///
    /// The distinction between midgame and engame values means we can be more 
    /// granular. E.g., a bishop is worth more in the endgame than a knight, 
    /// rooks become more valuable in the endgame, etc...
    pub fn material(&self, piece: Piece, trace: &mut impl Trace) -> S {
        trace.add(|t| {
            if piece.color().is_white() {
                t.piece_values[piece.piece_type()] += 1;
            } else {
                t.piece_values[piece.piece_type()] -= 1;
            }
        });

        if piece.color().is_white() {
            PARAMS.piece_values[piece.piece_type()]
        } else {
            -PARAMS.piece_values[piece.piece_type()]
        }
    }

    /// A positional score for each piece and the square it resides on,
    /// as determined by piece-specific "piece-square tables" (PSQTs).
    ///
    /// This captures a ton of different heuristics
    /// - The king should hide in the midgame, but come out in the endgame
    /// - Pawns should be pushed, especially in the endgame
    /// - Controlling the center
    /// - ...
    ///
    /// The tables are stored from black's perspective (so they read easier
    /// in text), so in order to get the correct value for White, we need to
    /// artificially mirror the square vertically.
    pub fn psqt(&self, piece: Piece, sq: Square, trace: &mut impl Trace) -> S {
        let sq = if piece.color().is_white() { sq.flip() } else { sq };

        trace.add(|t| {
            use PieceType::*;
            let term = if piece.color().is_white() { 1 } else { -1 };
            match piece.piece_type() {
                Pawn   => t.pawn_psqt[sq]   += term,
                Knight => t.knight_psqt[sq] += term,
                Bishop => t.bishop_psqt[sq] += term,
                Rook   => t.rook_psqt[sq]   += term,
                Queen  => t.queen_psqt[sq]  += term,
                King   => t.king_psqt[sq]   += term,
            };
        });

        if piece.color().is_white() {
            PIECE_SQUARE_TABLES[piece.piece_type()][sq]
        } else {
            -PIECE_SQUARE_TABLES[piece.piece_type()][sq]
        }
    }

    /// A score for pawns protecting the squares directly in front of the 
    /// friendly king.
    ///
    /// Assign a flat bonus for every pawn that is
    /// - on the three files surrounding the king,
    /// - 1 or 2 ranks in front of the king
    ///
    /// We assign different bonuses depending on how far the shield pawn is 
    /// removed from the king.
    pub fn pawn_shield<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_king = board.kings(us).first();
        let our_pawns = board.pawns(us);

        // Use the passed pawn masks to mask the squares in front of the king.
        let shield_mask = PASSED_PAWN_MASKS[us][our_king];
        let shield_pawns = shield_mask & our_pawns;

        for pawn in shield_pawns {
            // Get the (vertical) distance from the king, clamped to [1, 2],
            // and use it to assign the associated bonus.
            let distance = pawn.vdistance(our_king).min(3) - 1;
            total += PARAMS.pawn_shield[distance];

            trace.add(|t| t.pawn_shield[distance] += if WHITE { 1 } else { -1 });
        }

        total
    }

    // A score for pawns approaching the squares directly in front of the enemy
    // king.
    //
    /// Assign a flat bonus for every pawn that is
    /// - on the three files surrounding the king,
    /// - 1 or 2 ranks in front of the king
    ///
    /// We assign different bonuses depending on how far the shield pawn is 
    /// removed from the king.
    pub fn pawn_storm<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let them = !us;
        let their_king = board.kings(them).first();
        let our_pawns = board.pawns(us);

        // Use the passed pawn masks to mask the squares in front of the king.
        let storm_mask = PASSED_PAWN_MASKS[them][their_king];
        let storm_pawns = storm_mask & our_pawns;

        for pawn in storm_pawns {
            // Get the (vertical) distance from the king, clamped to [1, 2],
            // and use it to assign the associated bonus.
            let distance = pawn.vdistance(their_king).min(3) - 1;
            total += PARAMS.pawn_storm[distance];

            trace.add(|t| t.pawn_storm[distance] += if WHITE { 1 } else { -1 });
        }

        total
    }

    /// A score for keeping the king close to friendly passed powns, in order to
    /// protect them.
    ///
    /// For every passed pawn, we assign a bonus dependent on how far away they
    /// are from the friendly king. The distance is measured using the Chebyshev
    /// (infinity-, or max-) norm.
    pub fn passers_friendly_king<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace ) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_king = board.kings(us).first();

        for passer in self.pawn_structure.passed_pawns(us) {
            // Get the L_inf distance from the king, and use it to assign the 
            // associated bonus
            let distance = passer.max_dist(our_king);
            total += PARAMS.passers_friendly_king[distance - 1];

            trace.add(|t| t.passers_friendly_king[distance - 1] += if WHITE { 1 } else { -1 });
        }

        total
    }

    /// A penalty for having passers too close to the enemy king.
    ///
    /// For every passed pawn, we assign a penalty dependent on how close they
    /// are from the enemy king. The distance is measured using the Chebyshev
    /// (infinity-, or max-) norm.
    pub fn passers_enemy_king<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let their_king = board.kings(!us).first();

        for passer in self.pawn_structure.passed_pawns(us) {
            // Get the L_inf distance from the king, and use it to assign the 
            // associated bonus
            let distance = passer.max_dist(their_king);
            total += PARAMS.passers_enemy_king[distance - 1];

            trace.add(|t| t.passers_enemy_king[distance - 1] += if WHITE { 1 } else { -1 });
        }

        total
    }

    /// A bonus for knights that are positioned on outpost squares.
    ///
    /// Outpost squares are squares that cannot easily be attacked by pawns,
    /// and are defended by one of our own pawns.
    ///
    /// For the implementation of outpost squares, see [PawnStructure::new].
    pub fn knight_outposts<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let outpost_knights = board.knights(us) & self.pawn_structure.outposts(us);
        let count = outpost_knights.count() as i32;

        trace.add(|t| t.knight_outposts += if WHITE { count } else { -count });

        PARAMS.knight_outposts * count
    }

    /// A bonus for bishops that are positioned on outpost squares.
    ///
    /// Outpost squares are squares that cannot easily be attacked by pawns,
    /// and are defended by one of our own pawns.
    ///
    /// For the implementation of outpost squares, see [PawnStructure::new].
    pub fn bishop_outposts<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let outpost_bishops = board.bishops(us) & self.pawn_structure.outposts(us);
        let count = outpost_bishops.count() as i32;

        trace.add(|t| t.bishop_outposts += if WHITE { count } else { -count });

        PARAMS.bishop_outposts * count
    }

    /// A bonus for having a bishop pair on opposite colored squares.
    ///
    /// This does not actually check the square colors, and just assumes that if
    /// the player has two bishops, they are opposite colored (rather than, say,
    /// two same-color bishops through a promotion)
    pub fn bishop_pair<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };

        if board.bishops(us).count() == 2 {
            trace.add(|t| t.bishop_pair += if WHITE { 1 } else { -1 });

            PARAMS.bishop_pair
        } else {
            S::default()
        }
    }

    /// A bonus for having a rook on an open file
    ///
    /// Open files are files that have no pawns on them, and allow the rook to
    /// move freely along them without pawns blocking them in.
    ///
    /// For the implementation of open files, see [PawnStructure].
    pub fn rook_open_file<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let rooks_on_open = board.rooks(us) & self.pawn_structure.open_files();
        let count = rooks_on_open.count() as i32;

        trace.add(|t| t.rook_open_file += if WHITE { count } else { -count });

        PARAMS.rook_open_file * count
    }

    /// A bonus for having a rook on a semi-open file
    ///
    /// Semi-open files are files that have no friendly pawns on them, but do
    /// have enemy pawns on them. They allow rooks to move somewhat freely,
    /// since they aren't blocked by any friendly pawns.
    ///
    /// For the implementation of semi-open files, see [PawnStructure].
    pub fn rook_semiopen_file<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let rooks_on_semi = board.rooks(us) & self.pawn_structure.semi_open_files(us);
        let count = rooks_on_semi.count() as i32;

        trace.add(|t| t.rook_semiopen_file += if WHITE { count } else { -count });

        PARAMS.rook_semiopen_file * count
    }

    /// A bonus for having connected rooks on the back rank.
    ///
    /// Two rooks count as connected when they are withing direct line-of-sight
    /// of each other and are protecting one another.
    pub fn connected_rooks<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let mut total = S::default();
        let us = if WHITE { White } else { Black };

        let mut rooks = board.rooks(us);
        let back_rank = if WHITE { 0 } else { 7 };

        if let Some(first) = rooks.next() {
            if let Some(second) = rooks.next() {
                let on_back_rank = first.rank() == back_rank && second.rank() == back_rank;
                let connected = BETWEEN[first][second] & board.all_occupied() == Bitboard::EMPTY;

                if on_back_rank && connected {
                    total += PARAMS.connected_rooks;
                    trace.add(|t| t.connected_rooks += if WHITE { 1 } else { -1 });
                }
            }
        }

        total
    }

    /// A bonus for having a major piece (rook or queen) on the 7th/2nd rank.
    ///
    /// The idea is that these are powerful pieces on the 7th rank, because 
    /// they can trap the king on the 8th rank, and attack weak pawns on the 7th
    /// rank.
    ///
    /// As such, the terms assigns a bonus _only if_ the king is on the 8th rank
    /// or there are powns on the 7th.
    pub fn major_on_seventh<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let mut total = S::default();
        let us = if WHITE { White } else { Black };
        let seventh_rank = if WHITE { RANKS[6] } else { RANKS[1] };
        let eighth_rank = if WHITE { RANKS[7] } else { RANKS[0] };
        let pawns_on_seventh = !(board.pawns(!us) & seventh_rank).is_empty();
        let king_on_eighth = !(board.kings(!us) & eighth_rank).is_empty();
        let majors = board.rooks(us) | board.queens(us);

        if pawns_on_seventh || king_on_eighth {
            let count = (majors & seventh_rank).count() as i32;

            trace.add(|t| t.major_on_seventh += if WHITE { count } else { -count });

            total += PARAMS.major_on_seventh * count;
        }

        total
    }

    /// A bonus for having a queen on an open file.
    ///
    /// Identical in spirit and implementation to [Board::rook_open_file]
    pub fn queen_open_file<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let queens_on_open = board.queens(us) & self.pawn_structure.open_files();
        let count = queens_on_open.count() as i32;

        trace.add(|t| t.queen_open_file += if WHITE { count } else { -count });

        PARAMS.queen_open_file * count
    }

    /// A bonus for having a queen on a semi-open file.
    ///
    /// Identical in spirit and implementation to [Board::rook_semiopen_file]
    pub fn queen_semiopen_file<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let queens_on_semi = board.queens(us) 
            & self.pawn_structure.semi_open_files(us)
            & !self.pawn_structure.open_files();
        let count = queens_on_semi.count() as i32;

        trace.add(|t| t.queen_semiopen_file += if WHITE { count } else { -count });

        PARAMS.queen_semiopen_file * count
    }

    /// A score associated with how many squares a piece can move to.
    /// 
    /// This tries to take into account some extra considerations:
    /// 1. Disregard squares attacked by pawns
    /// 2. Disregard squares occupied by blocked pawns
    /// 3. Disregard squares not on the pinray when the piece is pinned
    ///
    /// NOTE: Because this function relies on generating attacked squares for 
    /// every single piece on the board, it is rather expensive. That's why we 
    /// also make it responsible for gathering relevant information derived from 
    /// the attacks to share with other evaluation terms.
    /// I kinda hate this, and it makes the order in which we evaluate the 
    /// individual eval terms important, which feels gross.
    /// FIXME: I'm pretty sure the blocked pawns thing is irrelevant?
    /// It's only relevant if I were to consider xray attacks, but then a lot 
    /// of the other calculated stuff (threats, king zone) would be invalid?
    pub fn mobility<const WHITE: bool>(&self, board: &Board, ctx: &mut EvalContext, trace: &mut impl Trace) -> S {
        use PieceType::*;
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_pawns = board.pawns(us);
        let their_pawns = board.pawns(!us);
        let their_minors = board.knights(!us) | board.bishops(!us);
        let their_rooks = board.rooks(!us);
        let their_queens = board.queens(!us);

        // Pawn threats
        let pawn_attacks = board.pawn_attacks(us);
        ctx.threats[us] |= pawn_attacks;
        ctx.attacked_by[us][Pawn] |= pawn_attacks;

        // King safety, threats and mobility
        let blockers = board.all_occupied();
        let enemy_king_zone = ctx.king_zones[!us];

        let pawn_attacks = board.pawn_attacks(!us);
        let blocked_pawns = our_pawns & their_pawns.backward::<WHITE>();

        let mobility_squares = !pawn_attacks & !blocked_pawns;

        let their_king = board.kings(!us).first();

        for sq in board.knights(us) {
            let attacks = sq.knight_squares();

            ctx.threats[us] |= attacks;
            ctx.attacked_by[us][Knight] |= attacks;

            // King safety
            let king_attacks = enemy_king_zone & attacks;
            ctx.king_attacks[!us] += king_attacks.count();

            // Mobility
            let mut available_squares = attacks & mobility_squares;

            if board.get_pinrays(us).contains(sq) {
                available_squares &= board.get_pinrays(us);
            }

            let sq_count = available_squares.count() as usize;
            total += PARAMS.knight_mobility[sq_count];

            trace.add(|t| t.knight_mobility[sq_count] += if WHITE { 1 } else { -1 });
        }

        for sq in board.bishops(us) {
            let attacks = sq.bishop_squares(blockers);

            ctx.threats[us] |= attacks;
            ctx.attacked_by[us][Bishop] |= attacks;

            // King safety
            let king_attacks = enemy_king_zone & attacks;
            ctx.king_attacks[!us] += king_attacks.count();

            // Long diagonal
            if (attacks & CENTER_SQUARES).count() > 1 {
                total += PARAMS.bishop_long_diagonal;
                trace.add(|t| t.bishop_long_diagonal += if WHITE { 1 } else { -1 });
            }

            // Mobility
            let mut available_squares = attacks & mobility_squares;

            if board.get_pinrays(us).contains(sq) {
                available_squares &= board.get_pinrays(us);
            }

            let sq_count = available_squares.count() as usize;
            total += PARAMS.bishop_mobility[sq_count];

            trace.add(|t| t.bishop_mobility[sq_count] += if WHITE { 1 } else { -1 });
        }

        for sq in board.rooks(us) {
            let attacks = sq.rook_squares(blockers);

            ctx.threats[us] |= attacks;
            ctx.attacked_by[us][Rook] |= attacks;

            // King safety
            let king_attacks = enemy_king_zone & attacks;
            ctx.king_attacks[!us] += king_attacks.count();

            // Mobility
            let mut available_squares = attacks & mobility_squares;

            if board.get_pinrays(us).contains(sq) {
                available_squares &= board.get_pinrays(us);
            }

            let sq_count = available_squares.count() as usize;
            total += PARAMS.rook_mobility[sq_count];

            trace.add(|t| t.rook_mobility[sq_count] += if WHITE { 1 } else { -1 });
        }

        for sq in board.queens(us) {
            let attacks = sq.queen_squares(blockers);

            ctx.threats[us] |= attacks;
            ctx.attacked_by[us][Queen] |= attacks;

            // King safety
            let king_attacks = enemy_king_zone & attacks;
            ctx.king_attacks[!us] += king_attacks.count();

            // Mobility
            let mut available_squares = attacks & mobility_squares;

            if board.get_pinrays(us).contains(sq) {
                available_squares &= board.get_pinrays(us);
            }

            let sq_count = available_squares.count() as usize;
            total += PARAMS.queen_mobility[sq_count];

            trace.add(|t| t.queen_mobility[sq_count] += if WHITE { 1 } else { -1 });
        }

        let king_attacks = ctx.king_zones[us];
        ctx.threats[us] |= king_attacks;
        ctx.attacked_by[us][King] |= king_attacks;

        total
    }

    /// A penalty for the amount of freedom the friendly king has.
    ///
    /// We quantify the "freedom" by placing a hypothetical queen on the king
    /// square, and seeing how many available squares she would have.
    ///
    /// The idea is that having many available queen squares correlates to 
    /// having many slider attack vectors.
    pub fn virtual_mobility<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let king_sq = board.kings(us).first();
        let blockers = board.all_occupied();
        let ours = board.occupied_by(us);
        let available_squares = king_sq.queen_squares(blockers) & !ours;
        let mobility = available_squares.count() as usize;

        trace.add(|t| t.virtual_mobility[mobility] += if WHITE { 1 } else { -1 });

        PARAMS.virtual_mobility[mobility]
    }

    /// A penalty for having many squares in the direct vicinity of the king 
    /// under attack.
    ///
    /// This uses the values that have been aggregated into an [EvalContext]
    /// The heavy lifting has been done in populating the [EvalContext] inside 
    /// [Board::mobility].
    pub fn king_zone<const WHITE: bool>(&self, ctx: &EvalContext, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let attacks = ctx.king_attacks[us];
        let attacks = usize::min(attacks as usize, 15);

        trace.add(|t| t.king_zone[attacks] += if WHITE { 1 } else { -1 });

        PARAMS.king_zone[attacks]
    }

    /// A penalty for having pieces attacked by less valuable pieces.
    ///
    /// There are many levels of granularity possible here, but we distinguish
    /// between:
    /// 
    /// 1. Pawn attacks on minor pieces
    /// 2. Pawn attacks on rooks
    /// 3. Pawn attacks on queens
    /// 4. Minor piece attacks on rooks
    /// 5. Minor piece attacks on queens
    /// 6. Rook attacks on queens
    ///
    /// This uses the values that have been aggregated into an [EvalContext]
    /// The heavy lifting has been done in populating the [EvalContext] inside 
    /// [Board::mobility].
    pub fn threats<const WHITE: bool>(&self, board: &Board, ctx: &EvalContext, trace: &mut impl Trace) -> S {
        use PieceType::*;
        let us = if WHITE { White } else { Black };
        let mut total = S::default();

        trace.add(|t| {
            let perspective = if WHITE { 1 } else { -1 };

            for victim in [Pawn, Knight, Bishop, Rook, Queen] {
                t.pawn_attacks[victim]   += perspective * (board.get_bb(victim, !us) & ctx.attacked_by[us][Pawn]).count() as i32;
                t.knight_attacks[victim] += perspective * (board.get_bb(victim, !us) & ctx.attacked_by[us][Knight]).count() as i32;
                t.bishop_attacks[victim] += perspective * (board.get_bb(victim, !us) & ctx.attacked_by[us][Bishop]).count() as i32;
                t.rook_attacks[victim]   += perspective * (board.get_bb(victim, !us) & ctx.attacked_by[us][Rook]).count() as i32;
                t.queen_attacks[victim]  += perspective * (board.get_bb(victim, !us) & ctx.attacked_by[us][Queen]).count() as i32;
            }
        });

        for victim in [Pawn, Knight, Bishop, Rook, Queen] {
            total += PARAMS.pawn_attacks[victim]   * (board.get_bb(victim, !us) & ctx.attacked_by[us][Pawn]  ).count() as i32;
            total += PARAMS.knight_attacks[victim] * (board.get_bb(victim, !us) & ctx.attacked_by[us][Knight]).count() as i32;
            total += PARAMS.bishop_attacks[victim] * (board.get_bb(victim, !us) & ctx.attacked_by[us][Bishop]).count() as i32;
            total += PARAMS.rook_attacks[victim]   * (board.get_bb(victim, !us) & ctx.attacked_by[us][Rook]  ).count() as i32;
            total += PARAMS.queen_attacks[victim]  * (board.get_bb(victim, !us) & ctx.attacked_by[us][Queen] ).count() as i32;
        }

        total
    }

    pub fn safe_checks<const WHITE: bool>(&self, board: &Board, ctx: &EvalContext, trace: &mut impl Trace) -> S {
        use PieceType::*;
        let us = if WHITE { White } else { Black };
        let their_king = board.kings(!us).first();
        let blockers = board.all_occupied();
        let pawn_pushes = (board.pawns(us)).forward::<WHITE>();
        let safe = !ctx.threats[!us];

        let pawn_checks = their_king.pawn_attacks(!us);
        let knight_checks = their_king.knight_squares();
        let bishop_checks = their_king.bishop_squares(board.all_occupied());
        let rook_checks = their_king.rook_squares(board.all_occupied());
        let queen_checks = bishop_checks | rook_checks;

        let pawn_checks = (ctx.attacked_by[us][Pawn] | pawn_pushes) & pawn_checks;
        let safe_pawn_checks = pawn_checks & !ctx.attacked_by[!us][Pawn];
        let unsafe_pawn_checks = pawn_checks & ctx.attacked_by[!us][Pawn];

        let knight_checks = knight_checks & ctx.attacked_by[us][Knight];
        let safe_knight_checks = knight_checks & safe;
        let unsafe_knight_checks = knight_checks & !safe;

        let bishop_checks = bishop_checks & ctx.attacked_by[us][Bishop];
        let safe_bishop_checks = bishop_checks & safe;
        let unsafe_bishop_checks = bishop_checks & !safe;

        let rook_checks = rook_checks & ctx.attacked_by[us][Rook];
        let safe_rook_checks = rook_checks & safe;
        let unsafe_rook_checks = rook_checks & !safe;

        let queen_checks = queen_checks & ctx.attacked_by[us][Queen];
        let safe_queen_checks = queen_checks & safe;
        let unsafe_queen_checks = queen_checks & !safe;

        trace.add(|t| {
            let perspective = if WHITE { 1 } else { -1 };
            t.safe_checks[Pawn]     += perspective * safe_pawn_checks.count() as i32;
            t.safe_checks[Knight]   += perspective * safe_knight_checks.count() as i32;
            t.safe_checks[Bishop]   += perspective * safe_bishop_checks.count() as i32;
            t.safe_checks[Rook]     += perspective * safe_rook_checks.count() as i32;
            t.safe_checks[Queen]    += perspective * safe_queen_checks.count() as i32;

            t.unsafe_checks[Pawn]   += perspective * unsafe_pawn_checks.count() as i32;
            t.unsafe_checks[Knight] += perspective * unsafe_knight_checks.count() as i32;
            t.unsafe_checks[Bishop] += perspective * unsafe_bishop_checks.count() as i32;
            t.unsafe_checks[Rook]   += perspective * unsafe_rook_checks.count() as i32;
            t.unsafe_checks[Queen]  += perspective * unsafe_queen_checks.count() as i32;
        });

        PARAMS.safe_checks[Pawn]   * safe_pawn_checks.count() as i32   +
        PARAMS.safe_checks[Knight] * safe_knight_checks.count() as i32 +
        PARAMS.safe_checks[Bishop] * safe_bishop_checks.count() as i32 + 
        PARAMS.safe_checks[Rook]   * safe_rook_checks.count() as i32   + 
        PARAMS.safe_checks[Queen]  * safe_queen_checks.count() as i32  +

        PARAMS.unsafe_checks[Pawn]   * unsafe_pawn_checks.count() as i32   +
        PARAMS.unsafe_checks[Knight] * unsafe_knight_checks.count() as i32 +
        PARAMS.unsafe_checks[Bishop] * unsafe_bishop_checks.count() as i32 + 
        PARAMS.unsafe_checks[Rook]   * unsafe_rook_checks.count() as i32   + 
        PARAMS.unsafe_checks[Queen]  * unsafe_queen_checks.count() as i32
    }

    pub fn knight_shelter<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let sheltered = board.knights(us).forward::<WHITE>() & board.pawns(us);
        let count = sheltered.count() as i32;

        trace.add(|t| t.knight_shelter += if WHITE { count } else { -count });

        PARAMS.knight_shelter * count
    }

    pub fn bishop_shelter<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let sheltered = board.bishops(us).forward::<WHITE>() & board.pawns(us);
        let count = sheltered.count() as i32;

        trace.add(|t| t.bishop_shelter += if WHITE { count } else { -count });

        PARAMS.bishop_shelter * count
    }

    pub fn bad_bishops<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let us = if WHITE { White } else { Black };
        let mut total: S = S::default();

        for bishop in board.bishops(us) {
            let squares = if DARK_SQUARES.contains(bishop) { 
                DARK_SQUARES 
            } else { 
                LIGHT_SQUARES 
            };

            let blocking_pawns = (board.pawns(us) & squares).count();

            total += PARAMS.bad_bishops[blocking_pawns as usize];

            trace.add(|t| t.bad_bishops[blocking_pawns as usize] += if WHITE { 1 } else { -1 });
        }

        total
    }

    /// Incremental passed-pawn stuff that only needs to be recomputed when a pawn
    /// or king moves (but also depends on the king, so can't be moved into the 
    /// pawn structure)
    /// NOTE: Should pawn structure include king, so all of this can be moved inside?
    pub fn passers<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let mut total = S::default();
        let us = if WHITE { White } else { Black };
        let them = if WHITE { Black } else { White };
        let our_king = board.kings(us).first();
        let their_king = board.kings(them).first();
        let perspective = if WHITE { 1 } else { -1 };

        for passer in self.pawn_structure.passed_pawns(us) {
            // Distance to friendly king
            let our_king_dist = passer.max_dist(our_king);
            total += PARAMS.passers_friendly_king[our_king_dist - 1];
            trace.add(|t| t.passers_friendly_king[our_king_dist - 1] += perspective);

            // Distance to friendly king
            let their_king_dist = passer.max_dist(their_king);
            total += PARAMS.passers_enemy_king[their_king_dist - 1];
            trace.add(|t| t.passers_enemy_king[their_king_dist - 1] += perspective);
        }

        total
    }

    /// Passed pawn related evaluation that has to be recomputed on each move.
    pub fn volatile_passers<const WHITE: bool>(
        &self,
        board: &Board, 
        ctx: &EvalContext, 
        trace: &mut impl Trace
    ) -> S {
        let us = if WHITE { White } else { Black };
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let them = if WHITE { Black } else { White };
        let our_king = board.kings(us).first();
        let their_king = board.kings(them).first();
        let perspective = if WHITE { 1 } else { -1 };
        let only_kp = board.occupied_by(them) == board.kings(them) | board.pawns(them);
        let tempo = board.current == them;

        for passer in self.pawn_structure.passed_pawns(us) {
            let stop_sq = passer.forward(us).unwrap();
            if board.get_at(stop_sq).is_none() && 
                !ctx.threats[!us].contains(stop_sq)
            {
                let rank = if WHITE { passer.rank() } else { 7 - passer.rank() };
                total += PARAMS.free_passer[rank];
                trace.add(|t| t.free_passer[rank] += if WHITE { 1 } else { -1 });
            }

            if ctx.threats[us].contains(passer) {
                let rank = if WHITE { passer.rank() } else { 7 - passer.rank() };
                total += PARAMS.protected_passer[rank];
                trace.add(|t| t.protected_passer[rank] += if WHITE { 1 } else { -1 });

            }

            // Square rule
            let queening_dist = if WHITE { 7 - passer.rank() } else { passer.rank() };
            let their_king_dist = passer.max_dist(their_king);

            if only_kp 
                && queening_dist <= 4
                && queening_dist < their_king_dist - tempo as usize 
            {
                total += PARAMS.square_rule;
                trace.add(|t| t.square_rule += perspective)
            }
        }

        total
    }

    /// Assign a score to how many pawn pushes are available that _would_
    /// threaten a piece.
    /// TODO: Should this also consider forks?
    ///
    /// Look at all available pushes that would attack non-pawn pieces, that are
    /// on safe squares (= not attacked by them, or attacked by one of their 
    /// non-pawn pieces and defended by us)
    pub fn push_threats<const WHITE: bool>(
        &self,
        board: &Board,
        ctx: &EvalContext,
        trace: &mut impl Trace
    ) -> S {
        use PieceType::*;

        let mut total = S::default();
        let us = if WHITE { White } else { Black };
        let them = if WHITE { Black } else { White };
        let perspective = if WHITE { 1 } else { -1 };
        let third = if WHITE { RANKS[2] } else { RANKS[5] };

        let targets = board.occupied_by(them) & !board.pawns(them);

        // A square is safe if it is 
        // 1. Not attacked by the opponent
        // 2. Attacked by an apponent piece (non-pawn), but also defended by us.
        let mut safe = !ctx.threats[them];
        safe |= ctx.threats[us] & !ctx.attacked_by[them][Pawn];

        let pushes = board.pawns(us).forward::<WHITE>() & !board.all_occupied();
        let double_pushes = (pushes & third).forward::<WHITE>() & !board.all_occupied();

        let safe_pushes = (pushes | double_pushes) & safe;
        let push_attacks = safe_pushes.forward_left::<WHITE>() | safe_pushes.forward_right::<WHITE>();
        let attacked = targets & push_attacks;

        for sq in attacked {
            let attacked = board.get_at(sq).unwrap().piece_type();
            total += PARAMS.push_threats[attacked];
            trace.add(|t| t.push_threats[attacked] += perspective);
        }

        total
    }
}
