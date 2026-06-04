
// ─── Shot types ─────────────────────────────────

// Raw result of a dart throw (independent of game type)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShotResult {
    pub sector: u32,     // 1..20, 25 (bull)
    pub multiplier: u32, // 1 (single), 2 (double), 3 (triple)
}

// Shot result methods
impl ShotResult {
    /// Calculates the total points scored by this shot.
    /// 
    /// Return 0 for invalid shots (e.g. multiplier > 3, sector out of range).
    pub fn points(&self) -> u32 {
        self.sector * self.multiplier
    }
}

// Outcome of applying a shot to the game
#[derive(Debug, Clone)]
pub struct ShotOutcome {
    /// Points actually scored (0 if bust)
    pub score: u32,
    /// Is this shot a bust?
    pub is_bust: bool,
    /// Is the player's turn over?
    pub turn_over: bool,
    /// Is the game finished?
    pub game_over: bool,
    /// Optional message (e.g. "Bust!", "Double out required")
    pub message: Option<String>,
}

// Player generic struct with game-specific data and history
#[derive(Clone)]
pub struct Player<D> {
    pub name: String,
    pub position: usize,
    pub data: D,
    pub history: Vec<TurnRecord>,
}

// Record of a player's turn (up to N darts, total score, bust status)
#[derive(Clone)]
pub struct TurnRecord {
    pub darts: Vec<ShotResult>,
    pub score: u32,
    pub bust: bool,
}

// Game phase contains infos about the current state of the game (playing, finished with winner index)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Playing,
    Finished(usize), // winner index
}

// Game variant trait defines the rules and logic for a specific dart game type
pub trait GameVariant: Send {
    /// Player-specific data type for this variant
    type PlayerData: Clone + Send;

    /// Initializes player data at the start of the game
    /// 
    /// Return the initial data for a player (e.g. starting score for X01).
    fn init_player_data(&self) -> Self::PlayerData;

    /// Number of darts allowed per turn (default: 3)
    /// 
    /// Return the number of darts each player can throw in a single turn.
    fn darts_per_turn(&self) -> u32 { 3 }

    /// Maximum number of players allowed (default: 4)
    /// 
    /// Return the maximum number of players that can participate in this game variant.
    fn max_players(&self) -> u32 { 4 }

    /// Name of the game variant (e.g. "X01", "Cricket")
    /// 
    /// Return a human-readable name for this game variant, used in UI and summaries.
    fn name(&self) -> &'static str;

    /// Resolves a shot against the player's data and returns the outcome.
    /// 
    /// This is where the core game logic lives: calculating scores, checking for busts,
    /// determining if the turn or game is over, and returning any relevant messages.
    fn resolve_shot(&self, player: &mut Self::PlayerData, shot: ShotResult) -> ShotOutcome;

    /// Checks if any player has won the game based on their data.
    /// 
    /// This is used to determine if the game has ended after a shot is applied.
    fn check_winner(&self, _players: &[Self::PlayerData]) -> Option<usize> {
        None
    }
}

// Game session struct that manages the state of an ongoing game, including players, current turn, and phase
pub struct GameSession<V: GameVariant> {
    pub variant: V,                             // The game variant being played (e.g. X01, Cricket)
    pub players: Vec<Player<V::PlayerData>>,    // List of players with their data and history
    pub current_player: usize,                  // Index of the current player in the players vector
    pub current_dart: u32,                      // 0..V::darts_per_turn()
    pub phase: GamePhase,                       // Current phase of the game (playing, finished)
}

// Implementation of game session methods, including applying shots, advancing turns, and summarizing state
impl<V: GameVariant> GameSession<V> {
    /// Creates a new game session with the specified variant and player names.
    /// 
    /// Arguments:
    /// - variant: The game variant to be played (e.g. X01, Cricket).
    /// - names: A vector of player names.
    pub fn new(variant: V, names: Vec<String>) -> Self {
        let players = names
            .into_iter()
            .enumerate()
            .map(|(i, name)| Player {
                name,
                position: i,
                data: variant.init_player_data(),
                history: vec![],
            })
            .collect();

        GameSession {
            variant,
            players,
            current_player: 0,
            current_dart: 0,
            phase: GamePhase::Playing,
        }
    }

    /// Applies a shot for the current player and updates the game state accordingly.
    /// 
    /// Aruments:
    /// - shot: The result of the dart throw (sector and multiplier).
    /// 
    /// Return a `ShotOutcome` that includes the score for this shot, whether it was a bust, if the turn is over, and if the game is finished.
    pub fn apply_shot(&mut self, shot: ShotResult) -> ShotOutcome {
        if !matches!(self.phase, GamePhase::Playing) {
            return ShotOutcome {
                score: 0,
                is_bust: false,
                turn_over: false,
                game_over: true,
                message: Some("Game is already over".into()),
            };
        }

        let player = &mut self.players[self.current_player];
        let outcome = self.variant.resolve_shot(&mut player.data, shot);

        // Increment dart count and determine if the turn ends
        self.current_dart += 1;
        let turn_max_reached = self.current_dart >= self.variant.darts_per_turn();
        let turn_ends = outcome.turn_over || turn_max_reached;

        // Log in history — start a new turn record if the previous one is complete
        let last_is_complete = player
            .history
            .last()
            .map(|t| t.darts.len() >= self.variant.darts_per_turn() as usize || t.bust)
            .unwrap_or(true);

        if last_is_complete {
            player.history.push(TurnRecord {
                darts: vec![shot],
                score: outcome.score,
                bust: outcome.is_bust,
            });
        } else if let Some(last_turn) = player.history.last_mut() {
            last_turn.darts.push(shot);
            last_turn.score = last_turn.score.saturating_add(outcome.score);
            last_turn.bust = outcome.is_bust;
        }

        // Check for game over
        if outcome.game_over
            || self
                .variant
                .check_winner(
                    &self
                        .players
                        .iter()
                        .map(|p| p.data.clone())
                        .collect::<Vec<_>>(),
                )
                == Some(self.current_player)
        {
            self.phase = GamePhase::Finished(self.current_player);
            return ShotOutcome {
                turn_over: true,
                game_over: true,
                ..outcome
            };
        }

        // Advance to next player if the turn is over
        if turn_ends {
            self.advance_to_next_player();
        }

        ShotOutcome {
            turn_over: turn_ends,
            ..outcome
        }
    }

    /// Advances the game to the next player's turn, resetting the dart count.
    /// 
    /// REturn nothing, but update the `current_player` index and reset `current_dart` to 0.
    fn advance_to_next_player(&mut self) {
        let next = (self.current_player + 1) % self.players.len();
        self.current_player = next;
        self.current_dart = 0;
    }

    /// Checks if the current player's turn is over based on the number of darts thrown.
    /// 
    /// Return true if the current player has thrown the maximum number of darts for their turn, false otherwise.
    pub fn is_turn_over(&self) -> bool {
        self.current_dart >= self.variant.darts_per_turn()
    }

    /// Readable summary of the current state
    pub fn summary(&self) -> String {
        let p = &self.players[self.current_player];
        format!(
            "{}'s turn — dart {}/{} ({})",
            p.name,
            self.current_dart + 1,
            self.variant.darts_per_turn(),
            self.variant.name(),
        )
    }
}