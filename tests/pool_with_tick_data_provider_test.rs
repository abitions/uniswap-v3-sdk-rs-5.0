

use uniswap_v3_sdk::prelude::*;
use uniswap_sdk_core::prelude::*;
use alloy_primitives::{address, U160, U256};
use uniswap_v3_sdk::extensions::EphemeralTickDataProvider;

#[tokio::test]
async fn test_pool_with_ephemeral_tick_map_data_provider() -> Result<(), uniswap_v3_sdk::error::Error> {
    // 设置环境变量（如果需要）
    dotenv::dotenv().ok();

    // 定义池子相关参数
    let factory_address = address!("0x38015D05f4fEC8AFe15D7cc0386a126574e8077B"); // Uniswap V3 Factory 地址
    let wbtc = Token::new(
        8453, // chain_id
        address!("0x4200000000000000000000000000000000000006"), // WBTC 地址
        1,  // decimals
        None,
        None,
        0,  // buy_fee_bps
        0   // sell_fee_bps
    );

    let weth = Token::new(
        8453, // chain_id
        address!("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"), // WETH 地址
        18, // decimals
        None,
        None,
        0,  // buy_fee_bps
        0   // sell_fee_bps
    );

    let fee_amount = FeeAmount::CUSTOM { fee: 1000, tick_spacing: 25 }; // 0.05% 费用等级
    let block_id = Some(alloy::eips::BlockId::from(29489623)); // 指定区块 ID

    // 创建 RPC Provider
    let rpc_url = "https://base-mainnet.g.alchemy.com/v2/f_Pz9YF9GkzFd_ZRb35cRNbsagBl7MGW".to_string();
    let provider = alloy::providers::ProviderBuilder::new()
        .disable_recommended_fillers()
        .on_http(rpc_url.parse().unwrap());

    // 使用 EphemeralTickMapDataProvider 创建池子
    let pool = Pool::<EphemeralTickMapDataProvider>::from_pool_key_with_tick_data_provider(
        8453, // chain_id
        factory_address,
        wbtc.address(),
        weth.address(),
        fee_amount,
        provider.clone(),
        block_id,
    )
    .await
    .unwrap();

    // 输入金额（以 WBTC 为单位）
    let amount_in = CurrencyAmount::from_raw_amount(wbtc.clone(), 1000000000000000000 as i128).unwrap(); // 1 WBTC (8 decimals)

    // 获取输出金额（以 WETH 为单位）
    let amount_out = pool.get_output_amount(&amount_in, None).unwrap();

    // 打印结果
    //println!("输入金额: {} {}", amount_in, wbtc.symbol().unwrap_or("WBTC"));
    println!("输出金额: {:?} ", amount_out);

    // 验证输出金额是否合理（根据实际情况调整断言）
    //assert!(amount_out.raw() > 0, "输出金额应大于 0");

    Ok(())
}

