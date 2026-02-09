/// Helper macro to serialize a single field to big-endian bytes.
#[macro_export]
#[doc(hidden)]
macro_rules! __cougr_serialize_field {
    ($bytes:ident, $env:ident, $value:expr, i32) => {
        $bytes.append(&soroban_sdk::Bytes::from_array($env, &$value.to_be_bytes()));
    };
    ($bytes:ident, $env:ident, $value:expr, u32) => {
        $bytes.append(&soroban_sdk::Bytes::from_array($env, &$value.to_be_bytes()));
    };
    ($bytes:ident, $env:ident, $value:expr, i64) => {
        $bytes.append(&soroban_sdk::Bytes::from_slice($env, &$value.to_be_bytes()));
    };
    ($bytes:ident, $env:ident, $value:expr, u64) => {
        $bytes.append(&soroban_sdk::Bytes::from_slice($env, &$value.to_be_bytes()));
    };
    ($bytes:ident, $env:ident, $value:expr, i128) => {
        $bytes.append(&soroban_sdk::Bytes::from_slice($env, &$value.to_be_bytes()));
    };
    ($bytes:ident, $env:ident, $value:expr, u8) => {
        $bytes.append(&soroban_sdk::Bytes::from_array($env, &[$value]));
    };
    ($bytes:ident, $env:ident, $value:expr, bool) => {
        $bytes.append(&soroban_sdk::Bytes::from_array(
            $env,
            &[if $value { 1u8 } else { 0u8 }],
        ));
    };
    ($bytes:ident, $env:ident, $value:expr, u128) => {
        $bytes.append(&soroban_sdk::Bytes::from_slice($env, &$value.to_be_bytes()));
    };
    ($bytes:ident, $env:ident, $value:expr, bytes32) => {
        $bytes.append(&soroban_sdk::Bytes::from_slice($env, &$value.to_array()));
    };
}

/// Helper macro to get the byte size of a field type.
#[macro_export]
#[doc(hidden)]
macro_rules! __cougr_field_size {
    (i32) => {
        4u32
    };
    (u32) => {
        4u32
    };
    (i64) => {
        8u32
    };
    (u64) => {
        8u32
    };
    (i128) => {
        16u32
    };
    (u8) => {
        1u32
    };
    (bool) => {
        1u32
    };
    (u128) => {
        16u32
    };
    (bytes32) => {
        32u32
    };
}

/// Helper macro to deserialize a single field from big-endian bytes.
#[macro_export]
#[doc(hidden)]
macro_rules! __cougr_deserialize_field {
    ($env:ident, $data:ident, $offset:expr, i32) => {{
        let val = i32::from_be_bytes([
            $data.get($offset)?,
            $data.get($offset + 1)?,
            $data.get($offset + 2)?,
            $data.get($offset + 3)?,
        ]);
        val
    }};
    ($env:ident, $data:ident, $offset:expr, u32) => {{
        let val = u32::from_be_bytes([
            $data.get($offset)?,
            $data.get($offset + 1)?,
            $data.get($offset + 2)?,
            $data.get($offset + 3)?,
        ]);
        val
    }};
    ($env:ident, $data:ident, $offset:expr, i64) => {{
        let val = i64::from_be_bytes([
            $data.get($offset)?,
            $data.get($offset + 1)?,
            $data.get($offset + 2)?,
            $data.get($offset + 3)?,
            $data.get($offset + 4)?,
            $data.get($offset + 5)?,
            $data.get($offset + 6)?,
            $data.get($offset + 7)?,
        ]);
        val
    }};
    ($env:ident, $data:ident, $offset:expr, u64) => {{
        let val = u64::from_be_bytes([
            $data.get($offset)?,
            $data.get($offset + 1)?,
            $data.get($offset + 2)?,
            $data.get($offset + 3)?,
            $data.get($offset + 4)?,
            $data.get($offset + 5)?,
            $data.get($offset + 6)?,
            $data.get($offset + 7)?,
        ]);
        val
    }};
    ($env:ident, $data:ident, $offset:expr, i128) => {{
        let val = i128::from_be_bytes([
            $data.get($offset)?,
            $data.get($offset + 1)?,
            $data.get($offset + 2)?,
            $data.get($offset + 3)?,
            $data.get($offset + 4)?,
            $data.get($offset + 5)?,
            $data.get($offset + 6)?,
            $data.get($offset + 7)?,
            $data.get($offset + 8)?,
            $data.get($offset + 9)?,
            $data.get($offset + 10)?,
            $data.get($offset + 11)?,
            $data.get($offset + 12)?,
            $data.get($offset + 13)?,
            $data.get($offset + 14)?,
            $data.get($offset + 15)?,
        ]);
        val
    }};
    ($env:ident, $data:ident, $offset:expr, u8) => {{
        let val: u8 = $data.get($offset)?;
        val
    }};
    ($env:ident, $data:ident, $offset:expr, bool) => {{
        let val = $data.get($offset)? != 0;
        val
    }};
    ($env:ident, $data:ident, $offset:expr, u128) => {{
        let val = u128::from_be_bytes([
            $data.get($offset)?,
            $data.get($offset + 1)?,
            $data.get($offset + 2)?,
            $data.get($offset + 3)?,
            $data.get($offset + 4)?,
            $data.get($offset + 5)?,
            $data.get($offset + 6)?,
            $data.get($offset + 7)?,
            $data.get($offset + 8)?,
            $data.get($offset + 9)?,
            $data.get($offset + 10)?,
            $data.get($offset + 11)?,
            $data.get($offset + 12)?,
            $data.get($offset + 13)?,
            $data.get($offset + 14)?,
            $data.get($offset + 15)?,
        ]);
        val
    }};
    ($env:ident, $data:ident, $offset:expr, bytes32) => {{
        let mut arr = [0u8; 32];
        let mut i = 0u32;
        while i < 32 {
            arr[i as usize] = $data.get($offset + i)?;
            i += 1;
        }
        soroban_sdk::BytesN::from_array($env, &arr)
    }};
}

