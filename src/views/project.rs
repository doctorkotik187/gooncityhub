use loco_rs::prelude::*;

use crate::models::_entities::projects;

/// Render a list view of `projects`.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn list(v: &impl ViewRenderer, items: &Vec<projects::Model>) -> Result<Response> {
    format::render().view(v, "project/list.html", data!({"items": items}))
}

/// Render a single `project` view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &projects::Model) -> Result<Response> {
    format::render().view(v, "project/show.html", data!({"item": item}))
}

/// Render a `project` create form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn create(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "project/create.html", data!({}))
}

/// Render a `project` edit form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn edit(v: &impl ViewRenderer, item: &projects::Model) -> Result<Response> {
    format::render().view(v, "project/edit.html", data!({"item": item}))
}
