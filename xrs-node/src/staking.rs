use solana_sdk::{pubkey::Pubkey, clock::Clock, program_error::ProgramError, sysvar::Sysvar};
use log::info;

#[allow(dead_code)]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount: u64,
    pub last_updated: i64,
    pub reputation_score: u64,
}

#[allow(dead_code)]
pub fn initialize_stake(owner: Pubkey, amount: u64) -> Result<StakeAccount, ProgramError> {
    if amount < 1_000_000_000_000 {
        return Err(ProgramError::InsufficientFunds);
    }
    let stake_account = StakeAccount {
        owner,
        amount,
        last_updated: Clock::get()?.unix_timestamp,
        reputation_score: 100,
    };
    info!("Stake initialized: owner={}, amount={} XRS", owner, amount / 1_000_000_000);
    Ok(stake_account)
}

#[allow(dead_code)]
pub fn stake(stake_account: &mut StakeAccount, amount: u64, total_stake: u64) -> Result<(), ProgramError> {
    if stake_account.amount + amount > total_stake / 10 {
        return Err(ProgramError::InvalidArgument);
    }
    stake_account.amount = stake_account.amount.checked_add(amount).ok_or(ProgramError::ArithmeticOverflow)?;
    stake_account.last_updated = Clock::get()?.unix_timestamp;
    stake_account.reputation_score += 10;
    info!(
        "Staked: owner={}, amount={} XRS, new total={} XRS",
        stake_account.owner,
        amount / 1_000_000_000,
        stake_account.amount / 1_000_000_000
    );
    Ok(())
}

#[allow(dead_code)]
pub fn unstake(stake_account: &mut StakeAccount, amount: u64) -> Result<(), ProgramError> {
    stake_account.amount = stake_account.amount.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
    stake_account.last_updated = Clock::get()?.unix_timestamp;
    info!(
        "Unstaked: owner={}, amount={} XRS, new total={} XRS",
        stake_account.owner,
        amount / 1_000_000_000,
        stake_account.amount / 1_000_000_000
    );
    Ok(())
}

#[allow(dead_code)]
pub fn claim_rewards(stake_account: &mut StakeAccount) -> Result<(), ProgramError> {
    let elapsed = Clock::get()?.unix_timestamp - stake_account.last_updated;
    let apy = 0.07;
    let reward = (stake_account.amount as f64 * apy * elapsed as f64 / (365 * 24 * 3600) as f64) as u64;
    stake_account.amount = stake_account.amount.checked_add(reward).ok_or(ProgramError::ArithmeticOverflow)?;
    stake_account.last_updated = Clock::get()?.unix_timestamp;
    info!(
        "Rewards claimed: owner={}, reward={} XRS, new total={} XRS",
        stake_account.owner,
        reward / 1_000_000_000,
        stake_account.amount / 1_000_000_000
    );
    Ok(())
}

#[allow(dead_code)]
pub fn slash(stake_account: &mut StakeAccount, amount: u64) -> Result<(), ProgramError> {
    stake_account.amount = stake_account.amount.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
    stake_account.reputation_score = stake_account.reputation_score.saturating_sub(25);
    info!(
        "Slashed: owner={}, amount={} XRS, new total={} XRS",
        stake_account.owner,
        amount / 1_000_000_000,
        stake_account.amount / 1_000_000_000
    );
    Ok(())
}