/// Implement `ComponentTrait` for a struct with fixed-size fields.
///
/// Generates serialization/deserialization using big-endian byte encoding.
///
/// # Supported field types
/// `i32` (4 bytes), `u32` (4 bytes), `i64` (8 bytes), `u64` (8 bytes),
/// `i128` (16 bytes), `u128` (16 bytes), `u8` (1 byte), `bool` (1 byte),
/// `bytes32` (32 bytes — use for `BytesN<32>` fields)
///
/// # Note
/// The symbol name must be at most 9 characters (Soroban `symbol_short!` limit).
///
/// # Example
/// ```ignore
/// #[contracttype]
/// #[derive(Clone, Debug)]
/// pub struct Position { pub x: i32, pub y: i32 }
///
/// impl_component!(Position, "position", Table, { x: i32, y: i32 });
/// ```
#[macro_export]
macro_rules! impl_component {
    ($struct_name:ident, $symbol:expr, $storage:ident, { $( $field:ident : $ftype:tt ),* $(,)? }) => {
        impl $crate::component::ComponentTrait for $struct_name {
            fn component_type() -> soroban_sdk::Symbol {
                soroban_sdk::symbol_short!($symbol)
            }

            fn serialize(&self, env: &soroban_sdk::Env) -> soroban_sdk::Bytes {
                let mut bytes = soroban_sdk::Bytes::new(env);
                $(
                    $crate::__cougr_serialize_field!(bytes, env, self.$field, $ftype);
                )*
                bytes
            }

            fn deserialize(_env: &soroban_sdk::Env, data: &soroban_sdk::Bytes) -> Option<Self> {
                let expected_len: u32 = 0 $( + $crate::__cougr_field_size!($ftype) )*;
                if data.len() != expected_len {
                    return None;
                }
                let mut _offset: u32 = 0;
                $(
                    let $field = $crate::__cougr_deserialize_field!(_env, data, _offset, $ftype);
                    _offset += $crate::__cougr_field_size!($ftype);
                )*
                Some(Self { $( $field ),* })
            }

            fn default_storage() -> $crate::component::ComponentStorage {
                $crate::component::ComponentStorage::$storage
            }
        }
    };
}

/// Implement `ComponentTrait` for a marker (unit) struct.
///
/// Marker components have no data and serialize to a single byte `[1]`.
///
/// # Example
/// ```ignore
/// pub struct SnakeHead;
///
/// impl_marker_component!(SnakeHead, "snkhead", Sparse);
/// ```
#[macro_export]
macro_rules! impl_marker_component {
    ($struct_name:ident, $symbol:expr, $storage:ident) => {
        impl $crate::component::ComponentTrait for $struct_name {
            fn component_type() -> soroban_sdk::Symbol {
                soroban_sdk::symbol_short!($symbol)
            }

            fn serialize(&self, env: &soroban_sdk::Env) -> soroban_sdk::Bytes {
                soroban_sdk::Bytes::from_array(env, &[1u8])
            }

            fn deserialize(_env: &soroban_sdk::Env, data: &soroban_sdk::Bytes) -> Option<Self> {
                if data.len() != 1 {
                    return None;
                }
                Some(Self)
            }

            fn default_storage() -> $crate::component::ComponentStorage {
                $crate::component::ComponentStorage::$storage
            }
        }
    };
}

/// Implement `ResourceTrait` for a struct with fixed-size fields.
///
/// Generates serialization/deserialization using big-endian byte encoding.
///
/// # Example
/// ```ignore
/// #[contracttype]
/// #[derive(Clone)]
/// pub struct GameState { pub score: i32, pub level: i32, pub is_game_over: bool }
///
/// impl_resource!(GameState, "gamestate", { score: i32, level: i32, is_game_over: bool });
/// ```
#[macro_export]
macro_rules! impl_resource {
    ($struct_name:ident, $symbol:expr, { $( $field:ident : $ftype:tt ),* $(,)? }) => {
        impl $crate::resource::ResourceTrait for $struct_name {
            fn resource_type() -> soroban_sdk::Symbol {
                soroban_sdk::symbol_short!($symbol)
            }

            fn serialize(&self, env: &soroban_sdk::Env) -> soroban_sdk::Bytes {
                let mut bytes = soroban_sdk::Bytes::new(env);
                $(
                    $crate::__cougr_serialize_field!(bytes, env, self.$field, $ftype);
                )*
                bytes
            }

            fn deserialize(_env: &soroban_sdk::Env, data: &soroban_sdk::Bytes) -> Option<Self> {
                let expected_len: u32 = 0 $( + $crate::__cougr_field_size!($ftype) )*;
                if data.len() != expected_len {
                    return None;
                }
                let mut _offset: u32 = 0;
                $(
                    let $field = $crate::__cougr_deserialize_field!(_env, data, _offset, $ftype);
                    _offset += $crate::__cougr_field_size!($ftype);
                )*
                Some(Self { $( $field ),* })
            }
        }
    };
}
