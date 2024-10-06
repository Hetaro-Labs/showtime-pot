use anchor_lang::prelude::*;

declare_id!("GcV7Ucvwg2t1J511GVdgUSnSNgoYXXgYCUHREw1Q1fA3");

#[program]
pub mod pot {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
