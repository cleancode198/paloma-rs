pub mod asset;
pub mod common;
pub mod factory;
pub mod generator;
pub mod generator_proxy;
pub mod maker;
pub mod oracle;
pub mod pair;
pub mod pair_anchor;
pub mod pair_stable_bluna;
pub mod querier;
pub mod restricted_vector;
pub mod router;
pub mod staking;
pub mod token;
pub mod vesting;
pub mod whitelist;
pub mod xastro_token;

#[cfg(test)]
mod mock_querier;

#[cfg(test)]
mod testing;

mod decimal_checked_ops {
    use cosmwasm_std::{Decimal, Fraction, OverflowError, Uint128, Uint256};
    use std::convert::TryInto;
    pub trait DecimalCheckedOps {
        fn checked_add(self, other: Decimal) -> Result<Decimal, OverflowError>;
        fn checked_mul_u128(self, other: Uint128) -> Result<Uint128, OverflowError>;
    }

    impl DecimalCheckedOps for Decimal {
        fn checked_add(self, other: Decimal) -> Result<Decimal, OverflowError> {
            self.numerator()
                .checked_add(other.numerator())
                .map(|_| self + other)
        }
        fn checked_mul_u128(self, other: Uint128) -> Result<Uint128, OverflowError> {
            if self.is_zero() || other.is_zero() {
                return Ok(Uint128::zero());
            }
            let multiply_ratio =
                other.full_mul(self.numerator()) / Uint256::from(self.denominator());
            if multiply_ratio > Uint256::from(Uint128::MAX) {
                Err(OverflowError::new(
                    cosmwasm_std::OverflowOperation::Mul,
                    self,
                    other,
                ))
            } else {
                Ok(multiply_ratio.try_into().unwrap())
            }
        }
    }
}

pub use decimal_checked_ops::DecimalCheckedOps;
