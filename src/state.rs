use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PlatForm {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub vesting_per: u64,
    pub vesting_period: u64,
    pub token_price: u64,

    pub init_stage: u64,
    pub stage_1: u64,
    pub stage_2: u64,
    pub stage_3: u64,
    pub stage_4: u64,
    pub platform_fess: u64,

    

}
impl Sealed for PlatForm {}
impl IsInitialized for PlatForm {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for PlatForm {
    const LEN: usize = 105;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, PlatForm::LEN];
        let (is_initialized,
            owner,
            vesting_per,
            vesting_period,
            token_price,
            init_stage,
            stage_1,
            stage_2,
            stage_3,
            stage_4,
            platform_fess,

            ) = array_refs![src, 1, 32, 8,8,8,8,8,8,8,8,8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(PlatForm {
            is_initialized,
            owner: Pubkey::new_from_array(*owner),
            vesting_per: u64::from_le_bytes(*vesting_per),
            vesting_period: u64::from_le_bytes(*vesting_period),
            token_price: u64::from_le_bytes(*token_price),

            init_stage: u64::from_le_bytes(*init_stage),
            stage_1: u64::from_le_bytes(*stage_1),
            stage_2: u64::from_le_bytes(*stage_2),
            stage_3: u64::from_le_bytes(*stage_3),
            stage_4: u64::from_le_bytes(*stage_4),
            platform_fess: u64::from_le_bytes(*platform_fess),




        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, PlatForm::LEN];
        let (is_initialized_dst, owner_dst, vesting_per_dst,vesting_period_dst,
            token_price_dst,init_stage_dst,stage_1_dst,stage_2_dst,stage_3_dst,stage_4_dst,platform_fess_dst) = mut_array_refs![dst, 1, 32, 8,8,8,8,8,8,8,8,8];
        let PlatForm {
            is_initialized,
            owner,
            vesting_per,
            vesting_period,
            token_price,
            init_stage,
            stage_1,
            stage_2,
            stage_3,
            stage_4,
            platform_fess
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        owner_dst.copy_from_slice(owner.as_ref());
        *vesting_per_dst = vesting_per.to_le_bytes();
        *vesting_period_dst = vesting_period.to_le_bytes();
        *token_price_dst = token_price.to_le_bytes();

        *init_stage_dst = init_stage.to_le_bytes();
        *stage_1_dst = stage_1.to_le_bytes();
        *stage_2_dst = stage_2.to_le_bytes();
        *stage_3_dst = stage_3.to_le_bytes();
        *stage_4_dst = stage_4.to_le_bytes();
        *platform_fess_dst = platform_fess.to_le_bytes();


    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct UserState {
    pub is_initialized: bool,
    pub user: Pubkey,
    pub buying_timestamp: u64,
    pub vesting_amount: u64,
    pub buying_amount: u64,
    pub counter: u64,


}
impl Sealed for UserState {}
impl IsInitialized for UserState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for UserState {
    const LEN: usize = 65;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, UserState::LEN];
        let (is_initialized, user, buying_timestamp, vesting_amount,buying_amount,counter) =
            array_refs![src, 1, 32, 8, 8,8,8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(UserState {
            is_initialized,
            user: Pubkey::new_from_array(*user),
            buying_timestamp: u64::from_le_bytes(*buying_timestamp),
            vesting_amount: u64::from_le_bytes(*vesting_amount),
            buying_amount: u64::from_le_bytes(*buying_amount),
            counter: u64::from_le_bytes(*counter),


        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, UserState::LEN];
        let (is_initialized_dst, user_dst, buying_timestamp_dst, vesting_amount_dst,buying_amount_dst,counter_dst) =
            mut_array_refs![dst, 1, 32, 8, 8,8,8];
        let UserState {
            is_initialized,
            user,
            buying_timestamp,
            vesting_amount,
            buying_amount,
            counter,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        user_dst.copy_from_slice(user.as_ref());
        *buying_timestamp_dst = buying_timestamp.to_le_bytes();
        *vesting_amount_dst = vesting_amount.to_le_bytes();
        *buying_amount_dst = buying_amount.to_le_bytes();
        *counter_dst = counter.to_le_bytes();


    }
}
