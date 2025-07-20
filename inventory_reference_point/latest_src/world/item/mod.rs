pub mod item;
pub mod items;

/// `define_items` is a declarative macro. (a.k.a. a "macro by example")
///
/// https://doc.rust-lang.org/book/ch20-05-macros.html
///
/// Macros compare a value to patterns that are associated with particular code: in this situation, the value is the literal Rust source code passed to the macro; the patterns are compared with the structure of that source code; and the code associated with each pattern, when matched, replaces the code passed to the macro. This all happens during compilation.
#[macro_export]
macro_rules! define_items {

    // this is the pattern the macro is matching against
    (
        // $( ... ),* is a repetition pattern meaning "match this pattern zero or more times, separated by commas"
        $(
            // $item:ident means "capture an identifier and call it $item"
            // $identifier:literal means "capture a string literal and call it $identifier"
            $item:ident => $identifier:literal

            // "as" is literal text that must appear exactly
            as

            // display_name:literal means "capture a string literal and call it display_name"
            $display_name:literal

            // "@" separates display name from sprite coordinates
            @

            // sprite coordinates as (x, y) tuple - use tt (token tree) instead of expr
            $sprite_coords:tt

            // ":" is literal text
            :

            // $props:expr means "capture an expression and call it $props"
            $props:expr
        ),*

        // $(,)? means "optionally match a trailing comma"
        $(,)?
    )

    // => separates the pattern (what to match) from the expansion (what to generate)
    =>

    // this (expansion) is repeated for each succesful capture
    {

        // $( ... )* is a repetition pattern meaning "expand this pattern for each capture"
        $(
            // each capture generates a public constant Item
            // $item expands to the captured identifier (like DIAMOND)
            // $identifier expands to the captured string literal (like "diamond")
            // $display_name expands to the display string (like "Diamond")
            // $sprite_coords expands to the sprite coordinates (like (0, 0))
            // $props expands to the captured expression (like ItemProperties::new())
            pub const $item: Item = Item {
                identifier: $identifier,
                display_name: $display_name,
                properties: $props,
                sprite_coords: $sprite_coords,
            };
        )*

        // AFTER each capture has been expanded,
        // we create a slice/array containing all items expanded from captures
        pub const ITEMS: &[(&str, &Item)] = &[
            $(
                // $identifier is the string like "diamond"
                // &$item is a reference to the const we created above like &DIAMOND
                ($identifier, &$item)
            ),*
        ];
    };
}