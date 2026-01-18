# Research: SNS Posting for Seismic Alerts

## Decisions

### Decision: HTTP Client for Webhooks (Discord, Google Chat)
**Rationale**: `reqwest` is the standard async HTTP client in Rust and supports multipart uploads required for Discord images.
**Choice**: `reqwest` with `json` and `multipart` features.

### Decision: AWS SDK for S3 and SNS
**Rationale**: Official `aws-sdk-rust` provides high-quality, async-first clients for S3 and SNS.
**Choice**: `aws-sdk-s3` and `aws-sdk-sns` crates.

### Decision: Async Execution Pattern
**Rationale**: SNS posting involves network latency. To avoid blocking the data pipeline, notifications should be spawned as independent Tokio tasks.
**Choice**: Use `tokio::spawn` within `pipeline.rs` to hand off events to an `SNSManager`.

### Decision: S3 ACL for Public Images
**Rationale**: `rsudp` implementation explicitly uses `public-read` ACL to ensure external platforms can fetch the image.
**Choice**: Use `ObjectCannedAcl::PublicRead` in `aws-sdk-s3`.

## Research Tasks

- [x] Check if `reqwest` is already in project? (No, need to add)
- [x] Check if `aws-sdk-rust` is already in project? (No, need to add)
- [x] Verify Discord multipart upload syntax for `reqwest`.
- [x] Verify S3 public URL format (Standard format: `https://{bucket}.s3.{region}.amazonaws.com/{key}`).
