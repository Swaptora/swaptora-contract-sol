use std::collections::HashMap;

use anchor_lang::{AccountDeserialize, AnchorSerialize, Discriminator, InstructionData};
use anchor_spl::metadata::mpl_token_metadata::{
    accounts::Metadata,
    types::{Collection, Key, TokenStandard},
    ID as TOKEN_METADATA_ID,
};
use mollusk_svm::{result::Check, Mollusk, MolluskContext};
use mollusk_svm_programs_token::{associated_token, token};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};
use swaptora_contract_sol::{
    instruction as escrow_instruction,
    state::{AssetItem, AssetKind, Offer, OfferStatus, MAX_ASSETS_PER_SIDE},
};

const SYSTEM_PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("11111111111111111111111111111111");
const TOKEN_2022_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const INITIAL_BALANCE: u64 = 20_000_000_000;
const PLATFORM_FEE: u64 = 10_000_000;
const MAKER_SOL: u64 = 1_250_000_000;
const TAKER_SOL: u64 = 700_000_000;

type Store = HashMap<Pubkey, Account>;
type TestContext = MolluskContext<Store>;

fn pk(value: anchor_lang::prelude::Pubkey) -> Pubkey {
    Pubkey::new_from_array(value.to_bytes())
}

fn anchor_pk(value: Pubkey) -> anchor_lang::prelude::Pubkey {
    anchor_lang::prelude::Pubkey::new_from_array(value.to_bytes())
}

fn program_id() -> Pubkey {
    pk(swaptora_contract_sol::ID)
}

fn system_account(lamports: u64) -> Account {
    Account {
        lamports,
        data: Vec::new(),
        owner: SYSTEM_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    }
}

fn setup() -> (TestContext, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
    let initializer = pk(swaptora_contract_sol::RELEASE_INITIALIZER);
    let fee_receiver = Pubkey::new_unique();
    let maker = Pubkey::new_unique();
    let taker = Pubkey::new_unique();
    let stranger = Pubkey::new_unique();
    let (config, _) = Pubkey::find_program_address(&[b"config"], &program_id());

    let mut accounts = HashMap::new();
    accounts.insert(initializer, system_account(INITIAL_BALANCE));
    accounts.insert(fee_receiver, system_account(0));
    accounts.insert(maker, system_account(INITIAL_BALANCE));
    accounts.insert(taker, system_account(INITIAL_BALANCE));
    accounts.insert(stranger, system_account(INITIAL_BALANCE));

    let mut mollusk = Mollusk::new(&program_id(), "target/deploy/swaptora_contract_sol");
    token::add_program(&mut mollusk);
    associated_token::add_program(&mut mollusk);
    let context = mollusk.with_context(accounts);

    let initialize = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(initializer, true),
            AccountMeta::new(config, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: escrow_instruction::InitializeConfig {
            fee_receiver: anchor_pk(fee_receiver),
            platform_fee_lamports: PLATFORM_FEE,
        }
        .data(),
    };
    context.process_and_validate_instruction(&initialize, &[Check::success()]);
    (context, config, fee_receiver, maker, taker, stranger)
}

fn mint_account(decimals: u8, supply: u64) -> Account {
    token::create_account_for_mint(Mint {
        mint_authority: COption::None,
        supply,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None,
    })
}

fn token_account(mint: Pubkey, owner: Pubkey, amount: u64) -> (Pubkey, Account) {
    associated_token::create_account_for_associated_token_account(TokenAccount {
        mint: anchor_pk(mint),
        owner: anchor_pk(owner),
        amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    })
}

fn token_balance(context: &TestContext, key: &Pubkey) -> u64 {
    let account = account(context, key);
    TokenAccount::unpack(&account.data)
        .expect("valid SPL token account")
        .amount
}

fn metadata_account(mint: Pubkey, collection: Pubkey) -> (Pubkey, Account) {
    metadata_account_with_verification(mint, collection, true)
}

fn metadata_account_with_verification(
    mint: Pubkey,
    collection: Pubkey,
    verified: bool,
) -> (Pubkey, Account) {
    metadata_account_with_collection(
        mint,
        Some(Collection {
            verified,
            key: anchor_pk(collection),
        }),
    )
}

fn standalone_metadata_account(mint: Pubkey) -> (Pubkey, Account) {
    metadata_account_with_collection(mint, None)
}

