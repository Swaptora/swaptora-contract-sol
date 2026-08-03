use anchor_lang::prelude::*;

#[event]
pub struct ConfigInitialized {
    pub config: Pubkey,
    pub fee_receiver: Pubkey,
    pub platform_fee_lamports: u64,
    pub version: u16,
    pub timestamp: i64,
}

#[event]
pub struct OfferCreated {
    pub offer: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub nonce: u64,
    pub expires_at: i64,
    pub assets_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct OfferAccepted {
    pub offer: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub assets_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct OfferCancelled {
    pub offer: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct OfferExpiredReclaimed {
    pub offer: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub caller: Pubkey,
    pub timestamp: i64,
}
