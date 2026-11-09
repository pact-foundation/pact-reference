@consumer
Feature: Matching rules combined with OR in integration JSON
  FFI consumers (Pact JS, Go, PHP, Python, Ruby, ...) define matching rules by embedding integration JSON in the
  bodies they pass to pactffi_with_body. A "pact:matcher:type" array combined with "pact:matcher:combine": "OR"
  is written to the Pact file as a single rule list combined with OR, which the V3 and V4 specifications support.
  See pact-foundation/pact-reference#249.

  Scenario Outline: Matching rules combined with OR are written to the <spec> Pact file
    Given a <spec> HTTP interaction with a POST request with the following JSON body:
      """
      {
        "id": {
          "pact:matcher:type": [
            { "pact:matcher:type": "regex", "regex": "^\\d+$" },
            { "pact:matcher:type": "regex", "regex": "^[a-z]+$" }
          ],
          "pact:matcher:combine": "<combine>",
          "value": "123"
        }
      }
      """
    When the Pact file for the test is generated
    Then the request body in the Pact file will be '{"id": "123"}'
    And the request body matching rules in the Pact file will be:
      """
      {
        "$.id": {
          "combine": "OR",
          "matchers": [
            { "match": "regex", "regex": "^\\d+$" },
            { "match": "regex", "regex": "^[a-z]+$" }
          ]
        }
      }
      """

    Examples:
      | spec | combine |
      | V3   | OR      |
      | V4   | OR      |
      | V3   | or      |

  Scenario Outline: The <spec> mock server accepts a value matching any of the matching rules combined with OR
    Given a <spec> HTTP interaction with a POST request with the following JSON body:
      """
      {
        "id": {
          "pact:matcher:type": [
            { "pact:matcher:type": "regex", "regex": "^\\d+$" },
            { "pact:matcher:type": "regex", "regex": "^[a-z]+$" }
          ],
          "pact:matcher:combine": "OR",
          "value": "123"
        }
      }
      """
    When the mock server is started with the interaction
    And a POST request is made to the mock server with the JSON body '<body>'
    Then the mock server status will be <status>

    Examples:
      | spec | body          | status |
      | V3   | {"id": "123"} | OK     |
      | V3   | {"id": "abc"} | OK     |
      | V3   | {"id": "A1"}  | NOT OK |
      | V4   | {"id": "123"} | OK     |
      | V4   | {"id": "abc"} | OK     |
      | V4   | {"id": "A1"}  | NOT OK |

  Scenario Outline: The <spec> mock server accepts null or an object matching its nested matching rules
    Given a <spec> HTTP interaction with a POST request with the following JSON body:
      """
      {
        "address": {
          "pact:matcher:type": [
            { "pact:matcher:type": "type" },
            { "pact:matcher:type": "null" }
          ],
          "pact:matcher:combine": "OR",
          "value": {
            "number": { "pact:matcher:type": "integer", "value": 100 },
            "street": { "pact:matcher:type": "type", "value": "Main Street" }
          }
        }
      }
      """
    When the mock server is started with the interaction
    And a POST request is made to the mock server with the JSON body '<body>'
    Then the mock server status will be <status>

    Examples:
      | spec | body                                                 | status |
      | V3   | {"address": {"number": 42, "street": "High Street"}} | OK     |
      | V3   | {"address": null}                                    | OK     |
      | V3   | {"address": {"number": "42", "street": "High St"}}   | NOT OK |
      | V4   | {"address": {"number": 42, "street": "High Street"}} | OK     |
      | V4   | {"address": null}                                    | OK     |
      | V4   | {"address": {"number": "42", "street": "High St"}}   | NOT OK |

  Scenario: Matching rules combined with OR replace a matcher wrapped as their example value
    A path has a single list of rules, so the OR rules replace the wrapped matcher's rule (as Pact-JVM's `or` does)
    and only the wrapped matcher's example value is kept.
    Given a V4 HTTP interaction with a POST request with the following JSON body:
      """
      {
        "id": {
          "pact:matcher:type": [
            { "pact:matcher:type": "integer" },
            { "pact:matcher:type": "null" }
          ],
          "pact:matcher:combine": "OR",
          "value": { "pact:matcher:type": "regex", "regex": "^\\d+$", "value": "100" }
        }
      }
      """
    When the Pact file for the test is generated
    Then the request body in the Pact file will be '{"id": "100"}'
    And the request body matching rules in the Pact file will be:
      """
      {
        "$.id": {
          "combine": "OR",
          "matchers": [ { "match": "integer" }, { "match": "null" } ]
        }
      }
      """

  Scenario: An invalid combine value is rejected
    Given a V4 HTTP interaction with a POST request with the following JSON body:
      """
      {
        "id": {
          "pact:matcher:type": [
            { "pact:matcher:type": "integer" },
            { "pact:matcher:type": "null" }
          ],
          "pact:matcher:combine": "XOR",
          "value": 1
        }
      }
      """
    When the Pact file for the test is generated
    Then the request body in the Pact file will be '{"id": null}'
    And the request body in the Pact file will have no matching rules
