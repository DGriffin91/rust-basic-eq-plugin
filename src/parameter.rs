use vst::util::AtomicFloat;

use crate::{atomic_bool::AtomicBool, units::Units};

pub struct Parameter {
    name: String,
    normalized_value: AtomicFloat,
    value: AtomicFloat,
    pub default: f32,
    pub min: f32,
    pub max: f32,
    display_func: fn(f32) -> String,
    pub transform_func: fn(f32) -> f32,
    pub inv_transform_func: fn(f32) -> f32,
    need_to_update_dsp: AtomicBool,
}

impl Parameter {
    pub fn new(
        name: &str,
        default: f32,
        min: f32,
        max: f32,
        display_func: fn(f32) -> String,
        transform_func: fn(f32) -> f32,
        inv_transform_func: fn(f32) -> f32,
    ) -> Parameter {
        Parameter {
            name: String::from(name),
            normalized_value: AtomicFloat::new(default.from_range(min, max)),
            value: AtomicFloat::new(default),
            default,
            min,
            max,
            display_func,
            transform_func,
            inv_transform_func,
            need_to_update_dsp: AtomicBool::new(true),
        }
    }

    pub fn get_normalized(&self) -> f32 {
        self.normalized_value.get()
    }

    pub fn get_normalized_default(&self) -> f32 {
        (self.inv_transform_func)(self.default.from_range(self.min, self.max))
    }

    pub fn set_normalized(&self, x: f32) {
        self.need_to_update_dsp.set(true);
        let x = x.max(0.0).min(1.0);
        self.normalized_value.set(x);
        self.value
            .set((self.transform_func)(x).to_range(self.min, self.max));
    }

    pub fn get(&self) -> f32 {
        self.value.get()
    }

    pub fn set(&self, x: f32) {
        self.need_to_update_dsp.set(true);
        let x = x.max(self.min).min(self.max);
        self.value.set(x);
        self.normalized_value
            .set((self.inv_transform_func)(x.from_range(self.min, self.max)));
    }

    pub fn get_display(&self) -> String {
        (self.display_func)(self.value.get())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn dsp_update(&self) -> bool {
        let need_to_update = self.need_to_update_dsp.get();
        self.need_to_update_dsp.set(false);
        need_to_update
    }
}
