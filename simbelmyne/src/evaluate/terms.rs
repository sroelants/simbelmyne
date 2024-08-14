use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::piece::{Color::*, Piece, PieceType};
use chess::square::Square;
use super::pawn_structure::PawnStructure;
use super::tuner::EvalTrace;
use chess::constants::RANKS;
use chess::movegen::lookups::BETWEEN;
use super::lookups::PASSED_PAWN_MASKS;
use super::piece_square_tables::PIECE_SQUARE_TABLES;
use super::{params::*, EvalContext, S};

/// An evaluation score for having a specific piece on the board.
///
/// This more or less corresponds to the classic heuristic values of
/// 100 (Pawn), 300 (Knight), 300 (Bishop), 500 (Rook), 900 (Queen),
/// but the values are tuned. 
///
/// The distinction between midgame and engame values means we can be more 
/// granular. E.g., a bishop is worth more in the endgame than a knight, 
/// rooks become more valuable in the endgame, etc...
pub fn material(piece: Piece, trace: Option<&mut EvalTrace>) -> S {
    #[cfg(feature = "texel")]
    if let Some(trace) = trace {
        if piece.color().is_white() {
            trace.piece_values[piece.piece_type()] += 1;
        } else {
            trace.piece_values[piece.piece_type()] -= 1;
        }
    }

    if piece.color().is_white() {
        PIECE_VALUES[piece.piece_type()]
    } else {
        -PIECE_VALUES[piece.piece_type()]
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
pub fn psqt(piece: Piece, sq: Square, trace: Option<&mut EvalTrace>) -> S {
    let sq = if piece.color().is_white() { sq.flip() } else { sq };

    #[cfg(feature = "texel")]
    if let Some(trace) = trace {
        use PieceType::*;
        let term = if piece.color().is_white() { 1 } else { -1 };
        match piece.piece_type() {
            Pawn   => trace.pawn_psqt[sq]   += term,
            Knight => trace.knight_psqt[sq] += term,
            Bishop => trace.bishop_psqt[sq] += term,
            Rook   => trace.rook_psqt[sq]   += term,
            Queen  => trace.queen_psqt[sq]  += term,
            King   => trace.king_psqt[sq]   += term,
        };
    }

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
pub fn pawn_shield<const WHITE: bool>(board: &Board, mut trace: Option<&mut EvalTrace>) -> S {
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
        total += PAWN_SHIELD_BONUS[distance];

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace {
            trace.pawn_shield[distance] += if WHITE { 1 } else { -1 };
        }
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
pub fn pawn_storm<const WHITE: bool>(board: &Board, mut trace: Option<&mut EvalTrace>) -> S {
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
        total += PAWN_STORM_BONUS[distance];

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
            trace.pawn_storm[distance] += if WHITE { 1 } else { -1 };
        }
    }

    total
}

/// A score for keeping the king close to friendly passed powns, in order to
/// protect them.
///
/// For every passed pawn, we assign a bonus dependent on how far away they
/// are from the friendly king. The distance is measured using the Chebyshev
/// (infinity-, or max-) norm.
pub fn passers_friendly_king<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, mut trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();

    let us = if WHITE { White } else { Black };
    let our_king = board.kings(us).first();

    for passer in pawn_structure.passed_pawns(us) {
        // Get the L_inf distance from the king, and use it to assign the 
        // associated bonus
        let distance = passer.max_dist(our_king);
        total += PASSERS_FRIENDLY_KING_BONUS[distance - 1];

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
            trace.passers_friendly_king[distance - 1] += if WHITE { 1 } else { -1 };
        }
    }

    total
}

/// A penalty for having passers too close to the enemy king.
///
/// For every passed pawn, we assign a penalty dependent on how close they
/// are from the enemy king. The distance is measured using the Chebyshev
/// (infinity-, or max-) norm.
pub fn passers_enemy_king<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, mut trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();

    let us = if WHITE { White } else { Black };
    let their_king = board.kings(!us).first();

    for passer in pawn_structure.passed_pawns(us) {
        // Get the L_inf distance from the king, and use it to assign the 
        // associated bonus
        let distance = passer.max_dist(their_king);
        total += PASSERS_ENEMY_KING_PENALTY[distance - 1];

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
            trace.passers_enemy_king[distance - 1] += if WHITE { 1 } else { -1 };
        }
    }

    total
}

