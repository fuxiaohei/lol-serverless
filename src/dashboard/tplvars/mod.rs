use crate::version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod breadcrumb;
pub use breadcrumb::{nav_active, BreadCrumb, BreadCrumbKey};

mod user;
pub use user::{AuthUser, Token};

mod project;
pub use project::Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    pub title: String,
    pub nav_active: HashMap<String, String>,
    pub breadcrumb: Vec<BreadCrumb>,
    pub user: Option<AuthUser>,
    pub version: String,
}

impl Page {
    /// new creates a new page var
    pub fn new(title: &str, breadcrumb: BreadCrumbKey, user: Option<AuthUser>) -> Self {
        Page {
            title: title.to_string(),
            nav_active: nav_active(&breadcrumb),
            breadcrumb: BreadCrumb::new(&breadcrumb),
            user,
            version: version::short(),
        }
    }
}

/// Vars is the template vars for whole site
#[derive(Serialize, Deserialize, Debug)]
pub struct Vars<T: Serialize> {
    pub page: Page,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Empty {}

impl<T: Serialize> Vars<T> {
    /// new creates a new vars
    pub fn new(page: Page, data: T) -> Self {
        Vars { page, data }
    }
}
