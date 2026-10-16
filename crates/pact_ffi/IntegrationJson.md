# FFI Integration JSON format

To make setting up matching rules and generators easier for language implementations using the Pact FFI functions, a
JSON format can be used with embedded rules in it. This JSON can be used for most interaction values (i.e. headers,
query parameters, etc.)

For each value were matching rules or generators are required, a JSON object can be passed in with the following 
attributes:

| Attribute             | Description                           | Values                                                                                                                                                                                                                              |
|-----------------------|---------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `pact:matcher:type`   | ID of the matcher to use              | One of: "regex", "equality", "include", "type", "number", "integer", "decimal", "boolean", "datetime", "date", "time", "null", "contentType", "arrayContains", "values", "statusCode", "notEmpty", "semver", "eachKey", "eachValue" |
| `pact:generator:type` | ID of the generator to use (optional) | One of: "RandomInt", "Uuid", "RandomDecimal", "RandomHexadecimal", "RandomString", "Regex", "Date", "Time", "DateTime", "RandomBoolean", "ProviderState", "MockServerURL"                                                           |
| `value`               | Example value to use                  | Can be any JSON value, including arrays and objects                                                                                                                                                                                 |

plus the attributes required for the matching rule and generator (if used). Required attributes for matching rules are
documented at [V4 matching rules](https://github.com/pact-foundation/pact-specification/tree/version-4#matching-rules) and
generators at [V4 generators](https://github.com/pact-foundation/pact-specification/tree/version-4#generators).

For example: Assuming we want to use a regex matcher on the request path. The `pactffi_with_request` function allows us
to pass the JSON value in as the path value. So instead of calling 

```js
pactffi_with_request(handle, "PUT", "/path/to/100")
```

we can use the JSON format with a regex matcher (which requires a "regex" attribute)

```js
pactffi_with_request(handle, "PUT", "{\"value\": \"/path/to/100\", \"pact:matcher:type\": \"regex\", \"regex\": \"\\\\/path\\\\/to\\\\/\\\\d+\"}")
```

## Matching on Paths

function: `pactffi_with_request`
parameter: `path`

For paths, the value used must be a string value (or a primitive value that will be converted to a string).

Example JSON:
```json
{
  "value": "/path/to/100", 
  "pact:matcher:type": "regex", 
  "regex": "\\/path\\/to\\/\\d+"
}
```

## Matching on query parameters

function: `pactffi_with_query_parameter_v2`
parameter: `value`

Query parameters can contain matching rule JSON in the value parameter. They can be used to match a single query parameter
(in which case the value must be convertable to a string), or be applied to repeated query parameters (in which case an
array should be used for the value).

Single parameter JSON:
```json
{
  "value": "100", 
  "pact:matcher:type": "regex", 
  "regex": "\\d+"
}
```

Multiple parameter JSON (this example doesn't make much sense, because query parameters are always string values, so 
will always have the same type. But it will check the minimum length):
```json
{
  "value": ["1"], 
  "pact:matcher:type": "type", 
  "min": 1
}
```

## Matching headers

function: `pactffi_with_header_v2`
parameter: `value`

Headers can contain matching rule JSON in the value parameter. They can be used to match a single header
(in which case the value must be convertable to a string), or be applied to repeated headers (in which case an
array should be used for the value).

Single header value JSON:
```json
{
  "value": "100", 
  "pact:matcher:type": "regex", 
  "regex": "\\d+"
}
```

Multiple header value JSON:
```json
{
  "value": ["1"],
  "pact:matcher:type": "regex",
  "regex": "\\d+"
}
```

## Matching JSON bodies

function: `pactffi_with_body`
parameter: `body`

The interaction JSON format can be embedded in any JSON body to define the matching rules. Just replace the value with
the JSON described above.

### Simple values

Replace the value of the attribute with the JSON format with the embedded matching rule.

For example, add a type matcher to an `id` attribute and a regex matcher to the `created` one:
```json
{
  "id": {
    "value":1,
    "pact:matcher:type": "type"
  },
  "created": {
    "value": "maybe",
    "pact:matcher:type": "regex", 
    "regex": "(yes|no|maybe)"
  }
}
```

### Adding matching rules to arrays (each like matchers)

"each like" type matching rule configurations can be applied by specifying the JSON format in the place of the collection,
and setting the matcher as a Type matcher. For example, in a higher level language DSL, you could define:

```js
{
    id: 100,
    first_name: 'Billy',
    last_name: 'Goat',
    animal: 'goat',
    interests: eachLike('walks in the garden/meadow')
}
```

Here the `interests` attribute would be expanded to

```json
{
  "id": 100,
  "first_name": "Billy",
  "last_name": "Goat",
  "animal": "goat",
  "interests": {
    "pact:matcher:type": "type",
    "value": ["walks in the garden/meadow"]
  }
}
```

## Supporting multiple matching rules

Matching rules can be combined. These rules will be evaluated with an AND (i.e. all the rules must match successfully 
for the result to be successful). The main reason to do this is to combine the `EachKey` and `EachValue` matching rules
on a map structure, but other rules make sense to combine (like the `include` matcher).

To provide multiple matchers, you need to provide an array format. 

For example, assume you have an API that returns results for a document store where the documents are keyed based on some index:
```json
{
  "results": {
    "AUK-155332": {
      "title": "...",
      "description": "...",
      "link": "http://....",
      "relatesTo": ["BAF-88654"]
    }
  }
}
```

Here you may want to provide a matching rule for the keys that they conform to the `AAA-NNNNNNN...` format, as well
as a type matcher for the values.

So the resulting intermediate JSON would be something like:
```json
{
  "results": {
    "pact:matcher:type": [
      {
        "pact:matcher:type": "each-key",
        "value": "AUK-155332",
        "rules": [
          {
            "pact:matcher:type": "regex",
            "regex": "\\w{3}-\\d+"
          }
        ]
      }, {
        "pact:matcher:type": "each-value",
        "rules": [
          {
            "pact:matcher:type": "type"
          }
        ]
      }
    ],
    "AUK-155332": {
      "title": "...",
      "description": "...",
      "link": "http://....",
      "relatesTo": ["BAF-88654"]
    }
  }
}
```

## Supporting matching rule definitions

You can use the [matching rule definition expressions](https://docs.rs/pact_models/latest/pact_models/matchingrules/expressions/index.html) 
in the `pact:matcher:type` field.

For example, with the previous document result JSON, you could then use the following for the `relatesTo` field:

```json
{
  "relatesTo": {
    "pact:matcher:type": "eachValue(matching(regex, '\\w{3}-\\d+', 'BAF-88654'))"
  }
}
```

You can then also combine matchers:

```json
{
  "relatesTo": {
    "pact:matcher:type": "atLeast(1), atMost(10), eachValue(matching(regex, '\\w{3}-\\d+', 'BAF-88654'))"
  }
}
```