/// A bonus for knights that are positioned on outpost squares.
///
/// Outpost squares are squares that cannot easily be attacked by pawns,
/// and are defended by one of our own pawns.
///
/// For the implementation of outpost squares, see [PawnStructure::new].
pub fn knight_outposts<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let outpost_knights = board.knights(us) & pawn_structure.outposts(us);
    let count = outpost_knights.count() as i32;

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        trace.knight_outposts += if WHITE { count } else { -count };
    }

    KNIGHT_OUTPOSTS * count
}

/// A bonus for bishops that are positioned on outpost squares.
///
/// Outpost squares are squares that cannot easily be attacked by pawns,
/// and are defended by one of our own pawns.
///
/// For the implementation of outpost squares, see [PawnStructure::new].
pub fn bishop_outposts<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let outpost_bishops = board.bishops(us) & pawn_structure.outposts(us);
    let count = outpost_bishops.count() as i32;

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        trace.bishop_outposts += if WHITE { count } else { -count };
    }

    BISHOP_OUTPOSTS * count
}

/// A bonus for having a bishop pair on opposite colored squares.
///
/// This does not actually check the square colors, and just assumes that if
/// the player has two bishops, they are opposite colored (rather than, say,
/// two same-color bishops through a promotion)
pub fn bishop_pair<const WHITE: bool>(board: &Board, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };

    if board.bishops(us).count() == 2 {
        #[cfg(feature = "texel")]
        if let Some(trace) = trace  {
            trace.bishop_pair += if WHITE { 1 } else { -1 };
        }

        BISHOP_PAIR_BONUS
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
pub fn rook_open_file<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let rooks_on_open = board.rooks(us) & pawn_structure.open_files();
    let count = rooks_on_open.count() as i32;

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        trace.rook_open_file += if WHITE { count } else { -count };
    }

    ROOK_OPEN_FILE_BONUS * count
}

/// A bonus for having a rook on a semi-open file
///
/// Semi-open files are files that have no friendly pawns on them, but do
/// have enemy pawns on them. They allow rooks to move somewhat freely,
/// since they aren't blocked by any friendly pawns.
///
/// For the implementation of semi-open files, see [PawnStructure].
pub fn rook_semiopen_file<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let rooks_on_semi = board.rooks(us) & pawn_structure.semi_open_files(us);
    let count = rooks_on_semi.count() as i32;

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        trace.rook_semiopen_file += if WHITE { count } else { -count };
    }

    ROOK_SEMIOPEN_FILE_BONUS * count
}

