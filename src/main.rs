use opentale_inventory::app::create_app;
use opentale_inventory::world::item::items::*;

fn main() {
    for (id, item) in ITEMS {
        println!("{} -> {:?}", id, item.properties);
    }

    println!("DIAMOND durability: {:?}", RING.properties.durability);

    create_app().run();

    println!("Closed the inventory demo successfully.");
}