use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use reqwest::Client;

declare_id!("your_program_id_here");

#[program]
pub mod project_management {
    use super::*;

    pub fn add_task(ctx: Context<AddTask>, id: String, title: String, description: String, status: String, concept: u64, testing: u64, production: u64) -> ProgramResult {
        let task = &mut ctx.accounts.task;
        task.id = id;
        task.title = title;
        task.description = description;
        task.status = status;
        task.rewards = Rewards {
            concept,
            testing,
            production,
        };
        Ok(())
    }

    pub fn add_contributor(
        ctx: Context<AddContributor>,
        id: String,
        name: String,
        email: String,
        icla_version: String,
        ipfs_hash: String,
    ) -> ProgramResult {
        let contributor = &mut ctx.accounts.contributor;
        contributor.id = id;
        contributor.name = name;
        contributor.email = email;
        contributor.wallet_address = ctx.accounts.user.key();

        let icla_acceptance = &mut ctx.accounts.icla_acceptance;
        icla_acceptance.contributor_id = id;
        icla_acceptance.icla_version = icla_version;
        icla_acceptance.accepted_at = Clock::get()?.unix_timestamp;

        // Validate the ICLA version and IPFS hash
        let icla = ICLA {
            version: icla_version,
            ipfs_hash,
        };

        // TODO: Retrieve the ICLA from IPFS using the provided hash and compare with the accepted version

        // Check if the refundable SPL token transfer is confirmed
        let refundable_amount = 1_000_000; // Adjust the amount as needed
        if ctx.accounts.user.lamports() < refundable_amount {
            return Err(ErrorCode::InsufficientRefundableAmount.into());
        }

        // Record the ICLA acceptance
        contributor.icla_accepted = true;
        contributor.icla_accepted_at = icla_acceptance.accepted_at;

        // Refund the SPL tokens back to the developer
        let refund_instruction = system_program::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.contributor.key(),
            refundable_amount,
        );
        invoke(
            &refund_instruction,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.contributor.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn add_contribution(ctx: Context<AddContribution>, task_id: String, contributor_id: String, commit_hash: String, merge_stage: String) -> ProgramResult {
        let contribution = &mut ctx.accounts.contribution;
        contribution.task_id = task_id;
        contribution.contributor_id = contributor_id;
        contribution.commit_hash = commit_hash;
        contribution.merge_stage = merge_stage;
        Ok(())
    }

    pub fn assign_task(ctx: Context<AssignTask>, task_id: String, contributor_wallet: String) -> ProgramResult {
        let task = &mut ctx.accounts.task;
        let contributor = &mut ctx.accounts.contributor;

        // Validate the task and contributor
        if task.id != task_id || contributor.wallet_address != contributor_wallet {
            return Err(ErrorCode::InvalidAssignment.into());
        }

        // Update the task status and contributor
        task.status = "assigned".to_string();
        task.assigned_contributor = contributor.wallet_address.to_string();

        Ok(())
    }

    pub fn mint_contribution_nft(ctx: Context<MintContributionNFT>, contribution_id: String) -> ProgramResult {
        let contribution = &ctx.accounts.contribution;
        let task = &ctx.accounts.task;
        let contributor = &ctx.accounts.contributor;

        // Validate the contribution
        if contribution.task_id != task.id || contribution.contributor_id != contributor.id {
            return Err(ErrorCode::InvalidContribution.into());
        }

        // Validate the merge stage
        if contribution.merge_stage != "testing" {
            return Err(ErrorCode::InvalidMergeStage.into());
        }

        // Validate ICLA acceptance
        if ctx.accounts.icla_acceptance.contributor_id != contributor.id {
            return Err(ErrorCode::ICLANotAccepted.into());
        }

        // Calculate the pool amount (65% of the task rewards)
        let pool_amount = task.rewards.testing * 65 / 100;

        // Transfer the pool amount to the pool account
        let transfer_instruction = system_program::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.pool_account.key(),
            pool_amount,
        );
        invoke(
            &transfer_instruction,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.pool_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Update the contribution with the pool amount and timestamp
        contribution.pool_amount = pool_amount;
        contribution.pool_timestamp = Clock::get()?.unix_timestamp;

        // Mint the NFT
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, 1)?;

        Ok(())
    }

