#[macro_export]
macro_rules! define_items {
    ($($item:ident => $identifier:literal as $display_name:literal: $props:expr),* $(,)?) => {
        $(
            pub const $item: Item = Item {
                identifier: $identifier,
                display_name: $display_name,
                properties: $props
            };
        )*

        pub const ITEMS: &[(&str, &Item)] = &[$( ($identifier, &$item) ),*];
    };
}