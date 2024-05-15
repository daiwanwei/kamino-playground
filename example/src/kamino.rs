use std::{collections::HashMap, error::Error, str::FromStr};

use anchor_client::{
    anchor_lang::{
        prelude::borsh::{to_vec, BorshDeserialize, BorshSerialize},
        InstructionData, ToAccountMetas,
    },
    solana_sdk::{
        instruction::Instruction,
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
        system_instruction::create_account,
        system_program::ID as SYSTEM_ID,
        sysvar::{instructions::ID as SYSVAR_INSTRUCTIONS_ID, rent::ID as SYSVAR_RENT_ID},
    },
    Program,
};
use anchor_spl::mint;
use encoding_rs::UTF_8;
use kamino_lend::{
    accounts as kamino_accounts, instruction as kamino_instruction, state as kamino_state,
    typedefs as kamino_typedefs,
    ID as KAMINO_LENDING_ID,
};
use spl_associated_token_account::get_associated_token_address;

pub fn create_lending_market(
    program: &Program<&Keypair>,
    payer: &Keypair,
    lending_market: &Keypair,
) -> Result<Signature, Box<dyn Error>> {
    let size = std::mem::size_of::<kamino_state::LendingMarket>() as u64 + 8;
    let market_authority = pda::get_market_authority(&lending_market.pubkey());
    let rpc_client = program.rpc();
    let res = program
        .request()
        .instruction(create_account(
            &payer.pubkey(),
            &lending_market.pubkey(),
            rpc_client.get_minimum_balance_for_rent_exemption(size as usize)?,
            size,
            &KAMINO_LENDING_ID,
        ))
        .instruction(
            instruction::init_lending_market(&payer.pubkey(), &lending_market.pubkey(), [0; 32])
                .unwrap(),
        )
        .signer(payer)
        .signer(lending_market)
        .send()?;
    Ok(res)
}

pub fn init_reserve(
    program: &Program<&Keypair>,
    payer: &Keypair,
    lending_market: &Keypair,
    reserve: &Keypair,
    mint: &Pubkey,
    token_program_id: &Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let size = std::mem::size_of::<kamino_state::Reserve>() as u64 + 8;
    let market_authority = pda::get_market_authority(&lending_market.pubkey());
    let [reserve_liquidity_supply, reserve_collateral_mint, reserve_collateral_supply, reserve_fee_vault] =
        get_reserve_pdas(&lending_market.pubkey(), mint);
    let rpc_client = program.rpc();
    let res = program
        .request()
        .instruction(create_account(
            &payer.pubkey(),
            &reserve.pubkey(),
            rpc_client.get_minimum_balance_for_rent_exemption(size as usize)?,
            size,
            &KAMINO_LENDING_ID,
        ))
        .instruction(
            instruction::init_reserve(
                &lending_market.pubkey(),
                &reserve.pubkey(),
                mint,
                &payer.pubkey(),
                token_program_id,
            )
            .unwrap(),
        )
        .signer(payer)
        .signer(reserve)
        .send()?;
    Ok(res)
}

pub fn update_reserve_config(
    program: &Program<&Keypair>,
    payer: &Keypair,
    lending_market: &Pubkey,
    reserve: &Pubkey,
    reserve_config: kamino_typedefs::ReserveConfig,
) -> Result<Signature, Box<dyn Error>> {
    let res = program
        .request()
        .instruction(instruction::upadte_entire_reserve_config(
            reserve,
            &payer.pubkey(),
            lending_market,
            reserve_config,
        )?)
        .signer(payer)
        .send()?;
    Ok(res)
}

pub fn init_user_metadata(
    program: &Program<&Keypair>,
    payer: &Keypair,
) -> Result<Signature, Box<dyn Error>> {
    let res = program
        .request()
        .instruction(instruction::init_user_metadata(&payer.pubkey()).unwrap())
        .signer(payer)
        .send()?;
    Ok(res)
}

