use land_helpers::version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod user;
pub use user::{Token, User};

mod projects;
pub use projects::Project;

/// Page is the template page vars, for every page
#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    pub title: String,
    pub nav_active: HashMap<String, String>,
    pub breadcrumb: Vec<BreadCrumb>,
    pub user: Option<User>,
    pub version: String,
    pub is_in_admin: bool,
}

impl Page {
    /// new creates a new page var
    pub fn new(title: &str, breadcrumb: BreadCrumbKey, user: Option<User>) -> Self {
        Page {
            title: title.to_string(),
            nav_active: nav_active(&breadcrumb),
            breadcrumb: BreadCrumb::new(&breadcrumb),
            user,
            version: version::short(),
            is_in_admin: false,
        }
    }
}

/// BreadCrumb enum
#[derive(strum::Display, Clone, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum BreadCrumbKey {
    Dashboard,
    Projects,
    Settings,
    NotFound,
    None,
}

/// BreadCrumb is breadcrumb value item
#[derive(Debug, Serialize, Deserialize)]
pub struct BreadCrumb {
    pub title: String,
    pub link: Option<String>,
}

impl BreadCrumb {
    /// single creates a single breadcrumb item
    pub fn single(name: &str) -> Self {
        BreadCrumb {
            title: name.to_string(),
            link: None,
        }
    }
    /// new creates breadcrumb items
    pub fn new(key: &BreadCrumbKey) -> Vec<BreadCrumb> {
        match key {
            BreadCrumbKey::Dashboard => vec![BreadCrumb::single("Dashboard")],
            BreadCrumbKey::Settings => vec![BreadCrumb::single("Settings")],
            BreadCrumbKey::Projects => {
                vec![BreadCrumb::single("Projects")]
            }
            BreadCrumbKey::None | BreadCrumbKey::NotFound => vec![],
        }
    }
}

/// nav_active sets active navbar items
pub fn nav_active(breadcrumb: &BreadCrumbKey) -> HashMap<String, String> {
    let mut nav_active = HashMap::new();
    // println!("breadcrumb: {:?}", breadcrumb.to_string());
    nav_active.insert(breadcrumb.to_string(), "active".to_string());
    nav_active
}

/// Vars is the template vars for whole site
#[derive(Serialize, Deserialize, Debug)]
pub struct Vars<T: Serialize> {
    pub page: Page,
    pub data: T,
}

/// Empty is an empty struct with serde support
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Empty {}

impl Empty {
    /// new_vars creates a new vars with empty data
    pub fn new_vars(title: &str, breadcrumb: BreadCrumbKey, user: Option<User>) -> Vars<Empty> {
        Vars {
            page: Page::new(title, breadcrumb, user),
            data: Empty::default(),
        }
    }
}

impl<T: Serialize> Vars<T> {
    /// new creates a new vars
    pub fn new(page: Page, data: T) -> Self {
        Vars { page, data }
    }
}
