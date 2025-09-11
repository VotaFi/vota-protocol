use anchor_lang::prelude::borsh::{BorshDeserialize, BorshSerialize};
use anchor_lang::Discriminator;
use solana_program::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OwnerInvokeInstructionV2 {
    pub index: u64,
    pub bump: u8,
    pub invoker: Pubkey,
    pub data: Vec<u8>,
}

impl Discriminator for OwnerInvokeInstructionV2 {
    const DISCRIMINATOR: [u8; 8] = [169, 161, 80, 52, 188, 19, 232, 97];
}

impl anchor_lang::InstructionData for OwnerInvokeInstructionV2 {
    fn data(&self) -> Vec<u8> {
        let mut d = Self::DISCRIMINATOR.to_vec();
        d.append(&mut self.try_to_vec().expect("Should always serialize"));
        d
    }
}

