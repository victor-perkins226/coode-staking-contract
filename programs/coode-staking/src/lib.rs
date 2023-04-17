use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint };
use anchor_lang::solana_program::{clock};
use crate::constants::*;

declare_id!("A8FFovyJCj8J235sKfUiorYZB9o8qoLSG3EzGG554Ws");

mod constants {
    use anchor_lang::prelude::Pubkey;

    pub const ADMIN_KEY: Pubkey = anchor_lang::solana_program::pubkey!("3ttYrBAp5D2sTG2gaBjg8EtrZecqBQSBuFRhsqHWPYxX"); 
    pub const COLLECTION_KEY: Pubkey = anchor_lang::solana_program::pubkey!("DyKv1WTgSyyPuHLi3SmFKMcoUDDif2KvRr55N8ZcU2oV");

    pub const DAY_TIME: u32 = 86400;
    pub const STATISTIC_SEEDS: &str = "statistic";
    pub const POOL_SEEDS: &str = "pool";
    pub const POOL_DATA_SEEDS: &str = "pool data";

    pub const START_TIME: u32 = 1681701520; // Mon Apr 17 2023 12:18:40
    pub const DAYS: [u8;13] = [30,31,30,31,31,30,31,30,31,30,29,31,30];

}

#[program]
pub mod degen_staking {
    use super::*;

    use anchor_lang::AccountsClose;
    
    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        let a_statistic = &mut ctx.accounts.statistic;
        a_statistic.staked_count = 0;

