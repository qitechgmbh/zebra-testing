macro_rules! responses {
    (
        $(
            $name:ident => $msg:expr
        ),* $(,)?
    ) => {
        $(
            pub const $name: &'static str = $msg;
        )*
    };
}

responses! {
    MISSING_PATH => 
        "Missing HTTP path. Supported paths: /weights, /plates, /orders, /logs",

    UNSUPPORTED_ROUTE => 
        "Unknown route. Supported: /weights, /plates, /orders, /logs",

    MISSING_METHOD => 
        "Missing HTTP method (must be GET)",

    INVALID_METHOD => 
        "Invalid method (only GET supported)",
}