use uniswap_v3_sdk::prelude::*;
use uniswap_v3_sdk::extensions::EphemeralTickDataProvider;
use alloy_primitives::address;

#[tokio::test]
async fn test_retrieve_and_display_all_ticks() -> Result<(), Error> {
    // 设置环境变量（如果需要）
    dotenv::dotenv().ok();

    // 使用一个真实的Uniswap V3池子地址
    // USDC/WETH 0.3% 池子地址
    let pool_address = address!("0x28a9ec9928a689410e82e579fd9418fbbf6452f3");

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
    let block_id = Some(alloy::eips::BlockId::from(28845421));

    println!("正在从池子 {} 获取tick数据...", pool_address);

    // 使用EphemeralTickDataProvider获取所有ticks数据
    let provider = EphemeralTickDataProvider::new(
        pool_address,
        provider,
        None, // tick_lower: 不限制下限
        None, // tick_upper: 不限制上限
        block_id,
    ).await?;

    // 输出基本信息
    println!("池子地址: {}", provider.pool);
    println!("Tick范围: {} 到 {}", provider.tick_lower, provider.tick_upper);
    println!("Tick间距: {}", provider.tick_spacing);
    println!("获取到的Tick数量: {}", provider.ticks.len());

    // 输出所有ticks数据
    println!("\n所有Tick数据:");
    println!("{:<10} | {:<20} | {:<20}", "Tick", "Liquidity Gross", "Liquidity Net");
    println!("{:-<10} | {:-<20} | {:-<20}", "", "", "");

    for tick in &provider.ticks {
        println!("{:<10} | {:<20} | {:<20}",
            tick.index,
            tick.liquidity_gross,
            tick.liquidity_net
        );
    }

    // 验证tick数据的有效性
    provider.ticks.validate_list(provider.tick_spacing);
    println!("\nTick数据验证通过!");

    // 转换为TickListDataProvider并测试
    let list_provider: TickListDataProvider = provider.into();
    println!("成功转换为TickListDataProvider");

    // 获取一个特定的tick进行测试
    if !list_provider.is_empty() {
        let sample_tick = list_provider[0].index;
        let tick_data = list_provider.get_tick(116000)?;
        println!("\n示例Tick数据 ({}): ", sample_tick);
        println!("Liquidity Gross: {}", tick_data.liquidity_gross);
        println!("Liquidity Net: {}", tick_data.liquidity_net);
    }

    Ok(())
}
