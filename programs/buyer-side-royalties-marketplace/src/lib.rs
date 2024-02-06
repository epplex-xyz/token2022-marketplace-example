use anchor_lang::prelude::*;

pub mod errors;
pub mod state;

pub mod context;
pub use context::*;

declare_id!("AhC8ej2B8LYF86ic16ZFZ4EGAxgcNz7Hvbx1pYdiAHqm");

#[program]
pub mod epplex_marketplace_example {
    use super::*;

    pub fn initialize_marketplace(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize()
    }

    pub fn cpi_list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.list(price)
    }

    pub fn cpi_delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.delist(ctx.bumps)
    }

    pub fn cpi_buy(ctx: Context<Buy>, price: u64) -> Result<()> {
        ctx.accounts.buy(price, ctx.bumps)
    }

}
