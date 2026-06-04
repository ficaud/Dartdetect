use crate::dart_game::game::
{
    GameVariant,
    ShotResult,
    ShotOutcome
};

// Implementation of the X01 game variant,
// which includes standard games like 301, 501, etc. with optional double-out rules.
pub struct X01 {
    pub starting_score: u32,
    pub require_double_out: bool,
    pub name: &'static str,
}

// Implementation of the X01 game variant, which includes standard games like 301, 501, etc. with optional double-out rules.
impl X01 {
    /// Creates a new X01 game variant with the specified starting score and double-out requirement.
    /// 
    /// Arguments:
    /// - starting_score: The initial score that players start with (e.g. 301, 501).
    /// - require_double_out: If true, players must finish on a double to win.
    /// 
    /// Returns an instance of the X01 game variant with the appropriate name based on the starting score.
    pub fn new(starting_score: u32, require_double_out: bool) -> Self {
        Self {
            starting_score,
            require_double_out,
            name: match starting_score {
                301 => "301",
                501 => "501",
                701 => "701",
                1001 => "1001",
                _ => "X01",
            },
        }
    }
}

// Implementation of the GameVariant trait for the X01 game,
// defining how shots are resolved and how winners are determined.
impl GameVariant for X01 {
    type PlayerData = u32;

    fn init_player_data(&self) -> u32 {
        self.starting_score
    }

    /// The number of darts allowed per turn in X01 is typically 3.
    fn darts_per_turn(&self) -> u32 { 3 }

    /// Returns the name of the X01 game variant.
    fn name(&self) -> &'static str { self.name }

    /// Resolves a shot for the X01 game variant, applying the rules for scoring, busts, and double-out if required.
    /// 
    /// Arguments:
    /// - player: A mutable reference to the player's current score (remaining points).
    /// - shot: The result of the dart throw (sector and multiplier).
    /// 
    /// Returns a `ShotOutcome` that includes the score for this shot, whether it was a bust,
    /// if the turn is over, and if the game is finished, along with any relevant messages.
    fn resolve_shot(&self, remaining: &mut u32, shot: ShotResult) -> ShotOutcome {
        let thrown = shot.points();

        // bust if exceeding remaining score
        if thrown > *remaining {
            return ShotOutcome {
                score: 0,
                is_bust: true,
                turn_over: true,
                game_over: false,
                message: Some("Bust!".into()),
            };
        }

        // double-out check if required
        if self.require_double_out && thrown == *remaining && shot.multiplier != 2 {
            return ShotOutcome {
                score: 0,
                is_bust: true,
                turn_over: true,
                game_over: false,
                message: Some("Must finish on a double!".into()),
            };
        }

        *remaining -= thrown;
        let game_over = *remaining == 0;

        ShotOutcome {
            score: thrown,
            is_bust: false,
            turn_over: game_over, // a finish ends the turn
            game_over,
            message: if game_over { Some("Game shot!".into()) } else { None },
        }
    }