        Ok(())
    }

    pub fn stake(ctx: Context<StakeContext>) -> Result<()> {
        let a_user = &ctx.accounts.user;
        let a_statistic = &mut ctx.accounts.statistic;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let a_mint = &ctx.accounts.mint;
        let a_token_program = &ctx.accounts.token_program;
        let a_token_account = &ctx.accounts.token_account;
        let a_edtion = &ctx.accounts.edition;
        let a_metadata_id = &ctx.accounts.metadata_id;
        let m_data = &mut ctx.accounts.metadata.try_borrow_data()?;
        let metadata = mpl_token_metadata::state::Metadata::deserialize(&mut &m_data[..])?;

        let collection_not_proper = metadata
            .data
            .creators
            .as_ref()
            .unwrap()
            .iter()
            .filter(|item| COLLECTION_KEY == item.address && item.verified)
            .count()
            == 0;

        require!(
            !collection_not_proper && metadata.mint == ctx.accounts.mint.key(),
            CustomError::InvalidNft
        );

        let cpi_ctx = CpiContext::new(
            a_token_program.to_account_info(),
            token::Approve {
                to: a_token_account.to_account_info(),
                delegate: a_pool.to_account_info(),
                authority: a_user.to_account_info(),
            },
        );
        token::approve(cpi_ctx, 1)?;

        let instruction = mpl_token_metadata::instruction::freeze_delegated_account(
            a_metadata_id.to_account_info().key(),
            a_pool.to_account_info().key(),
            a_token_account.to_account_info().key(),
            a_edtion.to_account_info().key(),
            a_mint.to_account_info().key(),
        );

        let (_pool, pool_bump) = Pubkey::find_program_address(
            &[POOL_SEEDS.as_ref(), a_user.to_account_info().key.as_ref()],
            ctx.program_id,
        );

        let pool_seeds = &[
            POOL_SEEDS.as_ref(),
            a_user.to_account_info().key.as_ref(),
            &[pool_bump],
        ];

        let pool_signer = &[&pool_seeds[..]];

        anchor_lang::solana_program::program::invoke_signed(
            &instruction,
            &[
                a_metadata_id.to_account_info().clone(),
                a_pool.to_account_info().clone(),
                a_token_account.to_account_info().clone(),
                a_edtion.to_account_info().clone(),
                a_mint.to_account_info().clone(),
            ],
            pool_signer,
        )?;

        a_statistic.staked_count += 1;

        a_pool.user = a_user.to_account_info().key();
        a_pool.staked_count += 1;

        a_pool_data.user = a_user.to_account_info().key();
        a_pool_data.mint = a_mint.to_account_info().key();

        let clock = clock::Clock::get().unwrap();
        a_pool_data.start_time = clock.unix_timestamp as u32;

        Ok(())
    }

    pub fn unstake(ctx: Context<UnstakeContext>) -> Result<()> {
        let a_user = &ctx.accounts.user;
        let a_statistic = &mut ctx.accounts.statistic;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let a_mint = &ctx.accounts.mint;
        let a_edition = &ctx.accounts.edition;
        let a_metadata_id = &ctx.accounts.metadata_id;
        let a_token_account = &ctx.accounts.token_account;
        let a_token_program = &ctx.accounts.token_program;
        let clock = clock::Clock::get().unwrap();
        
        let (_pool, pool_bump) =
            Pubkey::find_program_address(&[
                POOL_SEEDS.as_ref(), 
                a_user.to_account_info().key.as_ref()
        ], ctx.program_id);

        let pool_seeds = &[
            POOL_SEEDS.as_ref(),
            a_user.to_account_info().key.as_ref(),    
            &[pool_bump],
        ];

        let pool_signer = &[&pool_seeds[..]];

         let instuction = mpl_token_metadata::instruction::thaw_delegated_account(
            a_metadata_id.to_account_info().key(),
            a_pool.to_account_info().key(),
            a_token_account.to_account_info().key(),
            a_edition.to_account_info().key(),
            a_mint.to_account_info().key(),
        );

        anchor_lang::solana_program::program::invoke_signed(
            &instuction,
            &[
                a_metadata_id.to_account_info().clone(),
                a_pool.to_account_info().clone(),
                a_token_account.to_account_info().clone(),
                a_edition.to_account_info().clone(),
                a_mint.to_account_info().clone(),
            ],
            pool_signer,
        )?;

        let cpi_ctx = CpiContext::new(
            a_token_program.to_account_info(),
            token::Revoke {
                source: a_token_account.to_account_info(),
                authority: a_user.to_account_info(),
            },
        );
        token::revoke(cpi_ctx)?;

        a_statistic.staked_count -= 1;
        a_pool.staked_count -= 1 ;

        let mut start_time = a_pool_data.start_time;
        let mut current_time = clock.unix_timestamp as u32;
        let mut end_time = START_TIME;
        let mut total_reward = a_pool.total_reward;
        if start_time < current_time {
            for i in 0..12 {
                end_time += DAY_TIME * DAYS[i] as u32;
                if start_time <= end_time {
                    if end_time < current_time {
                        if total_reward >5 {
                            total_reward = total_reward - i as u64;
                        } else {
                            total_reward = 5
                        }
                        start_time = end_time;
                        msg!("i: {}, total_reward: {}", i, total_reward);
                    }
                    else {
                         if total_reward >5 {
                            total_reward = total_reward - i as u64;
                        } else {
                            total_reward = 5
                        }
                        msg!("i: {}, total_reward {}", i, total_reward);
                        break;
                    }
                }
        
            }
        }

        a_pool.total_reward = total_reward;

        Ok(())
    }

    pub fn claim(ctx: Context<ClaimContext>) -> Result<()> {
        let a_user = &ctx.accounts.user;
        let a_statistic = &mut ctx.accounts.statistic;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let clock = clock::Clock::get().unwrap();
        
        a_statistic.staked_count -= 1;
        a_pool.staked_count -= 1 ;

        let mut start_time = a_pool_data.start_time;
        let mut current_time = clock.unix_timestamp as u32;
        let mut end_time = START_TIME;
        let mut total_reward = a_pool.total_reward;
        if start_time < current_time {
            for i in 0..12 {
                end_time += DAY_TIME * DAYS[i] as u32;
                if start_time <= end_time {
                    if end_time < current_time {
                        if total_reward >5 {
                            total_reward = total_reward - i as u64;
                        } else {
                            total_reward = 5
                        }
                        start_time = end_time;
                        msg!("i: {}, total_reward: {}", i, total_reward);
                    }
                    else {
                         if total_reward >5 {
                            total_reward = total_reward - i as u64;
                        } else {
                            total_reward = 5
                        }
                        msg!("i: {}, total_reward {}", i, total_reward);
                        break;
                    }
                }
        
            }
        }

        a_pool.total_reward = total_reward;
        Ok(())

    }

}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    #[account(init, seeds = [STATISTIC_SEEDS.as_ref()], bump, payer = admin, space = 8 + 4)]
    pub statistic: Account<'info, Statistic>,
    #[account(mut, constraint = admin.key() == ADMIN_KEY)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct StakeContext<'info> {
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(init_if_needed, seeds = [POOL_SEEDS.as_ref(), user.key().as_ref()], bump, payer = user, space = 8 + 32 + 4 + 8 + 8)]
    pub pool: Account<'info, Pool>,
    #[account(init_if_needed, seeds = [POOL_DATA_SEEDS.as_ref(), user.key().as_ref(), mint.key().as_ref()], bump, payer = user, space = 8 + 32 + 32 + 4 + 4)]
    pub pool_data: Account<'info, PoolData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    /// CHECK: it's not dangerous
    pub metadata: AccountInfo<'info>,
    #[account(mut, constraint = token_account.mint == mint.key() && token_account.owner == user.key() && token_account.amount == 1)]
    pub token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: it's not dangerous
    pub edition: AccountInfo<'info>,
    /// CHECK: it's not dangerous
    pub metadata_id: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnstakeContext<'info> {
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(mut, constraint = pool.user == user.key())]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = pool_data.user == user.key() && pool_data.mint == mint.key())]
    pub pool_data: Account<'info, PoolData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: it's not dangerous
    pub edition: AccountInfo<'info>,
    /// CHECK: it's not dangerous
    pub metadata_id: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimContext<'info> {
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(mut, constraint = pool.user == user.key())]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = pool_data.user == user.key() && pool_data.mint == mint.key())]
    pub pool_data: Account<'info, PoolData>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}


#[account]
pub struct Statistic {
    pub staked_count: u32
}

#[account]
pub struct Pool {
    pub user: Pubkey,
    pub staked_count: u32,
    pub total_reward: u64,
    pub transfer_amount: u64,
}

#[account]
pub struct PoolData {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub start_time: u32,
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid Nft.")]
    InvalidNft,
    #[msg("Transfer too much.")]
    TooMuchTransfer
}
