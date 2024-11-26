use serde::Serialize;

#[derive(Serialize)]
pub struct PaginationItem {
    pub link: String,
    pub is_current: bool,
    pub page: u64,
}

#[derive(Serialize)]
pub struct Pagination {
    pub current_page: u64,
    pub total_pages: u64,
    pub total_count: u64,
    pub items: Vec<PaginationItem>,
}

impl Pagination {
    pub fn new(
        current: u64,
        size: u64,
        total_pages: u64,
        total_count: u64,
        link: &str,
    ) -> Pagination {
        let mut items = vec![];
        for i in 1..=total_pages {
            let page_link = if link.contains('?') {
                format!("{}&page={}&size={}", link, i, size)
            } else {
                format!("{}?page={}&size={}", link, i, size)
            };
            items.push(PaginationItem {
                link: page_link,
                is_current: i == current,
                page: i,
            });
        }
        Pagination {
            current_page: current,
            total_pages,
            total_count,
            items,
        }
    }
}