// Owner: ECS World - Borrow Wrappers, Entity Views, and Commands
pub struct Mut<'a, T> {
    value: &'a mut T,
}

impl<'a, T> Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for Mut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

pub struct Res<'a, T> {
    value: &'a T,
}

impl<'a, T> Deref for Res<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

pub struct ResMut<'a, T> {
    value: &'a mut T,
}

impl<'a, T> Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

pub struct EntityRef<'w> {
    world: &'w World,
    entity: Entity,
}

impl<'w> EntityRef<'w> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn contains<T: Component>(&self) -> bool {
        self.world.contains_component::<T>(self.entity)
    }

    pub fn get<T: Component>(&self) -> Option<&T> {
        self.world.get::<T>(self.entity)
    }

    pub fn require<T: Component>(&self) -> Result<&T, EntityError> {
        self.world.require::<T>(self.entity)
    }
}

pub struct EntityMut<'w> {
    world: &'w mut World,
    entity: Entity,
}

impl<'w> EntityMut<'w> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn contains<T: Component>(&self) -> bool {
        self.world.contains_component::<T>(self.entity)
    }

    pub fn get<T: Component>(&self) -> Option<&T> {
        self.world.get::<T>(self.entity)
    }

    pub fn get_mut<T: Component>(&mut self) -> Option<Mut<'_, T>> {
        self.world.get_mut::<T>(self.entity)
    }

    pub fn require<T: Component>(&self) -> Result<&T, EntityError> {
        self.world.require::<T>(self.entity)
    }

    pub fn require_mut<T: Component>(&mut self) -> Result<Mut<'_, T>, EntityError> {
        self.world.require_mut::<T>(self.entity)
    }

    pub fn insert<B: Bundle>(&mut self, bundle: B) -> Result<(), EntityError> {
        self.world.insert(self.entity, bundle)
    }

    pub fn remove<B: Bundle>(&mut self) -> Result<B, EntityError> {
        self.world.remove::<B>(self.entity)
    }

    pub fn despawn(self) -> Result<(), EntityError> {
        self.world.despawn(self.entity)
    }
}

trait WorldCommand {
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), CommandError>;
}

impl<F> WorldCommand for F
where
    F: FnOnce(&mut World) -> Result<(), CommandError> + 'static,
{
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), CommandError> {
        (*self)(world)
    }
}

pub struct Commands {
    queue: Vec<Box<dyn WorldCommand>>,
}

impl Commands {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn spawn<B: Bundle + 'static>(&mut self, bundle: B) {
        self.queue.push(Box::new(move |world: &mut World| {
            world.spawn(bundle);
            Ok(())
        }));
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue.push(Box::new(move |world: &mut World| {
            world.despawn(entity)?;
            Ok(())
        }));
    }

    pub fn insert<B: Bundle + 'static>(&mut self, entity: Entity, bundle: B) {
        self.queue.push(Box::new(move |world: &mut World| {
            world.insert(entity, bundle)?;
            Ok(())
        }));
    }

    pub fn remove<B: Bundle + 'static>(&mut self, entity: Entity) {
        self.queue.push(Box::new(move |world: &mut World| {
            let _: B = world.remove(entity)?;
            Ok(())
        }));
    }

    pub fn apply(self, world: &mut World) -> Result<(), CommandError> {
        for command in self.queue {
            command.apply(world)?;
        }
        Ok(())
    }
}

