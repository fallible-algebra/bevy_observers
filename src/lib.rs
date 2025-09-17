//! This crate provides the [`observers`] macro. See its documentation for further info.

pub use bevy_ecs;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};

/// A macro for setting [`Observer`]s on an entity from within a [`Bundle`]. It is similar to the [`children`] macro, but for observers.
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_bundled_observers::observers;
/// # #[derive(EntityEvent)] struct Collect { entity: Entity };
///
/// fn coin() -> impl Bundle {
///     (
///         Name::new("Coin"),
///         observers![|_: On<Collect>| {
///             info!("You collected a coin!");
///         }],
///     )
/// }
/// ```
#[macro_export]
macro_rules! observers {
    [$($observer:expr),*$(,)?] => {
       $crate::Observers(vec![$($crate::bevy_ecs::observer::Observer::new($observer)),*])
    };
}

/// A component that sets observers on an entity when inserted. This is the underlying mechanism for the [`observers`] macro.
///
/// The component is immediately emptied and promptly removed after insertion.
///
/// The code example below shows what the [`observers`] macro expands to.
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_bundled_observers::Observers;
/// # #[derive(EntityEvent)] struct OnCollect { entity: Entity };
///
/// fn coin() -> impl Bundle {
///     (
///         Name::new("Coin"),
///         Observers(vec![Observer::new(|_: On<OnCollect>| {
///             info!("You collected a coin!");
///         })]),
///     )
/// }
/// ```
#[derive(Component)]
#[component(on_insert = on_insert)]
pub struct Observers(pub Vec<Observer>);

fn on_insert(mut world: DeferredWorld, context: HookContext) {
    let mut component: Mut<Observers> = world.get_mut(context.entity).unwrap();

    let observers = core::mem::take(&mut component.0)
        .into_iter()
        .map(move |observer| observer.with_entity(context.entity));

    let mut commands = world.commands();
    commands.spawn_batch(observers);
    commands.entity(context.entity).remove::<Observers>();
}

#[cfg(test)]
mod test {
    use bevy_ecs::prelude::*;

    #[test]
    fn test() {
        #[derive(EntityEvent, Debug)]
        struct OnFoo {entity: Entity}

        #[derive(Component, Debug, PartialEq, Eq)]
        struct Bar(i32);

        let mut world = World::new();

        let entity = world
            .spawn((
                Bar(0),
                observers![|trigger: On<OnFoo>, mut query: Query<&mut Bar>| {
                    query.get_mut(trigger.entity).unwrap().0 += 1;
                }],
            ))
            .id();

        world.trigger(OnFoo { entity });

        assert_eq!(world.get::<Bar>(entity), Some(&Bar(1)));
    }
}
