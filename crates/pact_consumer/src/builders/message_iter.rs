use std::collections::VecDeque;
use std::env;
use std::panic::RefUnwindSafe;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use itertools::Itertools;

use maplit::hashmap;
use pact_models::generators::GeneratorTestMode;
use pact_models::message::Message;
use pact_models::pact::write_pact;
use pact_models::prelude::{MessagePact, Pact};
use pact_models::prelude::v4::V4Pact;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::sync_message::SynchronousMessage;
use pact_models::v4::V4InteractionType;
use tokio::runtime::Handle;
use tracing::{debug, error, info, warn};

use pact_matching::generators::{apply_generators_to_sync_message, generate_message};

/// Iterator over the messages build with the PactBuilder
pub struct MessageIterator<MT> {
  pact: Box<dyn Pact + Send + Sync + RefUnwindSafe>,
  message_list: VecDeque<MT>,
  // Output directory to write pact files to when done
  output_dir: Option<PathBuf>,
}

/// Construct a new iterator over the asynchronous messages in the pact
pub fn asynchronous_messages_iter(pact: V4Pact, output_dir: &Option<PathBuf>) -> MessageIterator<AsynchronousMessage> {
  MessageIterator {
    pact: pact.boxed(),
    message_list: pact.filter_interactions(V4InteractionType::Asynchronous_Messages)
      .iter()
      .map(|item| item.as_v4_async_message().unwrap())
      .collect(),
    output_dir: output_dir.clone()
  }
}

/// Construct a new iterator over the synchronous messages in the pact
pub fn synchronous_messages_iter(pact: V4Pact, output_dir: &Option<PathBuf>) -> MessageIterator<SynchronousMessage> {
  let original_messages = pact.filter_interactions(V4InteractionType::Synchronous_Messages)
    .iter()
    .map(|item| item.as_v4_sync_message().unwrap())
    .collect_vec();

  let (sx, rx) = channel();
  match Handle::try_current() {
    Ok(handle) => {
      let messages_to_generate = original_messages.clone();
      handle.spawn(async move {
        let mut messages = vec![];
        for message in messages_to_generate {
          let (req, res) = apply_generators_to_sync_message(&message, &GeneratorTestMode::Consumer, &hashmap! {}, &vec![], &hashmap! {}).await;
          messages.push(SynchronousMessage {
            request: req,
            response: res,
            ..      message
          });
        }
        let _ = sx.send(messages);
      })
    },
    Err(err) => {
      warn!("Could not access the Tokio runtime, will start a new one: {}", err);
      let messages_to_generate = original_messages.clone();
      tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not start a Tokio runtime for running async tasks")
        .spawn(async move {
          let mut messages = vec![];
          for message in messages_to_generate {
            let (req, res) = apply_generators_to_sync_message(&message, &GeneratorTestMode::Consumer, &hashmap! {}, &vec![], &hashmap! {}).await;
            messages.push(SynchronousMessage {
              request: req,
              response: res,
              ..       message
            });
          }
          let _ = sx.send(messages);
        })
    }
  };

  let message_list = match rx.recv() {
    Ok(messages) => messages,
    Err(err) => {
      error!("Was not able to apply generators to the messages: {}", err);
      original_messages
    }
  };

  MessageIterator {
    pact: pact.boxed(),
    message_list: message_list.iter().cloned().collect(),
    output_dir: output_dir.clone()
  }
}

/// Construct a new iterator over the messages in the Message Pact
// TODO: This needs a mechanism to pass in the test context and plugin data
pub fn messages_iter(pact: MessagePact, output_dir: &Option<PathBuf>) -> MessageIterator<Message> {
  let original_messages = pact.messages.clone();
  let (sx, rx) = channel();
  match Handle::try_current() {
    Ok(handle) => handle.spawn(async move {
      let mut messages = VecDeque::new();
      for message in original_messages {
        messages.push_back(generate_message(&message, &GeneratorTestMode::Consumer, &hashmap!{}, &vec![], &hashmap!{}).await);
      }
      let _ = sx.send(messages);
    }),
    Err(err) => {
      warn!("Could not access the Tokio runtime, will start a new one: {}", err);
      tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not start a Tokio runtime for running async tasks")
        .spawn(async move {
          let mut messages = VecDeque::new();
          for message in original_messages {
            messages.push_back(generate_message(&message, &GeneratorTestMode::Consumer, &hashmap!{}, &vec![], &hashmap!{}).await);
          }
          let _ = sx.send(messages);
        })
    }
  };

  MessageIterator {
    pact: pact.boxed(),
    message_list: rx.recv().expect("Did not receive any messages"),
    output_dir: output_dir.clone()
  }
}

impl <MT> Iterator for MessageIterator<MT> {
  type Item = MT;

  fn next(&mut self) -> Option<Self::Item> {
    self.message_list.pop_front()
  }
}

impl <MT> Drop for MessageIterator<MT> {
  fn drop(&mut self) {
    if !::std::thread::panicking() {

      // Write out the Pact file
      let output_dir = self.output_dir.as_ref().map(|dir| dir.to_string_lossy().to_string())
        .unwrap_or_else(|| {
          let val = env::var("PACT_OUTPUT_DIR");
          debug!("env:PACT_OUTPUT_DIR = {:?}", val);
          val.unwrap_or_else(|_| "target/pacts".to_owned())
        });
      let overwrite = env::var("PACT_OVERWRITE");
      debug!("env:PACT_OVERWRITE = {:?}", overwrite);

      let pact_file_name = self.pact.default_file_name();
      let mut path = PathBuf::from(output_dir);
      path.push(pact_file_name);

      info!("Writing pact out to '{}'", path.display());
      let specification = self.pact.specification_version();
      if let Err(err) = write_pact(self.pact.boxed(), path.as_path(), specification,
                                   overwrite.unwrap_or_else(|_| String::default()) == "true") {
        error!("Failed to write pact to file - {}", err);
        panic!("Failed to write pact to file - {}", err);
      }
    }
  }
}
