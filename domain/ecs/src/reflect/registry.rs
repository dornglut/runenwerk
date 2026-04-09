//! File: domain/ecs/src/reflect/registry.rs
//! Purpose: Global type registry for reflected ECS types.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::reflect::{Reflect, ReflectTypeId, TypeInfo};

#[derive(Debug, Default)]
pub struct TypeRegistry {
    next_id: u64,
    by_type_id: HashMap<TypeId, &'static TypeInfo>,
    by_reflect_id: HashMap<ReflectTypeId, &'static TypeInfo>,
    by_stable_name: HashMap<&'static str, &'static TypeInfo>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            by_type_id: HashMap::new(),
            by_reflect_id: HashMap::new(),
            by_stable_name: HashMap::new(),
        }
    }

    pub fn next_type_id(&mut self) -> ReflectTypeId {
        let id = ReflectTypeId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn register(&mut self, rust_type_id: TypeId, type_info: &'static TypeInfo) {
        self.by_type_id.insert(rust_type_id, type_info);
        self.by_reflect_id.insert(type_info.id, type_info);
        self.by_stable_name.insert(type_info.stable_name, type_info);
    }

    pub fn get_by_type_id(&self, rust_type_id: TypeId) -> Option<&'static TypeInfo> {
        self.by_type_id.get(&rust_type_id).copied()
    }

    pub fn get_by_reflect_id(&self, reflect_type_id: ReflectTypeId) -> Option<&'static TypeInfo> {
        self.by_reflect_id.get(&reflect_type_id).copied()
    }

    pub fn get_by_stable_name(&self, stable_name: &str) -> Option<&'static TypeInfo> {
        self.by_stable_name.get(stable_name).copied()
    }

    pub fn all_types(&self) -> impl Iterator<Item = &'static TypeInfo> + '_ {
        self.by_reflect_id.values().copied()
    }
}

static GLOBAL_TYPE_REGISTRY: OnceLock<Mutex<TypeRegistry>> = OnceLock::new();

pub fn global_type_registry() -> &'static Mutex<TypeRegistry> {
    GLOBAL_TYPE_REGISTRY.get_or_init(|| Mutex::new(TypeRegistry::new()))
}

pub fn allocate_reflect_type_id() -> ReflectTypeId {
    let mut registry = global_type_registry()
        .lock()
        .expect("type registry mutex poisoned");
    registry.next_type_id()
}

pub fn register_reflect_type<T>() -> &'static TypeInfo
where
    T: Reflect,
{
    let rust_type_id = TypeId::of::<T>();

    {
        let registry = global_type_registry()
            .lock()
            .expect("type registry mutex poisoned");

        if let Some(existing) = registry.get_by_type_id(rust_type_id) {
            return existing;
        }
    }

    let type_info = T::type_info();

    let mut registry = global_type_registry()
        .lock()
        .expect("type registry mutex poisoned");

    if let Some(existing) = registry.get_by_type_id(rust_type_id) {
        return existing;
    }

    registry.register(rust_type_id, type_info);
    type_info
}