fn metadata_account_with_collection(
    mint: Pubkey,
    collection: Option<Collection>,
) -> (Pubkey, Account) {
    let metadata = Metadata {
        key: Key::MetadataV1,
        update_authority: anchor_lang::prelude::Pubkey::new_unique(),
        mint: anchor_pk(mint),
        name: "Swaptora test NFT".to_owned(),
        symbol: "SWP".to_owned(),
        uri: "https://example.invalid/metadata.json".to_owned(),
        seller_fee_basis_points: 0,
        creators: None,
        primary_sale_happened: false,
        is_mutable: false,
        edition_nonce: None,
        token_standard: Some(TokenStandard::NonFungible),
        collection,
        uses: None,
        collection_details: None,
        programmable_config: None,
    };
    let mut data = Vec::new();
    metadata.serialize(&mut data).expect("serialize metadata");
    let (metadata_key, _) = Metadata::find_pda(&anchor_pk(mint));
    (
        pk(metadata_key),
        Account {
            lamports: 10_000_000,
            data,
            owner: pk(TOKEN_METADATA_ID),
            executable: false,
            rent_epoch: 0,
        },
    )
}

fn sol(amount: u64) -> AssetItem {
    AssetItem {
        kind: AssetKind::Sol,
        mint: anchor_lang::prelude::Pubkey::default(),
        amount,
    }
}

fn spl(mint: Pubkey, amount: u64) -> AssetItem {
    AssetItem {
        kind: AssetKind::SplToken,
        mint: anchor_pk(mint),
        amount,
    }
}

fn nft(mint: Pubkey) -> AssetItem {
    AssetItem {
        kind: AssetKind::Nft,
        mint: anchor_pk(mint),
        amount: 1,
    }
}

fn offer_addresses(maker: Pubkey, nonce: u64) -> (Pubkey, Pubkey) {
    let nonce_bytes = nonce.to_le_bytes();
    let (offer, _) =
        Pubkey::find_program_address(&[b"offer", maker.as_ref(), &nonce_bytes], &program_id());
    let (vault, _) = Pubkey::find_program_address(&[b"vault", offer.as_ref()], &program_id());
    (offer, vault)
}

fn create_sol_offer_ix(
    config: Pubkey,
    fee_receiver: Pubkey,
    maker: Pubkey,
    taker: Pubkey,
    nonce: u64,
    maker_assets: Vec<AssetItem>,
    taker_assets: Vec<AssetItem>,
) -> (Instruction, Pubkey, Pubkey) {
    let (offer, vault) = offer_addresses(maker, nonce);
    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(maker, true),
            AccountMeta::new_readonly(taker, false),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(offer, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(fee_receiver, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: escrow_instruction::CreateOffer {
            nonce,
            taker_address: anchor_pk(taker),
            maker_assets,
            taker_assets,
        }
        .data(),
    };
    (ix, offer, vault)
}

fn with_remaining(mut instruction: Instruction, remaining: Vec<AccountMeta>) -> Instruction {
    instruction.accounts.extend(remaining);
    instruction
}

fn accept_ix(
    config: Pubkey,
    offer: Pubkey,
    vault: Pubkey,
    maker: Pubkey,
    signer: Pubkey,
) -> Instruction {
    Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(maker, false),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(offer, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: escrow_instruction::AcceptOffer {}.data(),
    }
}

fn cancel_ix(offer: Pubkey, vault: Pubkey, maker: Pubkey) -> Instruction {
    Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(maker, true),
            AccountMeta::new(offer, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: escrow_instruction::CancelOffer {}.data(),
    }
}

fn claim_ix(offer: Pubkey, vault: Pubkey, maker: Pubkey, caller: Pubkey) -> Instruction {
    Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new_readonly(caller, true),
            AccountMeta::new(maker, false),
            AccountMeta::new(offer, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: escrow_instruction::ClaimExpiredOffer {}.data(),
    }
}

fn account(context: &TestContext, key: &Pubkey) -> Account {
    context
        .account_store
        .borrow()
        .get(key)
        .unwrap_or_else(|| panic!("missing account {key}"))
        .clone()
}

fn offer_state(context: &TestContext, key: &Pubkey) -> Offer {
    let account = account(context, key);
    let mut data: &[u8] = &account.data;
    Offer::try_deserialize(&mut data).expect("valid Offer account")
}

#[test]
fn sol_for_sol_is_atomic_and_fee_is_paid_once() {
    let (context, config, fee_receiver, maker, taker, _) = setup();
    let nonce = 1;
    let (create, offer, vault) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        nonce,
        vec![sol(MAKER_SOL)],
        vec![sol(TAKER_SOL)],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);

    assert_eq!(account(&context, &vault).lamports, MAKER_SOL);
    assert_eq!(account(&context, &fee_receiver).lamports, PLATFORM_FEE);
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Active);
    let maker_before_accept = account(&context, &maker).lamports;
    let taker_before_accept = account(&context, &taker).lamports;

    context.process_and_validate_instruction(
        &accept_ix(config, offer, vault, maker, taker),
        &[Check::success()],
    );

    assert_eq!(account(&context, &vault).lamports, 0);
    assert_eq!(
        account(&context, &maker).lamports,
        maker_before_accept + TAKER_SOL
    );
    assert_eq!(
        account(&context, &taker).lamports,
        taker_before_accept - TAKER_SOL + MAKER_SOL
    );
    assert_eq!(account(&context, &fee_receiver).lamports, PLATFORM_FEE);
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Accepted);

    let replay = context.process_instruction(&accept_ix(config, offer, vault, maker, taker));
    assert!(
        replay.program_result.is_err(),
        "accepted offer must not be reusable"
    );
}

