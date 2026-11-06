pub use sea_orm_migration::prelude::*;

mod m20261106_000001_create_note;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Chronological order. `sea-orm-cli migrate generate <name>` adds
        // new migrations here automatically.
        vec![Box::new(m20261106_000001_create_note::Migration)]
    }
}
