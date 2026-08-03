use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Maker and taker must be different wallets")]
    MakerEqualsTaker,
    #[msg("Asset list is empty or contains an invalid item")]
    InvalidAssetList,
    #[msg("An asset mint appears more than once on the same side")]
    DuplicateMint,
    #[msg("Asset count exceeds the immutable v1 limit")]
    TooManyAssets,
    #[msg("SPL mint is not in the immutable allowlist")]
    UnsupportedMint,
    #[msg("NFT collection is not in the immutable allowlist")]
    UnsupportedCollection,
    #[msg("NFT token standard is unsupported")]
    UnsupportedTokenStandard,
    #[msg("NFT amount must be exactly one")]
    InvalidNftAmount,
    #[msg("Token account, mint, metadata, owner, or token program is invalid")]
    InvalidTokenAccount,
    #[msg("Source account has insufficient balance")]
    InsufficientBalance,
    #[msg("Only the offer maker may perform this action")]
    UnauthorizedMaker,
    #[msg("Only the addressed offer taker may accept")]
    UnauthorizedTaker,
    #[msg("Offer is not active")]
    OfferNotActive,
    #[msg("Offer has expired")]
    OfferExpired,
    #[msg("Offer has not expired")]
    OfferNotExpired,
    #[msg("Vault holds less than the committed amount or is malformed")]
    VaultBalanceMismatch,
    #[msg("Recipient account does not match the committed owner and mint")]
    InvalidRecipientAccount,
    #[msg("Immutable fee configuration or receiver account is invalid")]
    FeeConfigurationInvalid,
    #[msg("Release initializer does not match the key compiled into this release")]
    UnauthorizedInitializer,
    #[msg("Allowlist exceeds the immutable v1 bound or contains duplicates")]
    InvalidAllowlist,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Unexpected, missing, duplicate, or writable remaining account")]
    InvalidRemainingAccounts,
}
