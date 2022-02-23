use borsh::{BorshDeserialize, BorshSerialize};
// Deployed Program Id: 5mpHLcQKE91D18QkYKsRwcJrV6DMv82zzfZJHDQd6BVv
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

fn process_instruction(
    program_id:&Pubkey,
    accounts:&[AccountInfo],
    instruction_data:&[u8]
    )->ProgramResult{
    //check for data
    if instruction_data.len()==0{
        return Err(ProgramError::InvalidInstructionData);
    }
    //1. create campaign
    if instruction_data[0]==0{
        return create_campaign(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
            );
    }else if instruction_data[0]==1{
        return withdraw(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
            );
    }else if instruction_data[0]==2{
        return donate(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
            );
    }
    msg!("Didn't find the endpoint required");
    return Err(ProgramError::InvalidInstructionData);
    Ok(())
}

entrypoint!(process_instruction);

#[derive(Debug)]
struct Name {
    pub eyes_color: String,
    pub name:String,
    pub height:i32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct CampaignDetails{
    pub admin: Pubkey,
    pub name: String,
    pub description: String,
    pub image_link: String,
    /// we will be using this to know the total amount 
    /// donated to a campaign.
    pub amount_donated: u64,
}

// By deriving the trait BorshDeserialize in our CampaignDetails struct we have added a method try_from_slice which takes in the parameter array of u8 and creates an object of CampaignDetails with it. 
fn create_campaign(
    program_id:&Pubkey,
    accounts:&[AccountInfo],
    instruction_data: &[u8]
    )->ProgramResult{

    let accounts_iter=&mut accounts.iter();

    let writng_account=next_account_info(accounts_iter)?;

    let creator_account=next_account_info(accounts_iter)?;

    if !creator_account.is_signer{
        msg!("Creator account should be a signer");
        return Err(ProgramError::IncorrectProgramId);
    }
    //
    if writng_account.is_signer{
        msg!("writng_account is not owned by the program");
        return Err(ProgramError::IncorrectProgramId)
    }
    let mut input_data=CampaignDetails::try_from_slice(&instruction_data)
        .expect("Instruction data deserialistion didn't work");

    if input_data.admin != *creator_account.key{
        msg!("Invalid Instruction data");
        return Err(ProgramError::InvalidInstructionData)
    }


    //get minimum balance
    let rent_exemption=Rent::get()?.minimum_balance(writng_account.data_len());
    //getting balance
    if **writng_account.lamports.borrow()<rent_exemption{
        msg!("The balance if the writng_account should be more then rent_exemption");
        return Err(ProgramError::InsufficientFunds);
    }

    //initial donation amount 
    input_data.amount_donated=0;

    //wrinting to accont 
    input_data.serialize(&mut &mut writng_account.data.borrow_mut()[..])?;

    Ok(())
}

//-----------------widthdraw request
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct WithdrawRequest {
    pub amount: u64,
}

fn withdraw(
    program_id:&Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
    )-> ProgramResult{
    let accounts_iter=&mut accounts.iter();
    let writing_account=next_account_info(accounts_iter)?;
    let admin_account=next_account_info(accounts_iter)?;

    if writing_account.owner != program_id{
        msg!("writng_account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    //admim account should be the owner
    if !admin_account.is_signer{
         msg!("writng_account isn't owned by program");
         return Err(ProgramError::IncorrectProgramId);
    }

    let campaign_data=CampaignDetails::try_from_slice(*writing_account.data.borrow())
            .expect("Error deserialising data");

    if campaign_data.admin != *admin_account.key{
        msg!("Only the account admin can withdraw");
        return Err(ProgramError::InvalidAccountData);
    }

    let input_data=WithdrawRequest::try_from_slice(&instruction_data)
        .expect("Instruction data serialization didn't work");

    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());

    //check for funds
    if **writing_account.lamports.borrow()-rent_exemption<input_data.amount{
        msg!("Insufficent balance");
        return Err(ProgramError::InsufficientFunds);
    }

    //transfer balance 
    // Transfer balance
    // We will decrease the balance of the program account, and increase the admin_account balance.
    **writing_account.try_borrow_mut_lamports()? -= input_data.amount;
    **admin_account.try_borrow_mut_lamports()? += input_data.amount;
    Ok(())
}

fn donate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
)->ProgramResult{
    let accounts_iter=&mut accounts.iter();
    let writing_account=next_account_info(accounts_iter)?;
    let donator_program_account = next_account_info(accounts_iter)?;
    let donator = next_account_info(accounts_iter)?;

      if writing_account.owner != program_id {
        msg!("writing_account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }
    if donator_program_account.owner != program_id {
        msg!("donator_program_account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }
    if !donator.is_signer {
        msg!("donator should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut campaign_data=CampaignDetails::try_from_slice(*writing_account.data.borrow())
        .expect("Error deserialising data");
    campaign_data.amount_donated+=**donator_program_account.lamports.borrow();
// <--------------------->
    **writing_account.try_borrow_mut_lamports()? += **donator_program_account.lamports.borrow();
    **donator_program_account.try_borrow_mut_lamports()? = 0;

    campaign_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    Ok(())
}