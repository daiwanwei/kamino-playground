use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use kamino_lend::kamino_lending;
#[derive(Accounts)]
pub struct Initialize {}

pub fn handler(ctx: Context<Initialize>, bump: u8) -> Result<()> {
    // kamino_lending::init_lending_market(ctx.accounts.lending_market.
    // to_account_info()? invoke_signed(
    //     &kamino_lending::accounts::InitLendingMarket {},
    //     &[
    //         ctx.accounts.lending_market_owner.to_account_info().key.as_ref(),
    //         ctx.accounts.lending_market_authority.to_account_info().key.as_ref(),
    //     ],
    //     &[&b"init_lending_market"[..]],
    // )?;
    Ok(())
}
