A prototype inventory system for an open-source game.

### Registration:
During game startup, registries go through different phases.
1. Registration: Our game and mods register content to internal registries
2. Freezing: Registries become immutable and assign final numeric index to underlying objects
3. Runtime: Registries are read-only, used for lookups by ID or index in O(1) time

### Structs
1. ResourceLocation
2. Item
3. ItemEntry
4. ItemRegistry
5. ItemStack
6. ItemRegistryResource
7. ItemPlugin


<!-- should re-build with slight reference to https://github.com/mwbryant/bevy_survival_crafting_game/blob/master/src/inventory.rs ?-->
