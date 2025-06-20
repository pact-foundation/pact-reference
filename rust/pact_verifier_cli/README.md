# Standalone Pact Verifier

This project provides a command line interface to verify pact files against a running provider. It is a single 
executable binary. It implements all the Pact specifications up to [V4](https://github.com/pact-foundation/pact-specification/tree/version-4).

[Online rust docs](https://docs.rs/pact_verifier_cli/)

The Pact Verifier works by taking all the interactions (requests and responses) from a number of pact files. For each 
interaction, it will make the request defined in the pact to a running service provider and check the response received
back against the one defined in the pact file. All mismatches will then be reported.

## Command line interface

The pact verifier is bundled as a single binary executable `pact_verifier_cli`. Running this without any options 
displays the standard help.

```console
$ pact_verifier_cli,ignore
Standalone pact verifier for provider pact verification

Usage: pact_verifier_cli [OPTIONS]

Options:
      --help     Print help and exit
  -v, --version  Print version information and exit

Logging options:
  -l, --loglevel <loglevel>  Log level to emit log events at (defaults to warn) [possible values: error, warn, info, debug, trace, none]
      --pretty-log           Emits excessively pretty, multi-line logs, optimized for human readability.
      --full-log             This emits human-readable, single-line logs for each event that occurs, with the current span context displayed before the formatted representation of the event.
      --compact-log          Emit logs optimized for short line lengths.
  -j, --json <json-file>     Generate a JSON report of the verification [env: PACT_VERIFIER_JSON_REPORT=]
  -x, --junit <junit-file>   Generate a JUnit XML report of the verification (requires the junit feature) [env: PACT_VERIFIER_JUNIT_REPORT=]
      --no-colour            Disables ANSI escape codes in the output [aliases: --no-color]

Loading pacts options:
  -f, --file <file>
          Pact file to verify (can be repeated)
  -d, --dir <dir>
          Directory of pact files to verify (can be repeated)
  -u, --url <url>
          URL of pact file to verify (can be repeated)
  -b, --broker-url <broker-url>
          URL of the pact broker to fetch pacts from to verify (requires the provider name parameter) [env: PACT_BROKER_BASE_URL=]
      --webhook-callback-url <webhook-callback-url>
          URL of a Pact to verify via a webhook callback. Requires the broker-url to be set. [env: PACT_WEBHOOK_CALLBACK_URL=]
      --ignore-no-pacts-error
          Do not fail if no pacts are found to verify

Authentication options:
      --user <user>          Username to use when fetching pacts from URLS [env: PACT_BROKER_USERNAME=]
      --password <password>  Password to use when fetching pacts from URLS [env: PACT_BROKER_PASSWORD=]
  -t, --token <token>        Bearer token to use when fetching pacts from URLS [env: PACT_BROKER_TOKEN=]

Provider options:
  -h, --hostname <hostname>
          Provider hostname (defaults to localhost) [env: PACT_PROVIDER_HOSTNAME=]
  -p, --port <port>
          Provider port (defaults to protocol default 80/443) [env: PACT_PROVIDER_PORT=]
      --transport <transport>
          Provider protocol transport to use (http, https, grpc, etc.) [env: PACT_PROVIDER_TRANSPORT=] [default: http]
      --transports <transports>
          Allows multiple protocol transports to be configured (http, https, grpc, etc.) with their associated port numbers separated by a colon. For example, use --transports http:8080 grpc:5555 to configure both.
  -n, --provider-name <provider-name>
          Provider name (defaults to provider) [env: PACT_PROVIDER_NAME=]
      --base-path <base-path>
          Base path to add to all requests [env: PACT_PROVIDER_BASE_PATH=]
      --request-timeout <request-timeout>
          Sets the HTTP request timeout in milliseconds for requests to the target API and for state change requests. [env: PACT_PROVIDER_REQUEST_TIMEOUT=]
  -H, --header <custom-header>
          Add a custom header to be included in the calls to the provider. Values must be in the form KEY=VALUE, where KEY and VALUE contain ASCII characters (32-127) only. Can be repeated.
      --disable-ssl-verification
          Disables validation of SSL certificates

Provider state options:
  -s, --state-change-url <state-change-url>
          URL to post state change requests to [env: PACT_PROVIDER_STATE_CHANGE_URL=]
      --state-change-as-query
          State change request data will be sent as query parameters instead of in the request body [env: PACT_PROVIDER_STATE_CHANGE_AS_QUERY=]
      --state-change-teardown
          State change teardown requests are to be made after each interaction [env: PACT_PROVIDER_STATE_CHANGE_TEARDOWN=]

Filtering interactions:
      --filter-description <filter-description>
          Only validate interactions whose descriptions match this filter (regex format) [env: PACT_DESCRIPTION=]
      --filter-state <filter-state>
          Only validate interactions whose provider states match this filter (regex format) [env: PACT_PROVIDER_STATE=]
      --filter-no-state
          Only validate interactions that have no defined provider state [env: PACT_PROVIDER_NO_STATE=]
  -c, --filter-consumer <filter-consumer>
          Consumer name to filter the pacts to be verified (can be repeated)

Publishing options:
      --publish
          Enables publishing of verification results back to the Pact Broker. Requires the broker-url and provider-version parameters.
      --provider-version <provider-version>
          Provider version that is being verified. This is required when publishing results.
      --build-url <build-url>
          URL of the build to associate with the published verification results.
      --provider-tags <provider-tags>
          Provider tags to use when publishing results. Accepts comma-separated values.
      --provider-branch <provider-branch>
          Provider branch to use when publishing results

Pact Broker options:
      --consumer-version-tags <consumer-version-tags>
          Consumer tags to use when fetching pacts from the Broker. Accepts comma-separated values.
      --consumer-version-selectors <consumer-version-selectors>
          Consumer version selectors to use when fetching pacts from the Broker. Accepts a JSON string as per https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/. Can be repeated.
      --enable-pending
          Enables Pending Pacts
      --include-wip-pacts-since <include-wip-pacts-since>
          Allow pacts that don't match given consumer selectors (or tags) to  be verified, without causing the overall task to fail. For more information, see https://pact.io/wip

Development options:
      --exit-on-first-error  Stops the verifier at the first failure
      --last-failed          Only runs the interactions that failed on the previous verifier run. Requires --json-file to have been set

```

## Options

### Log Level

You can control the log level with the `-l, --loglevel <loglevel>` option. It defaults to warn, and the options that 
you can specify are: error, warn, info, debug, trace, none.

### Pact File Sources

You can specify the pacts to verify with the following options. They can be repeated to set multiple sources.

| Option                          | Type        | Description                                                                                                          |
|---------------------------------|-------------|----------------------------------------------------------------------------------------------------------------------|
| `-f, --file <file>`             | File        | Loads a pact from the given file                                                                                     |
| `-u, --url <url>`               | URL         | Loads a pact from a URL resource                                                                                     |
| `-d, --dir <dir>`               | Directory   | Loads all the pacts from the given directory                                                                         |
| `-b, --broker-url <broker-url>` | Pact Broker | Loads all the pacts for the provider from the pact broker. Requires the `-n, --provider-name <provider-name>` option |

#### Verifying a Pact via a webhook callback

The Pact Broker allows for Pacts to be verified via a callback that supplies the URL to the Pact to verify. To verify
just the Pact from the webhook call, use the `--webhook-callback-url` set to the supplied URL in conjunction with the 
`--broker-url` option.

### Provider Options

The running provider can be specified with the following options:

| Option                                | Description                                                                                                   |
|---------------------------------------|---------------------------------------------------------------------------------------------------------------|
| `-h, --hostname <hostname>`           | The provider hostname, defaults to `localhost`                                                                |
| `-p, --port <port>`                   | The provider port (defaults to protocol default 80/443)                                                       |
| `-n, --provider-name <provider-name>` | The name of the provider. Required if you are loading pacts from a pact broker                                |
| `--base-path <base-path>`             | If the provider is mounted on a sub-path, you can use this option to set the base path to add to all requests |
| `--transport <transport>`             | Protocol transport to use. Defaults to HTTP.                                                                  |

### Filtering the interactions

The interactions that are verified can be filtered by the following options:

#### `-c, --filter-consumer <filter-consumer>`

This will only verify the interactions of matching consumers. You can specify multiple consumers by either separating
the names with a comma, or repeating the option.

#### `--filter-description <filter-description>`

This option will filter the interactions that are verified that match by description. You can use a regular expression
to match.

#### `--filter-state <filter-state>`

This option will filter the interactions that are verified that match by provider state. You can use a regular
expression to match. Can't be used with the `--filter-no-state` option.

#### `--filter-no-state`

This option will filter the interactions that are verified that don't have a defined provider state. Can't be used
with the `--filter-state` option.

### State change requests

[Provider states](https://docs.pact.io/getting_started/provider_states) are a mechanism to define the state that the 
provider needs to be in to be able to verify a particular request. This is achieved by setting a state change URL that
will receive a POST request with the provider state before the actual request is made.

*NOTE:* For verifying messages fetched via HTTP, the provider state is also passed in the request to fetch the message,
so the state change URL is not required.

For example, if a Pact file being verified has a provider state *"a user exists in the database"* and the provider state 
URL is set to `http://localhost:8080/provider-state`, then the following request would be made before the interaction
is verified:

```http request
POST /provider-state HTTP/1.1
Host: localhost:8080
content-type: application/json

{
    "state": "a user exists in the database",
    "params": {},
    "action": "setup"
}
```

If any parameters are configured for the provider state, they will be passed in the *"params"* attribute.

The provider state callback can return a map (JSON object) of values that can then be used by things like the 
Provider State Generator to inject values. The returned map of values will fall back to the provider state parameters.

#### `-s, --state-change-url <state-change-url>`

This sets the absolute URL that the POST requests will be made to before each actual request. If this value is not
set, the state change request will not be made. 

#### `--state-change-as-query`

By default, the state for the state change request will be sent as a JSON document in the body of the request. This 
option forces it to be sent as query parameters instead.

#### `--state-change-teardown`

This option will cause the verifier to also make a tear down request after the main request is made. It will receive a 
field in the body or a query parameter named `action` with the value `teardown`.

### Consumer Version Selectors (`--consumer-version-selectors`)

Accepts a set of [Consumer Version Selectors](https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/) encoded as JSON.

An example of a well-formed argument value might be:

```sh
--consumer-version-selectors '{"branch": "master"}'
```

### Development options

There are two options that support filtering interactions to help with setting up your Pact tests. For instance, setting
up provider state can be quite involved.

#### Exit on first error

The `--exit-on-first-error` option will terminate the verification run on the first error, and will skip all remaining interactions 
and Pact files.

### Only run previously failed interactions

The `--last-failed` option will only execute interactions that have previously failed. It requires the `--json` option
(the previous state will be loaded from this file).

## Example run

This will verify all the pacts for the `happy_provider` found in the pact broker (running on localhost) against the provider running on localhost port 5050. Only the pacts for the consumers `Consumer` and `Consumer2` will be verified.

```console,ignore
$ pact_verifier_cli -b http://localhost -n 'happy_provider' -p 5050 --filter-consumer Consumer --filter-consumer Consumer2
21:59:28 [WARN] pact_matching::models: No metadata found in pact file "http://localhost/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0", assuming V1.1 specification
21:59:28 [WARN] pact_matching::models: No metadata found in pact file "http://localhost/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0", assuming V1.1 specification

Verifying a pact between Consumer and happy_provider
  Given I am friends with Fred
    WARNING: State Change ignored as there is no state change URL
  Given I have no friends
    WARNING: State Change ignored as there is no state change URL
  a request to unfriend but no friends
    returns a response which
      has status code 200 (OK)
      includes headers
      has a matching body (OK)
  a request friends
    returns a response which
      has status code 200 (FAILED)
      includes headers
        "Content-Type" with value "application/json" (FAILED)
      has a matching body (FAILED)
  a request to unfriend
    returns a response which
      has status code 200 (OK)
      includes headers
        "Content-Type" with value "application/json" (OK)
      has a matching body (FAILED)


Verifying a pact between Consumer2 and happy_provider
  Given I am friends with Fred
    WARNING: State Change ignored as there is no state change URL
  Given I have no friends
    WARNING: State Change ignored as there is no state change URL
  a request to unfriend but no friends
    returns a response which
      has status code 200 (OK)
      includes headers
      has a matching body (OK)
  a request friends
    returns a response which
      has status code 200 (FAILED)
      includes headers
        "Content-Type" with value "application/json" (FAILED)
      has a matching body (FAILED)
  a request to unfriend
    returns a response which
      has status code 200 (OK)
      includes headers
        "Content-Type" with value "application/json" (OK)
      has a matching body (FAILED)


Failures:

0) Verifying a pact between Consumer and happy_provider - a request friends returns a response which has a matching body
    expected 'application/json' body but was 'text/plain'

1) Verifying a pact between Consumer and happy_provider - a request friends returns a response which has status code 200
    expected 200 but was 404

2) Verifying a pact between Consumer and happy_provider - a request friends returns a response which includes header 'Content-Type' with value 'application/json'
    Expected header 'Content-Type' to have value 'application/json' but was 'text/plain'

3) Verifying a pact between Consumer and happy_provider Given I am friends with Fred - a request to unfriend returns a response which has a matching body
    $.body -> Type mismatch: Expected Map {"reply":"Bye"} but received  "Ok"


4) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which has a matching body
    expected 'application/json' body but was 'text/plain'

5) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which has status code 200
    expected 200 but was 404

6) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which includes header 'Content-Type' with value 'application/json'
    Expected header 'Content-Type' to have value 'application/json' but was 'text/plain'

7) Verifying a pact between Consumer2 and happy_provider Given I am friends with Fred - a request to unfriend returns a response which has a matching body
    $.body -> Type mismatch: Expected Map {"reply":"Bye"} but received  "Ok"



There were 8 pact failures

```

## Verifying message pacts

Message pacts can be verified, the messages just need to be fetched from an HTTP endpoint. The verifier will send a
POST request to the configured provider and expect the message payload in the response. The POST request will include
the description and any provider states configured in the Pact file for the message, formatted as JSON.

Example POST request:

```json
{
    "description": "Test Message",
    "providerStates":[ {"name": "message exists"} ]
}
```

### Verifying metadata

Message metadata can be included as base64 encoded key/value pairs in the response, packed into the `Pact-Message-Metadata` HTTP header, and will be compared against any expected metadata in the pact file.

The values may contain any valid JSON.

For example, given this metadata:

```json
{
  "Content-Type": "application/json",
  "topic": "baz",
  "number": 27,
  "complex": {
    "foo": "bar"
  }
}
```

we would encode it into a base64 string, giving us `ewogICJDb250ZW50LVR5cGUiOiAiYXBwbGljYXRpb24vanNvbiIsCiAgInRvcGljIjogImJheiIsCiAgIm51bWJlciI6IDI3LAogICJjb21wbGV4IjogewogICAgImZvbyI6ICJiYXIiCiAgfQp9Cg==`.

## TLS and Certificate Management

Pact uses the [rustls-native-certs](https://lib.rs/crates/rustls-native-certs) crate, which will respect the platform's native certificate store when operating as a TLS client:

This is supported on Windows, macOS and Linux:

* On Windows, certificates are loaded from the system certificate store. The schannel crate is used to access the Windows certificate store APIs.
* On macOS, certificates are loaded from the keychain. The user, admin and system trust settings are merged together as documented by Apple. The security-framework crate is used to access the keystore APIs.
* On Linux and other UNIX-like operating systems, the openssl-probe crate is used to discover the filename of the system CA bundle.

On Linux the standard OpenSSL environment variables `SSL_CERT_FILE` and `SSL_CERT_DIR` will also be respected.

## Verifying V4 Pact files

### Pact files that require plugins

Pact files that require plugins can be verified with version 0.9.0-beta.0+. For details on how plugins work, see the
[Pact plugin project](https://github.com/pact-foundation/pact-plugins).

Each required plugin is defined in the `plugins` section in the Pact metadata in the Pact file. The plugins will be 
loaded from the plugin directory. By default, this is `~/.pact/plugins` or the value of the `PACT_PLUGIN_DIR` environment 
variable. Each plugin required by the Pact file must be installed there. You will need to follow the installation 
instructions for each plugin, but the default is to unpack the plugin into a sub-directory `<plugin-name>-<plugin-version>`
(i.e., for the Protobuf plugin 0.0.0 it will be `protobuf-0.0.0`). The plugin manifest file must be present for the
plugin to be able to be loaded.

### Verifying both HTTP and message interactions

V4 Pact files can support both HTTP and message-based interactions in the same file. In this case, the be able to 
handle the verification for both types of interactions you need to use the `--transports <transports>` option. This will
allow configuring different ports to send the different requests to.

For example, `--transports http:8080 message:8081` will send HTTP requests to port 8080 and message requests to port 8081.
