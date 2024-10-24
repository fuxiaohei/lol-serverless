use sea_orm_migration::{MigrationTrait, MigratorTrait};

mod m01_settings_table;
mod m02_user_table;
mod m03_usertoken_table;
mod m04_project_table;
mod m05_playground_table;
mod m06_deployment_table;
mod m07_deploystate_table;
mod m08_projectenvs_table;
mod m09_wasm_artifacts_table;
mod m10_worker_node_table;
mod m11_deploy_task_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m01_settings_table::Migration),
            Box::new(m02_user_table::Migration),
            Box::new(m03_usertoken_table::Migration),
            Box::new(m04_project_table::Migration),
            Box::new(m05_playground_table::Migration),
            Box::new(m06_deployment_table::Migration),
            Box::new(m07_deploystate_table::Migration),
            Box::new(m08_projectenvs_table::Migration),
            Box::new(m09_wasm_artifacts_table::Migration),
            Box::new(m10_worker_node_table::Migration),
            Box::new(m11_deploy_task_table::Migration),
        ]
    }
}
