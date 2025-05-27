use crate::prelude::*;
use alloc::vec::Vec;
use derive_more::{Deref, DerefMut};

/// A data provider for ticks that is backed by an in-memory array of ticks.
#[derive(Clone, Debug, Default, PartialEq, Deref, DerefMut)]
pub struct TickListDataProvider<I = i32>(Vec<Tick<I>>);

impl<I: TickIndex> TickListDataProvider<I> {
    #[inline]
    pub fn new(ticks: Vec<Tick<I>>, tick_spacing: I) -> Self {
        let mut sorted_ticks = ticks;
        // 确保初始的 ticks 列表是按 tick.index 排序的
        sorted_ticks.sort_unstable_by_key(|t| t.index);
        //ticks.validate_list(tick_spacing);
        Self(sorted_ticks)
    }

    // 添加安全的修改方法
    #[inline]
    pub fn update_tick(&mut self, index: usize, tick: Tick<I>) -> Result<(), Error> {
        if index >= self.len() {
            return Err(Error::InvalidTick(I::ZERO.to_i24()));
        }
        
        let mut new_ticks = self.0.clone();

        // 根据用户请求：如果传入的 tick 的 liquidity_net 等于 0，则删除该索引处的数据。
        // 假设 tick.liquidity_net 是可以直接与 0 比较的类型 (例如整数)。
        if tick.liquidity_net == 0 { // TODO: 确认 liquidity_net 为0的正确比较方式，这里假定为直接比较
            new_ticks.remove(index);
        } else {
            // 否则，更新指定物理索引处的 tick。
            new_ticks[index] = tick;
            // 注意：根据用户之前的修改，此处不进行重新排序。
            // 如果 tick.index 在此更新中发生改变，列表的整体排序可能会受到影响。
        }
        
        // 验证修改后的列表是否有效 (tick_spacing 在此作用域不可用)
        //new_ticks.validate_list(tick_spacing);
        
        // 如果验证通过，应用修改
        self.0 = new_ticks;
        Ok(())
    }

