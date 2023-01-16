<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/lvkdotsh/redeez/raw/master/public/redeez_white.png" />
    <img alt="redeez" src="https://github.com/v3xlabs/redeez/raw/master/public/redeez_black.png" width="400px" />
  </picture>
</p>

<div align="center">
  <a href="https://crates.io/crates/redeez">
    <img src="https://img.shields.io/crates/v/redeez.svg" alt="crates.io" />
  </a>
  <a href="https://crates.io/crates/redeez">
    <img src="https://img.shields.io/crates/d/redeez.svg" alt="download count badge" />
  </a>
  <a href="https://docs.rs/redeez">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg" alt="docs.rs" />
  </a>
</div>

A simplified general-purpose queueing system for Rust apps.

## Example

```rust
// Create a new Redeez object, and define your queues
let mut queues = Redeez::new(redis)
        .queue("avatars:resize", |job| -> Result<()> {
            // -- snip --

            Ok(())
        })
        .queue("images:resize", resize_images);

// Start queue workers in the background
queues.listen();

// Dispatch some jobs into the queue
queues.dispatch("images:resize", json!(["image1.jpg", "image2.jpg"]));
queues.dispatch("avatars:resize", json!(["avatar1.jpg", "avatar2.jpg"]));

// When shutting your program down, stop listening for jobs
queues.shutdown();
```

## Acknowledgements

This project is very heavily _inspired_ by [v3xlabs' `redeez`](https://github.com/v3xlabs/redeez) npm package. Extra thanks to [@lucemans](https://github.com/lucemans) for helping me understand Redis.

## License

Redeez is released under the MIT License. See the [LICENSE](LICENSE) file for details.
