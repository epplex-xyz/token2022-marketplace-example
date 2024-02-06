pub use anchor_lang::prelude::*;

pub use crate::state::Marketplace;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [b"marketplace", admin.key().as_ref()],
        bump,
        space = Marketplace::INIT_SPACE,
    )]
    pub marketplace: Account<'info, Marketplace>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info>{
    pub fn initialize(
        &mut self,
    ) -> Result<()> {
        self.marketplace.set_inner(
            Marketplace {
                admin: self.admin.key(),
            }
        );

        Ok(())
    }
}