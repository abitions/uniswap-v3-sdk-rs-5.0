//! ## Tick Map
//! [`TickMap`] provides a way to access tick data directly from a hashmap, supposedly more
//! efficient than [`TickList`].

use crate::prelude::*;
use alloc::vec::Vec;
use alloy_primitives::{aliases::I24, map::rustc_hash::FxHashMap, uint, U256};

#[derive(Clone, Debug)]
pub struct TickMap<I = I24> {
    pub bitmap: TickBitMap<I>,  // 用于快速查找已初始化tick的位图
    pub inner: FxHashMap<I, Tick<I>>,  // 存储tick数据的HashMap
    pub tick_spacing: I,         // tick间距
}

impl<I: TickIndex> TickMap<I> {
    #[inline]
    #[must_use]
    pub fn new(ticks: Vec<Tick<I>>, tick_spacing: I) -> Self {
        ticks.validate_list(tick_spacing);
        let mut bitmap = TickBitMap::default();
        
        // 构建bitmap用于快速查找
        for tick in &ticks {
            let compressed = tick.index.compress(tick_spacing);
            let (word_pos, bit_pos) = compressed.position();
            let word = bitmap.get(&word_pos).unwrap_or(&U256::ZERO);
            // 在bitmap中标记tick的位置
            bitmap.insert(word_pos, word | (uint!(1_U256) << bit_pos));
        }

        Self {
            bitmap,
            // 构建HashMap用于存储tick数据
            inner: FxHashMap::from_iter(ticks.into_iter().map(|tick| (tick.index, tick))),
            tick_spacing,
        }
    }
}

impl<I: TickIndex> TickDataProvider for TickMap<I> {
    type Index = I;

    #[inline]
    fn get_tick(&self, tick: Self::Index) -> Result<&Tick<Self::Index>, Error> {
        self.inner
            .get(&tick)
            .ok_or(Error::InvalidTick(tick.to_i24()))
    }

    #[inline]
    fn next_initialized_tick_within_one_word(
        &self,
        tick: Self::Index,
        lte: bool,
        tick_spacing: Self::Index,
    ) -> Result<(Self::Index, bool), Error> {
        self.bitmap
            .next_initialized_tick_within_one_word(tick, lte, tick_spacing)
    }
}