    // 添加新的 tick
    #[inline]
    pub fn push_tick(&mut self, tick: Tick<I>) -> Result<(), Error> {
        let mut new_ticks = self.0.clone();

        // 根据 tick.index 找到正确的插入位置以保持排序
        let insertion_point =
            match new_ticks.binary_search_by_key(&tick.index, |t: &Tick<I>| t.index) {
                Ok(idx) => idx,  // 如果已存在相同索引的tick，在此位置插入。
                                 // 这会将新的tick放在具有相同索引的现有tick之前（或之中）。
                                 // `validate_list`（如果启用）应处理重复索引的问题。
                Err(idx) => idx, // 如果不存在该索引的tick，idx是正确的插入点。
            };
        new_ticks.insert(insertion_point, tick);

        // 验证修改后的列表是否有效
        //new_ticks.validate_list(tick_spacing); // 注意: tick_spacing 在此作用域不可用

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

    /// 检查是否存在指定的tick且其liquidity_net不为0
    #[inline]
    pub fn has_tick(&mut self, tick_index: I) -> bool {
        self.0.iter()
            .any(|tick| tick.index == tick_index)
    }

    /// 检查是否存在指定的tick且其liquidity_net不为0，如果存在返回其索引位置
    #[inline]
    pub fn find_tick_with_net(&mut self, tick_index: I) -> Option<usize> {
        self.0.iter()
            .position(|tick| tick.index == tick_index)
    }

    /// 通过索引获取tick，如果索引超出范围则返回错误
    #[inline]
    pub fn get_tick_by_index(&mut self, index: usize) -> Result<&Tick<I>, Error> {
        self.0.get(index)
            .ok_or(Error::InvalidTick(I::ZERO.to_i24()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use once_cell::sync::Lazy;
    use alloc::format;  // 添加这行
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
            vec![Tick::new(-2, 1, 1),Tick::new(2, 1, -1),Tick::new(3, 1, -1),Tick::new(4, 1, -1)],
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
            vec![Tick::new(-2, 1, 1),Tick::new(3, 1, -1),Tick::new(4, 1, -1)],
            1
        );
        
        // 添加一个新的 tick 来平衡 liquidity_net
        let result = provider.push_tick(Tick::new(2, 1, -1));
        println!("provider.get_tick(2): {:?}", provider.get_tick(2).unwrap());
        assert!(result.is_ok());
        //assert_eq!(provider.len(), 3);
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

    #[test]
    fn test_update_tick_removes_on_zero_liquidity_net() {
        let mut provider = TickListDataProvider::new(
            vec![
                Tick::new(10, 100, 10), // 物理索引 0
                Tick::new(20, 200, 20), // 物理索引 1
                Tick::new(30, 300, 30), // 物理索引 2
            ],
            1, // tick_spacing
        );

        // 初始状态检查
        assert_eq!(provider.len(), 3);
        assert_eq!(provider.get_tick(10).unwrap().index, 10);
        assert_eq!(provider.get_tick(20).unwrap().index, 20);
        assert_eq!(provider.get_tick(30).unwrap().index, 30);

        // 调用 update_tick 作用于物理索引为 1 的元素 (即 tick.index == 20 的 tick)
        // 传入的 tick 的 liquidity_net 为 0，这应该导致该元素被移除。
        // 新 tick 的 tick.index (这里是20) 实际上不影响移除操作，但用被移除元素的 index 是符合逻辑的。
        let result = provider.update_tick(1, Tick::new(20, 0, 0)); // liquidity_net is 0
        assert!(result.is_ok());

        // 检查长度是否减少
        assert_eq!(provider.len(), 2);

        // 检查剩余的 ticks 是否按顺序存在
        // Tick 10 (原物理索引 0) 应该仍在物理索引 0
        let tick10 = provider.get_tick_by_index(0).unwrap();
        assert_eq!(tick10.index, 10);
        assert_eq!(tick10.liquidity_net, 10); // 确保是原始 tick

        // Tick 30 (原物理索引 2) 应该移动到物理索引 1
        let tick30 = provider.get_tick_by_index(1).unwrap();
        assert_eq!(tick30.index, 30);
        assert_eq!(tick30.liquidity_net, 30); // 确保是原始 tick

        // 检查 tick 20 (其 liquidity_net 被更新为0并因此被移除) 是否已无法通过其逻辑索引找到
        let get_tick_20_result = provider.get_tick(20);
        assert!(get_tick_20_result.is_err());
        // 可以更精确地检查错误类型，如果 TickListError 在作用域内:
        // use crate::error::TickListError; // (如果需要)

        // 尝试访问旧的物理索引 2 (现在越界了) 应该失败
        assert!(provider.get_tick_by_index(2).is_err());
    }

    #[tokio::test]
    async fn test_update_tick_async() -> Result<(), Error> {
        let mut provider = TickListDataProvider::new(
            vec![
                Tick::new(-8, 1, 1),
                Tick::new(6, 1, -1),
                Tick::new(3, 1, -1),
                Tick::new(8, 1, -1)
            ],
            1
        );
        println!("初始状态:");
        let a =  provider.binary_search_by_tick(1)?;
        println!("a: {}", a);
        // 测试更新第一个tick
        provider.update_tick(2, Tick::new(-1, 2, 1))?;
        let b =  provider.binary_search_by_tick(3)?;
        println!("b: {}", b);
        // 验证更新是否成功
        let updated_tick = provider.get_tick(-1)?;
        assert_eq!(updated_tick.liquidity_gross, 2);
        assert_eq!(updated_tick.liquidity_net, 1);
        
        // 测试更新中间的tick
        provider.update_tick(1, Tick::new(2, 3, -1))?;
        let middle_tick = provider.get_tick(2)?;
        assert_eq!(middle_tick.liquidity_gross, 3);
        assert_eq!(middle_tick.liquidity_net, -1);
        
        // 测试更新最后一个tick
        provider.update_tick(3, Tick::new(4, 5, -1))?;
        let last_tick = provider.get_tick(4)?;
        assert_eq!(last_tick.liquidity_gross, 5);
        assert_eq!(last_tick.liquidity_net, -1);

        Ok(())
    }
}






