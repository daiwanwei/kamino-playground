pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7PUjTUEsbk6d84ABAp84v91ZPggk24rantFnNxdEq8V");

#[program]
pub mod kamino_playground {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx, 10)
    }
}
