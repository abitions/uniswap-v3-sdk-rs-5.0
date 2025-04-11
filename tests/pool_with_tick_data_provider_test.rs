
use uniswap_v3_sdk::prelude::*;
use uniswap_sdk_core::prelude::*;
use alloy_primitives::{address, U160, U256};
#[tokio::test]
async fn test_new_with_tick_data_provider() -> Result<(), uniswap_v3_sdk::error::Error> {
    // 创建两个代币
    let token0 = Token::new(
        1, // chain_id
        address!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"), // WBTC
        8,  // decimals
        None,
        None,
        0,      // buy_fee_bps
        0       // sell_fee_bps
    );

    let token1 = Token::new(
        8453, // chain_id
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        18, // decimals
        None,
        None,
        0,      // buy_fee_bps
        0       // sell_fee_bps
    );

    // 设置基本参数
    let fee_amount = FeeAmount::LOW; // 0.05%
    let sqrt_price_x96 = U256::from(159746326648098440207237290464052u128);
    let liquidity = 1_000_000_u128; // 示例流动性

    // 创建tick数据
    let tick_spacing = fee_amount.tick_spacing().as_i32();
    let ticks = vec![
        // 创建两个tick作为示例
        Tick::new(
            nearest_usable_tick(MIN_TICK, fee_amount.tick_spacing()).as_i32(),
            liquidity,
            liquidity as i128,
        ),
        Tick::new(
            nearest_usable_tick(MAX_TICK, fee_amount.tick_spacing()).as_i32(),
            liquidity,
            -(liquidity as i128),
        ),
    ];

    // 创建TickListDataProvider
    let tick_data_provider = TickListDataProvider::new(ticks, tick_spacing);
    tick_data_provider.binary_search_by_tick(1);
    // 创建Pool实例
    let pool = Pool::new_with_tick_data_provider(
        token0,
        token1,
        fee_amount,
        U160::from(sqrt_price_x96),
        liquidity,
        tick_data_provider,
    )?;

    // 验证Pool的基本属性
    assert_eq!(pool.liquidity, liquidity);
    pool.get_output_amount(input_amount, sqrt_price_limit_x96);
    
    // 测试获取tick数据
    let min_tick = nearest_usable_tick(MIN_TICK, fee_amount.tick_spacing()).as_i32();
    let tick_data = pool.tick_data_provider.get_tick(min_tick)?;
    assert_eq!(tick_data.liquidity_gross, liquidity);
    assert_eq!(tick_data.liquidity_net, liquidity as i128);

    // 测试token排序是否正确
    assert!(pool.token0.address() < pool.token1.address(), "Token order should be correct");

    Ok(())
}

