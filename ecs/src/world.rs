use std::collections::HashMap;
use crate::archetype::Archetype;
use crate::component::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

pub struct World {
    next_entity: usize,
    pub archetypes: Vec<Archetype>,
    pub entity_archetype: HashMap<Entity, usize>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            archetypes: Vec::new(),
            entity_archetype: HashMap::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_entity);
        self.next_entity += 1;

        let index = self.archetypes.iter().position(|a| a.entities.is_empty())
          .unwrap_or_else(|| {
              self.archetypes.push(Archetype::new());
              self.archetypes.len() - 1
          });

        self.archetypes[index].entities.push(entity);
        self.entity_archetype.insert(entity, index);
        entity
    }

    pub fn add_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let index = *self.entity_archetype.get(&entity).expect("Entity not found");
        let archetype = &mut self.archetypes[index];

        if !archetype.has_component::<T>() {
            archetype.add_component_vec::<T>(vec![component]);
        } else {
            archetype.get_component_vec_mut::<T>().unwrap().push(component);
        }
    }

    pub fn query_mut2<T: Component, U: Component>(&mut self) -> Vec<(Entity, &mut T, &U)> {
        let mut results = Vec::new();

        for archetype in &mut self.archetypes {
            // split borrow: mutable for T, immutable for U
            let entities = &archetype.entities;
            let t_vec_opt = archetype.get_component_vec_mut::<T>();
            let u_vec_opt = archetype.get_component_vec::<U>();

            if let (Some(t_vec), Some(u_vec)) = (t_vec_opt, u_vec_opt) {
                for i in 0..t_vec.len() {
                    results.push((entities[i], &mut t_vec[i], &u_vec[i]));
                }
            }
        }

        results
    }
}
