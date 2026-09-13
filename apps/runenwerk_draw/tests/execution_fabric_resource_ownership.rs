use engine::prelude::App;
use engine::runtime::{
    RuntimeJobExecutorConfig, RuntimeJobExecutorResource, RuntimeProductCacheResource,
};
use product::{
    ProductDescriptorCore, ProductFamily, ProductIdentity, ProductKind, ProductLineage,
    ProductScaleBand, ProductScope,
};
use runenwerk_draw::runtime::DrawingAppPlugin;

#[test]
fn drawing_app_plugin_installs_product_execution_resources() {
    let mut app = App::headless();
    app.add_plugin(DrawingAppPlugin);

    let executor = app
        .world()
        .resource::<RuntimeJobExecutorResource>()
        .expect("DrawingAppPlugin should install its product job executor");
    assert_eq!(
        executor.config(),
        &RuntimeJobExecutorConfig::worker_pool(2, 64)
    );
    assert!(app.world().resource::<RuntimeProductCacheResource>().is_ok());
}

#[test]
fn drawing_app_plugin_preserves_explicit_product_execution_resources() {
    let mut app = App::headless();
    app.insert_resource(RuntimeJobExecutorResource::with_config(
        RuntimeJobExecutorConfig::serial(),
    ));
    let mut cache = RuntimeProductCacheResource::default();
    cache.record_accepted_descriptor(test_descriptor());
    app.insert_resource(cache);

    app.add_plugin(DrawingAppPlugin);

    let executor = app
        .world()
        .resource::<RuntimeJobExecutorResource>()
        .expect("explicit runtime job executor should remain installed");
    assert_eq!(executor.config(), &RuntimeJobExecutorConfig::serial());

    let cache = app
        .world()
        .resource::<RuntimeProductCacheResource>()
        .expect("explicit runtime product cache should remain installed");
    assert_eq!(cache.snapshot().entry_count, 1);
}

fn test_descriptor() -> ProductDescriptorCore {
    ProductDescriptorCore::new(
        ProductIdentity::new(641),
        ProductFamily::Texture,
        ProductKind::new("test.execution-fabric-resource-ownership"),
        ProductScope::non_spatial("test"),
        ProductScaleBand::Preview,
        ProductLineage::new("runenwerk.draw.execution-fabric-resource-ownership", 1)
            .with_source_revision("1")
            .with_source_key("source:641"),
    )
}
