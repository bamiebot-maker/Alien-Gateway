#![no_std]

pub mod errors;

/// Shared authorization and panic helpers for onchain contracts.
pub mod auth {
    use soroban_sdk::{Address, Env, Error};

    /// Requires auth from the provided address.
    pub fn require_address_auth(address: &Address) {
        address.require_auth();
    }

    /// Returns the value or panics with the provided contract error.
    pub fn unwrap_or_panic<T, E>(env: &Env, value: Option<T>, error: E) -> T
    where
        E: Copy + Into<Error>,
    {
        value.unwrap_or_else(|| env.panic_with_error(error))
    }

    /// Requires the caller to authorize and match the expected owner.
    pub fn require_matching_auth<E>(env: &Env, caller: &Address, owner: &Address, error: E)
    where
        E: Copy + Into<Error>,
    {
        caller.require_auth();
        if caller != owner {
            env.panic_with_error(error);
        }
    }
}

/// Shared storage helpers for persistent TTL handling.
pub mod storage {
    use core::fmt::Debug;

    use soroban_sdk::{Env, IntoVal, TryFromVal, Val};

    /// Bump amount: ~30 days (at ~5s per ledger close).
    pub const PERSISTENT_BUMP_AMOUNT: u32 = 518_400;
    /// Lifetime threshold: ~7 days — entries are extended when remaining TTL drops below this.
    pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;

    /// Extends the TTL for a persistent storage key using the shared policy.
    pub fn bump_persistent<K>(env: &Env, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Writes a persistent value and bumps its TTL.
    pub fn set_persistent<K, V>(env: &Env, key: &K, value: &V)
    where
        K: IntoVal<Env, Val>,
        V: IntoVal<Env, Val>,
    {
        env.storage().persistent().set(key, value);
        bump_persistent(env, key);
    }

    /// Reads a persistent value.
    pub fn get_persistent<K, V>(env: &Env, key: &K) -> Option<V>
    where
        V::Error: Debug,
        K: IntoVal<Env, Val>,
        V: TryFromVal<Env, Val>,
    {
        env.storage().persistent().get(key)
    }

    /// Reads a persistent value and bumps its TTL if present.
    pub fn get_persistent_with_ttl<K, V>(env: &Env, key: &K) -> Option<V>
    where
        V::Error: Debug,
        K: IntoVal<Env, Val>,
        V: TryFromVal<Env, Val>,
    {
        let value = env.storage().persistent().get(key);
        if value.is_some() {
            bump_persistent(env, key);
        }
        value
    }

    /// Reads an instance value.
    pub fn get_instance<K, V>(env: &Env, key: &K) -> Option<V>
    where
        V::Error: Debug,
        K: IntoVal<Env, Val>,
        V: TryFromVal<Env, Val>,
    {
        env.storage().instance().get(key)
    }

    /// Writes an instance value.
    pub fn set_instance<K, V>(env: &Env, key: &K, value: &V)
    where
        K: IntoVal<Env, Val>,
        V: IntoVal<Env, Val>,
    {
        env.storage().instance().set(key, value);
    }
}