#[test]
fn wrong_taker_cannot_accept_and_failed_execution_does_not_move_assets() {
    let (context, config, fee_receiver, maker, taker, stranger) = setup();
    let (create, offer, vault) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        2,
        vec![sol(MAKER_SOL)],
        vec![sol(TAKER_SOL)],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);
    let maker_before = account(&context, &maker).lamports;
    let stranger_before = account(&context, &stranger).lamports;
    let vault_before = account(&context, &vault).lamports;

    let result = context.process_instruction(&accept_ix(config, offer, vault, maker, stranger));
    assert!(result.program_result.is_err());
    assert_eq!(account(&context, &maker).lamports, maker_before);
    assert_eq!(account(&context, &stranger).lamports, stranger_before);
    assert_eq!(account(&context, &vault).lamports, vault_before);
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Active);
}

#[test]
fn maker_can_cancel_before_deadline_and_cannot_cancel_twice() {
    let (context, config, fee_receiver, maker, taker, _) = setup();
    let maker_before_create = account(&context, &maker).lamports;
    let (create, offer, vault) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        3,
        vec![sol(MAKER_SOL)],
        vec![],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);
    context.process_and_validate_instruction(&cancel_ix(offer, vault, maker), &[Check::success()]);

    assert_eq!(account(&context, &vault).lamports, 0);
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Cancelled);
    // Offer rent and the non-refundable platform fee remain paid; escrowed SOL is returned.
    assert!(account(&context, &maker).lamports > maker_before_create - PLATFORM_FEE - MAKER_SOL);
    let second = context.process_instruction(&cancel_ix(offer, vault, maker));
    assert!(second.program_result.is_err());
}

#[test]
fn third_party_can_reclaim_expired_offer_only_to_maker() {
    let (mut context, config, fee_receiver, maker, taker, stranger) = setup();
    let (create, offer, vault) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        4,
        vec![sol(MAKER_SOL)],
        vec![sol(TAKER_SOL)],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);
    let expires_at = offer_state(&context, &offer).expires_at;
    let maker_before_claim = account(&context, &maker).lamports;
    let stranger_before_claim = account(&context, &stranger).lamports;

    context.mollusk.sysvars.clock.unix_timestamp = expires_at + 1;
    let late_accept = context.process_instruction(&accept_ix(config, offer, vault, maker, taker));
    assert!(late_accept.program_result.is_err());
    let late_cancel = context.process_instruction(&cancel_ix(offer, vault, maker));
    assert!(late_cancel.program_result.is_err());

    context.process_and_validate_instruction(
        &claim_ix(offer, vault, maker, stranger),
        &[Check::success()],
    );
    assert_eq!(
        account(&context, &maker).lamports,
        maker_before_claim + MAKER_SOL
    );
    assert_eq!(account(&context, &stranger).lamports, stranger_before_claim);
    assert_eq!(account(&context, &vault).lamports, 0);
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Reclaimed);
    assert!(context
        .process_instruction(&claim_ix(offer, vault, maker, stranger))
        .program_result
        .is_err());
}

#[test]
fn malformed_asset_arrays_are_rejected_by_the_deployed_program() {
    let (context, config, fee_receiver, maker, taker, _) = setup();
    let duplicate_sol = vec![sol(1), sol(2)];
    let (duplicate_ix, _, _) =
        create_sol_offer_ix(config, fee_receiver, maker, taker, 5, duplicate_sol, vec![]);
    assert!(context
        .process_instruction(&duplicate_ix)
        .program_result
        .is_err());

    let too_many = (0..=MAX_ASSETS_PER_SIDE)
        .map(|_| sol(1))
        .collect::<Vec<_>>();
    let (too_many_ix, _, _) =
        create_sol_offer_ix(config, fee_receiver, maker, taker, 6, too_many, vec![]);
    assert!(context
        .process_instruction(&too_many_ix)
        .program_result
        .is_err());

    let (empty_ix, _, _) =
        create_sol_offer_ix(config, fee_receiver, maker, taker, 7, vec![], vec![]);
    assert!(context
        .process_instruction(&empty_ix)
        .program_result
        .is_err());
}

