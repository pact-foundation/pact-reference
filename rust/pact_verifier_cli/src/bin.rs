use std::time::Duration;
use tokio::time::sleep;

use pact_verifier_cli::{handle_cli, init_windows};

fn main() {
  init_windows();

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("Could not start a Tokio runtime for running async tasks");

  let result = runtime.block_on(async {
    let result = handle_cli().await;

    // Add a small delay to let asynchronous tasks to complete
    sleep(Duration::from_millis(200)).await;

    result
  });

  runtime.shutdown_timeout(Duration::from_millis(500));

  if let Err(err) = result {
    std::process::exit(err);
  }
}