pub fn init_obligation(
    program: &Program<&Keypair>,
    payer: &Keypair,
    lending_market: &Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let res = program
        .request()
        .instruction(
            instruction::init_obligation(
                &payer.pubkey(),
                lending_market,
                &spl_token::ID,
                0,
                0,
                [0; 32],
                [0; 32],
            )
            .unwrap(),
        )
        .signer(payer)
        .send()?;
    Ok(res)
}

pub fn deposit_reserve_liquidity_and_obligation_collateral(
    program: &Program<&Keypair>,
    payer: &Keypair,
    lending_market: &Pubkey,
    reserve: &Pubkey,
    mint: &Pubkey,
    liquidity_amount: u64,
    token_program_id: &Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let pyth_oracle = Pubkey::from_str("E4v1BBgoso9s64TQvmyownAVJbhbEPGyzA3qn4n46qj9").unwrap();
    let res = program
        .request()
        .instruction(instruction::refresh_reserve(reserve, lending_market, &pyth_oracle).unwrap())
        .instruction(instruction::refresh_obligation(&payer.pubkey(), lending_market).unwrap())
        .instruction(
            instruction::deposit_reserve_liquidity_and_obligation_collateral(
                lending_market,
                &payer.pubkey(),
                mint,
                reserve,
                &token_program_id,
                liquidity_amount,
            )
            .unwrap(),
        )
        .signer(payer)
        .send()?;
    println!("{:?}", res);
    Ok(res)
}

pub fn get_reserve_pdas(lending_market: &Pubkey, mint: &Pubkey) -> [Pubkey; 4] {
    return [
        pda::get_reserve_liquidity_supply(lending_market, mint),
        pda::get_reserve_collateral_mint(lending_market, mint),
        pda::get_reserve_collateral_supply(lending_market, mint),
        pda::get_reserve_fee_vault(lending_market, mint),
    ];
}

pub mod instruction {
    use std::error::Error;

    use anchor_client::{
        anchor_lang::{prelude::borsh::to_vec, InstructionData, ToAccountMetas},
        solana_sdk::{
            instruction::Instruction,
            pubkey::Pubkey,
            system_program::ID as SYSTEM_ID,
            sysvar::{instructions::ID as SYSVAR_INSTRUCTIONS_ID, rent::ID as SYSVAR_RENT_ID},
        },
    };
    use kamino_lend::{
        accounts, instruction,
        typedefs::{InitObligationArgs, ReserveConfig},
        ID as KAMINO_LENDING_ID,
    };
    use spl_associated_token_account::get_associated_token_address;

    use super::pda::get_user_obligation;
    use crate::kamino::pda;

    pub fn init_lending_market(
        owner: &Pubkey,
        lending_market: &Pubkey,
        quote_currency: [u8; 32],
    ) -> Result<Instruction, Box<dyn Error>> {
        let market_authority = pda::get_market_authority(&lending_market);
        Ok(Instruction {
            program_id: KAMINO_LENDING_ID,
            accounts: accounts::InitLendingMarket {
                lending_market_owner: *owner,
                lending_market: *lending_market,
                lending_market_authority: market_authority,
                system_program: SYSTEM_ID,
                rent: SYSVAR_RENT_ID,
            }
            .to_account_metas(Some(true)),
            data: instruction::InitLendingMarket { _quote_currency: quote_currency }.data(),
        })
    }

