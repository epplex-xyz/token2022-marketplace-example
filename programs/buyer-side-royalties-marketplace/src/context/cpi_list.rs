use anchor_lang::solana_program::program::invoke;
pub use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::ID as INSTRUCTIONS_ID
};

pub use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use spl_token_2022::instruction::transfer_checked;
use spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi;

pub use crate::{state::*, errors::*};

#[derive(Accounts)]
pub struct List<'info> {
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
        init,
        payer = lister,
        seeds = [b"listing", mint.key().as_ref(), marketplace.key().as_ref()],
        bump,
        space = Listing::INIT_SPACE,
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

impl<'info> List<'info> {
    pub fn list(
        &mut self,
        price: u64,
    ) -> Result<()> {

        self.listing.set_inner(
            Listing {
                lister: self.lister.key(),
                mint: self.mint.key(),
                price,
            }
        );

        let mut cpi_instruction = transfer_checked(
            &self.token_program.key(),
            &self.lister_ata.key(),
            &self.mint.key(),
            &self.listing_ata.key(),
            &self.lister.key(),
            &[], // add them later, to avoid unnecessary clones
            1u64,
            0u8,
        )?;
    
        let mut cpi_account_infos = vec![
            self.lister_ata.to_account_info(),
            self.mint.to_account_info(),
            self.listing_ata.to_account_info(),
            self.lister.to_account_info(),
        ];

        let additional_accounts = vec![
            self.transfer_hook_program_id.to_account_info(),
            self.metas_account_list.to_account_info(),
            self.sysvar_instruction.to_account_info()
        ];

        add_extra_accounts_for_execute_cpi(
            &mut cpi_instruction,
            &mut cpi_account_infos,
            &self.transfer_hook_program_id.key(),
            self.lister_ata.to_account_info(),
            self.mint.to_account_info(),
            self.listing_ata.to_account_info(),
            self.lister.to_account_info(),
            1u64,
            &additional_accounts,
        )?;

        invoke(&cpi_instruction, &cpi_account_infos)?;

        Ok(())
    }
}