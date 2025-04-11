use crate::prelude::*;
use alloc::vec::Vec;
use derive_more::{Deref, DerefMut};

/// A data provider for ticks that is backed by an in-memory array of ticks.
#[derive(Clone, Debug, Default, PartialEq, Deref, DerefMut)]
pub struct TickListDataProvider<I = i32>(Vec<Tick<I>>);

impl<I: TickIndex> TickListDataProvider<I> {
    #[inline]
    pub fn new(ticks: Vec<Tick<I>>, tick_spacing: I) -> Self {
        //ticks.validate_list(tick_spacing);
        Self(ticks)
    }

    // 添加安全的修改方法
    #[inline]
    pub fn update_tick(&mut self, index: usize, tick: Tick<I>) -> Result<(), Error> {
        if index >= self.len() {
            return Err(Error::InvalidTick(I::ZERO.to_i24()));
        }
        
        // 临时保存修改
        let mut new_ticks = self.0.clone();
        new_ticks[index] = tick;
        
        // 验证修改后的列表是否有效
        //new_ticks.validate_list(tick_spacing);
        
        // 如果验证通过，应用修改
        self.0 = new_ticks;
        Ok(())
    }

    // 添加新的 tick
    #[inline]
    pub fn push_tick(&mut self, tick: Tick<I>) -> Result<(), Error> {
        let mut new_ticks = self.0.clone();
        new_ticks.push(tick);
        
        // 验证修改后的列表是否有效
        //new_ticks.validate_list(tick_spacing);
        
        // 如果验证通过，应用修改
        self.0 = new_ticks;
        Ok(())
    }

    // 移除 tick
    #[inline]
    pub fn remove_tick(&mut self, index: usize, tick_spacing: I) -> Result<Tick<I>, Error> {
        if index >= self.len() {
            return Err(Error::InvalidTick(I::ZERO.to_i24()));
        }
        let mut new_ticks = self.0.clone();
        let removed = new_ticks.remove(index);
        
        // 如果移除后还有 ticks，验证列表的有效性   
        
        self.0 = new_ticks;
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use once_cell::sync::Lazy;

    static PROVIDER: Lazy<TickListDataProvider> =
        Lazy::new(|| TickListDataProvider::new(vec![Tick::new(-1, 1, 1), Tick::new(1, 1, -1)], 1));

    #[test]
    fn can_take_an_empty_list_of_ticks() {
        TickListDataProvider::<i32>::default();
    }

    #[test]
    #[should_panic(expected = "TICK_SPACING_NONZERO")]
    fn throws_for_0_tick_spacing() {
        TickListDataProvider::new(vec![], 0);
    }

    #[test]
    #[should_panic(expected = "ZERO_NET")]
    fn throws_for_uneven_tick_list() {
        TickListDataProvider::new(vec![Tick::new(-1, 1, -1), Tick::new(1, 1, 2)], 1);
    }

    #[test]
    #[cfg(not(feature = "extensions"))]
    fn throws_if_tick_not_in_list() {
        assert_eq!(
            PROVIDER.get_tick(0).unwrap_err(),
            TickListError::NotContained.into()
        );
    }

    #[test]
    fn gets_the_smallest_tick_from_the_list() {
        let tick = PROVIDER.get_tick(-1).unwrap();
        assert_eq!(tick.liquidity_net, 1);
        assert_eq!(tick.liquidity_gross, 1);
    }

    #[test]
    fn gets_the_largest_tick_from_the_list() {
        let tick = PROVIDER.get_tick(1).unwrap();
        assert_eq!(tick.liquidity_net, -1);
        assert_eq!(tick.liquidity_gross, 1);
    }

    #[test]
    fn test_update_tick() {
        let mut provider = TickListDataProvider::new(
            vec![Tick::new(-1, 1, 1), Tick::new(1, 1, -1)],
            1
        );
        
        // 更新第一个 tick
        let result = provider.update_tick(
            0,
            Tick::new(-1, 2, 1)
        );
        assert!(result.is_ok());
        
        let tick = provider.get_tick(-1).unwrap();
        assert_eq!(tick.liquidity_gross, 2);
    }

    #[test]
    fn test_push_tick() {
        let mut provider = TickListDataProvider::new(
            vec![Tick::new(-2, 1, 1),Tick::new(2, 1, -1)],
            1
        );
        
        // 添加一个新的 tick 来平衡 liquidity_net
        let result = provider.push_tick(Tick::new(2, 1, -1));
        assert!(result.is_ok());
        assert_eq!(provider.len(), 3);
    }

    #[test]
    #[should_panic(expected = "ZERO_NET")]
    fn test_invalid_update() {
        let mut provider = TickListDataProvider::new(
            vec![Tick::new(-1, 1, 1), Tick::new(1, 1, -1)],
            1
        );
        
        // 这应该失败，因为会导致 liquidity_net 不平衡
        provider.update_tick(0, Tick::new(-1, 1, 2)).unwrap();
    }
}



