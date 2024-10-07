use anchor_lang::prelude::*;
use std::mem::size_of;
use std::collections::HashSet;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

declare_id!("GcV7Ucvwg2t1J511GVdgUSnSNgoYXXgYCUHREw1Q1fA3");

const BASIC_RENT:u64 = LAMPORTS_PER_SOL / 10000 * 9;

#[program]
pub mod pot {

    use std::u64;

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

        let from_account = &ctx.accounts.signer;
        let to_account = &ctx.accounts.stake_account;

        let mut stake_amount:u64 = data.lamports;
        // haha! someone needs to pay for the rent :p
        if to_account.get_lamports() == 0 {
            stake_amount += BASIC_RENT;
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

    pub fn create_event(ctx: Context<CreateEvent>, data: CreateEventArg) -> Result<()> {

        let event = &mut ctx.accounts.event;
        // if bet_lamports > 0, this event needs betting, 
        let (is_betted, bet_lamports) = if data.bet_lamports > 0 {
            (false,data.bet_lamports)
        }else{
            // this event doesn't need betting
            (true,0)
        };

        // remove duplication
        let mut guest_keys: HashSet<Pubkey> = data.guests.into_iter().collect();
        guest_keys.insert(ctx.accounts.signer.key());
        //msg!("guest_key, {:?} {:?}", &guest_keys.len(), &guest_keys);
        for k in guest_keys.into_iter() {
            let g = EventGuest {
                key: k,
                is_betted: is_betted,
                agree_vote: false,
                is_claimed: false,
            };
            event.guests.push(g);
        }

        event.event_name = data.event_name;
        event.event_description = data.event_description;
        event.event_start_time = data.event_start_time;
        event.event_end_time = data.event_end_time;
        event.host = ctx.accounts.signer.key();
        event.attendees = vec![];
        event.signed = 0;
        event.bet_lamports = bet_lamports;

        //msg!("event, {:?}", &ctx.accounts.event);
        Ok(())
    }

    pub fn bet_on_event(ctx: Context<BetOnEvent>, data: BetOnEventArg) -> Result<()> {
        let bet_account = &ctx.accounts.bet_pot;
        let host = &ctx.accounts.host;
        //let event = &ctx.accounts.event;
        let signer = &ctx.accounts.signer;

        let mut bet_amount:u64 = ctx.accounts.event.bet_lamports;
        for g in &mut ctx.accounts.event.guests {
            if g.key == signer.key() {
                if g.is_betted {
                    return err!(MyError::BetOnEventAlreadyBetted);
                };


                if bet_account.get_lamports() == 0 {
                    bet_amount += BASIC_RENT;
                }

                let transfer_inst = anchor_lang::solana_program::system_instruction::transfer(
                    &signer.key(), 
                    &bet_account.key(), 
                    bet_amount,
                );
                anchor_lang::solana_program::program::invoke(
                    &transfer_inst,
                    &[
                        signer.to_account_info(),
                        bet_account.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                )?;

                g.is_betted = true;
                msg!("guest: {:?}", &g);
                return Ok(());

            };
        }
        return err!(MyError::BetOnEventNotAGuest);
    }

    pub fn declare_event_attendees(ctx: Context<DeclareEventAttendees>, data: DeclareEventAttendeesArg) -> Result<()> {
        // TODO: check host === signer === event.host
        // TODO: attendees includes in betted guests 
        // TODO: create attendees list 
        // TODO: host vote for itself 

        //let event = &mut ctx.accounts.event;
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
#[derive(Default, Debug)]    
pub struct Event {    
    host: Pubkey,
    guests: Vec<EventGuest>,
    event_name: String,    
    event_description: String, 
    event_start_time: i64,
    event_end_time: i64,
    bet_lamports: u64,
    attendees: Vec<Pubkey>,
    signed: u8,
} 

impl Event {
    pub fn get_init_account_size(data: &CreateEventArg) -> usize {
        let mut size = 8 + 4 * 2; // anchor id + vectorx2    
        size += data.event_name.len();
        size += data.event_description.len();
        size += size_of::<Event>();    
        size += size_of::<EventGuest>() * (data.guests.len() + 1);    
        size += size_of::<Pubkey>() * (data.guests.len() + 1);    
        size
    }
}




#[account]    
#[derive(Default, Debug)]    
pub struct EventGuest {    
    key: Pubkey,
    is_betted: bool,
    agree_vote: bool,
    is_claimed: bool,
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

#[account]
#[derive(Debug)]
pub struct CreateEventArg{
    event_name: String,
    event_description: String, 
    event_start_time: i64,
    event_end_time: i64,
    bet_lamports: u64,
    guests: Vec<Pubkey>,
}

#[account]
#[derive(Debug)]
pub struct BetOnEventArg{
    event_name: String,
}

#[account]
#[derive(Debug)]
pub struct DeclareEventAttendeesArg {
    event_name: String,
    attendees: Vec<Pubkey>,
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

#[derive(Accounts)]
#[instruction(data: CreateEventArg)]
pub struct CreateEvent<'info> {
    #[account(seeds = [b"profile", signer.key().as_ref()], bump )]
    pub profile: Account<'info, Profile>,


    #[account(init, payer = signer, space = Event::get_init_account_size(&data), 
        seeds = [b"event", signer.key().as_ref(), data.event_name.as_ref()], bump )]
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(data: BetOnEventArg)]
pub struct BetOnEvent<'info> {
    #[account(seeds = [b"profile", signer.key().as_ref()], bump )]
    pub profile: Account<'info, Profile>,

    /// CHECK: it is ok
    #[account()]
    pub host: SystemAccount<'info>,

    #[account(mut, seeds = [b"event", host.key().as_ref(), data.event_name.as_ref()], bump )]
    pub event: Account<'info, Event>,

    /// CHECK: it is ok
    #[account(mut, seeds = [b"bet_pot", host.key().as_ref()], bump )]
    pub bet_pot: SystemAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(data: DeclareEventAttendeesArg)]
pub struct DeclareEventAttendees<'info> {

    /// CHECK: it is ok
    #[account()]
    pub host: SystemAccount<'info>,

    #[account(mut, seeds = [b"event", host.key().as_ref(), data.event_name.as_ref()], bump )]
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}



#[error_code]
pub enum MyError {
    #[msg("BetOnEvent: Already Betted!")]
    BetOnEventAlreadyBetted,

    #[msg("BetOnEvent: Not a guest!")]
    BetOnEventNotAGuest,
}

pub enum TheError {
    MyError(MyError),
}


