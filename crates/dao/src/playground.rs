use crate::{
    models::playground,
    projects::{self, Language},
    DB,
};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder,
};
use short_uuid::ShortUuid;

/// Status is the status of a playground
pub type Status = projects::Status;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

/// create creates a new playground
pub async fn create(
    owner_id: i32,
    project_id: i32,
    language: Language,
    source: String,
    visible: bool,
) -> Result<playground::Model> {
    let uuid = ShortUuid::generate().to_string();
    let now = chrono::Utc::now();
    let p = playground::Model {
        id: 0,
        owner_id,
        project_id,
        uuid,
        language: language.to_string(),
        source,
        version: String::new(),
        status: Status::Active.to_string(),
        visiblity: if visible {
            Visibility::Public.to_string()
        } else {
            Visibility::Private.to_string()
        },
        created_at: now,
        deleted_at: None,
    };
    let mut active_model = p.into_active_model();
    active_model.id = Default::default();
    let p = active_model.insert(DB.get().unwrap()).await?;

    // set old playground to disabled, only one playground can be active at a time
    set_old_disabled(project_id, p.id).await?;

    Ok(p)
}

/// get_by_project gets a playground by project
pub async fn get_by_project(project_id: i32) -> Result<Option<playground::Model>> {
    let db = DB.get().unwrap();
    let p = playground::Entity::find()
        .filter(playground::Column::ProjectId.eq(project_id))
        .filter(playground::Column::Status.eq(Status::Active.to_string()))
        .order_by_desc(playground::Column::Id)
        .one(db)
        .await?;
    Ok(p)
}

async fn set_old_disabled(project_id: i32, current_playground_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    playground::Entity::update_many()
        .filter(playground::Column::ProjectId.eq(project_id))
        .filter(playground::Column::Id.ne(current_playground_id))
        .filter(playground::Column::Status.eq(Status::Active.to_string()))
        .col_expr(
            playground::Column::Status,
            Expr::value(Status::Disabled.to_string()),
        )
        .exec(db)
        .await?;
    Ok(())
}