#[test]
fn exactly_eight_assets_on_one_side_are_accepted() {
    let mints = (0..MAX_ASSETS_PER_SIDE)
        .map(|_| Pubkey::new_unique())
        .collect::<Vec<_>>();
    let (context, config, fee_receiver, maker, taker, _) = setup();
    {
        let mut store = context.account_store.borrow_mut();
        for mint in &mints {
            store.insert(*mint, mint_account(6, 1_000));
        }
    }
    let taker_assets = mints.iter().map(|mint| spl(*mint, 1)).collect();
    let (create, offer, vault) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        8,
        vec![sol(1)],
        taker_assets,
    );
    let create = with_remaining(
        create,
        mints
            .iter()
            .map(|mint| AccountMeta::new_readonly(*mint, false))
            .collect(),
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);
    assert_eq!(offer_state(&context, &offer).maker_assets.len(), 1);
    assert_eq!(offer_state(&context, &offer).taker_assets.len(), 8);
    context.process_and_validate_instruction(&cancel_ix(offer, vault, maker), &[Check::success()]);
}

#[test]
fn instruction_surface_contains_no_admin_mutation_calls() {
    let forbidden = [
        "set_fee",
        "set_pause",
        "allow_",
        "disallow_",
        "emergency_withdraw",
    ];
    let exposed = [
        "initialize_config",
        "create_offer",
        "accept_offer",
        "cancel_offer",
        "claim_expired_offer",
    ];
    for name in exposed {
        for prefix in forbidden {
            assert!(!name.contains(prefix));
        }
    }
    assert_eq!(Offer::DISCRIMINATOR.len(), 8);
}

#[test]
fn spl_for_spl_uses_classic_token_cpis_and_returns_vault_rent() {
    let maker_mint = Pubkey::new_unique();
    let taker_mint = Pubkey::new_unique();
    let (context, config, fee_receiver, maker, taker, _) = setup();
    let nonce = 20;
    let (offer, vault) = offer_addresses(maker, nonce);

    let (maker_source, maker_source_account) = token_account(maker_mint, maker, 500);
    let (taker_source, taker_source_account) = token_account(taker_mint, taker, 900);
    let (maker_destination, maker_destination_account) = token_account(taker_mint, maker, 0);
    let (taker_destination, taker_destination_account) = token_account(maker_mint, taker, 0);
    let (vault_ata, _) = token_account(maker_mint, vault, 0);
    {
        let mut store = context.account_store.borrow_mut();
        store.insert(maker_mint, mint_account(6, 500));
        store.insert(taker_mint, mint_account(6, 900));
        store.insert(maker_source, maker_source_account);
        store.insert(taker_source, taker_source_account);
        store.insert(maker_destination, maker_destination_account);
        store.insert(taker_destination, taker_destination_account);
    }

    let (create, _, _) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        nonce,
        vec![spl(maker_mint, 200)],
        vec![spl(taker_mint, 300)],
    );
    let create = with_remaining(
        create,
        vec![
            AccountMeta::new_readonly(maker_mint, false),
            AccountMeta::new(maker_source, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new_readonly(taker_mint, false),
        ],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);
    assert_eq!(token_balance(&context, &maker_source), 300);
    assert_eq!(token_balance(&context, &vault_ata), 200);
    let maker_before_accept = account(&context, &maker).lamports;
    let vault_rent = account(&context, &vault_ata).lamports;

    let accept = with_remaining(
        accept_ix(config, offer, vault, maker, taker),
        vec![
            // taker -> maker
            AccountMeta::new_readonly(taker_mint, false),
            AccountMeta::new(taker_source, false),
            AccountMeta::new(maker_destination, false),
            // vault -> taker, plus canonical maker refund ATA for grief surplus
            AccountMeta::new_readonly(maker_mint, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new(taker_destination, false),
            AccountMeta::new(maker_source, false),
        ],
    );
    let mut substituted = accept.clone();
    substituted.accounts[12].pubkey = maker_source;
    let source_before_substitution = token_balance(&context, &maker_source);
    let vault_before_substitution = token_balance(&context, &vault_ata);
    assert!(context
        .process_instruction(&substituted)
        .program_result
        .is_err());
    assert_eq!(
        token_balance(&context, &maker_source),
        source_before_substitution
    );
    assert_eq!(
        token_balance(&context, &vault_ata),
        vault_before_substitution
    );
    context.process_and_validate_instruction(&accept, &[Check::success()]);

    assert_eq!(token_balance(&context, &taker_source), 600);
    assert_eq!(token_balance(&context, &maker_destination), 300);
    assert_eq!(token_balance(&context, &taker_destination), 200);
    assert_eq!(token_balance(&context, &maker_source), 300);
    assert_eq!(account(&context, &vault_ata).lamports, 0);
    assert_eq!(
        account(&context, &maker).lamports,
        maker_before_accept + vault_rent
    );
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Accepted);
}