/// A bonus for having connected rooks on the back rank.
///
/// Two rooks count as connected when they are withing direct line-of-sight
/// of each other and are protecting one another.
pub fn connected_rooks<const WHITE: bool>(board: &Board, trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();
    let us = if WHITE { White } else { Black };

    let mut rooks = board.rooks(us);
    let back_rank = if WHITE { 0 } else { 7 };

    if let Some(first) = rooks.next() {
        if let Some(second) = rooks.next() {
            let on_back_rank = first.rank() == back_rank && second.rank() == back_rank;
            let connected = BETWEEN[first][second] & board.all_occupied() == Bitboard::EMPTY;

            if on_back_rank && connected {
                total += CONNECTED_ROOKS_BONUS;

                #[cfg(feature = "texel")]
                if let Some(trace) = trace  {
                    trace.connected_rooks += if WHITE { 1 } else { -1 };
                }
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
pub fn major_on_seventh<const WHITE: bool>(board: &Board, trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();
    let us = if WHITE { White } else { Black };
    let seventh_rank = if WHITE { RANKS[6] } else { RANKS[1] };
    let eighth_rank = if WHITE { RANKS[7] } else { RANKS[0] };
    let pawns_on_seventh = !(board.pawns(!us) & seventh_rank).is_empty();
    let king_on_eighth = !(board.kings(!us) & eighth_rank).is_empty();
    let majors = board.rooks(us) | board.queens(us);

    if pawns_on_seventh || king_on_eighth {
        let count = (majors & seventh_rank).count() as i32;

        #[cfg(feature = "texel")]
        if let Some(trace) = trace  {
            trace.major_on_seventh += if WHITE { count } else { -count };
        }

        total += MAJOR_ON_SEVENTH_BONUS * count;
    }

    total
}

/// A bonus for having a queen on an open file.
///
/// Identical in spirit and implementation to [Board::rook_open_file]
pub fn queen_open_file<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let queens_on_open = board.queens(us) & pawn_structure.open_files();
    let count = queens_on_open.count() as i32;

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        trace.queen_open_file += if WHITE { count } else { -count };
    }

    QUEEN_OPEN_FILE_BONUS * count
}

/// A bonus for having a queen on a semi-open file.
///
/// Identical in spirit and implementation to [Board::rook_semiopen_file]
pub fn queen_semiopen_file<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let queens_on_semi = board.queens(us) 
        & pawn_structure.semi_open_files(us)
        & !pawn_structure.open_files();
    let count = queens_on_semi.count() as i32;

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        trace.queen_semiopen_file += if WHITE { count } else { -count };
    }

    QUEEN_SEMIOPEN_FILE_BONUS * count
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
pub fn mobility<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, ctx: &mut EvalContext, mut trace: Option<&mut EvalTrace>) -> S {
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
    ctx.pawn_attacks_on_minors[us] += (pawn_attacks & their_minors).count() as i32;
    ctx.pawn_attacks_on_rooks[us] += (pawn_attacks & their_rooks).count() as i32;
    ctx.pawn_attacks_on_queens[us] += (pawn_attacks & their_queens).count() as i32;
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

        // Threats
        ctx.minor_attacks_on_rooks[us] += (attacks & their_rooks).count() as i32;
        ctx.minor_attacks_on_queens[us] += (attacks & their_queens).count() as i32;

        // Mobility
        let mut available_squares = attacks & mobility_squares;

        if board.get_pinrays(us).contains(sq) {
            available_squares &= board.get_pinrays(us);
        }

        let sq_count = available_squares.count() as usize;
        total += KNIGHT_MOBILITY_BONUS[sq_count];

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
            trace.knight_mobility[sq_count] += if WHITE { 1 } else { -1 };
        }
    }

    for sq in board.bishops(us) {
        let attacks = sq.bishop_squares(blockers);

        ctx.threats[us] |= attacks;
        ctx.attacked_by[us][Bishop] |= attacks;

        // King safety
        let king_attacks = enemy_king_zone & attacks;
        ctx.king_attacks[!us] += king_attacks.count();

        // Threats
        ctx.minor_attacks_on_rooks[us] += (attacks & their_rooks).count() as i32;
        ctx.minor_attacks_on_queens[us] += (attacks & their_queens).count() as i32;

        // Mobility
        let mut available_squares = attacks & mobility_squares;

        if board.get_pinrays(us).contains(sq) {
            available_squares &= board.get_pinrays(us);
        }

        let sq_count = available_squares.count() as usize;
        total += BISHOP_MOBILITY_BONUS[sq_count];

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
            trace.bishop_mobility[sq_count] += if WHITE { 1 } else { -1 };
        }

    }

    for sq in board.rooks(us) {
        let attacks = sq.rook_squares(blockers);

        ctx.threats[us] |= attacks;
        ctx.attacked_by[us][Rook] |= attacks;

        // King safety
        let king_attacks = enemy_king_zone & attacks;
        ctx.king_attacks[!us] += king_attacks.count();

        // Threats
        ctx.rook_attacks_on_queens[us] += (attacks & their_queens).count() as i32;

        // Mobility
        let mut available_squares = attacks & mobility_squares;

        if board.get_pinrays(us).contains(sq) {
            available_squares &= board.get_pinrays(us);
        }

        let sq_count = available_squares.count() as usize;
        total += ROOK_MOBILITY_BONUS[sq_count];

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
            trace.rook_mobility[sq_count] += if WHITE { 1 } else { -1 };
        }

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
        total += QUEEN_MOBILITY_BONUS[sq_count];

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
            trace.queen_mobility[sq_count] += if WHITE { 1 } else { -1 };
        }
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
pub fn virtual_mobility<const WHITE: bool>(board: &Board, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let king_sq = board.kings(us).first();
    let blockers = board.all_occupied();
    let ours = board.occupied_by(us);
    let available_squares = king_sq.queen_squares(blockers) & !ours;
    let mobility = available_squares.count() as usize;

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        trace.virtual_mobility[mobility] += if WHITE { 1 } else { -1 };
    }

    VIRTUAL_MOBILITY_PENALTY[mobility]
}

