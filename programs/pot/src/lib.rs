use anchor_lang::prelude::*;
use std::mem::size_of;

declare_id!("GcV7Ucvwg2t1J511GVdgUSnSNgoYXXgYCUHREw1Q1fA3");

#[program]
pub mod pot {

    use std::u64;

    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

    use super::*;

    pub fn create_account(ctx: Context<CreateAccount>, data: CreateAccountArg) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.profile.name = data.name; 
        ctx.accounts.profile.time_created = now;
        ctx.accounts.profile.time_updated= now;
        Ok(())
    }

    pub fn add_stake(ctx: Context<AddStake>, data: AddStakeArg) -> Result<()> {
        msg!("stake_account: {:?}", &ctx.accounts.stake_account);

        const RENT:u64 = LAMPORTS_PER_SOL / 10000 * 9;

        let from_account = &ctx.accounts.signer;
        let to_account = &ctx.accounts.stake_account;

        let mut stake_amount:u64 = data.lamports;
        // haha! someone needs to pay for the rent :p
        if to_account.get_lamports() == 0 {
            stake_amount += RENT;
        }

        let transfer_inst = anchor_lang::solana_program::system_instruction::transfer(
            &from_account.key(), 
            &to_account.key(), 
            stake_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_inst,
            &[
                from_account.to_account_info(),
                to_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        for i in &mut ctx.accounts.stake_list.list {
            if i.staker == from_account.key() {
                i.lamports += data.lamports;
                msg!("stake_list-mid, {:?}", &ctx.accounts.stake_list.list);
                return Ok(());
            }
        }

        let staker:Staker = Staker {staker: from_account.key(), lamports: data.lamports}; 
        ctx.accounts.stake_list.list.push(staker); 
        msg!("stake_list, {:?}", &ctx.accounts.stake_list.list);

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


#[account]    
#[derive(Default, Debug)]    
pub struct Staker {    
    staker: Pubkey,    
    lamports: u64,
} 

#[account]    
#[derive(Default, Debug)]    
pub struct StakerList {    
    list: Vec<Staker>   
} 

impl StakerList {
    fn get_account_size(&self) -> usize {
        let mut size = 8 + 4; // anchor id + vector    
        size += size_of::<StakerList>();    
        size += size_of::<Staker>() * (&self.list.len() + 1);    
        size
    }
}

#[account]
#[derive(Debug)]
pub struct CreateAccountArg{
    name: String,
}

#[account]
#[derive(Debug)]
pub struct AddStakeArg{
    target: Pubkey,
    lamports: u64,
}


#[derive(Accounts)]
#[instruction(data: CreateAccountArg)]
pub struct CreateAccount<'info> {
    #[account(init, payer = signer, space = 8+4+size_of::<Profile>()+data.name.len(), seeds = [b"profile", signer.key().as_ref()], bump )]
    pub profile: Account<'info, Profile>,


    #[account(init, payer = signer, space = 8+4+size_of::<StakerList>(), seeds = [b"staker_list", signer.key().as_ref()], bump )]
    pub stake_list: Account<'info, StakerList>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(data: AddStakeArg)]
pub struct AddStake<'info> {
    /// CHECK: it is ok
    #[account(mut, seeds = [b"stake_account", data.target.key().as_ref()], bump )]
    pub stake_account: SystemAccount<'info>,

    #[account(mut, realloc = stake_list.get_account_size() , realloc::payer = signer, realloc::zero = false, 
        seeds = [b"staker_list", data.target.key().as_ref()], bump )]
    pub stake_list: Account<'info, StakerList>,

    #[account(mut, signer)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}



