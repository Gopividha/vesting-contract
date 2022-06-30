#![allow(clippy::too_many_arguments)]

use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::mem;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum PriveteSellInstruction {
    // Init platform
    InitializePlatform{
        args: (u64, u64 ,u64,u64,u64,u64,u64,u64,u64,u64), 
      },

    //Private selling
    PrivateSell{
        amount:u64
    },

    //Claim vesting amount
    Claim,

}

impl PriveteSellInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidAccountData)?;

        Ok(match tag {
            0 => Self::InitializePlatform{
                args: Self::unpack_data(rest)?,            },
            1 => Self::PrivateSell{
                amount: Self::unpack_amount(rest)?,
            },
            2 => Self::Claim,

            _ => return Err(ProgramError::InvalidAccountData),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(mem::size_of::<Self>());
        match &*self {
            Self::InitializePlatform {args } => {
                buf.push(0);
                buf.extend_from_slice(&args.0.to_le_bytes());
                buf.extend_from_slice(&args.1.to_le_bytes());
                buf.extend_from_slice(&args.2.to_le_bytes());
                buf.extend_from_slice(&args.3.to_le_bytes());
                buf.extend_from_slice(&args.4.to_le_bytes());
                buf.extend_from_slice(&args.5.to_le_bytes());
                buf.extend_from_slice(&args.6.to_le_bytes());
                buf.extend_from_slice(&args.7.to_le_bytes());
                buf.extend_from_slice(&args.8.to_le_bytes());
                buf.extend_from_slice(&args.9.to_le_bytes());


            }
            Self::PrivateSell { amount } => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }

            _ => todo!(),
        }
        buf
    }

    fn unpack_data(input: &[u8]) -> Result<(u64,u64,u64,u64,u64,u64,u64,u64,u64,u64), ProgramError> {
        let amount1 = input
            .get(0..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        let amount2 = input
            .get(8..16)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        let amount3 = input
            .get(16..24)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
    
        let amount4 = input
            .get(24..32)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;

        let amount5 = input
            .get(32..40)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;

        let amount6 = input
            .get(40..48)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        let amount7 = input
            .get(48..56)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        
        let amount8 = input
            .get(56..64)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;

        let amount9 = input
            .get(64..72)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        
        let amount10 = input
            .get(72..80)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok((amount1,amount2,amount3,amount4,amount5,amount6,amount7,amount8,amount9,amount10))
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidAccountData)?;
        Ok(amount)
    }
}
