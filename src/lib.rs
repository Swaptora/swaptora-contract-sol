use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token},
};

pub mod error;
pub mod events;
pub mod state;
mod utils;

use error::EscrowError;
use events::*;
use state::*;
use utils::*;

declare_id!("CUziHakzRiAYkYE5kz5Sb3DzyWor4p51QRpQ99HvKib8");

/// Replace this public key before a release build. It intentionally grants only the
/// one-shot right to create the immutable Config account.
pub const RELEASE_INITIALIZER: Pubkey = Pubkey::new_from_array([42u8; 32]);

#[program]
pub mod swaptora_contract_sol {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        fee_receiver: Pubkey,
        platform_fee_lamports: u64,
        allowed_spl_mints: Vec<Pubkey>,
        allowed_nft_collections: Vec<Pubkey>,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.release_initializer.key(),
            RELEASE_INITIALIZER,
            EscrowError::UnauthorizedInitializer
        );
        require!(
            fee_receiver != Pubkey::default() && platform_fee_lamports > 0,
            EscrowError::FeeConfigurationInvalid
        );
        validate_allowlists(&allowed_spl_mints, &allowed_nft_collections)?;

        let config = &mut ctx.accounts.config;
        config.fee_receiver = fee_receiver;
        config.platform_fee_lamports = platform_fee_lamports;
        config.max_assets_per_side = MAX_ASSETS_PER_SIDE as u8;
        config.version = CONFIG_VERSION;
        config.allowed_spl_mints = allowed_spl_mints;
        config.allowed_nft_collections = allowed_nft_collections;
        config.bump = ctx.bumps.config;

        let now = Clock::get()?.unix_timestamp;
        emit!(ConfigInitialized {
            config: config.key(),
            fee_receiver,
            platform_fee_lamports,
            version: CONFIG_VERSION,
            timestamp: now,
        });
        Ok(())
    }

    pub fn create_offer<'info>(
        ctx: Context<'info, CreateOffer<'info>>,
        nonce: u64,
        taker_address: Pubkey,
        maker_assets: Vec<AssetItem>,
        taker_assets: Vec<AssetItem>,
    ) -> Result<()> {
        let maker = ctx.accounts.maker.key();
        require_keys_neq!(maker, taker_address, EscrowError::MakerEqualsTaker);
        require!(
            taker_address != Pubkey::default(),
            EscrowError::UnauthorizedTaker
        );
        require_keys_eq!(
            ctx.accounts.taker.key(),
            taker_address,
            EscrowError::UnauthorizedTaker
        );
        validate_asset_lists(&maker_assets, &taker_assets)?;
        let create_accounts = expected_remaining_for_validation(&maker_assets)
            .checked_add(expected_remaining_for_terms(&taker_assets))
            .ok_or_else(|| error!(EscrowError::ArithmeticOverflow))?;
        require!(
            ctx.remaining_accounts.len() == create_accounts,
            EscrowError::InvalidRemainingAccounts
        );
        require_keys_eq!(
            ctx.accounts.fee_receiver.key(),
            ctx.accounts.config.fee_receiver,
            EscrowError::FeeConfigurationInvalid
        );

        let clock = Clock::get()?;
        let expires_at = clock
            .unix_timestamp
            .checked_add(OFFER_TTL_SECONDS)
            .ok_or_else(|| error!(EscrowError::ArithmeticOverflow))?;

        let offer = &mut ctx.accounts.offer;
        offer.maker = maker;
        offer.taker = taker_address;
        offer.nonce = nonce;
        offer.status = OfferStatus::Active;
        offer.created_at = clock.unix_timestamp;
        offer.expires_at = expires_at;
        offer.maker_assets = maker_assets.clone();
        offer.taker_assets = taker_assets.clone();
        offer.platform_fee_lamports = ctx.accounts.config.platform_fee_lamports;
        offer.config_version = ctx.accounts.config.version;
        offer.bump = ctx.bumps.offer;
        offer.vault_bump = ctx.bumps.vault;

        let mut cursor = 0usize;
        for asset in &maker_assets {
            if asset.is_sol() {
                continue;
            }
            let mint_info = &ctx.remaining_accounts[cursor];
            let source_info = &ctx.remaining_accounts[cursor + 1];
            let vault_ata_info = &ctx.remaining_accounts[cursor + 2];
            let metadata_info =
                matches!(asset.kind, AssetKind::Nft).then(|| &ctx.remaining_accounts[cursor + 3]);
            cursor += asset.remaining_account_count_for_validation();

            let decimals = validate_asset_mint_and_metadata(
                asset,
                mint_info,
                metadata_info,
                &ctx.accounts.config,
            )?;
            validate_source_ata(source_info, &maker, &asset.mint, asset.amount)?;
            let expected_vault_ata =
                anchor_spl::associated_token::get_associated_token_address_with_program_id(
                    &ctx.accounts.vault.key(),
                    &asset.mint,
                    &token::ID,
                );
            require_keys_eq!(
                *vault_ata_info.key,
                expected_vault_ata,
                EscrowError::InvalidTokenAccount
            );

            create_ata_idempotent(
                ctx.accounts.maker.to_account_info(),
                vault_ata_info.clone(),
                ctx.accounts.vault.to_account_info(),
                mint_info.clone(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.associated_token_program.to_account_info(),
            )?;
            transfer_tokens(
                source_info.clone(),
                mint_info.clone(),
                vault_ata_info.clone(),
                ctx.accounts.maker.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                asset.amount,
                decimals,
                None,
            )?;
        }

        // A taker does not need to own the requested assets when the offer is created,
        // but their mint/metadata must already satisfy the immutable v1 allowlists.
        for asset in &taker_assets {
            if asset.is_sol() {
                continue;
            }
            let mint_info = &ctx.remaining_accounts[cursor];
            let metadata_info =
                matches!(asset.kind, AssetKind::Nft).then(|| &ctx.remaining_accounts[cursor + 1]);
            cursor += asset.remaining_account_count_for_terms();
            validate_asset_mint_and_metadata(
                asset,
                mint_info,
                metadata_info,
                &ctx.accounts.config,
            )?;
        }

        let sol_amount = checked_sol_total(&maker_assets)?;
        if sol_amount > 0 {
            anchor_lang::system_program::transfer(
                CpiContext::new(
                    anchor_lang::system_program::ID,
                    anchor_lang::system_program::Transfer {
                        from: ctx.accounts.maker.to_account_info(),
                        to: ctx.accounts.vault.to_account_info(),
                    },
                ),
                sol_amount,
            )?;
        }
        anchor_lang::system_program::transfer(
            CpiContext::new(
                anchor_lang::system_program::ID,
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.maker.to_account_info(),
                    to: ctx.accounts.fee_receiver.to_account_info(),
                },
            ),
            ctx.accounts.config.platform_fee_lamports,
        )?;

        emit!(OfferCreated {
            offer: offer.key(),
            maker,
            taker: taker_address,
            nonce,
            expires_at,
            assets_hash: asset_hash(&maker_assets, &taker_assets)?,
            timestamp: clock.unix_timestamp,
        });
        Ok(())
    }

    pub fn accept_offer<'info>(ctx: Context<'info, AcceptOffer<'info>>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let offer = &ctx.accounts.offer;
        require!(
            offer.status == OfferStatus::Active,
            EscrowError::OfferNotActive
        );
        require!(now < offer.expires_at, EscrowError::OfferExpired);
        require_keys_eq!(
            ctx.accounts.taker.key(),
            offer.taker,
            EscrowError::UnauthorizedTaker
        );
        require_keys_eq!(
            ctx.accounts.maker.key(),
            offer.maker,
            EscrowError::UnauthorizedMaker
        );
        require!(
            offer.config_version == ctx.accounts.config.version
                && offer.platform_fee_lamports == ctx.accounts.config.platform_fee_lamports,
            EscrowError::FeeConfigurationInvalid
        );

        let expected = expected_remaining_for_validation(&offer.taker_assets)
            .checked_add(expected_remaining_for_accept_from_vault(
                &offer.maker_assets,
            ))
            .ok_or_else(|| error!(EscrowError::ArithmeticOverflow))?;
        require!(
            ctx.remaining_accounts.len() == expected,
            EscrowError::InvalidRemainingAccounts
        );

        let maker = offer.maker;
        let taker = offer.taker;
        let maker_assets = offer.maker_assets.clone();
        let taker_assets = offer.taker_assets.clone();
        let offer_key = offer.key();
        let vault_bump = [offer.vault_bump];
        let vault_seeds: &[&[u8]] = &[b"vault", offer_key.as_ref(), &vault_bump];
        let signer_seeds: &[&[&[u8]]] = &[vault_seeds];

        let mut cursor = 0usize;
        for asset in &taker_assets {
            if asset.is_sol() {
                anchor_lang::system_program::transfer(
                    CpiContext::new(
                        anchor_lang::system_program::ID,
                        anchor_lang::system_program::Transfer {
                            from: ctx.accounts.taker.to_account_info(),
                            to: ctx.accounts.maker.to_account_info(),
                        },
                    ),
                    asset.amount,
                )?;
                continue;
            }
            let mint_info = &ctx.remaining_accounts[cursor];
            let source_info = &ctx.remaining_accounts[cursor + 1];
            let maker_destination = &ctx.remaining_accounts[cursor + 2];
            let metadata_info =
                matches!(asset.kind, AssetKind::Nft).then(|| &ctx.remaining_accounts[cursor + 3]);
            cursor += asset.remaining_account_count_for_validation();

            let decimals = validate_asset_mint_and_metadata(
                asset,
                mint_info,
                metadata_info,
                &ctx.accounts.config,
            )?;
            validate_source_ata(source_info, &taker, &asset.mint, asset.amount)?;
            let expected_destination =
                anchor_spl::associated_token::get_associated_token_address_with_program_id(
                    &maker,
                    &asset.mint,
                    &token::ID,
                );
            require_keys_eq!(
                *maker_destination.key,
                expected_destination,
                EscrowError::InvalidRecipientAccount
            );
            create_ata_idempotent(
                ctx.accounts.taker.to_account_info(),
                maker_destination.clone(),
                ctx.accounts.maker.to_account_info(),
                mint_info.clone(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.associated_token_program.to_account_info(),
            )?;
            validate_destination_ata(maker_destination, &maker, &asset.mint)?;
            transfer_tokens(
                source_info.clone(),
                mint_info.clone(),
                maker_destination.clone(),
                ctx.accounts.taker.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                asset.amount,
                decimals,
                None,
            )?;
        }

        for asset in &maker_assets {
            if asset.is_sol() {
                continue;
            }
            let mint_info = &ctx.remaining_accounts[cursor];
            let vault_ata_info = &ctx.remaining_accounts[cursor + 1];
            let taker_destination = &ctx.remaining_accounts[cursor + 2];
            let maker_refund = &ctx.remaining_accounts[cursor + 3];
            let metadata_info =
                matches!(asset.kind, AssetKind::Nft).then(|| &ctx.remaining_accounts[cursor + 4]);
            cursor += asset.remaining_account_count_for_accept_from_vault();

            let decimals = validate_asset_mint_and_metadata(
                asset,
                mint_info,
                metadata_info,
                &ctx.accounts.config,
            )?;
            let vault_account = validate_vault_ata(
                vault_ata_info,
                &ctx.accounts.vault.key(),
                &asset.mint,
                asset.amount,
            )?;
            let expected_destination =
                anchor_spl::associated_token::get_associated_token_address_with_program_id(
                    &taker,
                    &asset.mint,
                    &token::ID,
                );
            require_keys_eq!(
                *taker_destination.key,
                expected_destination,
                EscrowError::InvalidRecipientAccount
            );
            create_ata_idempotent(
                ctx.accounts.taker.to_account_info(),
                taker_destination.clone(),
                ctx.accounts.taker.to_account_info(),
                mint_info.clone(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.associated_token_program.to_account_info(),
            )?;
            transfer_tokens(
                vault_ata_info.clone(),
                mint_info.clone(),
                taker_destination.clone(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                asset.amount,
                decimals,
                Some(signer_seeds),
            )?;
            let surplus = vault_account.amount - asset.amount;
            if surplus > 0 {
                validate_destination_ata(maker_refund, &maker, &asset.mint)?;
                transfer_tokens(
                    vault_ata_info.clone(),
                    mint_info.clone(),
                    maker_refund.clone(),
                    ctx.accounts.vault.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                    surplus,
                    decimals,
                    Some(signer_seeds),
                )?;
            } else {
                // The account is still part of the canonical schema, even when no donation
                // needs to be returned. Validate it so callers cannot append arbitrary accounts.
                validate_destination_ata(maker_refund, &maker, &asset.mint)?;
            }
            close_token_account(
                vault_ata_info.clone(),
                ctx.accounts.maker.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                signer_seeds,
            )?;
        }

        let committed_sol = checked_sol_total(&maker_assets)?;
        let vault_lamports = ctx.accounts.vault.lamports();
        require!(
            vault_lamports >= committed_sol,
            EscrowError::VaultBalanceMismatch
        );
        if committed_sol > 0 {
            transfer_sol_from_vault(
                &ctx.accounts.vault,
                &ctx.accounts.taker.to_account_info(),
                &ctx.accounts.system_program,
                committed_sol,
                signer_seeds,
            )?;
        }
        let donation = vault_lamports - committed_sol;
        if donation > 0 {
            transfer_sol_from_vault(
                &ctx.accounts.vault,
                &ctx.accounts.maker.to_account_info(),
                &ctx.accounts.system_program,
                donation,
                signer_seeds,
            )?;
        }

        let offer = &mut ctx.accounts.offer;
        offer.status = OfferStatus::Accepted;
        emit!(OfferAccepted {
            offer: offer.key(),
            maker,
            taker,
            assets_hash: asset_hash(&maker_assets, &taker_assets)?,
            timestamp: now,
        });
        Ok(())
    }

    pub fn cancel_offer<'info>(ctx: Context<'info, CancelOffer<'info>>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(
            ctx.accounts.offer.status == OfferStatus::Active,
            EscrowError::OfferNotActive
        );
        require!(
            now < ctx.accounts.offer.expires_at,
            EscrowError::OfferExpired
        );
        require_keys_eq!(
            ctx.accounts.maker.key(),
            ctx.accounts.offer.maker,
            EscrowError::UnauthorizedMaker
        );
        refund_maker_assets(
            &ctx.accounts.offer,
            &ctx.accounts.vault,
            &ctx.accounts.maker.to_account_info(),
            &ctx.accounts.token_program,
            &ctx.accounts.system_program,
            ctx.remaining_accounts,
        )?;
        let offer = &mut ctx.accounts.offer;
        offer.status = OfferStatus::Cancelled;
        emit!(OfferCancelled {
            offer: offer.key(),
            maker: offer.maker,
            taker: offer.taker,
            timestamp: now,
        });
        Ok(())
    }

    pub fn claim_expired_offer<'info>(ctx: Context<'info, ClaimExpiredOffer<'info>>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(
            ctx.accounts.offer.status == OfferStatus::Active,
            EscrowError::OfferNotActive
        );
        require!(
            now > ctx.accounts.offer.expires_at,
            EscrowError::OfferNotExpired
        );
        require_keys_eq!(
            ctx.accounts.maker.key(),
            ctx.accounts.offer.maker,
            EscrowError::UnauthorizedMaker
        );
        refund_maker_assets(
            &ctx.accounts.offer,
            &ctx.accounts.vault,
            &ctx.accounts.maker.to_account_info(),
            &ctx.accounts.token_program,
            &ctx.accounts.system_program,
            ctx.remaining_accounts,
        )?;
        let offer = &mut ctx.accounts.offer;
        offer.status = OfferStatus::Reclaimed;
        emit!(OfferExpiredReclaimed {
            offer: offer.key(),
            maker: offer.maker,
            taker: offer.taker,
            caller: ctx.accounts.caller.key(),
            timestamp: now,
        });
        Ok(())
    }
}

