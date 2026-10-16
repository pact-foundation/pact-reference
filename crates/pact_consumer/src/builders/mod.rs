//! Support for building the types in `pact_matching::models`. This could
//! theoretically be moved into `pact_matching::models` at some future date,
//! but that's currently undergoing heavy construction.

mod interaction_builder;
mod http_part_builder;
mod pact_builder;
mod pact_builder_async;
pub mod plugin_builder;
mod request_builder;
mod response_builder;
mod message_builder;
mod message_iter;
mod sync_message_builder;

pub use self::interaction_builder::*;
pub use self::http_part_builder::*;
pub use self::pact_builder::*;
pub use self::pact_builder_async::*;
pub use self::request_builder::*;
pub use self::response_builder::*;
pub use self::message_builder::*;
pub use self::sync_message_builder::*;

#[test]
fn basic_builder_example() {
    let pact = PactBuilder::new("Consumer", "Provider")
        .interaction("GET /greeting/hello", "", |mut i| {
            i.given("a greeting named hello");
            i.request.method("GET").path("/greeting/hello");
            i.response
                .status(200)
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "message": "Hello!",
                }));
            i
        })
        .build();

    assert_eq!(pact.consumer().name, "Consumer");
    assert_eq!(pact.provider().name, "Provider");
    assert_eq!(pact.interactions().len(), 1);
    let interaction = &pact.interactions()[0];
    assert_eq!(&interaction.description(), "GET /greeting/hello");
    assert_eq!(interaction.provider_states()[0].name, "a greeting named hello");
}
