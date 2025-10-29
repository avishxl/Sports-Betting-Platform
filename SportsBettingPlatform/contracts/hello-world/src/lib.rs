#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, log, symbol_short, Address, Env, String, Symbol,
};

// Structure to store bet details
#[contracttype]
#[derive(Clone)]
pub struct Bet {
    pub bet_id: u64,
    pub bettor: Address,
    pub amount: i128,
    pub predicted_winner: String,
    pub is_settled: bool,
    pub won: bool,
}

// Structure to store match details
#[contracttype]
#[derive(Clone)]
pub struct Match {
    pub match_id: u64,
    pub team_a: String,
    pub team_b: String,
    pub winner: String,
    pub is_finished: bool,
    pub total_pool: i128,
}

// Mapping match_id to Match
#[contracttype]
pub enum MatchBook {
    Match(u64),
}

// Mapping bet_id to Bet
#[contracttype]
pub enum BetBook {
    Bet(u64),
}

// Counter for matches
const MATCH_COUNT: Symbol = symbol_short!("M_COUNT");
// Counter for bets
const BET_COUNT: Symbol = symbol_short!("B_COUNT");

#[contract]
pub struct SportsBettingContract;

#[contractimpl]
impl SportsBettingContract {
    // Function 1: Create a new match (Admin only)
    pub fn create_match(env: Env, team_a: String, team_b: String) -> u64 {
        let mut match_count: u64 = env.storage().instance().get(&MATCH_COUNT).unwrap_or(0);
        match_count += 1;

        let new_match = Match {
            match_id: match_count,
            team_a: team_a.clone(),
            team_b: team_b.clone(),
            winner: String::from_str(&env, ""),
            is_finished: false,
            total_pool: 0,
        };

        env.storage()
            .instance()
            .set(&MatchBook::Match(match_count), &new_match);
        env.storage().instance().set(&MATCH_COUNT, &match_count);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Match created with ID: {}", match_count);
        match_count
    }

    // Function 2: Place a bet on a match
    pub fn place_bet(
        env: Env,
        bettor: Address,
        match_id: u64,
        amount: i128,
        predicted_winner: String,
    ) -> u64 {
        // Verify the bettor
        bettor.require_auth();

        // Get match details
        let mut match_data = Self::view_match(env.clone(), match_id);

        // Ensure match exists and is not finished
        if match_data.match_id == 0 {
            log!(&env, "Match does not exist");
            panic!("Match does not exist");
        }

        if match_data.is_finished {
            log!(&env, "Match already finished, cannot place bet");
            panic!("Match already finished");
        }

        // Create bet
        let mut bet_count: u64 = env.storage().instance().get(&BET_COUNT).unwrap_or(0);
        bet_count += 1;

        let new_bet = Bet {
            bet_id: bet_count,
            bettor: bettor.clone(),
            amount: amount,
            predicted_winner: predicted_winner.clone(),
            is_settled: false,
            won: false,
        };

        // Update match pool
        match_data.total_pool += amount;

        env.storage()
            .instance()
            .set(&BetBook::Bet(bet_count), &new_bet);
        env.storage()
            .instance()
            .set(&MatchBook::Match(match_id), &match_data);
        env.storage().instance().set(&BET_COUNT, &bet_count);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Bet placed with ID: {}", bet_count);
        bet_count
    }

    // Function 3: Declare match winner and settle bets (Admin only)
    pub fn declare_winner(env: Env, match_id: u64, winner: String) {
        let mut match_data = Self::view_match(env.clone(), match_id);

        if match_data.match_id == 0 {
            log!(&env, "Match does not exist");
            panic!("Match does not exist");
        }

        if match_data.is_finished {
            log!(&env, "Match already finished");
            panic!("Match already finished");
        }

        // Set winner and mark match as finished
        match_data.winner = winner.clone();
        match_data.is_finished = true;

        env.storage()
            .instance()
            .set(&MatchBook::Match(match_id), &match_data);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Winner declared for match ID: {}", match_id);
    }

    // Function 4: Claim winnings for a bet
    pub fn claim_winnings(env: Env, bet_id: u64) -> i128 {
        let mut bet_data = Self::view_bet(env.clone(), bet_id);

        if bet_data.bet_id == 0 {
            log!(&env, "Bet does not exist");
            panic!("Bet does not exist");
        }

        // Verify the bettor
        bet_data.bettor.require_auth();

        if bet_data.is_settled {
            log!(&env, "Bet already settled");
            panic!("Bet already settled");
        }

        // Get associated match
        let match_data = Self::view_match(env.clone(), bet_id);

        if !match_data.is_finished {
            log!(&env, "Match not finished yet");
            panic!("Match not finished yet");
        }

        // Check if bet won
        let payout: i128;
        if bet_data.predicted_winner == match_data.winner {
            bet_data.won = true;
            // Simple 2x payout for winning bets
            payout = bet_data.amount * 2;
            log!(&env, "Congratulations! You won: {}", payout);
        } else {
            payout = 0;
            log!(&env, "Sorry, you lost this bet");
        }

        bet_data.is_settled = true;
        env.storage()
            .instance()
            .set(&BetBook::Bet(bet_id), &bet_data);
        env.storage().instance().extend_ttl(5000, 5000);

        payout
    }

    // View function: Get match details
    pub fn view_match(env: Env, match_id: u64) -> Match {
        env.storage()
            .instance()
            .get(&MatchBook::Match(match_id))
            .unwrap_or(Match {
                match_id: 0,
                team_a: String::from_str(&env, ""),
                team_b: String::from_str(&env, ""),
                winner: String::from_str(&env, ""),
                is_finished: false,
                total_pool: 0,
            })
    }

    // View function: Get bet details
    pub fn view_bet(env: Env, bet_id: u64) -> Bet {
        env.storage()
            .instance()
            .get(&BetBook::Bet(bet_id))
            .unwrap_or(Bet {
                bet_id: 0,
                bettor: Address::from_string(&String::from_str(&env, "")),
                amount: 0,
                predicted_winner: String::from_str(&env, ""),
                is_settled: false,
                won: false,
            })
    }
}
