use anchor_lang::prelude::*;

declare_id!("C6UkYinrZPK8PA52MsLAVworj7KvHL2VLLP85gmjerVR");

#[program]
pub mod token_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
