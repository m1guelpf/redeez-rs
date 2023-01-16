use anyhow::Result;
use redeez::{Job, Redeez};
use serde_json::json;

#[tokio::main]
async fn main() {
    let redis = redis::Client::open("redis://127.0.0.1:6379").unwrap();

    let mut queues = Redeez::new(redis)
        .queue("avatars:resize", resize_avatars)
        .queue("images:resize", resize_images);
    queues.listen();

    queues.dispatch("images:resize", json!(["image1.jpg", "image2.jpg"]));
    queues.dispatch("avatars:resize", json!(["avatar1.jpg", "avatar2.jpg"]));

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    queues.shutdown();
}

fn resize_avatars(job: Job) -> Result<()> {
    let images = job
        .payload
        .as_array()
        .expect("Failed to get avatars from payload.");

    for image in images {
        println!("Resizing avatar {image}...");

        // -- snip --
    }

    Ok(())
}

fn resize_images(job: Job) -> Result<()> {
    let images = job
        .payload
        .as_array()
        .expect("Failed to get images from payload.");

    for image in images {
        println!("Resizing image {image}...");

        // -- snip --
    }

    Ok(())
}
