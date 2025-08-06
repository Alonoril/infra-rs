# PostgreSQL Extension for Ethereum Big Numbers

This module provides SeaORM type mappings for Ethereum's unsigned integer types (u64, U128, U256) to PostgreSQL numeric types.

## Why This Extension?

Ethereum uses large unsigned integers that don't map directly to PostgreSQL types:
- `u64`: Used for gas limits, nonces, block numbers
- `U128`: Used for some token balances 
- `U256`: Used for wei values, large token balances

PostgreSQL's `BIGINT` can only store up to `i64::MAX`, and we need proper type safety for these values.

## Types Provided

### `DbU64`
- Wrapper for `u64` values
- Maps to PostgreSQL `BIGINT`
- Handles conversion safely

### `DbU128`
- Wrapper for `ruint::U128` values
- Maps to PostgreSQL `NUMERIC(39, 0)`
- Handles values up to 340 undecillion

### `DbU256`
- Wrapper for `ruint::U256` values
- Maps to PostgreSQL `NUMERIC(78, 0)`
- Handles values up to 115 quattuorvigintillion

## Usage Example

```rust
use sql_infra::pg_ext::{DbU64, DbU128, DbU256};
use sea_orm::entity::prelude::*;
use ruint::aliases::{U128, U256};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "transactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    
    #[sea_orm(column_type = "BigInteger")]
    pub gas_used: DbU64,
    
    #[sea_orm(column_type = "Decimal(Some((39, 0)))")]
    pub amount: DbU128,
    
    #[sea_orm(column_type = "Decimal(Some((78, 0)))")]
    pub total_wei: DbU256,
}
```

## SQL Schema

```sql
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    gas_used BIGINT NOT NULL,
    amount NUMERIC(39, 0) NOT NULL,
    total_wei NUMERIC(78, 0) NOT NULL
);
```

## Important Notes

1. **Large Number Handling**: This module uses the `bigdecimal` crate to handle arbitrary precision numbers, ensuring all Ethereum unsigned integers can be stored and retrieved without precision loss. No string fallback is needed.

2. **Performance**: For best performance, use `DbU64` when values fit in u64 range. Only use `DbU128` and `DbU256` when necessary.

3. **Conversions**: All types implement `From` traits for easy conversion:
   ```rust
   let gas: u64 = 21000;
   let db_gas = DbU64::from(gas);
   let gas_back: u64 = db_gas.into();
   ```

4. **Integration with Alloy**: These types work seamlessly with alloy-rs types:
   ```rust
   use alloy::primitives::U256 as AlloyU256;
   
   let alloy_value = AlloyU256::from(1000u64);
   let db_value = DbU256::from(U256::from_le_bytes(alloy_value.to_le_bytes()));
   ```