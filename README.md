# ladder_reward

## Project Title
ladder_reward

## Project Description
Competitive online games and esports platforms need a transparent, repeatable way to distribute rewards at the end of each ranked season. Off-chain spreadsheets and centralized ledgers are easy to tamper with, and players have no cryptographic proof that the prize pool is intact or that the final ladder has not been edited after the fact. `ladder_reward` solves this by encoding a recurring season-based ranked-ladder reward system into a Soroban smart contract on Stellar. An admin opens a season with a fixed reward pool and a top-N cap, players self-report their finishing rank, the admin finalizes the official ladder at season end, and the top N players claim tier-based rewards. Because the contract is on-chain, every action — season creation, rank reports, finalization, and individual claims — leaves a permanent, publicly verifiable trail.

## Project Vision
Our long-term vision is to become a trust-minimized backbone for competitive gaming reward distribution on Stellar. By moving the "who won and who gets paid" logic on-chain, we remove the need for players to blindly trust a game operator's bookkeeping. In the future the same primitive should support clan wars, leaderboard tournaments, seasonal battle passes, and cross-game loyalty point economies — all of which need the same building block: a season, a top ladder, and a tier reward curve that nobody can quietly change after the fact.

## Key Features
- **Season-based design.** Admins can open unlimited seasons via distinct `season_id` symbols; the same contract instance serves an entire competitive year.
- **Tiered reward curve.** 50% to rank 1, 25% to rank 2, 15% to rank 3, and the remaining 10% split equally among ranks 4..N — a predictable, fair distribution written directly into the contract.
- **Self-reported ranks with admin finalization.** Players record their claim on-chain via `report_rank`, but the authoritative ladder is committed by the admin through `finalize_season`, preventing a single player from arbitrarily claiming a top rank.
- **Single-claim enforcement.** Each player in the top ladder can only call `claim_reward` once per season — the contract tracks claims and refuses double-spends.
- **Transparent remaining pool.** A simple `season_reward` view returns how much of the prize pool is still unclaimed, so dashboards and community spectators can verify the distribution in real time.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** gaming dApp — see `contracts/ladder_reward/src/lib.rs` for the full ladder_reward business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CAMJQCDFMEPOTF3TDGJSAZHXOWZFTAOTW2J4V5NL7MXFFHPSSTL4G3L7`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/391382b4f3aa7095d8181b05cc13c6d0e446b976905f6df4037d85047d4be4ce`


## Future Scope
- **Multi-asset prize pools.** Extend `Season` to carry a token `Address` and integrate with a Stellar asset contract so the reward pool is held in escrow and released automatically on `claim_reward`.
- **Configurable reward curves.** Replace the hard-coded 50/25/15/10% split with an admin-provided basis-points table stored per season, enabling battle passes, fixed-pool tournaments, and winner-take-all formats.
- **Tie-breaker support.** Allow a final score alongside the rank so the admin can finalize ladders that contain ties without reordering the slot list.
- **Off-chain signature attestation.** Accept a `finalize_season` call co-signed by an oracle or a multisig of referees so a single rogue admin cannot publish a fake ladder.
- **Frontend dashboard.** A small web UI that calls `season_reward` and `claim_reward` via Freighter so players and spectators can watch the pool drain in real time as the top-N claims land.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `ladder_reward` (gaming)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
