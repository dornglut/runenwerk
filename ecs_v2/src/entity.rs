#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

#[derive(Default)]
pub struct EntityAllocator {
    next_id: u32,
    free_list: Vec<u32>,
    generations: Vec<u32>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(&mut self) -> Entity {
        if let Some(id) = self.free_list.pop() {
            Entity {
                id,
                generation: self.generations[id as usize],
            }
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.generations.push(0);
            Entity { id, generation: 0 }
        }
    }

    pub fn free(&mut self, entity: Entity) {
        if let Some(generation) = self.generations.get_mut(entity.id as usize) {
            *generation = generation.saturating_add(1);
            self.free_list.push(entity.id);
        }
    }
}
