#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate camera;
extern crate chrono;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use chrono::{DateTime, Utc};
use rocket::State;
use rocket_contrib::Json;
use std::collections::HashMap;

type Cameras<'a> = State<'a, HashMap<String, camera::Camera>>;

#[derive(Debug, Serialize)]
struct Camera {
    name: String,
    url: String,
    images_url: String,
}

#[derive(Debug, Serialize)]
struct Image {
    datetime: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Default, FromForm)]
struct Pagination {
    page: Option<usize>,
    per_page: Option<usize>,
}

impl<'a> From<&'a camera::Camera> for Camera {
    fn from(camera: &'a camera::Camera) -> Camera {
        let name = camera.name().to_string();
        let url = format!("/cameras/{}", name);
        let images_url = format!("{}/images", url);
        Camera {
            images_url: images_url,
            name: name,
            url: url,
        }
    }
}

impl From<camera::Image> for Image {
    fn from(image: camera::Image) -> Image {
        Image {
            datetime: image.datetime(),
        }
    }
}

impl Pagination {
    // TODO can we just implement a `.paginate()` method on an iterator?
    fn skip(&self) -> usize {
        (self.page() - 1) * self.per_page()
    }

    fn take(&self) -> usize {
        self.per_page()
    }

    fn page(&self) -> usize {
        // TODO protect against pages less than one
        self.page.unwrap_or(1)
    }

    fn per_page(&self) -> usize {
        // TODO protect against negative
        self.per_page.unwrap_or(30)
    }
}

#[get("/cameras")]
fn cameras(cameras: Cameras) -> Json<Vec<Camera>> {
    Json(cameras.values().map(|c| c.into()).collect())
}

#[get("/cameras/<name>")]
fn camera(name: String, cameras: Cameras) -> Option<Json<Camera>> {
    cameras.get(&name).map(|c| Json(c.into()))
}

#[get("/cameras/<name>/images")]
fn camera_images(name: String, cameras: Cameras) -> Option<Json<Vec<Image>>> {
    let pagination = Pagination::default();
    camera_images_paginated(name, pagination, cameras)
}

#[get("/cameras/<name>/images?<pagination>")]
fn camera_images_paginated(
    name: String,
    pagination: Pagination,
    cameras: Cameras,
) -> Option<Json<Vec<Image>>> {
    cameras
        .get(&name)
        .and_then(|camera| camera.images().ok())
        .map(|images| {
            Json(
                images
                    .into_iter()
                    .rev()
                    .skip(pagination.skip())
                    .take(pagination.take())
                    .map(|i| i.into())
                    .collect(),
            )
        })
}

fn main() {
    let cameras: HashMap<String, camera::Camera> =
        camera::from_path("/Users/rdcrlpjg/iridiumcam/StarDot", "StarDot")
            .unwrap()
            .into_iter()
            .map(|c| (c.name().to_string(), c))
            .collect();
    rocket::ignite()
        .manage(cameras)
        .mount(
            "/",
            routes![cameras, camera, camera_images, camera_images_paginated],
        )
        .launch();
}
