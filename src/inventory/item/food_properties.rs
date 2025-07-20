use std::rc::Rc;

// todo create proper structs for MobEffectInstance's and AttributeModifiers during full game implementation
type MobEffectInstance = String;


// preface from rc.rs:
// Shared references in Rust disallow mutation by default, and Rc is no exception: you cannot
// generally obtain a mutable reference to something inside an Rc. If you need mutability,
// put a Cell or RefCell inside the Rc.
/// We are explicitly using Rc instead of Box for two reasons:
/// 1. `Box<dyn Fn() -> MobEffectInstance>` cannot be cloned because trait objects don't implement Clone
/// 2. `Rc<T>` (Reference Counted) can be cloned - it clones the reference, not the underlying data.
type EffectSupplier = Rc<dyn Fn() -> MobEffectInstance>;

#[derive(Clone)]
pub struct FoodProperties {
    nutrition: u32,
    saturation_modifier: f32,
    is_meat: bool,
    can_always_eat: bool,
    fast_food: bool,
    effects: Vec<(EffectSupplier, f32)>,
}

impl FoodProperties {

    pub fn get_nutrition(&self) -> u32 {
        self.nutrition
    }

    pub fn get_saturation_modifier(&self) -> f32 {
        self.saturation_modifier
    }

    pub fn is_meat(&self) -> bool {
        self.is_meat
    }

    pub fn can_always_eat(&self) -> bool {
        self.can_always_eat
    }

    pub fn is_fast_food(&self) -> bool {
        self.can_always_eat
    }

    pub fn get_effects(&self) -> Vec<(EffectSupplier, f32)> {
        self.effects.clone()
    }
}

impl std::fmt::Debug for FoodProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FoodProperties")
            .field("nutrition", &self.nutrition)
            .field("saturation_modifier", &self.saturation_modifier)
            .field("is_meat", &self.is_meat)
            .field("can_always_eat", &self.can_always_eat)
            .field("fast_food", &self.fast_food)
            .field("effects", &format!("[{} effects]", self.effects.len()))
            .finish()
    }
}