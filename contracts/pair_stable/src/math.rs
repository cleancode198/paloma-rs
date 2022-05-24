use cosmwasm_std::{Uint128, Uint256};
use std::convert::TryFrom;

const N_COINS_SQUARED: u8 = 4;
const ITERATIONS: u8 = 32;

pub const N_COINS: u8 = 2;
pub const MAX_AMP: u64 = 1_000_000;
pub const MAX_AMP_CHANGE: u64 = 10;
pub const MIN_AMP_CHANGING_TIME: u64 = 86400;
pub const AMP_PRECISION: u64 = 100;

/// ## Description
/// Calculates the ask amount (the amount of tokens swapped to).
/// ## Params
/// * **offer_pool** is an object of type [`u128`]. This is the amount of offer tokens currently in a stableswap pool.
///
/// * **ask_pool** is an object of type [`u128`]. This is the amount of ask tokens currently in a stableswap pool.
///
/// * **offer_amount** is an object of type [`u128`]. This is the amount of offer tokens to swap.
///
/// * **amp** is an object of type [`u64`]. This is the pool's amplification parameter.
pub fn calc_ask_amount(
    offer_pool: u128,
    ask_pool: u128,
    offer_amount: u128,
    amp: u64,
) -> Option<u128> {
    let leverage = amp.checked_mul(u64::from(N_COINS)).unwrap();
    let new_offer_pool = offer_pool.checked_add(offer_amount)?;

    let d = compute_d(leverage, offer_pool, ask_pool).unwrap();

    let new_ask_pool = compute_new_balance(leverage, new_offer_pool, d)?;

    let amount_swapped = ask_pool.checked_sub(new_ask_pool)?;
    Some(amount_swapped)
}

/// ## Description
/// Calculates the amount to be swapped (the offer amount).
/// ## Params
/// * **offer_pool** is an object of type [`u128`]. This is the amount of offer tokens currently in a stableswap pool.
///
/// * **ask_pool** is an object of type [`u128`]. This is the amount of ask tokens currently in a stableswap pool.
///
/// * **ask_amount** is an object of type [`u128`]. This is the amount of offer tokens to swap.
///
/// * **amp** is an object of type [`u64`]. This is the pool's amplification parameter.
pub fn calc_offer_amount(
    offer_pool: u128,
    ask_pool: u128,
    ask_amount: u128,
    amp: u64,
) -> Option<u128> {
    let leverage = amp.checked_mul(u64::from(N_COINS)).unwrap();
    let new_ask_pool = ask_pool.checked_sub(ask_amount)?;

    let d = compute_d(leverage, offer_pool, ask_pool).unwrap();

    let new_offer_pool = compute_new_balance(leverage, new_ask_pool, d)?;

    let amount_swapped = new_offer_pool.checked_sub(offer_pool)?;
    Some(amount_swapped)
}

/// ## Description
/// Computes the stableswap invariant (D).
///
/// * **Equation**
///
/// A * sum(x_i) * n**n + D = A * D * n**n + D**(n+1) / (n**n * prod(x_i))
///
/// ## Params
/// * **leverage** is an object of type [`u128`].
///
/// * **amount_a** is an object of type [`u128`].
///
/// * **amount_b** is an object of type [`u128`].
pub fn compute_d(leverage: u64, amount_a: u128, amount_b: u128) -> Option<u128> {
    let amount_a_times_coins = checked_u8_mul(&Uint256::from(amount_a), N_COINS)?
        .checked_add(Uint256::from(1u8))
        .ok()?;
    let amount_b_times_coins = checked_u8_mul(&Uint256::from(amount_b), N_COINS)?
        .checked_add(Uint256::from(1u8))
        .ok()?;
    let sum_x = amount_a.checked_add(amount_b)?; // sum(x_i), a.k.a S
    if sum_x == 0 {
        Some(0)
    } else {
        let mut d_previous: Uint256;
        let mut d: Uint256 = sum_x.into();

        // Newton's method to approximate D
        for _ in 0..ITERATIONS {
            let mut d_product = d;
            d_product = d_product
                .checked_mul(d)
                .ok()?
                .checked_div(amount_a_times_coins)
                .ok()?;
            d_product = d_product
                .checked_mul(d)
                .ok()?
                .checked_div(amount_b_times_coins)
                .ok()?;
            d_previous = d;
            // d = (leverage * sum_x + d_p * n_coins) * d / ((leverage - 1) * d + (n_coins + 1) * d_p);
            d = calculate_step(&d, leverage, sum_x, &d_product)?;
            // Equality with the precision of 1
            if d == d_previous {
                break;
            }
        }
        Uint128::try_from(d).ok().map(|i| i.into())
    }
}

