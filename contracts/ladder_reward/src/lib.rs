#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

/// LadderReward is an on-chain ranked-ladder reward system for competitive
/// games. An admin opens a season with a fixed reward pool and a `top_n`
/// cap; players self-report their rank during the season; the admin
/// finalizes the ladder at season end; the top `top_n` players then claim
/// tier-based rewards. The contract is recurring-season by design — the
/// same admin can open new seasons with different `season_id`s over time.
#[contract]
pub struct LadderReward;

#[contracttype]
#[derive(Clone)]
pub struct Season {
    pub admin: Address,
    pub reward_pool: u64,
    pub top_n: u32,
    pub finalized: bool,
    pub total_claimed: u64,
}

#[contractimpl]
impl LadderReward {
    /// Open a new ranked season.
    ///
    /// The caller (`admin`) authenticates and is recorded as the season
    /// operator. `reward_pool` is the total amount of in-game reward units
    /// (e.g. loyalty points, off-chain credits — not native XLM) that will
    /// be split across the top `top_n` players. `top_n` must be > 0 and
    /// `reward_pool` must be > 0. The season is created in the *active*
    /// state; it can only be finalized once.
    pub fn create_season(
        env: Env,
        admin: Address,
        season_id: Symbol,
        reward_pool: u64,
        top_n: u32,
    ) {
        admin.require_auth();

        if top_n == 0 {
            panic!("top_n must be greater than zero");
        }
        if reward_pool == 0 {
            panic!("reward_pool must be greater than zero");
        }

        let key = (Symbol::new(&env, "season"), season_id);
        if env.storage().instance().has(&key) {
            panic!("season already exists");
        }

        let season = Season {
            admin,
            reward_pool,
            top_n,
            finalized: false,
            total_claimed: 0,
        };
        env.storage().instance().set(&key, &season);
    }

    /// Self-report a player's rank for a given season.
    ///
    /// This is a soft, on-chain signal — typically used by the off-chain
    /// game server or tournament platform to record what a player claims
    /// their finishing rank to be. The authoritative ladder is still
    /// chosen by the admin in `finalize_season`. `rank` must be > 0;
    /// a player may re-report to overwrite their previous submission.
    pub fn report_rank(env: Env, player: Address, season_id: Symbol, rank: u32) {
        player.require_auth();

        if rank == 0 {
            panic!("rank must be greater than zero");
        }

        let key = (Symbol::new(&env, "report"), season_id, player);
        env.storage().instance().set(&key, &rank);
    }

    /// Admin finalizes a season by submitting the ordered top ladder.
    ///
    /// `top_addresses` must have length exactly `top_n` and is ordered
    /// best-to-worst: index 0 is rank 1, index `top_n - 1` is rank N.
    /// After this call the season is closed for new reports and players
    /// in the ladder can claim their tier reward. Only the original
    /// season admin can call this.
    pub fn finalize_season(
        env: Env,
        admin: Address,
        season_id: Symbol,
        top_addresses: Vec<Address>,
    ) {
        admin.require_auth();

        let key = (Symbol::new(&env, "season"), season_id.clone());
        let mut season: Season = env
            .storage()
            .instance()
            .get(&key)
            .expect("season not found");

        if season.admin != admin {
            panic!("only the season admin can finalize");
        }
        if season.finalized {
            panic!("season already finalized");
        }
        if top_addresses.len() != season.top_n {
            panic!("top_addresses length must equal top_n");
        }

        season.finalized = true;
        env.storage().instance().set(&key, &season);

        let ladder_key = (Symbol::new(&env, "ladder"), season_id);
        env.storage().instance().set(&ladder_key, &top_addresses);
    }

    /// A player claims their tier reward for a finalized season.
    ///
    /// Returns the amount paid out to the caller. The reward is
    /// distributed as: 50% to rank 1, 25% to rank 2, 15% to rank 3,
    /// and the remaining 10% split equally among ranks 4..top_n.
    /// Players outside the top ladder, or who have already claimed,
    /// cannot claim again. This function does not perform any on-chain
    /// asset transfer; the returned amount is the off-chain-creditable
    /// tier share.
    pub fn claim_reward(env: Env, player: Address, season_id: Symbol) -> u64 {
        player.require_auth();

        let season_key = (Symbol::new(&env, "season"), season_id.clone());
        let season: Season = env
            .storage()
            .instance()
            .get(&season_key)
            .expect("season not found");

        if !season.finalized {
            panic!("season not finalized yet");
        }

        let claim_key = (Symbol::new(&env, "claim"), season_id.clone(), player.clone());
        if env
            .storage()
            .instance()
            .get::<_, bool>(&claim_key)
            .unwrap_or(false)
        {
            panic!("reward already claimed");
        }

        let ladder_key = (Symbol::new(&env, "ladder"), season_id.clone());
        let ladder: Vec<Address> = env
            .storage()
            .instance()
            .get(&ladder_key)
            .expect("ladder not set");

        let mut rank: u32 = 0;
        let len = ladder.len();
        for i in 0..len {
            if let Some(addr) = ladder.get(i) {
                if addr == player {
                    rank = i + 1;
                    break;
                }
            }
        }

        if rank == 0 || rank > season.top_n {
            panic!("player is not in the top ladder");
        }

        let payout = Self::tier_reward(season.reward_pool, rank, season.top_n);

        env.storage().instance().set(&claim_key, &true);

        let mut updated = season;
        updated.total_claimed += payout;
        env.storage().instance().set(&season_key, &updated);

        payout
    }

    /// View the remaining unclaimed reward pool for the season.
    ///
    /// Returns `reward_pool - total_claimed`. Useful for dashboards
    /// showing how much of a season's prize is still up for grabs.
    pub fn season_reward(env: Env, season_id: Symbol) -> u64 {
        let key = (Symbol::new(&env, "season"), season_id);
        let season: Season = env
            .storage()
            .instance()
            .get(&key)
            .expect("season not found");

        season.reward_pool - season.total_claimed
    }

    /// Compute the tier reward share for a given rank in a top_n ladder.
    /// Distribution: 50% / 25% / 15% / (10% split among ranks 4..top_n).
    fn tier_reward(pool: u64, rank: u32, top_n: u32) -> u64 {
        if rank == 1 {
            pool * 50 / 100
        } else if rank == 2 {
            pool * 25 / 100
        } else if rank == 3 {
            pool * 15 / 100
        } else if rank <= top_n {
            let remainder = pool * 10 / 100;
            let slots = top_n - 3;
            if slots == 0 {
                0
            } else {
                remainder / u64::from(slots)
            }
        } else {
            0
        }
    }
}