#[test]
fn multi_asset_accept_rolls_back_when_one_taker_balance_is_missing() {
    let maker_mint = Pubkey::new_unique();
    let taker_mint_ok = Pubkey::new_unique();
    let taker_mint_missing = Pubkey::new_unique();
    let (context, config, fee_receiver, maker, taker, _) = setup();
    let nonce = 21;
    let (offer, vault) = offer_addresses(maker, nonce);

    let (maker_source, maker_source_account) = token_account(maker_mint, maker, 100);
    let (taker_ok, taker_ok_account) = token_account(taker_mint_ok, taker, 80);
    let (taker_missing, taker_missing_account) = token_account(taker_mint_missing, taker, 1);
    let (maker_dest_ok, maker_dest_ok_account) = token_account(taker_mint_ok, maker, 0);
    let (maker_dest_missing, maker_dest_missing_account) =
        token_account(taker_mint_missing, maker, 0);
    let (taker_destination, taker_destination_account) = token_account(maker_mint, taker, 0);
    let (vault_ata, _) = token_account(maker_mint, vault, 0);
    {
        let mut store = context.account_store.borrow_mut();
        store.insert(maker_mint, mint_account(0, 100));
        store.insert(taker_mint_ok, mint_account(0, 80));
        store.insert(taker_mint_missing, mint_account(0, 1));
        store.insert(maker_source, maker_source_account);
        store.insert(taker_ok, taker_ok_account);
        store.insert(taker_missing, taker_missing_account);
        store.insert(maker_dest_ok, maker_dest_ok_account);
        store.insert(maker_dest_missing, maker_dest_missing_account);
        store.insert(taker_destination, taker_destination_account);
    }

    let (create, _, _) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        nonce,
        vec![spl(maker_mint, 40)],
        vec![spl(taker_mint_ok, 50), spl(taker_mint_missing, 2)],
    );
    let create = with_remaining(
        create,
        vec![
            AccountMeta::new_readonly(maker_mint, false),
            AccountMeta::new(maker_source, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new_readonly(taker_mint_ok, false),
            AccountMeta::new_readonly(taker_mint_missing, false),
        ],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);

    let balances_before = (
        token_balance(&context, &taker_ok),
        token_balance(&context, &taker_missing),
        token_balance(&context, &maker_dest_ok),
        token_balance(&context, &maker_dest_missing),
        token_balance(&context, &vault_ata),
    );
    let accept = with_remaining(
        accept_ix(config, offer, vault, maker, taker),
        vec![
            AccountMeta::new_readonly(taker_mint_ok, false),
            AccountMeta::new(taker_ok, false),
            AccountMeta::new(maker_dest_ok, false),
            AccountMeta::new_readonly(taker_mint_missing, false),
            AccountMeta::new(taker_missing, false),
            AccountMeta::new(maker_dest_missing, false),
            AccountMeta::new_readonly(maker_mint, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new(taker_destination, false),
            AccountMeta::new(maker_source, false),
        ],
    );
    let result = context.process_instruction(&accept);
    assert!(result.program_result.is_err());
    assert_eq!(
        (
            token_balance(&context, &taker_ok),
            token_balance(&context, &taker_missing),
            token_balance(&context, &maker_dest_ok),
            token_balance(&context, &maker_dest_missing),
            token_balance(&context, &vault_ata),
        ),
        balances_before
    );
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Active);
}