    /// Checks if any player has won the game by reaching exactly 0 points.
    /// 
    /// Arguments:
    /// - players: A slice of player scores (remaining points).
    /// 
    /// Returns an `Option<usize>` with the index of the winning player if there is a winner,
    /// or `None` if no player has won yet.
    fn check_winner(&self, players: &[u32]) -> Option<usize> {
        players.iter().position(|r| *r == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dart_game::game::{GameSession, ShotResult};

    fn t(sector: u32) -> ShotResult {
        ShotResult { sector, multiplier: 3 }
    }

    fn d(sector: u32) -> ShotResult {
        ShotResult { sector, multiplier: 2 }
    }

    fn s(sector: u32) -> ShotResult {
        ShotResult { sector, multiplier: 1 }
    }

    /// Simulates a perfect 9-dart finish (501, double out).
    /// Sequence: T20 T20 T20 | T20 T20 T20 | T20 T19 D12
    #[test]
    fn nine_dart_finish() {
        let mut session = GameSession::new(
            X01::new(501, true),
            vec!["Phil Taylor".into()],
        );

        // Turn 1 — 180
        let outcome = session.apply_shot(t(20));
        assert!(!outcome.turn_over);
        let outcome = session.apply_shot(t(20));
        assert!(!outcome.turn_over);
        let outcome = session.apply_shot(t(20));
        assert!(outcome.turn_over);
        assert!(!outcome.game_over);

        // Turn 2 — 180 → 321 remaining
        let outcome = session.apply_shot(t(20));
        assert!(!outcome.turn_over);
        let outcome = session.apply_shot(t(20));
        assert!(!outcome.turn_over);
        let outcome = session.apply_shot(t(20));
        assert!(outcome.turn_over);
        assert!(!outcome.game_over);

        // Turn 3 — 141 checkout: T20 T19 D12
        let outcome = session.apply_shot(t(20));   // 60 → 81 remaining
        assert!(!outcome.turn_over);
        assert_eq!(session.players[0].data, 81);

        let outcome = session.apply_shot(t(19));   // 57 → 24 remaining
        assert!(!outcome.turn_over);
        assert_eq!(session.players[0].data, 24);

        let outcome = session.apply_shot(d(12));   // D12 → 0
        assert!(outcome.game_over);
        assert!(outcome.turn_over);
        assert_eq!(outcome.message.as_deref(), Some("Game shot!"));
        assert_eq!(session.phase, crate::dart_game::game::GamePhase::Finished(0));

        // Total of 9 darts
        assert_eq!(session.players[0].history.len(), 3);
        let total_darts: usize = session.players[0].history.iter()
            .map(|t| t.darts.len())
            .sum();
        assert_eq!(total_darts, 9);
    }

    /// Two-player match where both have a slow start, then Player 0 checks out.
    /// Sequence:
    ///   P0:  60  60  60 → 180 (321 left)
    ///   P1:  20  20  20 →  60 (441 left)
    ///   P0: 100 100 100 → bust (exceeds 321)
    ///   P1:  60  60  60 → 180 (261 left)
    ///   P0: 100 100 100 → bust again
    ///   P1: 141 checkout: T20 T19 D12 → 261 - 141 = 120 (not finished, need double)
    ///   P0:  60 100  60 → bust
    ///   P1:  T20  T20  D12 → 120 - 120 = 0 🎯
    #[test]
    fn two_player_match() {
        let mut session = GameSession::new(
            X01::new(501, true),
            vec!["Alice".into(), "Bob".into()],
        );

        println!("=== Match start: Alice vs Bob (501, double out) ===\n");

        // ─── Round 1 ───────────────────────────────
        println!("── Round 1 ──");

        // Alice
        println!("Alice:");
        let _ = session.apply_shot(t(20));
        let _ = session.apply_shot(t(20));
        let outcome = session.apply_shot(t(20));
        println!("  T20 T20 T20 → 180 ({} left)", session.players[0].data);
        assert!(outcome.turn_over);
        assert_eq!(session.players[0].data, 321);

        // Bob
        println!("Bob:");
        let _ = session.apply_shot(s(20));
        let _ = session.apply_shot(s(20));
        let outcome = session.apply_shot(s(20));
        println!("  S20 S20 S20 → 60 ({} left)", session.players[1].data);
        assert!(outcome.turn_over);
        assert_eq!(session.players[1].data, 441);

        // ─── Round 2 ───────────────────────────────
        println!("\n── Round 2 ──");

        // Alice
        println!("Alice:");
        let _ = session.apply_shot(s(20)); // 321 → 301
        let _ = session.apply_shot(s(20)); // 301 → 281
        let outcome = session.apply_shot(t(20)); // 281 - 60 = 221
        println!("  S20 S20 T20 → 100 ({} left)", session.players[0].data);
        assert!(!outcome.is_bust);
        assert!(outcome.turn_over);
        assert_eq!(session.players[0].data, 221);

        // Bob
        println!("Bob:");
        let _ = session.apply_shot(t(20));
        let _ = session.apply_shot(t(20));
        let outcome = session.apply_shot(t(20));
        println!("  T20 T20 T20 → 180 ({} left)", session.players[1].data);
        assert!(outcome.turn_over);
        assert_eq!(session.players[1].data, 261);

        // ─── Round 3 ───────────────────────────────
        println!("\n── Round 3 ──");

        // Alice
        println!("Alice:");
        let _ = session.apply_shot(t(20)); // 221 - 60 = 161
        let _ = session.apply_shot(t(20)); // 161 - 60 = 101
        let outcome = session.apply_shot(t(20)); // 101 - 60 = 41
        println!("  T20 T20 T20 → 180 ({} left)", session.players[0].data);
        assert!(!outcome.is_bust);
        assert!(outcome.turn_over);
        assert_eq!(session.players[0].data, 41);

        // Bob
        println!("Bob:");
        let _ = session.apply_shot(t(20));
        let _ = session.apply_shot(t(20));
        let _ = session.apply_shot(t(20));
        println!("  T20 T20 T20 → 180 ({} left)", session.players[1].data);
        assert_eq!(session.players[1].data, 81);

        // ─── Round 4 ───────────────────────────────
        println!("\n── Round 4 ──");

        // Alice
        println!("Alice:");
        let _ = session.apply_shot(s(1));  // 41 - 1 = 40
        let _ = session.apply_shot(d(10)); // 40 - 20 = 20
        let outcome = session.apply_shot(s(20)); // S20 on 20 → bust (not a double)
        println!("  S1 D10 S20 → BUST! ({} left, needs double)", session.players[0].data);
        assert!(outcome.is_bust);
        assert_eq!(session.players[0].data, 20);

        // Bob
        println!("Bob:");
        let _ = session.apply_shot(t(19)); // 81 - 57 = 24
        let outcome = session.apply_shot(d(12)); // D12 → 0 🎯
        println!("  T19 D12 → 0 🏆 Bob wins!");
        assert!(outcome.game_over);
        assert_eq!(session.phase, crate::dart_game::game::GamePhase::Finished(1));
        assert_eq!(session.players[1].data, 0);

        println!("\n=== Match finished: Bob wins ===");
    }

}