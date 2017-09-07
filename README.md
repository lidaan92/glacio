# glacio

Rust crates for managing and disseminating our glacier research data.
These data include:

- Status information from the [ATLAS system](http://atlas.lidar.io), a remote LiDAR scanner located at the Helheim Glacier in southeast Greenland.
- Remote camera images from cameras located all around the world, in particular in Alaska and Greenland.

These data are transmitted from remote sites to host servers via satellite connections.

The **glacio** crate provides a Rust API for accessing and inspecting these remote data, once they're on our host systems.
The **glacio-http** crate uses the Rust API to create an HTTP API (using [iron](http://ironframework.io/)).
Finally, **glacio-bin** provides an executable that can be used to start the HTTP API or query the entire glacier data system.
