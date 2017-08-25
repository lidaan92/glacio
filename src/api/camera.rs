use iron::Request;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub path: String,
}

#[derive(Serialize, Debug)]
pub struct Summary {
    name: String,
    description: String,
    url: String,
    images_url: String,
}

#[derive(Serialize, Debug)]
pub struct Detail;

#[derive(Serialize, Debug)]
pub struct ImageSummary;

impl Config {
    pub fn summary(&self, request: &Request) -> Summary {
        let url = url_for!(request, "camera", "name" => self.name.clone());
        let images_url = url_for!(request, "camera_images", "name" => self.name.clone());
        Summary {
            name: self.name.clone(),
            description: self.description.clone(),
            url: url.as_ref().to_string(),
            images_url: images_url.as_ref().to_string(),
        }
    }

    pub fn detail(&self, request: &Request) -> Detail {
        unimplemented!()
    }

    pub fn images(&self, request: &Request) -> Vec<ImageSummary> {
        unimplemented!()
    }
}
