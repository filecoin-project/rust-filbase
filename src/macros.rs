#[macro_export]
macro_rules! hex_arr {
    ($size:expr, $matches:expr, $field:expr) => {{
        let v = $matches.value_of($field).expect("missing required field");
        <[u8; $size] as hex::FromHex>::from_hex(v)
    }};
}
#[macro_export]
macro_rules! hex_vec_arr {
    ($size:expr, $matches:expr, $field:expr) => {{
        $matches
            .values_of($field)
            .expect("missing required field")
            .map(<[u8; $size] as hex::FromHex>::from_hex)
            .collect::<Result<Vec<_>, _>>()
    }};
}
#[macro_export]
macro_rules! hex_vec_vec {
    ($matches:expr, $field:expr) => {{
        $matches
            .values_of($field)
            .expect("missing required field")
            .map(<Vec<u8> as hex::FromHex>::from_hex)
            .collect::<Result<Vec<_>, _>>()
    }};
}
#[macro_export]
macro_rules! hex_vec {
    ($matches:expr, $field:expr) => {{
        let v = $matches.value_of($field).expect("missing required field");
        <Vec<u8> as hex::FromHex>::from_hex(v)
    }};
}
