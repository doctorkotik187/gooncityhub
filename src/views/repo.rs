use loco_rs::prelude::*;

use crate::models::_entities::repos;

/// Render a list view of `repos`.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn list(v: &impl ViewRenderer, items: &Vec<repos::Model>) -> Result<Response> {
    format::render().view(v, "repo/list.html", data!({"items": items}))
}

/// Render a single `repo` view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &repos::Model) -> Result<Response> {
    format::render().view(v, "repo/show.html", data!({"item": item}))
}

/// Render a `repo` create form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn create(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "repo/create.html", data!({}))
}

/// Render a `repo` edit form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn edit(v: &impl ViewRenderer, item: &repos::Model) -> Result<Response> {
    format::render().view(v, "repo/edit.html", data!({"item": item}))
}
