use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BreadCrumb enum
#[derive(strum::Display, Clone, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum BreadCrumbKey {
    Dashboard,
    Settings,
    Administration,
    Projects,
    ProjectNew,
    ProjectSingle,
    ProjectSettings,
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
    /// new creates breadcrumb items
    pub fn new(key: &BreadCrumbKey) -> Vec<BreadCrumb> {
        match key {
            BreadCrumbKey::Dashboard => vec![BreadCrumb {
                title: "Dashboard".to_string(),
                link: None,
            }],
            BreadCrumbKey::Settings => vec![BreadCrumb {
                title: "Settings".to_string(),
                link: None,
            }],
            BreadCrumbKey::Administration => vec![BreadCrumb {
                title: "Administration".to_string(),
                link: None,
            }],
            BreadCrumbKey::Projects | BreadCrumbKey::ProjectNew => vec![BreadCrumb {
                title: "Projects".to_string(),
                link: None,
            }],
            BreadCrumbKey::ProjectSingle | BreadCrumbKey::ProjectSettings => vec![BreadCrumb {
                title: "Projects".to_string(),
                link: Some("/projects".to_string()),
            }],
            BreadCrumbKey::None | BreadCrumbKey::NotFound => vec![],
        }
    }
}

/// nav_active sets active navbar items
pub fn nav_active(breadcrumb: &BreadCrumbKey) -> HashMap<String, String> {
    let mut nav_active = HashMap::new();
    nav_active.insert(breadcrumb.to_string(), "active".to_string());
    nav_active
}