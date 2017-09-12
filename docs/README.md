# Overview

This describes the glacio HTTP API.
For documentation of the glacio Rust API, use `cargo doc --all` in the repository root, or `cargo doc -p glacio --open` to open the **glacio** crate's documentation.

## Access

All API access is over HTTP via `http://api.glac.io`.
All data is received as JSON, with the exception of redirect URLs.
All datetimes are returned as ISO 8601, e.g. `2017-09-12T16:12:42-06:00`.

## Pagination

Requests that return multiple items return 30 items by default.
To change that number, use the `?per_page` parameter.
To select the page, use the `?page` parameter.
Note that page numbering is 1-indexed.

```
curl 'http://api.glac.io/cameras?per_page=2&page=2'
```

# List all cameras

List all cameras configured in the system.

```
GET /cameras
```

## Response

```json
[
  {
    "name": "ATLAS_CAM",
    "description": "A really swell camera.",
    "url": "http://api.glac.io/cameras/ATLAS_CAM",
    "images_url": "http://api.glac.io/cameras/ATLAS_CAM/images",
    "interval": 3
  }
]
```

# Get a camera

```
GET /cameras/:name
```

## Response

Very close to the same as the summary information, but includes information on the latest image.

```json
[
  {
    "name": "ATLAS_CAM",
    "description": "A really swell camera.",
    "url": "http://api.glac.io/cameras/ATLAS_CAM",
    "images_url": "http://api.glac.io/cameras/ATLAS_CAM/images",
    "interval": 3,
    "latest_image": {
      "datetime": "2017-09-12T21:25:00+00:00",
      "url": "http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170912_212500.jpg"
    }
  }
]
```

# List a camera's images

```
GET /cameras/:name/images
```

## Parameters

Results are paginated, so use `?page` and `?per_page`.
Images are returned most recent first (descending datetime order).

## Response

```json
[
  {
    "datetime": "2017-09-12T21:25:00+00:00",
    "url": "http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170912_212500.jpg"
  },
  {
    "datetime": "2017-09-12T18:25:00+00:00",
    "url": "http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170912_182500.jpg"
  }
]
```

# Redirect to a camera's latest image

```
GET /cameras/:name/images/latest/redirect
```

## Response

```
Status: 302 Found
Location: http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170912_212500.jpg
```

# Get the ATLAS system's status

```
GET /atlas/status
```

## Response

```json
{
  "last_heartbeat_received": "2017-09-12T22:02:21+00:00",
  "batteries": [
    {
      "id": 1,
      "state_of_charge": 93.317
    },
    {
      "id": 2,
      "state_of_charge": 93.043
    }
  ],
  "efoys": [
    {
      "id": 1,
      "state": "auto off",
      "active_cartridge": "1.1",
      "active_cartridge_consumed": 7.597,
      "voltage": 26.55,
      "current": -0.03,
      "cartridges": [
        {
          "name": "1.1",
          "fuel_percentage": 5.0374985
        },
        {
          "name": "1.2",
          "fuel_percentage": 100
        },
        {
          "name": "2.1",
          "fuel_percentage": 100
        },
        {
          "name": "2.2",
          "fuel_percentage": 100
        }
      ]
    },
    {
      "id": 2,
      "state": "auto off",
      "active_cartridge": "1.2",
      "active_cartridge_consumed": 0.281,
      "voltage": 26.55,
      "current": -0.02,
      "cartridges": [
        {
          "name": "1.1",
          "fuel_percentage": 0
        },
        {
          "name": "1.2",
          "fuel_percentage": 96.487495
        },
        {
          "name": "2.1",
          "fuel_percentage": 100
        },
        {
          "name": "2.2",
          "fuel_percentage": 100
        }
      ]
    }
  ],
  "are_riegl_systems_on": true,
  "timeseries": {
    "datetimes": [
      "2017-07-17T16:03:25+00:00"
    ],
    "states_of_charge": {
      "1": [
        34.677
      ],
      "2": [
        42.321
      ]
    },
    "efoy_current": {
      "1": [
        0.01
      ],
      "2": [
        0.02
      ]
    },
    "efoy_voltage": {
      "1": [
        26.2
      ],
      "2": [
        26.3
      ]
    },
    "efoy_fuel_percentage": {
      "1": [
        98.2
      ],
      "2": [
        97.2
      ]
    }
  }
}
```
