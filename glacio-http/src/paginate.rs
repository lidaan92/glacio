//! Pagination support for Iron requests.
//!
//! Contains a trait, `Paginate`, that is implemented for `Iterator`, that can be used to subset an
//! iterator based on Iron request parameters.

use Result;
use iron::{Plugin, Request};
use params::{Params, Value};
use std::iter::{Skip, Take};

/// The default page, if one is not specified in the request.
///
/// We 1-index pages because Github does.
pub const DEFAULT_PAGE: usize = 1;

/// The default number of items per page.
pub const DEFAULT_PER_PAGE: usize = 30;

/// The maximum number of items that can be returned per page.
///
/// If more than this amount is requested, the number of items returned is clamped to 100.
pub const MAX_PER_PAGE: usize = 100;

/// Use an Iron request, specifically its parameters, to paginate over an iterator.
///
/// The parameters used:
///
/// - `per_page`: How many items to return per page. Defaults to `DEFAULT_PER_PAGE`.
/// - `page`: The (1-indexed) page to return. This is 1-indexed because Github's is, and I'm just
/// copying them.
///
/// An example paginated request might look like this:
///
/// ```bash
/// curl http://localhost:3000/cameras/ATLAS_CAM/images?page=2&per_page=10
/// ```
pub trait Paginate<I> {
    /// Creates a pagination iterator from a request.
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
