// Owner: Engine App Runtime - Plugin Tuple Wiring
pub trait IntoPlugins {
    fn add_to_app(self, app: &mut App);
}

impl<P> IntoPlugins for P
where
    P: Plugin + 'static,
{
    fn add_to_app(self, app: &mut App) {
        app.add_plugin(self);
    }
}

impl IntoPlugins for Box<dyn Plugin> {
    fn add_to_app(self, app: &mut App) {
        app.add_boxed_plugin(self);
    }
}

impl IntoPlugins for Vec<Box<dyn Plugin>> {
    fn add_to_app(self, app: &mut App) {
        for plugin in self {
            app.add_boxed_plugin(plugin);
        }
    }
}

macro_rules! impl_into_plugins_tuple {
    ($(($name:ident, $index:tt)),+ $(,)?) => {
        impl<$($name),+> IntoPlugins for ($($name,)+)
        where
            $($name: IntoPlugins,)+
        {
            fn add_to_app(self, app: &mut App) {
                $(
                    self.$index.add_to_app(app);
                )+
            }
        }
    };
}

impl_into_plugins_tuple!((A, 0), (B, 1));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_into_plugins_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);
