//! ## Ephemeral Tick Map Data Provider
//! A data provider that fetches ticks using an [ephemeral contract](https://github.com/Aperture-Finance/Aperture-Lens/blob/904101e4daed59e02fd4b758b98b0749e70b583b/contracts/EphemeralGetPopulatedTicksInRange.sol) in a single `eth_call`.

use crate::prelude::*;
use alloy::{eips::BlockId, network::Network, providers::Provider};
use alloy_primitives::{aliases::I24, Address};
use derive_more::Deref;

/// A data provider that fetches ticks using an ephemeral contract in a single `eth_call`.
#[derive(Clone, Debug, Deref)]
pub struct EphemeralTickMapDataProvider<I = I24> {
    pub pool: Address,          // 池子地址
    pub tick_lower: I,          // tick范围下限
    pub tick_upper: I,          // tick范围上限
    pub tick_spacing: I,        // tick间距
    pub block_id: Option<BlockId>, // 区块ID
    #[deref]
    pub tick_map: TickMap<I>,   // 存储tick数据的HashMap结构
}

impl<I: TickIndex> EphemeralTickMapDataProvider<I> {
    #[inline]
    pub async fn new<N, P>(
        pool: Address,
        provider: P,
        tick_lower: Option<I>,
        tick_upper: Option<I>, 
        block_id: Option<BlockId>,
    ) -> Result<Self, Error>
    where
        N: Network,
        P: Provider<N>,
    {
        // 首先通过EphemeralTickDataProvider获取tick数据
        let provider = 
            EphemeralTickDataProvider::new(pool, provider, tick_lower, tick_upper, block_id)
                .await?;
        
        // 构造EphemeralTickMapDataProvider实例
        // 将tick数据转换为TickMap形式存储
        Ok(Self {
            pool,
            tick_lower: provider.tick_lower,
            tick_upper: provider.tick_upper,
            tick_spacing: provider.tick_spacing,
            block_id,
            tick_map: TickMap::new(provider.ticks, provider.tick_spacing), // 创建tick的HashMap索引
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use alloy_primitives::address;

    const TICK_SPACING: i32 = 10;

    #[tokio::test]
    async fn test_ephemeral_tick_map_data_provider() -> Result<(), Error> {
        let provider = EphemeralTickMapDataProvider::new(
            address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"),
            PROVIDER.clone(),
            None,
            None,
            *BLOCK_ID,
        )
        .await?;
        // [-887270, -92110, 100, 110, 22990, ...]
        let tick = provider.get_tick(-92110)?;
        assert_eq!(tick.liquidity_gross, 398290794261);
        assert_eq!(tick.liquidity_net, 398290794261);
        let (tick, initialized) = provider.next_initialized_tick_within_one_word(
            MIN_TICK_I32 + TICK_SPACING,
            true,
            TICK_SPACING,
        )?;
        assert_eq!(tick, -887270);
        assert!(initialized);
        let (tick, initialized) =
            provider.next_initialized_tick_within_one_word(-92120, true, TICK_SPACING)?;
        assert_eq!(tick, -92160);
        assert!(!initialized);
        let (tick, initialized) =
            provider.next_initialized_tick_within_one_word(0, false, TICK_SPACING)?;
        assert_eq!(tick, 100);
        assert!(initialized);
        let (tick, initialized) =
            provider.next_initialized_tick_within_one_word(110, false, TICK_SPACING)?;
        assert_eq!(tick, 2550);
        assert!(!initialized);
        Ok(())
    }
}

