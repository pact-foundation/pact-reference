use cucumber::World;
use tracing_subscriber::EnvFilter;

use crate::shared_steps::provider::ProviderWorld;

pub mod shared_steps;

#[tokio::main]
async fn main() {
  let format = tracing_subscriber::fmt::format().pretty();
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .event_format(format)
    .init();

  ProviderWorld::cucumber()
    .fail_on_skipped()
    .max_concurrent_scenarios(1)
    .after(|_feature, _, _scenario, _status, world| Box::pin(async move {
      if let Some(world) = world {
        let _ = world.provider_server.shutdown();
        for broker in &mut world.mock_brokers {
          let _ = broker.shutdown();
        }
      }
    }))
    .filter_run_and_exit("pact-compatibility-suite/features/V1", |feature, _rule, scenario| {
      feature.tags.iter().any(|tag| tag == "provider") &&
        !scenario.tags.iter().any(|t| t == "wip")
    })
    .await;
}