    pub fn init_reserve(
        lending_market: &Pubkey,
        reserve: &Pubkey,
        reserve_liquidity_mint: &Pubkey,
        lending_market_owner: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Instruction, Box<dyn Error>> {
        let market_authority = pda::get_market_authority(&lending_market);
        let reserve_liquidity_supply =
            pda::get_reserve_liquidity_supply(lending_market, reserve_liquidity_mint);
        let reserve_collateral_mint =
            pda::get_reserve_collateral_mint(lending_market, reserve_liquidity_mint);
        let reserve_collateral_supply =
            pda::get_reserve_collateral_supply(lending_market, reserve_liquidity_mint);
        let reserve_fee_vault = pda::get_reserve_fee_vault(lending_market, reserve_liquidity_mint);
        Ok(Instruction {
            program_id: KAMINO_LENDING_ID,
            accounts: accounts::InitReserve {
                reserve: *reserve,
                reserve_liquidity_supply,
                reserve_collateral_mint,
                reserve_collateral_supply,
                fee_receiver: reserve_fee_vault,
                reserve_liquidity_mint: *reserve_liquidity_mint,
                lending_market_owner: *lending_market_owner,
                lending_market: *lending_market,
                lending_market_authority: market_authority,
                system_program: SYSTEM_ID,
                rent: SYSVAR_RENT_ID,
                token_program: *token_program,
            }
            .to_account_metas(Some(true)),
            data: instruction::InitReserve {}.data(),
        })
    }

    pub fn upadte_entire_reserve_config(
        reserve: &Pubkey,
        lending_market_owner: &Pubkey,
        lending_market: &Pubkey,
        reserve_config: ReserveConfig,
    ) -> Result<Instruction, Box<dyn Error>> {
        let encoded_config = to_vec::<ReserveConfig>(&reserve_config).unwrap();
        Ok(Instruction {
            program_id: KAMINO_LENDING_ID,
            accounts: accounts::UpdateEntireReserveConfig {
                reserve: *reserve,
                lending_market_owner: *lending_market_owner,
                lending_market: *lending_market,
            }
            .to_account_metas(Some(true)),
            data: instruction::UpdateEntireReserveConfig {
                _mode: 25,
                _value: encoded_config.try_into().unwrap(),
            }
            .data(),
        })
    }

    pub fn init_user_metadata(user: &Pubkey) -> Result<Instruction, Box<dyn Error>> {
        Ok(Instruction {
            program_id: KAMINO_LENDING_ID,
            accounts: accounts::InitUserMetadata {
                user_metadata: pda::get_user_metadata(user),
                owner: *user,
                fee_payer: *user,
                referrer_user_metadata: KAMINO_LENDING_ID,
                system_program: SYSTEM_ID,
                rent: SYSVAR_RENT_ID,
            }
            .to_account_metas(Some(true)),
            data: instruction::InitUserMetadata { _user_lookup_table: Pubkey::default() }.data(),
        })
    }

    pub fn init_obligation(
        owner: &Pubkey,
        lending_market: &Pubkey,
        token_program: &Pubkey,
        tag: u8,
        id: u8,
        seed1: [u8; 32],
        seed2: [u8; 32],
    ) -> Result<Instruction, Box<dyn Error>> {
        Ok(Instruction {
            program_id: KAMINO_LENDING_ID,
            accounts: accounts::InitObligation {
                obligation_owner: *owner,
                fee_payer: *owner,
                obligation: pda::get_user_obligation(lending_market, owner),
                lending_market: *lending_market,
                seed1_account: Pubkey::from(seed1),
                seed2_account: Pubkey::from(seed2),
                owner_user_metadata: pda::get_user_metadata(owner),
                token_program: *token_program,
                system_program: SYSTEM_ID,
                rent: SYSVAR_RENT_ID,
            }
            .to_account_metas(Some(true)),
            data: instruction::InitObligation { _args: InitObligationArgs { tag, id } }.data(),
        })
    }

    pub fn refresh_reserve(
        reserve: &Pubkey,
        lending_market: &Pubkey,
        pyth_oracle: &Pubkey,
    ) -> Result<Instruction, Box<dyn Error>> {
        Ok(Instruction {
            program_id: KAMINO_LENDING_ID,
            accounts: accounts::RefreshReserve {
                reserve: *reserve,
                lending_market: *lending_market,
                pyth_oracle: *pyth_oracle,
                switchboard_price_oracle: KAMINO_LENDING_ID,
                switchboard_twap_oracle: KAMINO_LENDING_ID,
                scope_prices: KAMINO_LENDING_ID,
            }
            .to_account_metas(Some(true)),
            data: instruction::RefreshReserve {}.data(),
        })
    }

    pub fn refresh_obligation(
        user: &Pubkey,
        lending_market: &Pubkey,
    ) -> Result<Instruction, Box<dyn Error>> {
        Ok(Instruction {
            program_id: KAMINO_LENDING_ID,
            accounts: accounts::RefreshObligation {
                obligation: pda::get_user_obligation(lending_market, user),
                lending_market: *lending_market,
            }
            .to_account_metas(Some(true)),
            data: instruction::RefreshObligation {}.data(),
        })
    }

    pub fn deposit_reserve_liquidity_and_obligation_collateral(
        lending_market: &Pubkey,
        user: &Pubkey,
        mint: &Pubkey,
        reserve: &Pubkey,
        token_program: &Pubkey,
        deposit_amount: u64,
    ) -> Result<Instruction, Box<dyn Error>> {
        let [reserve_liquidity_supply, reserve_collateral_mint, reserve_collateral_supply, reserve_fee_vault] = [
            pda::get_reserve_liquidity_supply(lending_market, mint),
            pda::get_reserve_collateral_mint(lending_market, mint),
            pda::get_reserve_collateral_supply(lending_market, mint),
            pda::get_reserve_fee_vault(lending_market, mint),
        ];
        Ok(Instruction {
            program_id: KAMINO_LENDING_ID,
            accounts: accounts::DepositReserveLiquidityAndObligationCollateral {
                owner: *user,
                obligation: get_user_obligation(lending_market, user),
                user_source_liquidity: get_associated_token_address(user, mint),
                reserve: *reserve,
                lending_market: *lending_market,
                lending_market_authority: pda::get_market_authority(lending_market),
                reserve_destination_deposit_collateral: reserve_collateral_supply,
                reserve_liquidity_supply,
                reserve_collateral_mint,
                token_program: *token_program,
                placeholder_user_destination_collateral: KAMINO_LENDING_ID,
                instruction_sysvar_account: SYSVAR_INSTRUCTIONS_ID,
            }
            .to_account_metas(Some(true)),
            data: instruction::DepositReserveLiquidityAndObligationCollateral {
                _liquidity_amount: deposit_amount,
            }
            .data(),
        })
    }
}

pub mod types {
    use anchor_client::solana_sdk::pubkey::Pubkey;
    pub struct ReserveConfigParams {
        pub loan_to_value_pct: u8,
        pub max_liquidation_bonus_bps: u16,
        pub min_liquidation_bonus_bps: u16,
        pub bad_debt_liquidation_bonus_bps: u16,
        pub liquidation_threshold: u8,
        pub borrow_fee_sf: u64,
        pub flash_loan_fee_sf: u64,
        pub protocol_take_rate: u8,
        pub elevation_groups: [u8; 20],
        pub price_feed: Option<Pubkey>,
        pub borrow_limit: u64,
    }

