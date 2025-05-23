use uniswap_v3_sdk::prelude::*;
use uniswap_v3_sdk::extensions::EphemeralTickDataProvider;
use alloy_primitives::address;
use std::fs::File;
use std::io::Write;
use serde::Serialize;

#[derive(Serialize)]
struct TickData {
    tick: i32,
    liquidity_gross: u128,
    liquidity_net: i128,
}

#[tokio::test]
async fn test_retrieve_and_display_all_ticks() -> Result<(), Error> {
    // 设置环境变量（如果需要）
    dotenv::dotenv().ok();

    // 使用一个真实的Uniswap V3池子地址
    // USDC/WETH 0.3% 池子地址
    let pool_address = address!("0x482fe995c4a52bc79271ab29a53591363ee30a89");

    // 创建provider
    let rpc_url = "https://base-mainnet.g.alchemy.com/v2/PaDrdtZbgVIWgYyp8s2HE9mJstDh9E-q".to_string();

    let provider = alloy::providers::ProviderBuilder::new()
        .disable_recommended_fillers()
        .on_http(rpc_url.parse().unwrap());

    // 创建区块ID（可选）
    let block_id = Some(alloy::eips::BlockId::from(29992826));

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

    // 将ticks数据转换为JSON格式
    let ticks_json: Vec<TickData> = provider.ticks.iter().map(|tick| TickData {
        tick: tick.index,
        liquidity_gross: tick.liquidity_gross,
        liquidity_net: tick.liquidity_net,
    }).collect();

    // After creating ticks_json, handle file operations separately
    let file_path = "E:\\web3\\rust-code-for-base\\tmp\\tick3.json";
    
    // Separate block for file operations with its own error handling
    {
        let json_data = serde_json::to_string_pretty(&ticks_json)
            .expect("Failed to serialize JSON");
            
        let mut file = File::create(file_path)
            .expect("Failed to create file");
            
        file.write_all(json_data.as_bytes())
            .expect("Failed to write to file");
    }
    
    println!("\nTick数据已写入到文件: {}", file_path);
    
   


    Ok(())
}