    pub fn release_pool_funds(ctx: Context<ReleasePoolFunds>, contribution_id: String) -> ProgramResult {
        let contribution = &mut ctx.accounts.contribution;
        let task = &ctx.accounts.task;
        let contributor = &ctx.accounts.contributor;

        // Validate the contribution
        if contribution.task_id != task.id || contribution.contributor_id != contributor.id {
            return Err(ErrorCode::InvalidContribution.into());
        }

        // Check if the contribution is merged to staging
        if contribution.merge_stage != "staging" {
            return Err(ErrorCode::InvalidMergeStage.into());
        }

        // Calculate the amount to be paid to the contributor (35% of the task rewards)
        let payout_amount = task.rewards.testing * 35 / 100;

        // Transfer the payout amount to the contributor
        let transfer_instruction = system_program::transfer(
            &ctx.accounts.pool_account.key(),
            &ctx.accounts.contributor.key(),
            payout_amount,
        );
        invoke(
            &transfer_instruction,
            &[
                ctx.accounts.pool_account.to_account_info(),
                ctx.accounts.contributor.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Update the contribution status
        contribution.status = "completed".to_string();

        Ok(())
    }

    pub fn burn_stagnant_pool_funds(ctx: Context<BurnStagnantPoolFunds>, contribution_id: String) -> ProgramResult {
        let contribution = &mut ctx.accounts.contribution;
        let task = &ctx.accounts.task;
        let contributor = &ctx.accounts.contributor;

        // Validate the contribution
        if contribution.task_id != task.id || contribution.contributor_id != contributor.id {
            return Err(ErrorCode::InvalidContribution.into());
        }

        // Check if the contribution is stagnant (3 months since pool creation)
        let current_timestamp = Clock::get()?.unix_timestamp;
        if current_timestamp - contribution.pool_timestamp < 3 * 30 * 24 * 60 * 60 {
            return Err(ErrorCode::PoolNotStagnant.into());
        }

        // Burn the remaining pool funds
        let burn_instruction = system_program::transfer(
            &ctx.accounts.pool_account.key(),
            &ctx.accounts.burn_account.key(),
            ctx.accounts.pool_account.lamports(),
        );
        invoke(
            &burn_instruction,
            &[
                ctx.accounts.pool_account.to_account_info(),
                ctx.accounts.burn_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Update the contribution status
        contribution.status = "stagnant".to_string();

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Rewards {
    pub concept: u64,
    pub testing: u64,
    pub production: u64,
}

impl Rewards {
    pub const LEN: usize = 8 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub rewards: Rewards,
    pub assigned_contributor: String,
}

impl Task {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + Rewards::LEN + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Contributor {
    pub id: String,
    pub name: String,
    pub email: String,
    pub wallet_address: Pubkey,
    pub icla_accepted: bool,
    pub icla_accepted_at: i64,
}

impl Contributor {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 1 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Contribution {
    pub task_id: String,
    pub contributor_id: String,
    pub commit_hash: String,
    pub merge_stage: String,
    pub pool_amount: u64,
    pub pool_timestamp: i64,
    pub status: String,
}

impl Contribution {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ICLA {
    pub version: String,
    pub ipfs_hash: String,
}

impl ICLA {
    pub const LEN: usize = 8 + 32 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ICLAAcceptance {
    pub contributor_id: String,
    pub icla_version: String,
    pub accepted_at: i64,
}

impl ICLAAcceptance {
    pub const LEN: usize = 8 + 32 + 32 + 8;
}

#[derive(Accounts)]
pub struct AddTask<'info> {
    #[account(init, payer = user, space = 8 + Task::LEN)]
    pub task: Account<'info, Task>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddContributor<'info> {
    #[account(init, payer = user, space = 8 + Contributor::LEN)]
    pub contributor: Account<'info, Contributor>,
    #[account(init, payer = user, space = 8 + ICLAAcceptance::LEN)]
    pub icla_acceptance: Account<'info, ICLAAcceptance>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddContribution<'info> {
    #[account(init, payer = user, space = 8 + Contribution::LEN)]
    pub contribution: Account<'info, Contribution>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AssignTask<'info> {
    #[account(mut)]
    pub task: Account<'info, Task>,
    #[account(mut)]
    pub contributor: Account<'info, Contributor>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintContributionNFT<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub contribution: Account<'info, Contribution>,
    pub task: Account<'info, Task>,
    pub contributor: Account<'info, Contributor>,
    pub icla_acceptance: Account<'info, ICLAAcceptance>,
    #[account(mut)]
    pub pool_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReleasePoolFunds<'info> {
    #[account(mut)]
    pub contribution: Account<'info, Contribution>,
    pub task: Account<'info, Task>,
    #[account(mut)]
    pub contributor: Account<'info, Contributor>,
    #[account(mut)]
    pub pool_account: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnStagnantPoolFunds<'info> {
    #[account(mut)]
    pub contribution: Account<'info, Contribution>,
    pub task: Account<'info, Task>,
    pub contributor: Account<'info, Contributor>,
    #[account(mut)]
    pub pool_account: AccountInfo<'info>,
    #[account(mut)]
    pub burn_account: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[error]
pub enum ErrorCode {
    #[msg("Invalid contribution")]
    InvalidContribution,
    #[msg("Invalid task assignment")]
    InvalidAssignment,
    #[msg("Invalid merge stage for minting NFT")]
    InvalidMergeStage,
    #[msg("ICLA not accepted by the contributor")]
    ICLANotAccepted,
    #[msg("Insufficient refundable amount")]
    InsufficientRefundableAmount,
    #[msg("Pool is not stagnant yet")]
    PoolNotStagnant,
}

async fn validate_contribution(contribution: &Contribution, github_token: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("https://api.github.com/repos/owner/repo/commits/{}", contribution.commit_hash);
    let response = client
        .get(&url)
        .header("Authorization", format!("token {}", github_token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if response.status().is_success() {
        // Commit exists, validate the merge stage
        let merge_stage = match contribution.merge_stage.as_str() {
            "concept" => "concept",
            "testing" => "staging",
            "production" => "main",
            _ => return Ok(()),
        };

        let url = format!("https://api.github.com/repos/owner/repo/pulls?state=closed&base={}", merge_stage);
        let response = client
            .get(&url)
            .header("Authorization", format!("token {}", github_token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if response.status().is_success() {
            // Pull request merged to the specified branch
            println!("Contribution validated for stage: {}", contribution.merge_stage);
        }
    }

    Ok(())
}