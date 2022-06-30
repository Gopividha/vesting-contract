use num_traits::CheckedDiv;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    system_instruction::create_account,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};

use crate::{
    error::FarmError,
    instruction::PriveteSellInstruction,
    state::{PlatForm, UserState},
};
use spl_associated_token_account;
use spl_token::{instruction::transfer, state::Account as TokenAccount};
use std::cell::RefCell;
use std::str::FromStr;
pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = PriveteSellInstruction::unpack(instruction_data)?;
        match instruction {
            PriveteSellInstruction::InitializePlatform {args} => {
                msg!("Instruction:INIT PLATFORM");
                return Self::process_init_platform(accounts, program_id,args);
            }
            //PrivateSell means it is from buy from the user and sell from the vesting account
            PriveteSellInstruction::PrivateSell { amount} => {
                msg!("Instruction:Sell!!!!!");
                return Self::process_sell(accounts, program_id,amount);
            }
            PriveteSellInstruction::Claim {} => {
                msg!("Instruction:claim");
                return Self::process_claim(accounts, program_id);
            }
            
        }
    }

    pub fn process_init_platform(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        args:(u64,u64,u64,u64,u64,u64,u64,u64,u64,u64),
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let platform_state_account = next_account_info(account_info_iter)?;

        let owner_account = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?; //PDA of the vesting program

        let admin_reward_token_account = next_account_info(account_info_iter)?;

        let pda_reward_token_account = next_account_info(account_info_iter)?; //from PDA created the PDA_tokenAccount from the frontend
         
        let system_program_id = next_account_info(account_info_iter)?;

        let token_program = next_account_info(account_info_iter)?;

        //creating the state account of the vesting program
        invoke(
            &create_account(
                owner_account.key,
                platform_state_account.key,
                Rent::default().minimum_balance(PlatForm::LEN),
                PlatForm::LEN as u64,
                program_id,
            ),
            &[
                owner_account.clone(), //payer of the account - owner
                platform_state_account.clone(), //state account key pair of the program id created by owner
                system_program_id.clone(), // always prefer to send from outside which is use to create the account
            ],
        )?;
        msg!("Platfom_state_account {}", platform_state_account.key);

        //pda to store staked tokens
        let pda_prefix = "Private_selling";

        let pda_seed = &[pda_prefix.as_bytes(), platform_state_account.key.as_ref()];

        //it will always give the same public key for the same owner
        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);

        //PDA created for the vesting program with it's vesting account created
        msg!("pda {}", pda);

        //unpack the object of the platform state account so that we can update the data
        let mut platform_data =
            PlatForm::unpack_unchecked(&platform_state_account.try_borrow_data()?)?;
        msg!("after unpack");

        //if owner initialises the platform again then it will throw the error
        if platform_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        platform_data.is_initialized = true;
        platform_data.owner = *owner_account.key;
        platform_data.vesting_per = args.1; //vesting percentage - what user will get immediately after the txn
        platform_data.vesting_period=args.2; //vesting time in sec => 240/4 = 60s for each txn to take place 
        // token price for 1 USDC (can be any SPL token based on my config) with our SOLG token. 
        platform_data.token_price=args.3; //Hence, as per current login => 1 USDC = 10 SOLG
        platform_data.init_stage=args.4;//The percentage of the token qty at immediate txn of token buying which user will receive
        platform_data.stage_1=args.5; //The percentage of the token qty at 1st settlement (60s) of token buying which user will receive
        platform_data.stage_2=args.6; //The percentage of the token qty at IInd settlement (120s) of token buying which user will receive
        platform_data.stage_3=args.7; //The percentage of the token qty at IIIrd settlement (180s) of token buying which user will receive
        platform_data.stage_4=args.8;//The percentage of the token qty at IVth(Final) settlement (240s) of token buying which user will receive
        platform_data.platform_fess=args.9;//The percentage of the token qty at IVth(Final) settlement (240s) of token buying which user will receive


        let transfer_token = transfer(
            token_program.key, //official solana token program
            admin_reward_token_account.key, // owner account which will initialise the platform
            pda_reward_token_account.key, //pda associated account of the PDA created against a SOLG mint i.e token_mint from FRONTEND
            owner_account.key, //payer of the transaction
            &[], //owner already signed from the FRONTEND
            args.0.clone(), //amount which will be sent to the vesting contract
        )?;
        msg!("Calling the token program to transfer LP tokens pdatoken account...");

        //owner already signed from the FRONTEND, that's why not needed to call invoked_signed
        invoke(
            &transfer_token,
            &[
                admin_reward_token_account.clone(),
                pda_reward_token_account.clone(),
                owner_account.clone(),
                token_program.clone(), //use to transfer the SPL or SOL token
            ],
        )?;
        msg!("platform state{:?}",platform_data);

        PlatForm::pack(
            platform_data,
            &mut platform_state_account.try_borrow_mut_data()?,
        )?;

        Ok(())
    }

    pub fn process_user_init(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        msg!("entry ********************************");
        let account_info_iter = &mut accounts.iter();

        let user = next_account_info(account_info_iter)?;
        let user_state_account = next_account_info(account_info_iter)?;
        let platform_state_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;



        let pda_seed = &[(user.key).as_ref(), (platform_state_account.key).as_ref()];

        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);


        if pda != *user_state_account.key {
            msg!("pda wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        //Vesting account is creating the user associated account for the user against the vesting account
        //Hence Vesting PAD will be used to create the associated account of the user and invoked_signed will be used

        invoke_signed(
            &create_account(
                user.key,
                user_state_account.key,
                Rent::default().minimum_balance(UserState::LEN),
                UserState::LEN as u64,
                program_id,
            ),
            &[
                user.clone(),
                user_state_account.clone(),
                system_program.clone(),
            ],
            //PDA of the vesting contract
            //PDA will always be of the vesting program 
            //generated PDA string can store the string address of the user key or any other key which can be used later to find the users related to the vesting state account (indirectly for the vesting program)
            &[&[&(user.key).as_ref(), &(platform_state_account.key).as_ref()[..], &[nonce]]],
        )?;

        let mut user_data = UserState::unpack_unchecked(&user_state_account.try_borrow_data()?)?;


        // let mut platform_data = PlatForm::unpack_unchecked(&platform_state_account.try_borrow_data()?)?;

        if user_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        msg!("after unpacks ******************");

        // user_data.is_initialized = true;
        user_data.user = *user.key;
        user_data.buying_timestamp = 0;
        user_data.vesting_amount = 0;
        user_data.buying_amount = 0;
        user_data.counter = 0;

        // platform_data. => mutation

        UserState::pack(user_data, &mut user_state_account.try_borrow_mut_data()?)?;

        // PlatForm::pack(platform_data, &mut platform_state_account.try_borrow_mut_data()?)?;
        msg!("after pack ******************");

        Ok(())
    }

    pub fn process_sell(accounts: &[AccountInfo], program_id: &Pubkey,amount:u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        //user = buyer in context of FRONTEND
        //user = receiver in context of Vesting Program/Contract
        msg!("entered ******************");
        let user = next_account_info(account_info_iter)?;
        let user_state_account = next_account_info(account_info_iter)?; //User state account corresponding to the vesting account i.e platform_state_account and vesting program id
        let user_pda_token_account = next_account_info(account_info_iter)?;


        let platform_state = next_account_info(account_info_iter)?; //id of the platform state account
        let user_sending_token_account = next_account_info(account_info_iter)?; //user USDC token mint associated account

        let user_reciving_token_account = next_account_info(account_info_iter)?; //user SOLG token mint associated account
        let owner_recining_token_account = next_account_info(account_info_iter)?;//(treasory wallet) USDC token mint associated account

        // Vesting program main PDA associated account holds all the tokens initialised from the admin at the time of creating the platform state account
        let pda_token_account = next_account_info(account_info_iter)?;//PDA associated account of the vesting state account
        let pda_account = next_account_info(account_info_iter)?;//PDA of the vesting state account

        let token_program = next_account_info(account_info_iter)?;//Solana token program
        let system_program = next_account_info(account_info_iter)?;//Solana system program

        let user_pda_seed = &[(user.key).as_ref(), (platform_state.key).as_ref()];

        let (user_state, nonce1) = Pubkey::find_program_address(user_pda_seed, program_id);


        //PDA or state account of the user is being checked, if the user is legit who is trying to make the transaction
        if user_state != *user_state_account.key {
            msg!("user_state_acc wrong");
            return Err(ProgramError::InvalidAccountData);
        }

     
        //owner is not program id then it is first time for the user interacting with the vesting contract
        if user_state_account.owner != program_id {
            let user_init_accounts = &[
                user.clone(), //user
                user_state_account.clone(), //user state account
                platform_state.clone(), // platform state
                system_program.clone(), // system program
            ];

            Self::process_user_init(user_init_accounts, program_id);
        };
        let mut user_data = UserState::unpack_unchecked(&user_state_account.try_borrow_data()?)?;

        if user_data.is_initialized == true{
            return Err(ProgramError::AccountAlreadyInitialized)?;
        }



        //pda to store staked tokens
        let pda_prefix = "Private_selling";
        let pda_seed = &[pda_prefix.as_bytes(), (platform_state.key).as_ref()];

        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);
        msg!("pda ********{}", pda_account.key);

        if pda != *pda_account.key {
            msg!("wrong pda");
            return Err(ProgramError::InvalidAccountData);
        }

        msg!("1111");
        msg!("222");


        let mut platform_state_info =
            PlatForm::unpack_unchecked(&platform_state.try_borrow_data()?)?;

        msg!("amount{}",amount.clone());
        msg!("platform_fess{}",platform_state_info.platform_fess);


        let transfer_token = transfer(
            token_program.key, 
            user_sending_token_account.key, //USDC from the user (buyer)
            owner_recining_token_account.key, //USDC to the owner (treasory)
            user.key, //authority of the token sender
            &[], //not needed as already signed from the FRONTEND
            amount.clone()+((amount * platform_state_info.platform_fess)/100),
        )?;
        msg!("Calling the token program to transfer tokens user to owner token account...");
        invoke(
            &transfer_token,
            &[
                user_sending_token_account.clone(), //source
                owner_recining_token_account.clone(), //destination
                user.clone(), //authority
                token_program.clone(), //token program
            ],
        )?;

        //set up clock
        let system_clock = Clock::get()?;
        msg!("222");

        user_data.buying_timestamp = system_clock.unix_timestamp.clone() as u64;


        let total_token_recived_to_user=amount*platform_state_info.token_price;

        let init_stage_amount=(total_token_recived_to_user*platform_state_info.init_stage)/100;



        //////
        let transfer_token = transfer(
            token_program.key,
            pda_token_account.key, //PDA of the vesting acount
            user_pda_token_account.key, //PDA of the user account which will hold all the vesting tokens 
            &pda, 
            &[],
            total_token_recived_to_user.clone(),
        )?;
        msg!("Calling the token program to transfer pda token acc to user token account...");
        invoke_signed(
            &transfer_token,
            &[
                pda_token_account.clone(),
                user_pda_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[
                pda_prefix.as_bytes(),
                &(platform_state.key).as_ref()[..],
                &[nonce],
            ]],
        )?;
        ////
    

        let transfer_token = transfer(
            token_program.key,
            user_pda_token_account.key,
            user_reciving_token_account.key,
            &user_state,
            &[],
            init_stage_amount.clone(),
        )?;
        msg!("Calling the token program to transfer pda token acc to user token account...");
        invoke_signed(
            &transfer_token,
            &[
                user_pda_token_account.clone(),
                user_reciving_token_account.clone(),
                user_state_account.clone(),
                token_program.clone(),
            ],
            &[&[
                &(user.key).as_ref()[..],
                &(platform_state.key).as_ref(),
                &[nonce1],
            ]],
        )?;
       

        user_data.vesting_amount = total_token_recived_to_user-init_stage_amount;
        user_data.buying_amount = total_token_recived_to_user;
        user_data.is_initialized = true;


        

        UserState::pack(user_data, &mut user_state_account.try_borrow_mut_data()?)?;
        PlatForm::pack(
            platform_state_info,
            &mut platform_state.try_borrow_mut_data()?,
        )?;
        msg!(" packed!!!!1111");

        Ok(())
    }

    pub fn process_claim(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        msg!("entered ******************");
        let user = next_account_info(account_info_iter)?;
        let user_state_account = next_account_info(account_info_iter)?;
        let user_pda_token_account = next_account_info(account_info_iter)?;

        
        let platform_state = next_account_info(account_info_iter)?;

        let pda_token_account = next_account_info(account_info_iter)?;
        let user_reciving_token_account = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;

        let token_program = next_account_info(account_info_iter)?;


        let user_pda_seed = &[(user.key).as_ref(), (platform_state.key).as_ref()];
        let (user_state, nonce1) = Pubkey::find_program_address(user_pda_seed, program_id);
        msg!("user state acc  {}", user_state);

        if user_state != *user_state_account.key {
            msg!("user_state_acc wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        //pda to store staked tokens
        let pda_prefix = "Private_selling";
        let pda_seed = &[pda_prefix.as_bytes(), (platform_state.key).as_ref()];

        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);

        if pda != *pda_account.key {
            msg!("error with farm pda");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut user_data = UserState::unpack_unchecked(&user_state_account.try_borrow_data()?)?;

        let mut platform_state_info =
            PlatForm::unpack_unchecked(&platform_state.try_borrow_data()?)?;

        let settlement_duration=platform_state_info.vesting_period/4;
        msg!("settlement_duration{}",settlement_duration);


        let system_clock = Clock::get()?;


        let user_duration=system_clock.unix_timestamp.clone() as u64 - user_data.buying_timestamp ;
        let mut claim_amount=0;
        msg!("user_duration{}",user_duration);
        /**
         * 240
         * 
         * condition 1
         * 60
         * 20 
         * 
         * condition 2
         * 60
         * 65
         * 
         * 3:
         * 60
         * 125
         * 100
         * stage 0 = 20
         * stage 1 = 20  - 20
         * stage 2 = 10  
         * stage 3 = 30
         * stage 4 = 20
         */

        //20
        if user_duration >=settlement_duration && user_duration < settlement_duration *2 {
            if user_data.counter==0 {
                claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_1)/100);
                user_data.counter=user_data.counter+1;

            }
           
        } else if user_duration >= settlement_duration *2 && user_duration < settlement_duration *3 {
            if user_data.counter != 2 {
                claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_2)/100); //40
                if user_data.counter==0{
                    claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_1)/100); // 80
                    user_data.counter=user_data.counter+2; //2
    
                };
                user_data.counter=user_data.counter+1; //2

            }
          
        } else if user_duration >=settlement_duration *3 && user_duration <settlement_duration * 4{
            if user_data.counter != 3{
                claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_3)/100);
                if user_data.counter==0{
                    claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_1)/100)
                                  +((user_data.buying_amount*platform_state_info.stage_2)/100);
                    user_data.counter=user_data.counter+3;
    
                };
                if user_data.counter==1{
                    claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_2)/100);
                    user_data.counter=user_data.counter+2;
    
                }
                user_data.counter=user_data.counter+1;

            }
         

        }else if user_duration >settlement_duration * 4{

            if user_data.counter !=4{
                claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_4)/100);
                if user_data.counter==0{
                    claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_1)/100)+
                    ((user_data.buying_amount*platform_state_info.stage_2)/100)+
                    ((user_data.buying_amount*platform_state_info.stage_3)/100);
                }
                else if user_data.counter==1{
                    claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_2)/100)+
                    ((user_data.buying_amount*platform_state_info.stage_3)/100);
                }
                else if user_data.counter==2{
                    claim_amount=claim_amount+((user_data.buying_amount*platform_state_info.stage_3)/100);
    
                };
                //no need for the user counter as it is false
                user_data.counter=user_data.counter+1;
                user_data.is_initialized=false;


            }
    
        }

        let transfer_token = transfer(
            token_program.key,
            user_pda_token_account.key,
            user_reciving_token_account.key,
            &user_state,
            &[],
            claim_amount.clone(),
        )?;
        msg!("Calling the token program to transfer pda token acc to user token account...");
        invoke_signed(
            &transfer_token,
            &[
                user_pda_token_account.clone(),
                user_reciving_token_account.clone(),
                user_state_account.clone(),
                token_program.clone(),
            ],
            &[&[
                &(user.key).as_ref()[..],
                &(platform_state.key).as_ref(),
                &[nonce1],
            ]],
        )?;

      
        user_data.vesting_amount =  user_data.vesting_amount-claim_amount;



       
        UserState::pack(user_data, &mut user_state_account.try_borrow_mut_data()?)?;
        PlatForm::pack(
            platform_state_info,
            &mut platform_state.try_borrow_mut_data()?,
        )?;

        Ok(())
    }

}
