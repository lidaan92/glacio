/// A remote camera that transmits pictures back to home.
#[derive(Clone, Debug, Serialize)]
pub struct Camera {
    name: String,
}

impl Camera {
    /// Creates a new camera with the given name.
    pub fn new(name: &str) -> Camera {
        Camera { name: name.to_string() }
    }

    /// Returns this camera's name.
    pub fn name(&self) -> &str {
        &self.name
    }
}
