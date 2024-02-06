use anchor_lang::prelude::*;

#[account]
pub struct Marketplace {
    pub admin: Pubkey,
}

impl Space for Marketplace {
    const INIT_SPACE: usize = 8 + 32;
}

#[account]
pub struct Listing {
    pub lister: Pubkey,
    pub mint: Pubkey,
    pub price: u64,
}

impl Space for Listing {
    const INIT_SPACE: usize = 8 + 32 + 32 + 8;
}