/// A penalty for having many squares in the direct vicinity of the king 
/// under attack.
///
/// This uses the values that have been aggregated into an [EvalContext]
/// The heavy lifting has been done in populating the [EvalContext] inside 
/// [Board::mobility].
pub fn king_zone<const WHITE: bool>(ctx: &EvalContext, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let attacks = ctx.king_attacks[us];
    let attacks = usize::min(attacks as usize, 15);

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        trace.king_zone[attacks] += if WHITE { 1 } else { -1 };
    }

    KING_ZONE_ATTACKS[attacks]
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
pub fn threats<const WHITE: bool>(ctx: &EvalContext, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        let perspective = if WHITE { 1 } else { -1 };
        trace.pawn_attacks_on_minors  += perspective * ctx.pawn_attacks_on_minors[us];
        trace.pawn_attacks_on_rooks   += perspective * ctx.pawn_attacks_on_rooks[us]; 
        trace.pawn_attacks_on_queens  += perspective * ctx.pawn_attacks_on_queens[us]; 
        trace.minor_attacks_on_rooks  += perspective * ctx.minor_attacks_on_rooks[us]; 
        trace.minor_attacks_on_queens += perspective * ctx.minor_attacks_on_queens[us]; 
        trace.rook_attacks_on_queens  += perspective * ctx.rook_attacks_on_queens[us]; 
    }

      PAWN_ATTACKS_ON_MINORS * ctx.pawn_attacks_on_minors[us] as i32
    + PAWN_ATTACKS_ON_ROOKS * ctx.pawn_attacks_on_rooks[us] as i32
    + PAWN_ATTACKS_ON_QUEENS * ctx.pawn_attacks_on_queens[us] as i32
    + MINOR_ATTACKS_ON_ROOKS * ctx.minor_attacks_on_rooks[us] as i32
    + MINOR_ATTACKS_ON_QUEENS * ctx.minor_attacks_on_queens[us] as i32
    + ROOK_ATTACKS_ON_QUEENS * ctx.rook_attacks_on_queens[us] as i32
}

pub fn safe_checks<const WHITE: bool>(board: &Board, ctx: &EvalContext, trace: Option<&mut EvalTrace>) -> S {
    use PieceType::*;
    let us = if WHITE { White } else { Black };
    let their_king = board.kings(!us).first();
    let blockers = board.all_occupied();
    let knight_checks = their_king.knight_squares();
    let diag_checks = their_king.bishop_squares(board.all_occupied());
    let hv_checks = their_king.rook_squares(board.all_occupied());
    let safe = !ctx.threats[!us];

    let knight_safe_checks = (
        ctx.attacked_by[us][Knight] & knight_checks & safe
    ).count() as i32;

    let bishop_safe_checks = (
        ctx.attacked_by[us][Bishop] & diag_checks & safe
    ).count() as i32;

    let rook_safe_checks = (
        ctx.attacked_by[us][Rook] & hv_checks & safe
    ).count() as i32;

    let queen_safe_checks = (
        ctx.attacked_by[us][Queen] & (hv_checks | diag_checks) & safe
    ).count() as i32;

    #[cfg(feature = "texel")]
    if let Some(trace) = trace  {
        let perspective = if WHITE { 1 } else { -1 };
        trace.safe_checks[Knight] += perspective * knight_safe_checks;
        trace.safe_checks[Bishop] += perspective * bishop_safe_checks;
        trace.safe_checks[Rook]   += perspective * rook_safe_checks;
        trace.safe_checks[Queen]  += perspective * queen_safe_checks;
    }

    SAFE_CHECKS[Knight] * knight_safe_checks +
    SAFE_CHECKS[Bishop] * bishop_safe_checks + 
    SAFE_CHECKS[Rook]   * rook_safe_checks + 
    SAFE_CHECKS[Queen]  * queen_safe_checks
}