fn transfer_sol_from_vault<'info>(
    vault: &UncheckedAccount<'info>,
    destination: &AccountInfo<'info>,
    _system_program: &Program<'info, System>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(
            anchor_lang::system_program::ID,
            anchor_lang::system_program::Transfer {
                from: vault.to_account_info(),
                to: destination.clone(),
            },
            signer_seeds,
        ),
        amount,
    )
}

fn refund_maker_assets<'info>(
    offer: &Account<'info, Offer>,
    vault: &UncheckedAccount<'info>,
    maker: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    remaining_accounts: &'info [AccountInfo<'info>],
) -> Result<()> {
    require!(
        remaining_accounts.len() == expected_remaining_for_refund(&offer.maker_assets),
        EscrowError::InvalidRemainingAccounts
    );
    let offer_key = offer.key();
    let vault_bump = [offer.vault_bump];
    let vault_seeds: &[&[u8]] = &[b"vault", offer_key.as_ref(), &vault_bump];
    let signer_seeds: &[&[&[u8]]] = &[vault_seeds];
    let mut cursor = 0usize;

    for asset in &offer.maker_assets {
        if asset.is_sol() {
            continue;
        }
        let mint_info = &remaining_accounts[cursor];
        let vault_ata_info = &remaining_accounts[cursor + 1];
        let maker_destination = &remaining_accounts[cursor + 2];
        cursor += asset.remaining_account_count_for_refund();

        require_keys_eq!(*mint_info.key, asset.mint, EscrowError::InvalidTokenAccount);
        let mint = unpack_mint(mint_info)?;
        let vault_account =
            validate_vault_ata(vault_ata_info, &vault.key(), &asset.mint, asset.amount)?;
        validate_destination_ata(maker_destination, &offer.maker, &asset.mint)?;
        transfer_tokens(
            vault_ata_info.clone(),
            mint_info.clone(),
            maker_destination.clone(),
            vault.to_account_info(),
            token_program.to_account_info(),
            vault_account.amount,
            mint.decimals,
            Some(signer_seeds),
        )?;
        close_token_account(
            vault_ata_info.clone(),
            maker.clone(),
            vault.to_account_info(),
            token_program.to_account_info(),
            signer_seeds,
        )?;
    }

    let committed_sol = checked_sol_total(&offer.maker_assets)?;
    let vault_lamports = vault.lamports();
    require!(
        vault_lamports >= committed_sol,
        EscrowError::VaultBalanceMismatch
    );
    if vault_lamports > 0 {
        transfer_sol_from_vault(vault, maker, system_program, vault_lamports, signer_seeds)?;
    }
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub release_initializer: Signer<'info>,
    #[account(
        init,
        payer = release_initializer,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u64, taker_address: Pubkey)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    /// CHECK: Address is fixed in the instruction data and Offer; it need not exist yet.
    #[account(address = taker_address @ EscrowError::UnauthorizedTaker)]
    pub taker: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = maker,
        space = 8 + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub offer: Account<'info, Offer>,
    /// CHECK: PDA is the per-offer native SOL and token authority vault.
    #[account(mut, seeds = [b"vault", offer.key().as_ref()], bump)]
    pub vault: UncheckedAccount<'info>,
    /// CHECK: Exact address is checked against immutable Config.
    #[account(mut)]
    pub fee_receiver: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    /// CHECK: Constrained against Offer in the handler.
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"offer", offer.maker.as_ref(), &offer.nonce.to_le_bytes()],
        bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,
    /// CHECK: Canonical PDA constrained by the Offer.
    #[account(mut, seeds = [b"vault", offer.key().as_ref()], bump = offer.vault_bump)]
    pub vault: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        mut,
        seeds = [b"offer", offer.maker.as_ref(), &offer.nonce.to_le_bytes()],
        bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,
    /// CHECK: Canonical PDA constrained by the Offer.
    #[account(mut, seeds = [b"vault", offer.key().as_ref()], bump = offer.vault_bump)]
    pub vault: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimExpiredOffer<'info> {
    pub caller: Signer<'info>,
    /// CHECK: Assets are always routed here; address is checked against Offer.
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"offer", offer.maker.as_ref(), &offer.nonce.to_le_bytes()],
        bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,
    /// CHECK: Canonical PDA constrained by the Offer.
    #[account(mut, seeds = [b"vault", offer.key().as_ref()], bump = offer.vault_bump)]
    pub vault: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
