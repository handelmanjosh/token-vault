
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, transfer, Transfer};

declare_id!("C6UkYinrZPK8PA52MsLAVworj7KvHL2VLLP85gmjerVR");

#[program]
pub mod token_vault {


    use super::*;
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    pub fn create(ctx: Context<Create>, amount: u64, end_time: u64) -> Result<()> {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.program_token_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                }
            ),
            amount,
        )?;
        ctx.accounts.vault.end_time = end_time;
        ctx.accounts.vault.amount = amount;
        ctx.accounts.vault.mint = ctx.accounts.mint.key();
        ctx.accounts.vault.from = ctx.accounts.signer.key();
        ctx.accounts.vault.to = ctx.accounts.other.key();
        ctx.accounts.vault.from_closed = false;
        ctx.accounts.vault.to_closed = false;
        Ok(())
    }
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        if ctx.accounts.signer.key() != ctx.accounts.vault.to {
            return Err(CustomError::InvalidAccount.into());
        }
        if ctx.accounts.vault.from_closed && ctx.accounts.vault.to_closed {
            return Err(CustomError::AccountClosed.into())
        }
        let time = Clock::get()?.unix_timestamp as u64; 
        if time < ctx.accounts.vault.end_time {
            return Err(CustomError::NotTime.into())
        }
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.program_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.program_authority.to_account_info(),
                },
                &[&[b"auth", &[ctx.bumps.program_authority]]]
            ),
            ctx.accounts.vault.amount
        )?;
        Ok(())
    }
    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        let vault = &mut Vault::try_from_slice(*ctx.accounts.vault.data.borrow_mut())?;
        if vault.to == ctx.accounts.signer.key() {
           vault.to_closed = true;
        } else if vault.from == ctx.accounts.signer.key() {
            vault.from_closed = true;
        } else {
            return Err(CustomError::Unauthorized.into())
        }
        if vault.to_closed && vault.from_closed {
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        to: ctx.accounts.user_token_account.to_account_info(),
                        from: ctx.accounts.program_token_account.to_account_info(),
                        authority: ctx.accounts.program_authority.to_account_info(),
                    },
                    &[&[b"auth", &[ctx.bumps.program_authority]]]
                ),
                vault.amount
            )?;
        }
        Ok(())
    }
    pub fn close(ctx: Context<Close>) -> Result<()> {
        if !ctx.accounts.vault.to_closed || !ctx.accounts.vault.from_closed {
            return Err(CustomError::AccountNotClosed.into())
        }
        Ok(())
    }
}
#[error_code]
pub enum CustomError {
    #[msg("Invalid Account")]
    InvalidAccount,
    #[msg("Not Time")]
    NotTime,
    #[msg("Account Closed")]
    AccountClosed,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Account Not Closed")]
    AccountNotClosed
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [b"auth"],
        bump,
        space = 8,
    )]
    /// CHECK: 
    pub program_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
#[account]
pub struct Vault {
    pub from: Pubkey,
    pub to: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub from_closed: bool,
    pub to_closed: bool,
    pub end_time: u64,
}
#[derive(Accounts)]
pub struct Create<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        seeds = [mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = program_authority,
    )]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = signer,
        seeds = [b"vault", signer.key().as_ref(), other.key().as_ref(), mint.key().as_ref()],
        bump,
        space = 32 + 32 + 32 + 8 + 1 + 1 + 8,
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        seeds = [b"auth"],
        bump,
    )]
    /// CHECK: 
    pub program_authority: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    /// CHECK: 
    pub other: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
    )]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    /// CHECK:
    pub other: AccountInfo<'info>,
    #[account[
        mut,
        close = signer,
        seeds = [b"vault", other.key().as_ref(), signer.key().as_ref(), mint.key().as_ref()],
        bump,
    ]]
    pub vault: Account<'info, Vault>,
    #[account(
        seeds = [b"auth"],
        bump,
    )]
    /// CHECK:
    pub program_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    /// CHECK: 
    pub vault: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump
    )]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"auth"],
        bump,
    )]
    pub program_authority: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut, 
        close = signer,
        seeds = [b"vault", from.key().as_ref(), to.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: 
    pub from: AccountInfo<'info>,
    /// CHECK: 
    pub to: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

