use Result;
use iron::{Plugin, Request};
use params::{Params, Value};
use std::iter::{Skip, Take};

const DEFAULT_PAGE: usize = 1;
const DEFAULT_PER_PAGE: usize = 30;
const MAX_PER_PAGE: usize = 100;

pub trait Paginate<I> {
    fn paginate(self, request: &mut Request) -> Result<Take<Skip<I>>>;
}

struct Pagination {
    page: usize,
    per_page: usize,
}

impl<I: Iterator> Paginate<I> for I {
    fn paginate(self, request: &mut Request) -> Result<Take<Skip<I>>> {
        let pagination = Pagination::new(request)?;
        Ok(self.skip(pagination.skip()).take(pagination.take()))
    }
}

impl Pagination {
    pub fn new(request: &mut Request) -> Result<Pagination> {
        let map = request.get::<Params>().unwrap();
        let mut page = match map.find(&["page"]) {
            Some(&Value::U64(page)) => page as usize,
            Some(&Value::String(ref page)) => page.parse::<usize>()?,
            _ => DEFAULT_PAGE,
        };
        if page == 0 {
            page = 1;
        }
        let mut per_page = match map.find(&["per_page"]) {
            Some(&Value::U64(per_page)) => per_page as usize,
            Some(&Value::String(ref per_page)) => per_page.parse::<usize>()?,
            _ => DEFAULT_PER_PAGE,
        };
        if per_page >= MAX_PER_PAGE {
            per_page = MAX_PER_PAGE;
        } else if per_page == 0 {
            per_page = DEFAULT_PER_PAGE;
        }
        Ok(Pagination {
               page: page,
               per_page: per_page,
           })
    }

    pub fn skip(&self) -> usize {
        self.per_page * (self.page - 1)
    }

    pub fn take(&self) -> usize {
        self.per_page
    }
}
