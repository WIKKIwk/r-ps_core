# Printer Driver Contract

This document records the target printer architecture for later `rp-scale` phases.

The goal is to add printers without changing core print flow for every brand.

## Source Of Truth

Current production behavior must still be studied from:

- `gscale-platform`
- `accord_mobile`
- `accord_mobile_server_rs`

Do not assume printer behavior from README files.

Before implementing a printer driver:

1. Read the production code path for that printer.
2. Record supported modes and blocked modes.
3. Preserve command protocol and observable mobile behavior.
4. Add tests for capability validation and generated printer commands.

## Design Rule

Core owns the print intent.

Driver owns printer-specific execution.

Core must not contain scattered logic like:

```text
if printer == "godex" { block_epc() }
if printer == "zebra" { allow_rfid() }
```

Instead, core asks the active driver what it supports.

## Capability Model

Each printer driver must expose typed capabilities.

Required fields:

- Stable driver id.
- Human-readable printer name.
- Thermal label support.
- RFID EPC write support.
- Barcode support.
- QR support.
- Verify-after-print support.
- Required print job fields.
- Explicit unsupported modes.

Example startup manifest shape:

```json
{
  "id": "godex",
  "name": "GoDEX",
  "thermal_label": true,
  "rfid_epc_write": false,
  "barcode": true,
  "qr": true,
  "verify_after_print": false,
  "required_fields": ["item_code", "item_name", "weight"],
  "unsupported_modes": ["rfid_epc_write"]
}
```

JSON is allowed here only as small startup config or public API shape.

After startup, runtime code must use typed Rust structs.

## Print Mode Policy

Default policy: reject unsupported modes.

Examples:

- GoDEX with thermal label request: allowed if `thermal_label=true`.
- GoDEX with RFID EPC write request: rejected if `rfid_epc_write=false`.
- Zebra RFID request: allowed only if the active Zebra driver declares `rfid_epc_write=true`.

Silent fallback is not allowed by default because it can hide production mistakes.

Fallback behavior requires explicit approval and a test.

## Mobile Discovery API

`rp-scale` should expose printer capabilities to the mobile app.

Target response shape:

```json
{
  "active_printer": {
    "id": "godex",
    "name": "GoDEX",
    "capabilities": {
      "thermal_label": true,
      "rfid_epc_write": false,
      "barcode": true,
      "qr": true,
      "verify_after_print": false
    },
    "required_fields": ["item_code", "item_name", "weight"],
    "unsupported_modes": ["rfid_epc_write"]
  }
}
```

The mobile app can use this response to disable impossible options.

Core still validates every request. UI validation is not enough.

## Driver Boundary

A printer driver should receive a prepared print job.

It should not own:

- ERP item lookup.
- Warehouse lookup.
- User permissions.
- Batch business decisions.
- Accord server authentication.

It should own:

- Device connection.
- Printer command generation.
- RFID command generation when supported.
- Printer status parsing.
- Printer-specific error mapping.

## Rust Shape

Expected later module shape:

```text
src/
  printer/
    mod.rs
    capabilities.rs
    driver.rs
    job.rs
    registry.rs
```

Suggested concepts:

- `PrinterCapabilities`
- `PrintJob`
- `PrintMode`
- `PrinterDriver`
- `PrinterRegistry`
- `PrinterError`

No file may exceed 500 lines.

## Test Rules

Every driver must test:

- Capability export.
- Allowed print modes.
- Rejected unsupported modes.
- Required field validation.
- Generated command bytes/text for known fixtures.
- Error mapping for known printer responses.

Contract tests should compare with production Go behavior where available.

## Open Decisions

These decisions need approval before implementation:

- Exact mobile API endpoint name.
- Exact fallback behavior when a printer cannot perform a requested mode.
- Exact manifest file format and location.
- Whether third-party printer drivers are compiled in or loaded from config.
