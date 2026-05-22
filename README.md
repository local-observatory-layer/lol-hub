# LOL HUB

LOL HUB receives samples from remote devices, typically sensors running
on ESP32 boards, and stores them in memory.

Devices can connect and receive an SSE stream -- initial state, and a
live stream of samples.

Each device also registers a descriptor, which includes some information
about the samples it sends, optionally including names, units, aggregation
strategies, TTLs, and rendering information.

## LOL HUD

LOL HUD integrates with LOL HUB, visualizing all samples according to the
details in each device's descriptor.

## HTTP API

### `POST /sample`

Ingest a sample from a device.

#### Request body

JSON object containing the sample payload. Each sample should include:

- `device_id` — stable device UUID
- `boot_id` — UUID generated once per boot
- `seq` — monotonically increasing `u64` sequence number for this boot
- sample-specific fields

Example:

```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "boot_id": "c1c8f8df-4c6d-4c71-8f1e-bf4cf8d48d9b",
  "seq": 42,
  "kind": "temperature",
  "value": 23.4
}
```

#### Responses

- `204 No Content` — sample was ingested successfully
- `202 Accepted` — sample was ingested successfully, but the server does not have a descriptor for this device yet; the device should submit its descriptor

No response body is returned.

---

### `POST /descriptor`

Register or update a device descriptor.

Devices should call this endpoint when `/sample` returns `202 Accepted`.

#### Request body

JSON object containing device metadata used by the UI and renderer.

Typical fields include:

- `device_id` — stable device UUID
- `name` — human-readable device name
- rendering metadata such as size, layout, or capabilities

Example:

```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "West Hall Sign",
  "render": {
    "width": 64,
    "height": 32,
    "color_mode": "rgb"
  }
}
```

#### Responses

- `204 No Content` — descriptor accepted