#[test]
fn nft_for_nft_supports_arbitrary_collections() {
    let maker_mint = Pubkey::new_unique();
    let taker_mint = Pubkey::new_unique();
    let maker_collection = Pubkey::new_unique();
    let taker_collection = Pubkey::new_unique();
    let (context, config, fee_receiver, maker, taker, _) = setup();
    let nonce = 30;
    let (offer, vault) = offer_addresses(maker, nonce);
    let (maker_metadata, maker_metadata_account) = metadata_account(maker_mint, maker_collection);
    let (taker_metadata, taker_metadata_account) =
        metadata_account_with_verification(taker_mint, taker_collection, false);
    let (maker_source, maker_source_account) = token_account(maker_mint, maker, 1);
    let (taker_source, taker_source_account) = token_account(taker_mint, taker, 1);
    let (maker_destination, maker_destination_account) = token_account(taker_mint, maker, 0);
    let (taker_destination, taker_destination_account) = token_account(maker_mint, taker, 0);
    let (vault_ata, _) = token_account(maker_mint, vault, 0);
    {
        let mut store = context.account_store.borrow_mut();
        store.insert(maker_mint, mint_account(0, 1));
        store.insert(taker_mint, mint_account(0, 1));
        store.insert(maker_metadata, maker_metadata_account);
        store.insert(taker_metadata, taker_metadata_account);
        store.insert(maker_source, maker_source_account);
        store.insert(taker_source, taker_source_account);
        store.insert(maker_destination, maker_destination_account);
        store.insert(taker_destination, taker_destination_account);
    }

    let (create, _, _) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        nonce,
        vec![nft(maker_mint)],
        vec![nft(taker_mint)],
    );
    let create = with_remaining(
        create,
        vec![
            AccountMeta::new_readonly(maker_mint, false),
            AccountMeta::new(maker_source, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new_readonly(maker_metadata, false),
            AccountMeta::new_readonly(taker_mint, false),
            AccountMeta::new_readonly(taker_metadata, false),
        ],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);
    assert_eq!(token_balance(&context, &maker_source), 0);
    assert_eq!(token_balance(&context, &vault_ata), 1);

    let accept = with_remaining(
        accept_ix(config, offer, vault, maker, taker),
        vec![
            AccountMeta::new_readonly(taker_mint, false),
            AccountMeta::new(taker_source, false),
            AccountMeta::new(maker_destination, false),
            AccountMeta::new_readonly(taker_metadata, false),
            AccountMeta::new_readonly(maker_mint, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new(taker_destination, false),
            AccountMeta::new(maker_source, false),
            AccountMeta::new_readonly(maker_metadata, false),
        ],
    );
    context.process_and_validate_instruction(&accept, &[Check::success()]);
    assert_eq!(token_balance(&context, &taker_source), 0);
    assert_eq!(token_balance(&context, &maker_destination), 1);
    assert_eq!(token_balance(&context, &taker_destination), 1);
    assert_eq!(account(&context, &vault_ata).lamports, 0);
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Accepted);
}

#[test]
fn nft_without_collection_is_supported_and_refundable() {
    let mint = Pubkey::new_unique();
    let (context, config, fee_receiver, maker, taker, _) = setup();
    let (offer, vault) = offer_addresses(maker, 31);
    let (metadata, metadata_account) = standalone_metadata_account(mint);
    let (source, source_account) = token_account(mint, maker, 1);
    let (vault_ata, _) = token_account(mint, vault, 0);
    {
        let mut store = context.account_store.borrow_mut();
        store.insert(mint, mint_account(0, 1));
        store.insert(metadata, metadata_account);
        store.insert(source, source_account);
    }
    let (create, _, _) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        31,
        vec![nft(mint)],
        vec![],
    );
    let create = with_remaining(
        create,
        vec![
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(source, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new_readonly(metadata, false),
        ],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);
    assert_eq!(token_balance(&context, &source), 0);
    assert_eq!(token_balance(&context, &vault_ata), 1);

    let cancel = with_remaining(
        cancel_ix(offer, vault, maker),
        vec![
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new(source, false),
        ],
    );
    context.process_and_validate_instruction(&cancel, &[Check::success()]);
    assert_eq!(token_balance(&context, &source), 1);
    assert_eq!(account(&context, &vault_ata).lamports, 0);
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Cancelled);
}