    impl Default for ReserveConfigParams {
        fn default() -> Self {
            Self {
                loan_to_value_pct: 75,
                max_liquidation_bonus_bps: 500,
                min_liquidation_bonus_bps: 200,
                bad_debt_liquidation_bonus_bps: 10,
                liquidation_threshold: 85,
                borrow_fee_sf: 0,     // Assuming ZERO_FRACTION is 0
                flash_loan_fee_sf: 0, // Assuming ZERO_FRACTION is 0
                protocol_take_rate: 0,
                elevation_groups: [0; 20],
                price_feed: None,
                borrow_limit: 10_000_000_000_000,
            }
        }
    }
}

pub mod utils {
    use crate::kamino::types::ReserveConfigParams;
    use anchor_client::solana_sdk::pubkey::Pubkey;
    use encoding_rs::UTF_8;
    use kamino_lend::typedefs::{
        BorrowRateCurve, CurvePoint, PriceHeuristic, PythConfiguration, ReserveConfig, ReserveFees,
        TokenInfo, WithdrawalCaps,
    };
    use std::str::FromStr;
    use std::{collections::HashMap, convert::TryInto};
    pub fn make_reserve_config(token_name: &str, params: ReserveConfigParams) -> ReserveConfig {
        let pyth_usdc_price =
            Pubkey::from_str("Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD").unwrap();
        let pyth_msol_price =
            Pubkey::from_str("E4v1BBgoso9s64TQvmyownAVJbhbEPGyzA3qn4n46qj9").unwrap();

        let mut price_to_oracle_map = HashMap::new();
        price_to_oracle_map.insert("SOL", pyth_msol_price);
        price_to_oracle_map.insert("STSOL", pyth_msol_price);
        price_to_oracle_map.insert("MSOL", pyth_msol_price);
        price_to_oracle_map.insert("USDC", pyth_usdc_price);
        price_to_oracle_map.insert("USDH", pyth_usdc_price);
        price_to_oracle_map.insert("USDT", pyth_usdc_price);
        price_to_oracle_map.insert("UXD", pyth_usdc_price);
        let pyth_config: PythConfiguration =
            PythConfiguration { price: *price_to_oracle_map.get(token_name).unwrap() };
        println!("{:?}", pyth_config);
        ReserveConfig {
            status: 0,
            loan_to_value_pct: params.loan_to_value_pct,
            liquidation_threshold_pct: params.liquidation_threshold,
            min_liquidation_bonus_bps: params.min_liquidation_bonus_bps,
            protocol_liquidation_fee_pct: 0,
            protocol_take_rate_pct: params.protocol_take_rate,
            asset_tier: 0,
            multiplier_side_boost: [1, 1],
            max_liquidation_bonus_bps: params.max_liquidation_bonus_bps,
            bad_debt_liquidation_bonus_bps: params.bad_debt_liquidation_bonus_bps,
            fees: ReserveFees {
                borrow_fee_sf: params.borrow_fee_sf,
                flash_loan_fee_sf: params.flash_loan_fee_sf,
                padding: [0; 8],
            },
            deposit_limit: 10_000_000_000_000,
            borrow_limit: params.borrow_limit,
            token_info: TokenInfo {
                name: encode_token_name(&token_name),
                heuristic: PriceHeuristic { lower: 0, upper: 0, exp: 0 },
                max_twap_divergence_bps: 0,
                max_age_price_seconds: 1_000_000_000,
                max_age_twap_seconds: 0,
                switchboard_configuration: Default::default(),
                pyth_configuration: pyth_config,
                scope_configuration: Default::default(),
                padding: [0; 20],
            },
            borrow_rate_curve: BorrowRateCurve {
                points: {
                    let mut curve_points = vec![
                        CurvePoint { utilization_rate_bps: 0, borrow_rate_bps: 1 },
                        CurvePoint { utilization_rate_bps: 100, borrow_rate_bps: 100 },
                        CurvePoint { utilization_rate_bps: 10000, borrow_rate_bps: 100000 },
                    ];
                    curve_points.extend(vec![
                        CurvePoint {
                            utilization_rate_bps: 10000,
                            borrow_rate_bps: 100000
                        };
                        8
                    ]);
                    curve_points.try_into().unwrap()
                },
            },
            deposit_withdrawal_cap: WithdrawalCaps {
                config_capacity: 0,
                current_total: 0,
                last_interval_start_timestamp: 0,
                config_interval_length_seconds: 0,
            },
            debt_withdrawal_cap: WithdrawalCaps {
                config_capacity: 0,
                current_total: 0,
                last_interval_start_timestamp: 0,
                config_interval_length_seconds: 0,
            },
            deleveraging_margin_call_period_secs: 259200, // 3 days
            borrow_factor_pct: 100,
            elevation_groups: params.elevation_groups,
            deleveraging_threshold_slots_per_bps: 7200, // 0.01% per hour
            multiplier_tag_boost: [1; 8],
            reserved0: [0; 2],
            reserved1: [0; 4],
        }
    }

