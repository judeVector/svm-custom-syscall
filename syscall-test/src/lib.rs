use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
    unix_timestamp::sol_get_unix_timestamp,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let unix_timestamp = sol_get_unix_timestamp();
    msg!("The Timestampp in unix is : {}", unix_timestamp);

    Ok(())
}
