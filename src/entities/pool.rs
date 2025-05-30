use crate::prelude::{Error, *};
use alloy_primitives::{ChainId, B256, I256, U160};
use uniswap_sdk_core::prelude::*;

/// Represents a V3 pool
#[derive(Clone, Debug)]
pub struct Pool<TP = NoTickDataProvider>
where
    TP: TickDataProvider,
{
    pub token0: Token,
    pub token1: Token,
    pub fee: FeeAmount,
    pub sqrt_ratio_x96: U160,
    pub liquidity: u128,
    pub tick_current: TP::Index,
    pub tick_data_provider: TP,
}

impl<TP> PartialEq for Pool<TP>
where
    TP: TickDataProvider<Index: PartialEq>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.token0 == other.token0
            && self.token1 == other.token1
            && self.fee == other.fee
            && self.sqrt_ratio_x96 == other.sqrt_ratio_x96
            && self.liquidity == other.liquidity
            && self.tick_current == other.tick_current
    }
}

impl Pool {
    /// Construct a pool
    ///
    /// ## Arguments
    ///
    /// * `token_a`: One of the tokens in the pool
    /// * `token_b`: The other token in the pool
    /// * `fee`: The fee in hundredths of a bips of the input amount of every swap that is collected
    ///   by the pool
    /// * `sqrt_ratio_x96`: The sqrt of the current ratio of amounts of token1 to token0
    /// * `liquidity`: The current value of in range liquidity
    /// * `tick_current`: The current tick of the pool
    #[inline]
    pub fn new(
        token_a: Token,
        token_b: Token,
        fee: FeeAmount,
        sqrt_ratio_x96: U160,
        liquidity: u128,
    ) -> Result<Self, Error> {
        Self::new_with_tick_data_provider(
            token_a,
            token_b,
            fee,
            sqrt_ratio_x96,
            liquidity,
            NoTickDataProvider,
        )
    }

    /// Compute the pool address
    ///
    /// ## Arguments
    ///
    /// * `token_a`: The first token of the pair, irrespective of sort order
    /// * `token_b`: The second token of the pair, irrespective of sort order
    /// * `fee`: The fee tier of the pool
    /// * `init_code_hash_manual_override`: Override the init code hash used to compute the pool
    ///   address if necessary
    /// * `factory_address_override`: Override the factory address used to compute the pool address
    ///   if necessary
    ///
    /// ## Returns
    ///
    /// The computed pool address
    ///
    /// ## Examples
    ///
    /// ```
    /// use alloy_primitives::{address, Address};
    /// use uniswap_sdk_core::{prelude::Token, token};
    /// use uniswap_v3_sdk::prelude::*;
    ///
    /// let usdc = token!(1, "A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", 6);
    /// let dai = token!(1, "6B175474E89094C44Da98b954EedeAC495271d0F", 18);
    /// let result = Pool::get_address(&usdc, &dai, FeeAmount::LOW, None, None);
    /// assert_eq!(result, address!("6c6Bc977E13Df9b0de53b251522280BB72383700"));
    /// ```
    #[inline]
    #[must_use]
    pub fn get_address(
        token_a: &Token,
        token_b: &Token,
        fee: FeeAmount,
        init_code_hash_manual_override: Option<B256>,
        factory_address_override: Option<Address>,
    ) -> Address {
        compute_pool_address(
            factory_address_override.unwrap_or(FACTORY_ADDRESS),
            token_a.address(),
            token_b.address(),
            fee,
            init_code_hash_manual_override,
            Some(token_a.chain_id()),
        )
    }
}

impl<TP: TickDataProvider> Pool<TP> {
    /// Returns the pool address
    #[inline]
    pub fn address(
        &self,
        init_code_hash_manual_override: Option<B256>,
        factory_address_override: Option<Address>,
    ) -> Address {
        Pool::get_address(
            &self.token0,
            &self.token1,
            self.fee,
            init_code_hash_manual_override,
            factory_address_override,
        )
    }

    #[inline]
    pub fn chain_id(&self) -> ChainId {
        self.token0.chain_id()
    }

    #[inline]
    pub fn tick_spacing(&self) -> TP::Index {
        TP::Index::from_i24(self.fee.tick_spacing())
    }

    /// Returns true if the token is either token0 or token1
    ///
    /// ## Arguments
    ///
    /// * `token`: The token to check
    ///
    /// returns: bool
    #[inline]
    pub fn involves_token(&self, token: &impl BaseCurrency) -> bool {
        self.token0.equals(token) || self.token1.equals(token)
    }

