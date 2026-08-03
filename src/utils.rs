use std::collections::BTreeSet;

use anchor_lang::{
    prelude::*,
    solana_program::{program_option::COption, program_pack::Pack},
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id},
    metadata::mpl_token_metadata::{
        accounts::Metadata, types::TokenStandard, ID as TOKEN_METADATA_ID,
    },
    token::{
        self,
        spl_token::state::{Account as SplAccount, AccountState, Mint},
    },
};

use crate::{
    error::EscrowError,
    state::{AssetItem, AssetKind, MAX_ASSETS_PER_SIDE},
};

pub fn validate_asset_lists(maker: &[AssetItem], taker: &[AssetItem]) -> Result<()> {
    require!(
        !(maker.is_empty() && taker.is_empty()),
        EscrowError::InvalidAssetList
    );
    require!(
        maker.len() <= MAX_ASSETS_PER_SIDE && taker.len() <= MAX_ASSETS_PER_SIDE,
        EscrowError::TooManyAssets
    );
    validate_asset_side(maker)?;
    validate_asset_side(taker)?;
    Ok(())
}

fn validate_asset_side(assets: &[AssetItem]) -> Result<()> {
    let mut mints = BTreeSet::new();
    let mut has_sol = false;
    for asset in assets {
        require!(asset.amount > 0, EscrowError::InvalidAssetList);
        match asset.kind {
            AssetKind::Sol => {
                require_keys_eq!(asset.mint, Pubkey::default(), EscrowError::InvalidAssetList);
                require!(!has_sol, EscrowError::DuplicateMint);
                has_sol = true;
            }
            AssetKind::SplToken => {
                require!(
                    asset.mint != Pubkey::default(),
                    EscrowError::InvalidAssetList
                );
                require!(mints.insert(asset.mint), EscrowError::DuplicateMint);
            }
            AssetKind::Nft => {
                require!(
                    asset.mint != Pubkey::default(),
                    EscrowError::InvalidAssetList
                );
                require!(asset.amount == 1, EscrowError::InvalidNftAmount);
                require!(mints.insert(asset.mint), EscrowError::DuplicateMint);
            }
        }
    }
    Ok(())
}

pub fn expected_remaining_for_validation(assets: &[AssetItem]) -> usize {
    assets
        .iter()
        .map(AssetItem::remaining_account_count_for_validation)
        .sum()
}

pub fn expected_remaining_for_refund(assets: &[AssetItem]) -> usize {
    assets
        .iter()
        .map(AssetItem::remaining_account_count_for_refund)
        .sum()
}

pub fn expected_remaining_for_accept_from_vault(assets: &[AssetItem]) -> usize {
    assets
        .iter()
        .map(AssetItem::remaining_account_count_for_accept_from_vault)
        .sum()
}

pub fn expected_remaining_for_terms(assets: &[AssetItem]) -> usize {
    assets
        .iter()
        .map(AssetItem::remaining_account_count_for_terms)
        .sum()
}

pub fn unpack_mint(mint_info: &AccountInfo<'_>) -> Result<Mint> {
    require_keys_eq!(
        *mint_info.owner,
        token::ID,
        EscrowError::InvalidTokenAccount
    );
    let data = mint_info.try_borrow_data()?;
    let mint = Mint::unpack(&data).map_err(|_| error!(EscrowError::InvalidTokenAccount))?;
    require!(mint.is_initialized, EscrowError::InvalidTokenAccount);
    require!(
        matches!(mint.freeze_authority, COption::None),
        EscrowError::InvalidTokenAccount
    );
    Ok(mint)
}

pub fn unpack_token_account(account_info: &AccountInfo<'_>) -> Result<SplAccount> {
    require_keys_eq!(
        *account_info.owner,
        token::ID,
        EscrowError::InvalidTokenAccount
    );
    let data = account_info.try_borrow_data()?;
    let account =
        SplAccount::unpack(&data).map_err(|_| error!(EscrowError::InvalidTokenAccount))?;
    require!(
        account.state == AccountState::Initialized,
        EscrowError::InvalidTokenAccount
    );
    Ok(account)
}

pub fn validate_source_ata(
    account_info: &AccountInfo<'_>,
    owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) -> Result<SplAccount> {
    let expected = get_associated_token_address_with_program_id(owner, mint, &token::ID);
    require_keys_eq!(
        *account_info.key,
        expected,
        EscrowError::InvalidTokenAccount
    );
    let account = unpack_token_account(account_info)?;
    require_keys_eq!(account.owner, *owner, EscrowError::InvalidTokenAccount);
    require_keys_eq!(account.mint, *mint, EscrowError::InvalidTokenAccount);
    require!(account.amount >= amount, EscrowError::InsufficientBalance);
    Ok(account)
}

