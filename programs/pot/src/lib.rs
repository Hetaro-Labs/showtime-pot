use anchor_lang::prelude::*;
use std::mem::{size_of, size_of_val};

declare_id!("GcV7Ucvwg2t1J511GVdgUSnSNgoYXXgYCUHREw1Q1fA3");

#[program]
pub mod pot {
    use super::*;

    pub fn create_profile(ctx: Context<CreateProfile>, data: CreateProfileArg) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.profile.name = data.name; 
        ctx.accounts.profile.time_created = now;
        ctx.accounts.profile.time_updated= now;
        Ok(())
    }

}


#[account]    
#[derive(Default, Debug)]    
pub struct Profile {    
    name: String,    
    time_created: i64,    
    time_updated: i64,    
}    

// Create Profile
#[account]
#[derive(Debug)]
pub struct CreateProfileArg{
    name: String,
}


#[derive(Accounts)]
#[instruction(data: CreateProfileArg)]
pub struct CreateProfile<'info> {
    #[account(init, payer = signer, space = 8+4+size_of::<Profile>()+data.name.len(), seeds = [b"profile", signer.key().as_ref()], bump )]
    pub profile: Account<'info, Profile>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


