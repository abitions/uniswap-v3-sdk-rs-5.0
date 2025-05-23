use uniswap_v3_sdk::prelude::*;
use uniswap_v3_sdk::extensions::EphemeralTickMapDataProvider;
use alloy_primitives::address;

#[tokio::test]
async fn test_retrieve_and_display_all_ticks_map() -> Result<(), Error> {
    // 设置环境变量（如果需要）
    dotenv::dotenv().ok();

    // 使用一个真实的Uniswap V3池子地址
    // USDC/WETH 0.3% 池子地址
    let pool_address = address!("0x3e66e55e97ce60096f74b7C475e8249f2D31a9fb");

    // 创建provider
    let rpc_url = match std::env::var("MAINNET_RPC_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("环境变量MAINNET_RPC_URL未设置，请设置后再运行测试");
            println!("例如: $env:MAINNET_RPC_URL = 'https://eth-mainnet.alchemyapi.io/v2/YOUR_API_KEY'");
            return Ok(());
        }   
    };

    let provider = alloy::providers::ProviderBuilder::new()
        .disable_recommended_fillers()
        .on_http(rpc_url.parse().unwrap());

    // 创建区块ID（可选）
    let block_id = Some(alloy::eips::BlockId::from(17000000));

    println!("正在从池子 {} 获取tick数据...", pool_address);

    // 使用EphemeralTickMapDataProvider获取所有ticks数据
    let provider = EphemeralTickMapDataProvider::new(
        pool_address,
        provider,
        None, // tick_lower: 不限制下限
        None, // tick_upper: 不限制上限
        None,
    ).await?;

    // 输出基本信息
    println!("池子地址: {}", provider.pool);
    println!("Tick范围: {} 到 {}", provider.tick_lower, provider.tick_upper);
    println!("Tick间距: {}", provider.tick_spacing);
    println!("获取到的Tick数量: {}", provider.tick_map.inner.len());

    // 输出所有ticks数据
    println!("\n所有Tick数据:");
    println!("{:<10} | {:<20} | {:<20}", "Tick", "Liquidity Gross", "Liquidity Net");
    println!("{:-<10} | {:-<20} | {:-<20}", "", "", "");

    // 获取所有已初始化的ticks
    let mut current_tick = provider.tick_lower;
    let mut initialized_ticks = Vec::new();

    while current_tick <= provider.tick_upper {
        // 尝试获取当前tick
        match provider.get_tick(current_tick) {
            Ok(tick) => {
                initialized_ticks.push((current_tick, tick));
            },
            Err(_) => {
                // 如果当前tick未初始化，尝试找到下一个初始化的tick
                match provider.next_initialized_tick_within_one_word(
                    current_tick,
                    false,
                    provider.tick_spacing
                ) {
                    Ok((next_tick, initialized)) => {
                        if initialized && next_tick > current_tick {
                            current_tick = next_tick;
                            continue;
                        }
                    },
                    Err(_) => {}
                }
            }
        }

        // 移动到下一个可能的tick
        current_tick = current_tick + provider.tick_spacing;
    }

    // 输出找到的已初始化ticks
    let initialized_ticks_count = initialized_ticks.len();
    for (index, tick) in initialized_ticks {
        println!("{:<10} | {:<20} | {:<20}",
            index,
            tick.liquidity_gross,
            tick.liquidity_net
        );
    }

    println!("\n找到 {} 个已初始化的ticks", initialized_ticks_count);

    // 测试一些特定的ticks
    // 这些tick值来自现有测试，应该在大多数池子中存在
    let test_ticks = [-92110, 100, 110, 22990];

    println!("\n测试特定的ticks:");
    for &test_tick in &test_ticks {
        match provider.get_tick(test_tick) {
            Ok(tick) => {
                println!("Tick {}: Liquidity Gross = {}, Liquidity Net = {}",
                    test_tick,
                    tick.liquidity_gross,
                    tick.liquidity_net
                );
            },
            Err(_) => {
                println!("Tick {} 未初始化", test_tick);
            }
        }
    }

    // 测试next_initialized_tick_within_one_word方法
    println!("\n测试next_initialized_tick_within_one_word方法:");
    let test_cases = [
        (provider.tick_lower, true),
        (0, false),
        (100, true),
        (110, false)
    ];

    for (tick, lte) in &test_cases {
        match provider.next_initialized_tick_within_one_word(*tick, *lte, provider.tick_spacing) {
            Ok((next_tick, initialized)) => {
                println!("从Tick {} 向{}: 下一个tick = {}, 已初始化 = {}",
                    tick,
                    if *lte { "下" } else { "上" },
                    next_tick,
                    initialized
                );
            },
            Err(e) => {
                println!("从Tick {} 向{}: 错误 = {}",
                    tick,
                    if *lte { "下" } else { "上" },
                    e
                );
            }
        }
    }

    Ok(())
}

