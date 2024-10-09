use anchor_lang::prelude::*;
use std::mem::size_of;
use std::collections::HashSet;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

declare_id!("GcV7Ucvwg2t1J511GVdgUSnSNgoYXXgYCUHREw1Q1fA3");

const BASIC_RENT:u64 = LAMPORTS_PER_SOL / 10000 * 9;

#[program]
pub mod pot {

    use std::{borrow::BorrowMut, u64};

    use super::*;

    /*
    * Every user must create a profile and some data accounts for interactions
    * */
    pub fn create_account(ctx: Context<CreateAccount>, data: CreateAccountArg) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.profile.name = data.name; 
        ctx.accounts.profile.time_created = now;
        ctx.accounts.profile.time_updated= now;

        Ok(())
    }

    /*
    * An user could stake on anoter target user in order to get 
    * higher chance to be notified
    * */
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


    /*
    * An event host creates an event with the guests they wanna invite 
    * 
    * */
    pub fn create_event(ctx: Context<CreateEvent>, data: CreateEventArg) -> Result<()> {


        let now = Clock::get()?.unix_timestamp;

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
        event.success = false;

        let event_item = EventItem {
            event_name: event.event_name.clone(),
            event_create_time: now,
        };

        ctx.accounts.event_list.events.push(event_item);

        Ok(())
    }

    /*
    * Event guests needs to bet on an event to win the pot share 
    * 
    * */
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
                return Ok(());

            };
        }
        return err!(MyError::BetOnEventNotAGuest);
    }

    /*
    * Event host needs to declare who attended the event,
    * and the attendees needs to agree on it to claim the reward after the event
    * 
    * */
    pub fn declare_event_attendees(ctx: Context<DeclareEventAttendees>, data: DeclareEventAttendeesArg) -> Result<()> {
        // check signer === event.host
        if ctx.accounts.signer.key() != ctx.accounts.event.host {
            return err!(MyError::AccountNotMatch); 
        }

        if ctx.accounts.event.attendees.len() != 0 {
            return err!(MyError::AttendeesAlreadyDeclared); 
        }
        // attendees includes in betted guests 
        let mut betted_guest_keys: HashSet<Pubkey> = HashSet::new();
        for g in &ctx.accounts.event.guests{
            if g.is_betted {
                betted_guest_keys.insert(g.key);
            }
        }
        let attendees_set: HashSet<Pubkey> = data.attendees.into_iter().collect();

        // create attendees list 
        for a in &attendees_set {
            if !betted_guest_keys.contains(a) {
                return err!(MyError::AccountNotMatch); 
            }
            ctx.accounts.event.attendees.push(a.to_owned());
        }

        // host vote for itself 
        for g in &mut ctx.accounts.event.guests {
            if g.key == ctx.accounts.signer.key() {
                g.agree_vote = true;
                ctx.accounts.event.signed += 1;
                return Ok(());
            }
        }

        Ok(())
    }

    /*
    * Attendees agree on the attendees declaration
    * 
    * */
    pub fn agree_event_attendees(ctx: Context<AgreeEventAttendees>, data: AgreeEventAttendeesArg) -> Result<()> {
        // TODO: check event.success and event
        //

        let now = Clock::get()?.unix_timestamp;
        if ctx.accounts.event.success == true {
            return err!(MyError::EventCompleted); 
        }

        if now > ctx.accounts.event.event_end_time {
            return err!(MyError::EventCompleted); 
        }

        if now < ctx.accounts.event.event_start_time {
            return err!(MyError::EventNotStarted); 
        }

        // check host == event.host
        if ctx.accounts.host.key() != ctx.accounts.event.host {
            return err!(MyError::AccountNotMatch); 
        }

        if ctx.accounts.event.attendees.len() == 0 {
            return err!(MyError::AttendeesNotDeclared); 
        }

        let attendees_set: HashSet<Pubkey> = data.attendees.into_iter().collect();

        if !attendees_set.contains(ctx.accounts.signer.key) {
            return err!(MyError::NotAttendee); 
        }

        for a in &ctx.accounts.event.attendees{
            if !attendees_set.contains(&a){
                return err!(MyError::AttendeesNotMatch); 
            }
        }

        let mut done = false; 
        for g in &mut ctx.accounts.event.guests {
            if g.key == ctx.accounts.signer.key() {
                if g.agree_vote == true {
                    return err!(MyError::AttendeesAlreadyVoted) 
                }
                g.agree_vote = true;
                ctx.accounts.event.signed += 1;
                if ctx.accounts.event.signed as usize == ctx.accounts.event.attendees.len(){
                    ctx.accounts.event.success = true;
                }; 

                done = true;
                break;
            }
        }

        if done {
            Ok(())
        }else{

            err!(MyError::AttendeesNotMatch) 
        }
    }


    /*
    * Once the event attendees have all agreed,
    * they can claim the bet pot
    * 
    * */
    pub fn claim_event_reward(ctx: Context<ClaimEventReward>, data: ClaimEventRewardArg) -> Result<()> {

        let bump = ctx.bumps.bet_pot;
        msg!("bump {:?}", bump);

        let host_key = ctx.accounts.host.key();
        let bet_pot_seeds = &[
            b"bet_pot",
            host_key.as_ref(),
            &[bump],
        ];

        if ctx.accounts.event.success == false {
            return err!(MyError::EventCompleted); 
        }

        // check host == event.host
        if ctx.accounts.host.key() != ctx.accounts.event.host {
            return err!(MyError::AccountNotMatch); 
        }

        if !ctx.accounts.event.attendees.contains(ctx.accounts.signer.key){
            return err!(MyError::AccountNotMatch); 
        }

        let bet_lamports = ctx.accounts.event.bet_lamports;
        let mut bet_total:u64 = 0;

        for g in &ctx.accounts.event.guests {
            if g.is_betted {
                bet_total += bet_lamports;
            }
        }

        let reward_per_attendee:u64 = bet_total / ctx.accounts.event.attendees.len() as u64;

        let mut done = false; 
        for g in &mut ctx.accounts.event.guests {
            if g.key == ctx.accounts.signer.key() {
                if g.is_claimed == true {
                    return err!(MyError::AttendeesAlreadyClaimed) 
                }

                //TO CHECK: this code should work for the program PDA...
                /*
                ctx.accounts.bet_pot.to_account_info().sub_lamports(reward_per_attendee)?;
                ctx.accounts.signer.to_account_info().add_lamports(reward_per_attendee)?;
                */

                // Transfer reward from the bet_pot PDA to the signer
                let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.bet_pot.key(),
                    &ctx.accounts.signer.key(),
                    reward_per_attendee,
                );

                anchor_lang::solana_program::program::invoke_signed(
                    &transfer_instruction,
                    &[
                        ctx.accounts.bet_pot.to_account_info(),
                        ctx.accounts.signer.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                    &[bet_pot_seeds],
                )?;

                // mark this guest is_claimed
                g.is_claimed = true;

                done = true;
                break;
            }
        }


        if done {
            Ok(())
        }else{
            err!(MyError::AttendeesNotMatch) 
        }

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
    success: bool,
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
#[derive(Default, Debug)]    
pub struct EventItem{    
    event_name: String,   
    event_create_time: i64,
} 

impl EventItem {

    pub fn get_init_account_size(event_name: &String) -> usize {
        let mut size = 8 + 4; // anchor id + vector    
        size += size_of::<EventItem>();    
        size += event_name.len();
        size
    }

    fn get_account_size(&self) -> usize {
        let mut size = 8 + 4; // anchor id + vector    
        size += size_of::<EventItem>();    
        size += &self.event_name.len();    
        size
    }
}

#[account]
#[derive(Default, Debug)]    
pub struct EventList{    
    events: Vec<EventItem>,   
} 

impl EventList {
    pub fn get_init_account_size() -> usize {
        let mut size = 8 + 4; // anchor id + vector    
        size
    }
    fn get_account_size(&self, event_name: &String) -> usize {
        let mut size = 8 + 4; // anchor id + vector    
        size += size_of::<EventList>();    
        size += EventItem::get_init_account_size(event_name);
        for item in &self.events {
            size += item.get_account_size();
        }
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

#[account]
#[derive(Debug)]
pub struct AgreeEventAttendeesArg {
    event_name: String,
    attendees: Vec<Pubkey>,
}

#[account]
#[derive(Debug)]
pub struct ClaimEventRewardArg {
    event_name: String,
}




#[derive(Accounts)]
#[instruction(data: CreateAccountArg)]
pub struct CreateAccount<'info> {
    #[account(init, payer = signer, space = 8+4+size_of::<Profile>()+data.name.len(), seeds = [b"profile", signer.key().as_ref()], bump )]
    pub profile: Account<'info, Profile>,


    #[account(init, payer = signer, space = 8+4+size_of::<StakerList>(), seeds = [b"staker_list", signer.key().as_ref()], bump )]
    pub stake_list: Account<'info, StakerList>,

    #[account(init, payer = signer, space = EventList::get_init_account_size(), seeds = [b"event_list", signer.key().as_ref()], bump )]
    pub event_list: Account<'info, EventList>,

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


    #[account(mut, 
        realloc = event_list.get_account_size(&data.event_name),
        realloc::payer = signer, realloc::zero = false,
        seeds = [b"event_list", signer.key().as_ref()], bump )]
    pub event_list: Account<'info, EventList>,

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

    #[account(mut, seeds = [b"event", signer.key().as_ref(), data.event_name.as_ref()], bump )]
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(data: AgreeEventAttendeesArg)]
pub struct AgreeEventAttendees<'info> {

    /// CHECK: it is ok
    #[account()]
    pub host: SystemAccount<'info>,

    #[account(mut, seeds = [b"event", host.key().as_ref(), data.event_name.as_ref()], bump )]
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(data: ClaimEventRewardArg)]
pub struct ClaimEventReward<'info> {

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





#[error_code]
pub enum MyError {
    #[msg("BetOnEvent: Already Betted!")]
    BetOnEventAlreadyBetted,

    #[msg("BetOnEvent: Not a guest!")]
    BetOnEventNotAGuest,


    #[msg("Account doesn't match!")]
    AccountNotMatch,

    #[msg("Attendees has already declared!")]
    AttendeesAlreadyDeclared,


    #[msg("Attendees has not declared!")]
    AttendeesNotDeclared,

    #[msg("Attendees do not match!")]
    AttendeesNotMatch,


    #[msg("Signer is not an attendee!")]
    NotAttendee,

    #[msg("Attendees has already voted!")]
    AttendeesAlreadyVoted,

    #[msg("Attendees has already claimed!")]
    AttendeesAlreadyClaimed,

    #[msg("Event has been completed!")]
    EventCompleted,

    #[msg("Event has not started!")]
    EventNotStarted,
}

