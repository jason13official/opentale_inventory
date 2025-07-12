use crate::world::inventory::item_stack::ItemStack;

pub fn format_item_display(stack: &ItemStack) -> String {
    if stack.size > 1 {
        format!("{}\n{}", stack.item.unwrap().display_name, stack.size)
    } else {
        stack.item.unwrap().display_name.to_string()
    }
}