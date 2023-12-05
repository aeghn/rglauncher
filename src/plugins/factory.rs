use gtk::Widget;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Mutex;

use super::{PluginPreview, PluginResult};

type CreatorFn<R> = fn() -> Box<dyn PluginPreview<R>>;

lazy_static::lazy_static! {
    static ref REGISTRY: Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>> = Mutex::new(HashMap::new());
}

pub fn register_inner<R, T>()
where
    R: PluginResult + 'static,
    T: PluginPreview<R> + 'static,
{
    let type_id = TypeId::of::<T>();
    let creator_fn: CreatorFn<R> = || Box::new(T::new());

    REGISTRY
        .lock()
        .unwrap()
        .insert(type_id, Box::new(creator_fn));
}

pub fn create_plugin_preview<R>(type_id: TypeId) -> Option<Box<dyn PluginPreview<R>>>
where
    R: PluginResult + 'static,
{
    if let Some(creator) = REGISTRY.lock().unwrap().get(&type_id) {
        if let Some(creator_fn) = creator.downcast_ref::<CreatorFn<R>>() {
            return Some(creator_fn());
        }
    }
    None
}

#[macro_export]
macro_rules! register_plugin_preview {
    ($result_type:ty, $preview_type:ty) => {
        #[ctor::ctor]
        fn register() {
            crate::plugins::factory::register_inner::<$result_type, $preview_type>();
        }
    };
}