/// ## Description
/// Helper function used to calculate the D invariant as a last step in the `compute_d` public function.
///
/// * **Equation**:
///
/// d = (leverage * sum_x + d_product * n_coins) * initial_d / ((leverage - 1) * initial_d + (n_coins + 1) * d_product)
fn calculate_step(
    initial_d: &Uint256,
    leverage: u64,
    sum_x: u128,
    d_product: &Uint256,
) -> Option<Uint256> {
    let leverage_mul =
        Uint256::from(leverage).checked_mul(sum_x.into()).ok()? / Uint256::from(AMP_PRECISION);
    let d_p_mul = checked_u8_mul(d_product, N_COINS)?;

    let l_val = leverage_mul
        .checked_add(d_p_mul)
        .ok()?
        .checked_mul(*initial_d)
        .ok()?;

    let leverage_sub = initial_d
        .checked_mul((leverage.checked_sub(AMP_PRECISION)?).into())
        .ok()?
        / Uint256::from(AMP_PRECISION);
    let n_coins_sum = checked_u8_mul(d_product, N_COINS.checked_add(1)?)?;

    let r_val = leverage_sub.checked_add(n_coins_sum).ok()?;

    l_val.checked_div(r_val).ok()
}

/// ## Description
/// Compute the swap amount `y` in proportion to `x`.
///
/// * **Solve for y**
///
/// y**2 + y * (sum' - (A*n**n - 1) * D / (A * n**n)) = D ** (n + 1) / (n ** (2 * n) * prod' * A)
///
/// y**2 + b*y = c
fn compute_new_balance(leverage: u64, new_source_amount: u128, d_val: u128) -> Option<u128> {
    // Upscale to Uint256
    let leverage: Uint256 = leverage.into();
    let new_source_amount: Uint256 = new_source_amount.into();
    let d_val: Uint256 = d_val.into();

    // sum' = prod' = x
    // c =  D ** (n + 1) / (n ** (2 * n) * prod' * A)
    let c = checked_u8_power(&d_val, N_COINS.checked_add(1)?)?
        .checked_mul(Uint256::from(AMP_PRECISION))
        .ok()?
        .checked_div(
            checked_u8_mul(&new_source_amount, N_COINS_SQUARED)?
                .checked_mul(leverage)
                .ok()?,
        )
        .ok()?;

    // b = sum' - (A*n**n - 1) * D / (A * n**n)
    let b = new_source_amount
        .checked_add(
            d_val
                .checked_mul(Uint256::from(AMP_PRECISION))
                .ok()?
                .checked_div(leverage)
                .ok()?,
        )
        .ok()?;

    // Solve for y by approximating: y**2 + b*y = c
    let mut y_prev: Uint256;
    let mut y = d_val;
    for _ in 0..ITERATIONS {
        y_prev = y;
        y = (checked_u8_power(&y, 2)?.checked_add(c).ok()?)
            .checked_div(
                checked_u8_mul(&y, 2)?
                    .checked_add(b)
                    .ok()?
                    .checked_sub(d_val)
                    .ok()?,
            )
            .ok()?;
        if y == y_prev {
            break;
        }
    }
    Uint128::try_from(y).ok().map(|i| i.into())
}

/// ## Description
/// Returns self to the power of b.
fn checked_u8_power(a: &Uint256, b: u8) -> Option<Uint256> {
    let mut result = *a;
    for _ in 1..b {
        result = result.checked_mul(*a).ok()?;
    }
    Some(result)
}

/// ## Description
/// Returns self multiplied by b.
fn checked_u8_mul(a: &Uint256, b: u8) -> Option<Uint256> {
    let mut result = *a;
    for _ in 1..b {
        result = result.checked_add(*a).ok()?;
    }
    Some(result)
}