#[test]
fn token_2022_and_frozen_classic_accounts_are_rejected() {
    let token_2022_mint = Pubkey::new_unique();
    let frozen_mint = Pubkey::new_unique();
    let (context, config, fee_receiver, maker, taker, _) = setup();

    let (token_2022_source, token_2022_source_account) = token_account(token_2022_mint, maker, 10);
    let (_, token_2022_vault) = offer_addresses(maker, 40);
    let (token_2022_vault_ata, _) = token_account(token_2022_mint, token_2022_vault, 0);
    let mut token_2022_mint_account = mint_account(6, 10);
    token_2022_mint_account.owner = TOKEN_2022_PROGRAM_ID;

    let (frozen_source, mut frozen_source_account) = token_account(frozen_mint, maker, 10);
    let (_, frozen_vault) = offer_addresses(maker, 41);
    let (frozen_vault_ata, _) = token_account(frozen_mint, frozen_vault, 0);
    let mut frozen_data = TokenAccount::unpack(&frozen_source_account.data).unwrap();
    frozen_data.state = AccountState::Frozen;
    TokenAccount::pack(frozen_data, &mut frozen_source_account.data).unwrap();
    {
        let mut store = context.account_store.borrow_mut();
        store.insert(token_2022_mint, token_2022_mint_account);
        store.insert(token_2022_source, token_2022_source_account);
        store.insert(frozen_mint, mint_account(6, 10));
        store.insert(frozen_source, frozen_source_account);
    }

    let (token_2022_create, _, _) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        40,
        vec![spl(token_2022_mint, 5)],
        vec![],
    );
    let token_2022_create = with_remaining(
        token_2022_create,
        vec![
            AccountMeta::new_readonly(token_2022_mint, false),
            AccountMeta::new(token_2022_source, false),
            AccountMeta::new(token_2022_vault_ata, false),
        ],
    );
    assert!(context
        .process_instruction(&token_2022_create)
        .program_result
        .is_err());
    assert_eq!(token_balance(&context, &token_2022_source), 10);

    let (frozen_create, _, _) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        41,
        vec![spl(frozen_mint, 5)],
        vec![],
    );
    let frozen_create = with_remaining(
        frozen_create,
        vec![
            AccountMeta::new_readonly(frozen_mint, false),
            AccountMeta::new(frozen_source, false),
            AccountMeta::new(frozen_vault_ata, false),
        ],
    );
    assert!(context
        .process_instruction(&frozen_create)
        .program_result
        .is_err());
    assert_eq!(
        TokenAccount::unpack(&account(&context, &frozen_source).data)
            .unwrap()
            .amount,
        10
    );
}