pub fn validate_destination_ata(
    account_info: &AccountInfo<'_>,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<SplAccount> {
    let expected = get_associated_token_address_with_program_id(owner, mint, &token::ID);
    require_keys_eq!(
        *account_info.key,
        expected,
        EscrowError::InvalidRecipientAccount
    );
    let account = unpack_token_account(account_info)
        .map_err(|_| error!(EscrowError::InvalidRecipientAccount))?;
    require_keys_eq!(account.owner, *owner, EscrowError::InvalidRecipientAccount);
    require_keys_eq!(account.mint, *mint, EscrowError::InvalidRecipientAccount);
    Ok(account)
}

pub fn validate_vault_ata(
    account_info: &AccountInfo<'_>,
    vault: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) -> Result<SplAccount> {
    let expected = get_associated_token_address_with_program_id(vault, mint, &token::ID);
    require_keys_eq!(
        *account_info.key,
        expected,
        EscrowError::VaultBalanceMismatch
    );
    let account = unpack_token_account(account_info)
        .map_err(|_| error!(EscrowError::VaultBalanceMismatch))?;
    require_keys_eq!(account.owner, *vault, EscrowError::VaultBalanceMismatch);
    require_keys_eq!(account.mint, *mint, EscrowError::VaultBalanceMismatch);
    require!(account.amount >= amount, EscrowError::VaultBalanceMismatch);
    Ok(account)
}

pub fn validate_asset_mint_and_metadata(
    asset: &AssetItem,
    mint_info: &AccountInfo<'_>,
    metadata_info: Option<&AccountInfo<'_>>,
) -> Result<u8> {
    require_keys_eq!(*mint_info.key, asset.mint, EscrowError::InvalidTokenAccount);
    let mint = unpack_mint(mint_info)?;

    match asset.kind {
        AssetKind::Sol => return err!(EscrowError::InvalidAssetList),
        AssetKind::SplToken => {}
        AssetKind::Nft => {
            require!(asset.amount == 1, EscrowError::InvalidNftAmount);
            require!(
                mint.decimals == 0 && mint.supply == 1,
                EscrowError::UnsupportedTokenStandard
            );
            let metadata_info =
                metadata_info.ok_or_else(|| error!(EscrowError::InvalidRemainingAccounts))?;
            require_keys_eq!(
                *metadata_info.owner,
                TOKEN_METADATA_ID,
                EscrowError::InvalidTokenAccount
            );
            let expected_metadata = Metadata::find_pda(&asset.mint).0;
            require_keys_eq!(
                *metadata_info.key,
                expected_metadata,
                EscrowError::InvalidTokenAccount
            );
            let metadata = Metadata::try_from(metadata_info)
                .map_err(|_| error!(EscrowError::InvalidTokenAccount))?;
            require_keys_eq!(metadata.mint, asset.mint, EscrowError::InvalidTokenAccount);
            require!(
                metadata.token_standard == Some(TokenStandard::NonFungible),
                EscrowError::UnsupportedTokenStandard
            );
        }
    }
    Ok(mint.decimals)
}

pub fn create_ata_idempotent<'info>(
    payer: AccountInfo<'info>,
    associated_token: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    _associated_token_program: AccountInfo<'info>,
) -> Result<()> {
    associated_token::create_idempotent(CpiContext::new(
        associated_token::ID,
        associated_token::Create {
            payer,
            associated_token,
            authority,
            mint,
            system_program,
            token_program,
        },
    ))
}

// Mirrors the four TransferChecked accounts plus amount/decimals/signer mode.
#[allow(clippy::too_many_arguments)]
pub fn transfer_tokens<'info>(
    from: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    _token_program: AccountInfo<'info>,
    amount: u64,
    decimals: u8,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let accounts = token::TransferChecked {
        from,
        mint,
        to,
        authority,
    };
    match signer_seeds {
        Some(seeds) => token::transfer_checked(
            CpiContext::new_with_signer(token::ID, accounts, seeds),
            amount,
            decimals,
        ),
        None => token::transfer_checked(CpiContext::new(token::ID, accounts), amount, decimals),
    }
}

pub fn close_token_account<'info>(
    account: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    _token_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    token::close_account(CpiContext::new_with_signer(
        token::ID,
        token::CloseAccount {
            account,
            destination,
            authority,
        },
        signer_seeds,
    ))
}

pub fn asset_hash(maker: &[AssetItem], taker: &[AssetItem]) -> Result<[u8; 32]> {
    let mut bytes = Vec::new();
    maker.serialize(&mut bytes)?;
    taker.serialize(&mut bytes)?;
    Ok(solana_keccak_hasher::hash(&bytes).to_bytes())
}

pub fn checked_sol_total(assets: &[AssetItem]) -> Result<u64> {
    assets
        .iter()
        .filter(|asset| asset.is_sol())
        .try_fold(0u64, |total, asset| {
            total
                .checked_add(asset.amount)
                .ok_or_else(|| error!(EscrowError::ArithmeticOverflow))
        })
}
