#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::response::Redirect;
use axum_extra::extract::Form;
use loco_rs::prelude::*;
use sea_orm::{sea_query::Order, QueryOrder};
use serde::{Deserialize, Serialize};

use crate::{
    models::_entities::repos::{ActiveModel, Column, Entity, Model},
    views,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub project_id: i32,
    pub name: String,
    pub owner: String,
    pub stars: i32,
    pub forks: i32,
    pub issues: i32,
    pub prs: i32,
    pub contributors: i32,
    pub commits_last_30d: i32,
    pub watchers: i32,
    pub license: Option<String>,
    pub last_fetch: DateTime,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.project_id = Set(self.project_id);
        item.name = Set(self.name.clone());
        item.owner = Set(self.owner.clone());
        item.stars = Set(self.stars);
        item.forks = Set(self.forks);
        item.issues = Set(self.issues);
        item.prs = Set(self.prs);
        item.contributors = Set(self.contributors);
        item.commits_last_30d = Set(self.commits_last_30d);
        item.watchers = Set(self.watchers);
        item.license = Set(self.license.clone());
        item.last_fetch = Set(self.last_fetch);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Entity::find()
        .order_by(Column::Id, Order::Desc)
        .all(&ctx.db)
        .await?;
    views::repo::list(&v, &item)
}

#[debug_handler]
pub async fn new(
    ViewEngine(v): ViewEngine<TeraView>,
    State(_ctx): State<AppContext>,
) -> Result<Response> {
    views::repo::create(&v)
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    item.update(&ctx.db).await?;
    Ok(Redirect::to("../repos"))
}

#[debug_handler]
pub async fn edit(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    views::repo::edit(&v, &item)
}

#[debug_handler]
pub async fn show(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    views::repo::show(&v, &item)
}

#[debug_handler]
pub async fn add(State(ctx): State<AppContext>, Form(params): Form<Params>) -> Result<Redirect> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    item.insert(&ctx.db).await?;
    Ok(Redirect::to("repos"))
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("repos/")
        .add("/", get(list))
        .add("/", post(add))
        .add("new", get(new))
        .add("{id}", get(show))
        .add("{id}/edit", get(edit))
        .add("{id}", delete(remove))
        .add("{id}", post(update))
}
