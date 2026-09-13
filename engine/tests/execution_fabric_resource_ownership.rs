use engine::prelude::App;
use engine::runtime::{RuntimeJobExecutorResource, RuntimeProductCacheResource};

#[test]
fn bare_app_does_not_install_execution_fabric_resources() {
    let app = App::headless();

    assert!(
        app.world()
            .resource::<RuntimeJobExecutorResource>()
            .is_err()
    );
    assert!(
        app.world()
            .resource::<RuntimeProductCacheResource>()
            .is_err()
    );
}
