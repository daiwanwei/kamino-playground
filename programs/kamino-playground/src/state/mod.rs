use anchor_lang::prelude::*;

#[account]
pub struct Account {
    pub authority: Pubkey,
    pub data: u64,
}