    pub fn encode_token_name(token_name: &str) -> [u8; 32] {
        let mut buffer = vec![0; 32];
        let encoded = UTF_8.encode(token_name).0;
        for (i, &byte) in encoded.iter().enumerate() {
            buffer[i] = byte;
        }
        buffer.try_into().unwrap()
    }
}

pub mod pda {
    use kamino_lend::ID as KAMINO_LENDING_ID;
    use spl_token::solana_program::pubkey::Pubkey;
    pub fn get_market_authority(lending_market: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[b"lma", lending_market.as_ref()], &KAMINO_LENDING_ID).0
    }

    pub fn get_reserve_liquidity_supply(lending_market: &Pubkey, mint: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[b"reserve_liq_supply", lending_market.as_ref(), mint.as_ref()],
            &KAMINO_LENDING_ID,
        )
        .0
    }
    pub fn get_reserve_collateral_mint(lending_market: &Pubkey, mint: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[b"reserve_coll_mint", lending_market.as_ref(), mint.as_ref()],
            &KAMINO_LENDING_ID,
        )
        .0
    }
    pub fn get_reserve_collateral_supply(lending_market: &Pubkey, mint: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[b"reserve_coll_supply", lending_market.as_ref(), mint.as_ref()],
            &KAMINO_LENDING_ID,
        )
        .0
    }
    pub fn get_reserve_fee_vault(lending_market: &Pubkey, mint: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[b"fee_receiver", lending_market.as_ref(), mint.as_ref()],
            &KAMINO_LENDING_ID,
        )
        .0
    }

    pub fn get_user_obligation(lending_market: &Pubkey, user: &Pubkey) -> Pubkey {
        let tag = 0u8;
        let id = 0u8;
        let seed1 = [0; 32];
        let seed2 = [0; 32];
        Pubkey::find_program_address(
            &[&[tag], &[id], user.as_ref(), lending_market.as_ref(), &seed1, &seed2],
            &KAMINO_LENDING_ID,
        )
        .0
    }

    pub fn get_user_metadata(user: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[b"user_meta", user.as_ref()], &KAMINO_LENDING_ID).0
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use anchor_client::{
        solana_sdk::{
            commitment_config::CommitmentConfig,
            signature::{read_keypair_file, Keypair},
            signer::Signer,
        },
        Client, Cluster,
    };

    use super::*;
    use crate::token::{create_token_mint, mint_token};

    fn set_up(payer: &Keypair, client: &Client<&Keypair>) -> (Pubkey, Pubkey, Pubkey) {
        let program = client.program(KAMINO_LENDING_ID).unwrap();
        let lending_market = Keypair::new();
        create_lending_market(&program, &payer, &lending_market).unwrap();
        let market_account =
            program.account::<kamino_state::LendingMarket>(lending_market.pubkey()).unwrap();
        println!("{:?}", market_account.lending_market_owner);
        let mint = Keypair::new();
        create_token_mint(&program.rpc(), &payer, &mint, &payer.pubkey(), &spl_token::ID).unwrap();
        mint_token(&program.rpc(), &payer, &mint.pubkey(), 100000000, &spl_token::ID).unwrap();
        let reserve = Keypair::new();
        init_reserve(&program, &payer, &lending_market, &reserve, &mint.pubkey(), &spl_token::ID)
            .unwrap();
        let reserve_config =
            utils::make_reserve_config("SOL", types::ReserveConfigParams::default());
        update_reserve_config(
            &program,
            &payer,
            &lending_market.pubkey(),
            &reserve.pubkey(),
            reserve_config,
        )
        .unwrap();
        if !check_if_user_metadata_initialized(&program, &payer) {
            init_user_metadata(&program, &payer).unwrap();
        }
        init_obligation(&program, &payer, &lending_market.pubkey()).unwrap();
        (lending_market.pubkey(), mint.pubkey(), reserve.pubkey())
    }

    fn check_if_user_metadata_initialized(program: &Program<&Keypair>, payer: &Keypair)->bool {
        let user_metadata = pda::get_user_metadata(&payer.pubkey());
        match program.account::<kamino_state::UserMetadata>(user_metadata) {
            Ok(user_metadata_account) => {
                assert_eq!(user_metadata_account.owner, payer.pubkey());
                true
            },
            Err(_) => false,
        }
    }

    #[test]
    fn test_deposit() {
        let path = "/Users/daiwanwei/.config/solana/id.json";
        let payer = read_keypair_file(&path).expect("invalid payer keypair file");
        let client: Client<&Keypair> =
            Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
        let (lending_market, mint, reserve) = set_up(&payer, &client);
        let program = client.program(KAMINO_LENDING_ID).unwrap();
        deposit_reserve_liquidity_and_obligation_collateral(
            &program,
            &payer,
            &lending_market,
            &reserve,
            &mint,
            100,
            &spl_token::ID,
        )
        .unwrap();
    }
}
