/// A remote camera that transmits pictures back to home.
#[derive(Debug, Serialize)]
pub struct Camera {
    name: String,
}

impl Camera {
    /// Creates a new camera with the given name.
    pub fn new(name: &str) -> Camera {
        Camera { name: name.to_string() }
    }
}
