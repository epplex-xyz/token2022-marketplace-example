pub use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::ID as INSTRUCTIONS_ID
};

pub use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use spl_token_2022::onchain::invoke_transfer_checked;

pub use crate::{state::*, errors::*};

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    pub lister: Signer<'info>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = lister,
    )]
    pub lister_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [b"marketplace", marketplace.admin.key().as_ref()],
        bump,
    )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
        mut,
        close = lister,
        seeds = [b"listing", mint.key().as_ref(), marketplace.key().as_ref()],
        bump,
        has_one = lister,
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = listing,
    )]
    pub listing_ata: InterfaceAccount<'info, TokenAccount>,

    // Transfer Hook Accounts
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    /// CHECK: no need to check it out, the invoke_transfer will check for us
    pub metas_account_list: AccountInfo<'info>,
    /// CHECK: no need to check it out, the invoke_transfer will check for us
    pub transfer_hook_program_id: AccountInfo<'info>,
    #[account(address = INSTRUCTIONS_ID)]
    /// CHECK: no need to check it out, we have a fixed address
    pub sysvar_instruction: AccountInfo<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> Delist<'info> {
    pub fn delist(
        &mut self,
        bumps: DelistBumps
    ) -> Result<()> {

        let mint_key = self.mint.key();
        let marketplace_key = self.marketplace.key();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"listing",
            &mint_key.as_ref(),
            &marketplace_key.as_ref(),
            &[bumps.listing],
        ]];

        invoke_transfer_checked(
            &self.token_program.key(),
            self.listing_ata.to_account_info(),
            self.mint.to_account_info(),
            self.lister_ata.to_account_info(),
            self.listing.to_account_info(),
            &[
                self.transfer_hook_program_id.to_account_info(),
                self.metas_account_list.to_account_info(),
                self.sysvar_instruction.to_account_info(),
            ],
            1u64,
            0u8,
            signer_seeds,
        )?;        

        Ok(())
    }
}