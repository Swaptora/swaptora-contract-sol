use anchor_lang::prelude::*;

pub const CONFIG_VERSION: u16 = 1;
pub const OFFER_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;
pub const MAX_ASSETS_PER_SIDE: usize = 8;
pub const MAX_ALLOWED_SPL_MINTS: usize = 32;
pub const MAX_ALLOWED_NFT_COLLECTIONS: usize = 32;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub fee_receiver: Pubkey,
    pub platform_fee_lamports: u64,
    pub max_assets_per_side: u8,
    pub version: u16,
    #[max_len(MAX_ALLOWED_SPL_MINTS)]
    pub allowed_spl_mints: Vec<Pubkey>,
    #[max_len(MAX_ALLOWED_NFT_COLLECTIONS)]
    pub allowed_nft_collections: Vec<Pubkey>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub nonce: u64,
    pub status: OfferStatus,
    pub created_at: i64,
    pub expires_at: i64,
    #[max_len(MAX_ASSETS_PER_SIDE)]
    pub maker_assets: Vec<AssetItem>,
    #[max_len(MAX_ASSETS_PER_SIDE)]
    pub taker_assets: Vec<AssetItem>,
    pub platform_fee_lamports: u64,
    pub config_version: u16,
    pub bump: u8,
    pub vault_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, InitSpace, PartialEq, Eq)]
pub struct AssetItem {
    pub kind: AssetKind,
    /// Pubkey::default() is the canonical sentinel for native SOL.
    pub mint: Pubkey,
    /// Lamports for SOL, base units for SPL, exactly one for NFT.
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, InitSpace, PartialEq, Eq)]
pub enum AssetKind {
    Sol,
    SplToken,
    Nft,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, InitSpace, PartialEq, Eq)]
pub enum OfferStatus {
    Active,
    Accepted,
    Cancelled,
    Reclaimed,
}

impl AssetItem {
    pub fn is_sol(&self) -> bool {
        matches!(self.kind, AssetKind::Sol)
    }

    pub fn remaining_account_count_for_validation(&self) -> usize {
        match self.kind {
            AssetKind::Sol => 0,
            AssetKind::SplToken => 3,
            AssetKind::Nft => 4,
        }
    }

    pub fn remaining_account_count_for_refund(&self) -> usize {
        if self.is_sol() {
            0
        } else {
            3
        }
    }

    pub fn remaining_account_count_for_accept_from_vault(&self) -> usize {
        match self.kind {
            AssetKind::Sol => 0,
            AssetKind::SplToken => 4,
            AssetKind::Nft => 5,
        }
    }

    pub fn remaining_account_count_for_terms(&self) -> usize {
        match self.kind {
            AssetKind::Sol => 0,
            AssetKind::SplToken => 1,
            AssetKind::Nft => 2,
        }
    }
}