    /// Returns the current mid price of the pool in terms of token0, i.e. the ratio of token1 over
    /// token0
    #[inline]
    pub fn token0_price(&self) -> Price<Token, Token> {
        let sqrt_ratio_x96 = self.sqrt_ratio_x96.to_big_int();
        Price::new(
            self.token0.clone(),
            self.token1.clone(),
            Q192_BIG_INT,
            sqrt_ratio_x96 * sqrt_ratio_x96,
        )
    }

    /// Returns the current mid price of the pool in terms of token1, i.e. the ratio of token0 over
    /// token1
    #[inline]
    pub fn token1_price(&self) -> Price<Token, Token> {
        let sqrt_ratio_x96 = self.sqrt_ratio_x96.to_big_int();
        Price::new(
            self.token1.clone(),
            self.token0.clone(),
            sqrt_ratio_x96 * sqrt_ratio_x96,
            Q192_BIG_INT,
        )
    }

    /// Return the price of the given token in terms of the other token in the pool.
    ///
    /// ## Arguments
    ///
    /// * `token`: The token to return price of
    ///
    /// returns: Price<Token, Token>
    #[inline]
    pub fn price_of(&self, token: &Token) -> Result<Price<Token, Token>, Error> {
        if self.token0.equals(token) {
            Ok(self.token0_price())
        } else if self.token1.equals(token) {
            Ok(self.token1_price())
        } else {
            Err(Error::InvalidToken)
        }
    }