#[test]
fn mixed_two_nft_sol_spl_for_nft_spl_executes_atomically() {
    let maker_nft_1 = Pubkey::new_unique();
    let maker_nft_2 = Pubkey::new_unique();
    let maker_spl = Pubkey::new_unique();
    let taker_nft = Pubkey::new_unique();
    let taker_spl = Pubkey::new_unique();
    let maker_collection = Pubkey::new_unique();
    let taker_collection = Pubkey::new_unique();
    let (context, config, fee_receiver, maker, taker, _) = setup();
    let nonce = 50;
    let (offer, vault) = offer_addresses(maker, nonce);

    let (maker_nft_1_meta, maker_nft_1_meta_account) =
        metadata_account(maker_nft_1, maker_collection);
    let (maker_nft_2_meta, maker_nft_2_meta_account) =
        metadata_account(maker_nft_2, maker_collection);
    let (taker_nft_meta, taker_nft_meta_account) = metadata_account(taker_nft, taker_collection);

    let (maker_nft_1_source, maker_nft_1_source_account) = token_account(maker_nft_1, maker, 1);
    let (maker_nft_2_source, maker_nft_2_source_account) = token_account(maker_nft_2, maker, 1);
    let (maker_spl_source, maker_spl_source_account) = token_account(maker_spl, maker, 1_000);
    let (taker_nft_source, taker_nft_source_account) = token_account(taker_nft, taker, 1);
    let (taker_spl_source, taker_spl_source_account) = token_account(taker_spl, taker, 800);

    let (maker_nft_destination, maker_nft_destination_account) = token_account(taker_nft, maker, 0);
    let (maker_spl_destination, maker_spl_destination_account) = token_account(taker_spl, maker, 0);
    let (taker_nft_1_destination, taker_nft_1_destination_account) =
        token_account(maker_nft_1, taker, 0);
    let (taker_nft_2_destination, taker_nft_2_destination_account) =
        token_account(maker_nft_2, taker, 0);
    let (taker_spl_destination, taker_spl_destination_account) = token_account(maker_spl, taker, 0);

    let (maker_nft_1_vault, _) = token_account(maker_nft_1, vault, 0);
    let (maker_nft_2_vault, _) = token_account(maker_nft_2, vault, 0);
    let (maker_spl_vault, _) = token_account(maker_spl, vault, 0);

    {
        let mut store = context.account_store.borrow_mut();
        store.insert(maker_nft_1, mint_account(0, 1));
        store.insert(maker_nft_2, mint_account(0, 1));
        store.insert(taker_nft, mint_account(0, 1));
        store.insert(maker_spl, mint_account(6, 1_000));
        store.insert(taker_spl, mint_account(6, 800));
        store.insert(maker_nft_1_meta, maker_nft_1_meta_account);
        store.insert(maker_nft_2_meta, maker_nft_2_meta_account);
        store.insert(taker_nft_meta, taker_nft_meta_account);
        store.insert(maker_nft_1_source, maker_nft_1_source_account);
        store.insert(maker_nft_2_source, maker_nft_2_source_account);
        store.insert(maker_spl_source, maker_spl_source_account);
        store.insert(taker_nft_source, taker_nft_source_account);
        store.insert(taker_spl_source, taker_spl_source_account);
        store.insert(maker_nft_destination, maker_nft_destination_account);
        store.insert(maker_spl_destination, maker_spl_destination_account);
        store.insert(taker_nft_1_destination, taker_nft_1_destination_account);
        store.insert(taker_nft_2_destination, taker_nft_2_destination_account);
        store.insert(taker_spl_destination, taker_spl_destination_account);
    }

    let maker_assets = vec![
        nft(maker_nft_1),
        sol(100_000_000),
        spl(maker_spl, 400),
        nft(maker_nft_2),
    ];
    let taker_assets = vec![nft(taker_nft), spl(taker_spl, 250)];
    let (create, _, _) = create_sol_offer_ix(
        config,
        fee_receiver,
        maker,
        taker,
        nonce,
        maker_assets,
        taker_assets,
    );
    let create = with_remaining(
        create,
        vec![
            AccountMeta::new_readonly(maker_nft_1, false),
            AccountMeta::new(maker_nft_1_source, false),
            AccountMeta::new(maker_nft_1_vault, false),
            AccountMeta::new_readonly(maker_nft_1_meta, false),
            AccountMeta::new_readonly(maker_spl, false),
            AccountMeta::new(maker_spl_source, false),
            AccountMeta::new(maker_spl_vault, false),
            AccountMeta::new_readonly(maker_nft_2, false),
            AccountMeta::new(maker_nft_2_source, false),
            AccountMeta::new(maker_nft_2_vault, false),
            AccountMeta::new_readonly(maker_nft_2_meta, false),
            AccountMeta::new_readonly(taker_nft, false),
            AccountMeta::new_readonly(taker_nft_meta, false),
            AccountMeta::new_readonly(taker_spl, false),
        ],
    );
    context.process_and_validate_instruction(&create, &[Check::success()]);
    assert_eq!(account(&context, &vault).lamports, 100_000_000);

    let taker_lamports_before = account(&context, &taker).lamports;
    let accept = with_remaining(
        accept_ix(config, offer, vault, maker, taker),
        vec![
            // taker NFT + SPL -> maker
            AccountMeta::new_readonly(taker_nft, false),
            AccountMeta::new(taker_nft_source, false),
            AccountMeta::new(maker_nft_destination, false),
            AccountMeta::new_readonly(taker_nft_meta, false),
            AccountMeta::new_readonly(taker_spl, false),
            AccountMeta::new(taker_spl_source, false),
            AccountMeta::new(maker_spl_destination, false),
            // maker NFT 1 -> taker
            AccountMeta::new_readonly(maker_nft_1, false),
            AccountMeta::new(maker_nft_1_vault, false),
            AccountMeta::new(taker_nft_1_destination, false),
            AccountMeta::new(maker_nft_1_source, false),
            AccountMeta::new_readonly(maker_nft_1_meta, false),
            // maker SPL -> taker
            AccountMeta::new_readonly(maker_spl, false),
            AccountMeta::new(maker_spl_vault, false),
            AccountMeta::new(taker_spl_destination, false),
            AccountMeta::new(maker_spl_source, false),
            // maker NFT 2 -> taker
            AccountMeta::new_readonly(maker_nft_2, false),
            AccountMeta::new(maker_nft_2_vault, false),
            AccountMeta::new(taker_nft_2_destination, false),
            AccountMeta::new(maker_nft_2_source, false),
            AccountMeta::new_readonly(maker_nft_2_meta, false),
        ],
    );
    context.process_and_validate_instruction(&accept, &[Check::success()]);

    assert_eq!(token_balance(&context, &maker_nft_destination), 1);
    assert_eq!(token_balance(&context, &maker_spl_destination), 250);
    assert_eq!(token_balance(&context, &taker_nft_1_destination), 1);
    assert_eq!(token_balance(&context, &taker_nft_2_destination), 1);
    assert_eq!(token_balance(&context, &taker_spl_destination), 400);
    assert_eq!(token_balance(&context, &maker_spl_source), 600);
    assert_eq!(token_balance(&context, &taker_spl_source), 550);
    assert_eq!(
        account(&context, &taker).lamports,
        taker_lamports_before + 100_000_000
    );
    assert_eq!(account(&context, &maker_nft_1_vault).lamports, 0);
    assert_eq!(account(&context, &maker_nft_2_vault).lamports, 0);
    assert_eq!(account(&context, &maker_spl_vault).lamports, 0);
    assert_eq!(offer_state(&context, &offer).status, OfferStatus::Accepted);
}
