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

Matching rules can be combined. By default, these rules will be evaluated with an AND (i.e. all the rules must match
successfully for the result to be successful). See [Combining matching rules with OR](#combining-matching-rules-with-or)
to require only one of the rules to match. The main reason to do this is to combine the `EachKey` and `EachValue` matching rules
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

### Combining matching rules with OR

The `pact:matcher:combine` attribute sets how the matching rules are combined. It can be either `AND` (the default,
all the rules must match) or `OR` (at least one of the rules must match). Any other value is an error, and the
matching rules will be ignored.

The top level `value` is the example used for the consumer test.

#### Alternatives with their own example values

Each matcher in an `OR` list can have its own `value`. This makes it a complete alternative: the value is the
example for that alternative, and can contain matchers of its own. The alternatives can be any shape, so a value
can be an object of one shape or another, an object or an array, an array or a string, and so on.

For example, a `contact` that can either be an object with an email, an object with a phone number, a non-empty
array of contacts, or null:

```json
{
  "contact": {
    "pact:matcher:type": [
      {
        "pact:matcher:type": "type",
        "value": {
          "email": { "pact:matcher:type": "regex", "regex": "^[^@ ]+@[^@ ]+$", "value": "fred@example.com" }
        }
      },
      {
        "pact:matcher:type": "type",
        "value": {
          "phone": { "pact:matcher:type": "regex", "regex": "^\\d+$", "value": "0123456789" }
        }
      },
      { "pact:matcher:type": "type", "min": 1, "value": [{ "email": "fred@example.com" }] },
      { "pact:matcher:type": "null", "value": null }
    ],
    "pact:matcher:combine": "OR",
    "value": { "email": "fred@example.com" }
  }
}
```

A value matches an alternative if it matches the alternative's example using the alternative's matchers, in the
same way as any other value and matchers.

In the Pact file, the alternatives are stored as the matchers of a rule list combined with `OR`. Each one has a
`variant` attribute with its example value and its matching rules, with paths relative to the example value:

```json
"$.contact": {
  "combine": "OR",
  "matchers": [
    {
      "match": "type",
      "variant": {
        "value": { "email": "fred@example.com" },
        "rules": {
          "$": { "combine": "AND", "matchers": [{ "match": "type" }] },
          "$.email": { "combine": "AND", "matchers": [{ "match": "regex", "regex": "^[^@ ]+@[^@ ]+$" }] }
        }
      }
    },
    ...
  ]
}
```

Implementations that don't support the `variant` attribute will ignore it, and use the matchers as a simple `OR`.

#### Alternatives without example values

If a matcher in an `OR` list doesn't have a `value`, it is applied to the top level example value. This is enough
when the alternatives can be checked without an example, like a field that is either a string or null:

```json
{
  "name": {
    "pact:matcher:type": [
      { "pact:matcher:type": "type" },
      { "pact:matcher:type": "null" }
    ],
    "pact:matcher:combine": "OR",
    "value": "Fred"
  }
}
```

Matchers that compare against the example (like `type`) can only describe the shape of the top level example, so
use alternatives with their own example values for anything else. Also, the `regex` and `include` matchers convert
non-string values to their JSON text before matching, so as an alternative they can accept numbers, booleans,
objects or arrays whose JSON text matches (for example, `^\w+$` matches the number `5`).

An alternative can also be a list of matchers combined with `AND` or `OR`, as long as it has its own `value`:

```json
{
  "id": {
    "pact:matcher:type": [
      {
        "pact:matcher:type": [
          { "pact:matcher:type": "regex", "regex": "^\\d+$" },
          { "pact:matcher:type": "include", "value": "1" }
        ],
        "value": "100"
      },
      { "pact:matcher:type": "null", "value": null }
    ],
    "pact:matcher:combine": "OR",
    "value": "100"
  }
}
```

#### Headers, query parameters, paths and XML

Alternatives also work for headers, query parameters, paths, status codes, and XML attributes and text. These
values are not JSON, so only the matchers for the alternative value itself are applied, and they are compared
against the alternative's example value (i.e. an `equality` alternative compares against its own `value`).

#### Limitations

* Use `OR` with care. Only the top level example value is used for the consumer test, so the other alternatives are
  never exercised against your consumer code.
* XML elements can not have alternatives with their own example values, only simple `OR` lists. This is because the
  matching rules for XML are keyed by the element names, so an alternative element would need the same name.
* Other Pact implementations may not support the `variant` attribute, and will use the matchers as a simple `OR`.
* The experimental V2 matching engine (`PACT_MATCHING_ENGINE=v2`) does not support `OR`.

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