    /// Construct a pool with a tick data provider
    ///
    /// ## Arguments
    ///
    /// * `token_a`: One of the tokens in the pool
    /// * `token_b`: The other token in the pool
    /// * `fee`: The fee in hundredths of a bips of the input amount of every swap that is collected
    ///   by the pool
    /// * `sqrt_ratio_x96`: The sqrt of the current ratio of amounts of token1 to token0
    /// * `liquidity`: The current value of in range liquidity
    /// * `tick_current`: The current tick of the pool
    /// * `tick_data_provider`: A tick data provider that can return tick data
    #[inline]
    pub fn new_with_tick_data_provider(
        token_a: Token,
        token_b: Token,
        fee: FeeAmount,
        sqrt_ratio_x96: U160,
        liquidity: u128,
        tick_data_provider: TP,
    ) -> Result<Self, Error> {
        let (token0, token1) = if token_a.sorts_before(&token_b)? {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        Ok(Self {
            token0,
            token1,
            fee,
            sqrt_ratio_x96,
            liquidity,
            tick_current: TP::Index::from_i24(sqrt_ratio_x96.get_tick_at_sqrt_ratio()?),
            tick_data_provider,
        })
    }

    fn _swap(
        &self,
        zero_for_one: bool,
        amount_specified: I256,
        sqrt_price_limit_x96: Option<U160>,
    ) -> Result<SwapState<TP::Index>, Error> {
        v3_swap(
            self.fee.into(),
            self.sqrt_ratio_x96,
            self.tick_current,
            self.liquidity,
            self.tick_spacing(),
            &self.tick_data_provider,
            zero_for_one,
            amount_specified,
            sqrt_price_limit_x96,
        )
    }

    /// Given an input amount of a token, return the computed output amount
    ///
    /// ## Arguments
    ///
    /// * `input_amount`: The input amount for which to quote the output amount
    /// * `sqrt_price_limit_x96`: The Q64.96 sqrt price limit
    ///
    /// returns: The output amount
    #[inline]
    pub fn get_output_amount(
        &self,
        input_amount: &CurrencyAmount<impl BaseCurrency>,
        sqrt_price_limit_x96: Option<U160>,
    ) -> Result<CurrencyAmount<Token>, Error> {
        if !self.involves_token(&input_amount.currency) {
            return Err(Error::InvalidToken);
        }

        let zero_for_one = input_amount.currency.equals(&self.token0);

        let SwapState {
            amount_specified_remaining,
            amount_calculated: output_amount,
            ..
        } = self._swap(
            zero_for_one,
            I256::from_big_int(input_amount.quotient()),
            sqrt_price_limit_x96,
        )?;

        if !amount_specified_remaining.is_zero() && sqrt_price_limit_x96.is_none() {
            return Err(Error::InsufficientLiquidity);
        }

        let output_token = if zero_for_one {
            &self.token1
        } else {
            &self.token0
        };
        CurrencyAmount::from_raw_amount(output_token.clone(), -output_amount.to_big_int())
            .map_err(Error::Core)
    }

    /// Given an input amount of a token, return the computed output amount, updating the pool state
    ///
    /// ## Arguments
    ///
    /// * `input_amount`: The input amount for which to quote the output amount
    /// * `sqrt_price_limit_x96`: The Q64.96 sqrt price limit
    ///
    /// returns: The output amount
    #[inline]
    pub fn get_output_amount_mut(
        &mut self,
        input_amount: &CurrencyAmount<impl BaseCurrency>,
        sqrt_price_limit_x96: Option<U160>,
    ) -> Result<CurrencyAmount<Token>, Error> {
        if !self.involves_token(&input_amount.currency) {
            return Err(Error::InvalidToken);
        }

        let zero_for_one = input_amount.currency.equals(&self.token0);

        let SwapState {
            amount_specified_remaining,
            amount_calculated: output_amount,
            sqrt_price_x96,
            liquidity,
            ..
        } = self._swap(
            zero_for_one,
            I256::from_big_int(input_amount.quotient()),
            sqrt_price_limit_x96,
        )?;

        if !amount_specified_remaining.is_zero() && sqrt_price_limit_x96.is_none() {
            return Err(Error::InsufficientLiquidity);
        }

        let output_token = if zero_for_one {
            &self.token1
        } else {
            &self.token0
        };

        self.sqrt_ratio_x96 = sqrt_price_x96;
        self.tick_current = TP::Index::from_i24(sqrt_price_x96.get_tick_at_sqrt_ratio()?);
        self.liquidity = liquidity;
        CurrencyAmount::from_raw_amount(output_token.clone(), -output_amount.to_big_int())
            .map_err(Error::Core)
    }

    /// Given a desired output amount of a token, return the computed input amount
    ///
    /// ## Arguments
    ///
    /// * `output_amount`: the output amount for which to quote the input amount
    /// * `sqrt_price_limit_x96`: The Q64.96 sqrt price limit. If zero for one, the price cannot be
    ///   less than this value after the swap. If one for zero, the price cannot be greater than
    ///   this value after the swap
    ///
    /// returns: The input amount
    #[inline]
    pub fn get_input_amount(
        &self,
        output_amount: &CurrencyAmount<impl BaseCurrency>,
        sqrt_price_limit_x96: Option<U160>,
    ) -> Result<CurrencyAmount<Token>, Error> {
        if !self.involves_token(&output_amount.currency) {
            return Err(Error::InvalidToken);
        }

        let zero_for_one = output_amount.currency.equals(&self.token1);

        let SwapState {
            amount_specified_remaining,
            amount_calculated: input_amount,
            ..
        } = self._swap(
            zero_for_one,
            I256::from_big_int(-output_amount.quotient()),
            sqrt_price_limit_x96,
        )?;

        if !amount_specified_remaining.is_zero() && sqrt_price_limit_x96.is_none() {
            return Err(Error::InsufficientLiquidity);
        }

        let input_token = if zero_for_one {
            &self.token0
        } else {
            &self.token1
        };
        CurrencyAmount::from_raw_amount(input_token.clone(), input_amount.to_big_int())
            .map_err(Error::Core)
    }

    /// Given a desired output amount of a token, return the computed input amount, updating the
    /// pool state
    ///
    /// ## Arguments
    ///
    /// * `output_amount`: the output amount for which to quote the input amount
    /// * `sqrt_price_limit_x96`: The Q64.96 sqrt price limit. If zero for one, the price cannot be
    ///   less than this value after the swap. If one for zero, the price cannot be greater than
    ///   this value after the swap
    ///
    /// returns: The input amount
    #[inline]
    pub fn get_input_amount_mut(
        &mut self,
        output_amount: &CurrencyAmount<impl BaseCurrency>,
        sqrt_price_limit_x96: Option<U160>,
    ) -> Result<CurrencyAmount<Token>, Error> {
        if !self.involves_token(&output_amount.currency) {
            return Err(Error::InvalidToken);
        }

        let zero_for_one = output_amount.currency.equals(&self.token1);

        let SwapState {
            amount_specified_remaining,
            amount_calculated: input_amount,
            sqrt_price_x96,
            liquidity,
            ..
        } = self._swap(
            zero_for_one,
            I256::from_big_int(-output_amount.quotient()),
            sqrt_price_limit_x96,
        )?;

        if !amount_specified_remaining.is_zero() && sqrt_price_limit_x96.is_none() {
            return Err(Error::InsufficientLiquidity);
        }

        let input_token = if zero_for_one {
            &self.token0
        } else {
            &self.token1
        };

        self.sqrt_ratio_x96 = sqrt_price_x96;
        self.tick_current = TP::Index::from_i24(sqrt_price_x96.get_tick_at_sqrt_ratio()?);
        self.liquidity = liquidity;
        CurrencyAmount::from_raw_amount(input_token.clone(), input_amount.to_big_int())
            .map_err(Error::Core)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use alloy_primitives::address;

    const ONE_ETHER: U160 = U160::from_limbs([10_u64.pow(18), 0, 0]);

    mod constructor {
        use super::*;

        #[test]
        #[should_panic(expected = "CHAIN_IDS")]
        fn cannot_be_used_for_tokens_on_different_chains() {
            let weth9 = WETH9::default().get(3).unwrap().clone();
            Pool::new(USDC.clone(), weth9, FeeAmount::MEDIUM, ONE_ETHER, 0).expect("CHAIN_IDS");
        }

        #[test]
        #[should_panic(expected = "ADDRESSES")]
        fn cannot_be_given_two_of_the_same_token() {
            Pool::new(USDC.clone(), USDC.clone(), FeeAmount::MEDIUM, ONE_ETHER, 0)
                .expect("ADDRESSES");
        }

        #[test]
        fn works_with_valid_arguments_for_empty_pool_medium_fee() {
            let weth9 = WETH9::default().get(1).unwrap().clone();
            Pool::new(USDC.clone(), weth9, FeeAmount::MEDIUM, ONE_ETHER, 0).unwrap();
        }

        #[test]
        fn works_with_valid_arguments_for_empty_pool_low_fee() {
            let weth9 = WETH9::default().get(1).unwrap().clone();
            Pool::new(USDC.clone(), weth9, FeeAmount::LOW, ONE_ETHER, 0).unwrap();
        }

        #[test]
        fn works_with_valid_arguments_for_empty_pool_lowest_fee() {
            let weth9 = WETH9::default().get(1).unwrap().clone();
            Pool::new(USDC.clone(), weth9, FeeAmount::LOWEST, ONE_ETHER, 0).unwrap();
        }

        #[test]
        fn works_with_valid_arguments_for_empty_pool_high_fee() {
            let weth9 = WETH9::default().get(1).unwrap().clone();
            Pool::new(USDC.clone(), weth9, FeeAmount::HIGH, ONE_ETHER, 0).unwrap();
        }
    }

    #[test]
    fn get_address_matches_an_example() {
        let result = Pool::get_address(&USDC, &DAI, FeeAmount::LOW, None, None);
        assert_eq!(result, address!("6c6Bc977E13Df9b0de53b251522280BB72383700"));
    }

    #[test]
    fn token0_always_is_the_token_that_sorts_before() {
        let pool = Pool::new(
            USDC.clone(),
            DAI.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        assert!(pool.token0.equals(&DAI.clone()));
        let pool = Pool::new(
            DAI.clone(),
            USDC.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        assert!(pool.token0.equals(&DAI.clone()));
    }

    #[test]
    fn token1_always_is_the_token_that_sorts_after() {
        let pool = Pool::new(
            USDC.clone(),
            DAI.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        assert!(pool.token1.equals(&USDC.clone()));
        let pool = Pool::new(
            DAI.clone(),
            USDC.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        assert!(pool.token1.equals(&USDC.clone()));
    }

    #[test]
    fn token0_price_returns_price_of_token0_in_terms_of_token1() {
        let pool = Pool::new(
            USDC.clone(),
            DAI.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(101e6 as u128, 100e18 as u128),
            0,
        )
        .unwrap();
        assert_eq!(pool.token0_price().to_significant(5, None).unwrap(), "1.01");
        let pool = Pool::new(
            DAI.clone(),
            USDC.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(101e6 as u128, 100e18 as u128),
            0,
        )
        .unwrap();
        assert_eq!(pool.token0_price().to_significant(5, None).unwrap(), "1.01");
    }

    #[test]
    fn token1_price_returns_price_of_token1_in_terms_of_token0() {
        let pool = Pool::new(
            USDC.clone(),
            DAI.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(101e6 as u128, 100e18 as u128),
            0,
        )
        .unwrap();
        assert_eq!(
            pool.token1_price().to_significant(5, None).unwrap(),
            "0.9901"
        );
        let pool = Pool::new(
            DAI.clone(),
            USDC.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(101e6 as u128, 100e18 as u128),
            0,
        )
        .unwrap();
        assert_eq!(
            pool.token1_price().to_significant(5, None).unwrap(),
            "0.9901"
        );
    }

    #[test]
    fn price_of_returns_price_of_token_in_terms_of_other_token() {
        let pool = Pool::new(
            USDC.clone(),
            DAI.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        assert_eq!(pool.price_of(&DAI.clone()).unwrap(), pool.token0_price());
        assert_eq!(pool.price_of(&USDC.clone()).unwrap(), pool.token1_price());
    }

    #[test]
    #[should_panic(expected = "InvalidToken")]
    fn price_of_throws_if_invalid_token() {
        let pool = Pool::new(
            USDC.clone(),
            DAI.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        pool.price_of(&WETH9::default().get(1).unwrap().clone())
            .unwrap();
    }

    #[test]
    fn chain_id_returns_token0_chain_id() {
        let pool = Pool::new(
            USDC.clone(),
            DAI.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        assert_eq!(pool.chain_id(), 1);
        let pool = Pool::new(
            DAI.clone(),
            USDC.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        assert_eq!(pool.chain_id(), 1);
    }

    #[test]
    fn involves_token() {
        let pool = Pool::new(
            USDC.clone(),
            DAI.clone(),
            FeeAmount::LOW,
            encode_sqrt_ratio_x96(1, 1),
            0,
        )
        .unwrap();
        assert!(pool.involves_token(&USDC.clone()));
        assert!(pool.involves_token(&DAI.clone()));
        assert!(!pool.involves_token(&WETH9::default().get(1).unwrap().clone()));
    }

    mod swaps {
        use super::*;
        use crate::utils::tick_math::{MAX_TICK, MIN_TICK};
        use once_cell::sync::Lazy;

        static POOL: Lazy<Pool<TickListDataProvider>> = Lazy::new(|| {
            Pool::new_with_tick_data_provider(
                USDC.clone(),
                DAI.clone(),
                FeeAmount::LOW,
                encode_sqrt_ratio_x96(1, 1),
                ONE_ETHER.into_limbs()[0] as u128,
                TickListDataProvider::new(
                    vec![
                        Tick::new(
                            nearest_usable_tick(MIN_TICK, FeeAmount::LOW.tick_spacing()).as_i32(),
                            ONE_ETHER.into_limbs()[0] as u128,
                            ONE_ETHER.into_limbs()[0] as i128,
                        ),
                        Tick::new(
                            nearest_usable_tick(MAX_TICK, FeeAmount::LOW.tick_spacing()).as_i32(),
                            ONE_ETHER.into_limbs()[0] as u128,
                            -(ONE_ETHER.into_limbs()[0] as i128),
                        ),
                    ],
                    FeeAmount::LOW.tick_spacing().as_i32(),
                ),
            )
            .unwrap()
        });

        #[test]
        fn get_output_amount_usdc_to_dai() {
            let output_amount = POOL
                .get_output_amount(
                    &CurrencyAmount::from_raw_amount(USDC.clone(), 100).unwrap(),
                    None,
                )
                .unwrap();
            assert!(output_amount.currency.equals(&DAI.clone()));
            assert_eq!(output_amount.quotient(), 98.into());
        }

        #[test]
        fn get_output_amount_dai_to_usdc() {
            let output_amount = POOL
                .get_output_amount(
                    &CurrencyAmount::from_raw_amount(DAI.clone(), 100).unwrap(),
                    None,
                )
                .unwrap();
            assert!(output_amount.currency.equals(&USDC.clone()));
            assert_eq!(output_amount.quotient(), 98.into());
        }

        #[test]
        fn get_input_amount_usdc_to_dai() {
            let input_amount = POOL
                .get_input_amount(
                    &CurrencyAmount::from_raw_amount(DAI.clone(), 98).unwrap(),
                    None,
                )
                .unwrap();
            assert!(input_amount.currency.equals(&USDC.clone()));
            assert_eq!(input_amount.quotient(), 100.into());
        }

        #[test]
        fn get_input_amount_dai_to_usdc() {
            let input_amount = POOL
                .get_input_amount(
                    &CurrencyAmount::from_raw_amount(USDC.clone(), 98).unwrap(),
                    None,
                )
                .unwrap();
            assert!(input_amount.currency.equals(&DAI.clone()));
            assert_eq!(input_amount.quotient(), 100.into());
        }
    }
}
