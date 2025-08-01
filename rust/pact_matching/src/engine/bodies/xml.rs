//! Builder for XML bodies

use bytes::Bytes;
use itertools::Itertools;
use kiss_xml::dom::Element;
use pact_models::content_types::ContentType;
use pact_models::path_exp::DocPath;
use pact_models::xml_utils::{group_children, text_nodes};

use crate::engine::{build_matching_rule_node, ExecutionPlanNode, NodeValue};
use crate::engine::bodies::{drop_indices, PlanBodyBuilder, remove_marker};
use crate::engine::context::PlanMatchingContext;
use crate::xml::resolve_attr_namespaces;

fn name(element: &Element) -> String {
  if let Some(namespace) = element.namespace() {
    format!("{}:{}", namespace, element.name())
  } else {
    element.name()
  }
}

/// Plan builder for XML bodies
#[derive(Clone, Debug)]
pub struct XMLPlanBuilder;

impl XMLPlanBuilder {
  /// Create a new instance
  pub fn new() -> Self {
    XMLPlanBuilder{}
  }

  fn process_element(
    &self,
    context: &PlanMatchingContext,
    element: &Element,
    index: Option<usize>,
    path: &DocPath,
    node: &mut ExecutionPlanNode
  ) {
    let name = name(element);
    let element_path = if path.ends_with(format!("['{}*']", name).as_str()) {
      path.clone()
    } else if let Some(index) = index {
      path.join_field(&name).join_index(index)
    } else {
      path.join_field(&name)
    };

    let mut presence_check = ExecutionPlanNode::action("if");
    presence_check
      .add(ExecutionPlanNode::action("check:exists")
        .add(ExecutionPlanNode::resolve_current_value(element_path.clone())));
    let mut item_node = ExecutionPlanNode::container(&element_path);

    if !element.attributes().is_empty() {
      let mut attributes_node = ExecutionPlanNode::container("attributes");
      self.process_attributes(&element_path, element, &mut attributes_node, context);
      item_node.add(attributes_node);
    }

    let mut text_node = ExecutionPlanNode::container("#text");
    self.process_text(&element_path, element, &mut text_node, context);
    item_node.add(text_node);

    self.process_children(context, &element_path, element, &mut item_node);
    presence_check.add(item_node);

    let mut error_node = ExecutionPlanNode::action("error");
    error_node
      .add(ExecutionPlanNode::value_node(
        format!("Was expecting an XML element {} but it was missing", element_path
          .as_json_pointer().unwrap_or_else(|_| element.name())
        )));
    presence_check.add(error_node);

    node.add(presence_check);
  }

  fn process_children(
    &self,
    context: &PlanMatchingContext,
    path: &DocPath,
    element: &Element,
    parent_node: &mut ExecutionPlanNode
  ) {
    let children = group_children(element);

    let no_markers = remove_marker(&path);
    let no_indices = drop_indices(&no_markers);
    let matchers = context.select_best_matcher_from(&no_markers, &no_indices)
      .filter(|matcher| matcher.is_type_matcher())
      .remove_duplicates();
    if !context.config.allow_unexpected_entries || !matchers.is_empty() {
      if element.child_elements().next().is_none() {
        parent_node.add(
          ExecutionPlanNode::action("expect:empty")
            .add(ExecutionPlanNode::resolve_current_value(path))
        );
      } else {
        parent_node.add(
          ExecutionPlanNode::action("expect:only-entries")
            .add(ExecutionPlanNode::value_node(children.keys().collect_vec()))
            .add(ExecutionPlanNode::resolve_current_value(path))
        );
      }
    }

    for (child_name, elements) in children {
      let p = path.join(child_name.as_str());

      let no_markers = remove_marker(&p);
      let no_indices = drop_indices(&no_markers);
      let matchers = context.select_best_matcher_from(&no_markers, &no_indices)
        .filter(|matcher| matcher.is_type_matcher())
        .remove_duplicates();
      if matchers.is_empty() {
        if !context.config.allow_unexpected_entries {
          parent_node.add(
            ExecutionPlanNode::action("expect:count")
              .add(ExecutionPlanNode::value_node(NodeValue::UINT(elements.len() as u64)))
              .add(ExecutionPlanNode::resolve_current_value(p.clone()))
              .add(
                ExecutionPlanNode::action("join")
                  .add(ExecutionPlanNode::value_node(
                    format!("Expected {} <{}> child element{} but there were ",
                            elements.len(), child_name.as_str(), if elements.len() > 1 { "s" } else { "" })))
                  .add(ExecutionPlanNode::action("length")
                    .add(ExecutionPlanNode::resolve_current_value(p.clone())))
              )
          );
        }

        if elements.len() == 1 {
          self.process_element(context, elements[0], Some(0), path, parent_node);
        } else {
          for (index, child) in elements.iter().enumerate() {
            self.process_element(context, child, Some(index), path, parent_node);
          }
        }
      } else {
        let rules = matchers
          .filter(|m| m.is_length_type_matcher());
        if !rules.is_empty() {
          parent_node.add(ExecutionPlanNode::annotation(format!("{} {}",
            p.last_field().unwrap_or_default(),
            rules.generate_description(true))));
          parent_node.add(build_matching_rule_node(&ExecutionPlanNode::value_node(elements[0]),
            &ExecutionPlanNode::resolve_current_value(&p), &rules, true, false));
        }

        let mut for_each_node = ExecutionPlanNode::action("for-each");
        let marker = format!("{}*", child_name);
        for_each_node.add(ExecutionPlanNode::value_node(marker.as_str()));
        for_each_node.add(ExecutionPlanNode::resolve_current_value(&p));
        let item_path = path.join(marker.as_str());

        self.process_element(context, elements[0], Some(0), &item_path, &mut for_each_node);

        parent_node.add(for_each_node);
      }
    }
  }

  fn process_text(
    &self,
    path: &DocPath,
    element: &Element,
    node: &mut ExecutionPlanNode,
    context: &PlanMatchingContext
  ) {
    let text_nodes = text_nodes(element);
    let p = path.join("#text");
    let no_markers = remove_marker(&p);
    let no_indices = drop_indices(&no_markers);
    let matchers = context.select_best_matcher_from(&no_markers, &no_indices)
      .remove_duplicates();
    if !matchers.type_matcher_defined() {
      if !matchers.is_empty() {
        node.add(ExecutionPlanNode::annotation(format!("{} {}", p.last_field().unwrap_or_default(),
                                                       matchers.generate_description(false))));
        let mut current_value = ExecutionPlanNode::action("to-string");
        current_value.add(ExecutionPlanNode::resolve_current_value(&p));
        node.add(build_matching_rule_node(&ExecutionPlanNode::value_node(text_nodes.join("")),
          &current_value, &matchers, false, false));
      } else {
        if text_nodes.is_empty() {
          node.add(ExecutionPlanNode::action("expect:empty")
            .add(ExecutionPlanNode::action("to-string")
              .add(ExecutionPlanNode::resolve_current_value(&p))));
        } else {
          let mut match_node = ExecutionPlanNode::action("match:equality");
          match_node
            .add(ExecutionPlanNode::value_node(NodeValue::STRING(text_nodes.join(""))))
            .add(ExecutionPlanNode::action("to-string")
              .add(ExecutionPlanNode::resolve_current_value(&p)))
            .add(ExecutionPlanNode::value_node(NodeValue::NULL))
            .add(ExecutionPlanNode::value_node(false));
          node.add(match_node);
        }
      }
    }
  }

  fn process_attributes(
    &self,
    path: &DocPath,
    element: &Element,
    node: &mut ExecutionPlanNode,
    context: &PlanMatchingContext
  ) {
    let attributes = resolve_attr_namespaces(element);
    let keys = attributes.keys()
      .filter(|key| key.as_str() != "xmlns" && !key.starts_with("xmlns:"))
      .cloned()
      .sorted()
      .collect_vec();
    for key in &keys {
      let p = path.join_field(format!("@{}", key));
      let value = attributes.get(key).unwrap();
      let mut item_node = ExecutionPlanNode::container(p.to_string());

      let mut presence_check = ExecutionPlanNode::action("if");
      let item_value = NodeValue::STRING(value.clone());
      presence_check
        .add(
          ExecutionPlanNode::action("check:exists")
            .add(ExecutionPlanNode::resolve_current_value(&p))
        );

      let no_indices = drop_indices(&p);
      let matchers = context.select_best_matcher(&p)
        .and_rules(&context.select_best_matcher(&no_indices))
        .remove_duplicates();
      if !matchers.is_empty() {
        item_node.add(ExecutionPlanNode::annotation(format!("@{} {}", key, matchers.generate_description(true))));
        presence_check.add(build_matching_rule_node(&ExecutionPlanNode::value_node(item_value),
          ExecutionPlanNode::action("xml:value")
            .add(ExecutionPlanNode::resolve_current_value(&p)),
          &matchers, false, false));
      } else {
        item_node.add(ExecutionPlanNode::annotation(format!("@{}={}", key, item_value.to_string())));
        let mut item_check = ExecutionPlanNode::action("match:equality");
        item_check
          .add(ExecutionPlanNode::value_node(item_value.clone()))
          .add(ExecutionPlanNode::action("xml:value")
            .add(ExecutionPlanNode::resolve_current_value(&p)))
          .add(ExecutionPlanNode::value_node(NodeValue::NULL))
          .add(ExecutionPlanNode::value_node(false));
        presence_check.add(item_check);
      }

      item_node.add(presence_check);
      node.add(item_node);
    }

    node.add(
      ExecutionPlanNode::action("expect:entries")
        .add(ExecutionPlanNode::value_node(NodeValue::SLIST(keys.clone())))
        .add(ExecutionPlanNode::action("xml:attributes")
          .add(ExecutionPlanNode::resolve_current_value(path.clone())))
        .add(
          ExecutionPlanNode::action("join")
            .add(ExecutionPlanNode::value_node("The following expected attributes were missing: "))
            .add(ExecutionPlanNode::action("join-with")
              .add(ExecutionPlanNode::value_node(", "))
              .add(
                ExecutionPlanNode::splat()
                  .add(ExecutionPlanNode::action("apply"))
              )
            )
        )
    );

    if !context.config.allow_unexpected_entries {
      if !context.config.allow_unexpected_entries {
        node.add(
          ExecutionPlanNode::action("expect:only-entries")
            .add(ExecutionPlanNode::value_node(keys.clone()))
            .add(ExecutionPlanNode::action("xml:attributes")
              .add(ExecutionPlanNode::resolve_current_value(path)))
        );
      }
    }
  }
}

impl PlanBodyBuilder for XMLPlanBuilder {
  fn namespace(&self) -> Option<String> {
    Some("xml".to_string())
  }
  fn supports_type(&self, content_type: &ContentType) -> bool {
    content_type.is_xml()
  }

  fn build_plan(&self, content: &Bytes, context: &PlanMatchingContext) -> anyhow::Result<ExecutionPlanNode> {
    let dom = kiss_xml::parse_str(String::from_utf8_lossy(&content))?;
    let root_element = dom.root_element();

    let mut body_node = ExecutionPlanNode::action("tee");
    body_node
      .add(ExecutionPlanNode::action("xml:parse")
        .add(ExecutionPlanNode::resolve_value(DocPath::new_unwrap("$.body"))));

    let path = DocPath::root();
    let mut root_node = ExecutionPlanNode::container(&path);
    self.process_element(context, root_element, None, &path, &mut root_node);

    body_node.add(root_node);

    Ok(body_node)
  }
}

#[cfg(test)]
mod tests {
  use bytes::Bytes;
  use pretty_assertions::assert_eq;
  use pact_models::matchingrules;
  use pact_models::matchingrules::MatchingRule;
  use crate::engine::bodies::{PlanBodyBuilder, XMLPlanBuilder};
  use crate::engine::context::{MatchingConfiguration, PlanMatchingContext};

  #[test_log::test]
  fn xml_plan_builder_with_very_simple_xml() {
    let builder = XMLPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#;
    let content = Bytes::copy_from_slice(xml.as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    assert_eq!(r#"%tee (
  %xml:parse (
    $.body
  ),
  :$ (
    %if (
      %check:exists (
        ~>$.blah
      ),
      :$.blah (
        :#text (
          %expect:empty (
            %to-string (
              ~>$.blah['#text']
            )
          )
        ),
        %expect:empty (
          ~>$.blah
        )
      ),
      %error (
        'Was expecting an XML element /blah but it was missing'
      )
    )
  )
)"#, buffer);
  }

  #[test_log::test]
  fn xml_plan_builder_with_allowed_unexpected_values() {
    let builder = XMLPlanBuilder::new();
    let context = PlanMatchingContext {
      config: MatchingConfiguration {
        allow_unexpected_entries: true,
        .. MatchingConfiguration::default()
      },
      .. PlanMatchingContext::default()
    };
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#;
    let content = Bytes::copy_from_slice(xml.as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    assert_eq!(r#"%tee (
  %xml:parse (
    $.body
  ),
  :$ (
    %if (
      %check:exists (
        ~>$.blah
      ),
      :$.blah (
        :#text (
          %expect:empty (
            %to-string (
              ~>$.blah['#text']
            )
          )
        )
      ),
      %error (
        'Was expecting an XML element /blah but it was missing'
      )
    )
  )
)"#, buffer);
  }

  #[test_log::test]
  fn xml_plan_builder_with_simple_xml() {
    let builder = XMLPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
      <config>
        <name>My Settings</name>
        <sound>
          <property name="volume" value="11" />
          <property name="mixer" value="standard" />
        </sound>
      </config>
    "#;
    let content = Bytes::copy_from_slice(xml.as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    assert_eq!(r#"%tee (
  %xml:parse (
    $.body
  ),
  :$ (
    %if (
      %check:exists (
        ~>$.config
      ),
      :$.config (
        :#text (
          %expect:empty (
            %to-string (
              ~>$.config['#text']
            )
          )
        ),
        %expect:only-entries (
          ['name', 'sound'],
          ~>$.config
        ),
        %expect:count (
          UINT(1),
          ~>$.config.name,
          %join (
            'Expected 1 <name> child element but there were ',
            %length (
              ~>$.config.name
            )
          )
        ),
        %if (
          %check:exists (
            ~>$.config.name[0]
          ),
          :$.config.name[0] (
            :#text (
              %match:equality (
                'My Settings',
                %to-string (
                  ~>$.config.name[0]['#text']
                ),
                NULL,
                BOOL(false)
              )
            ),
            %expect:empty (
              ~>$.config.name[0]
            )
          ),
          %error (
            'Was expecting an XML element /config/name/0 but it was missing'
          )
        ),
        %expect:count (
          UINT(1),
          ~>$.config.sound,
          %join (
            'Expected 1 <sound> child element but there were ',
            %length (
              ~>$.config.sound
            )
          )
        ),
        %if (
          %check:exists (
            ~>$.config.sound[0]
          ),
          :$.config.sound[0] (
            :#text (
              %expect:empty (
                %to-string (
                  ~>$.config.sound[0]['#text']
                )
              )
            ),
            %expect:only-entries (
              ['property'],
              ~>$.config.sound[0]
            ),
            %expect:count (
              UINT(2),
              ~>$.config.sound[0].property,
              %join (
                'Expected 2 <property> child elements but there were ',
                %length (
                  ~>$.config.sound[0].property
                )
              )
            ),
            %if (
              %check:exists (
                ~>$.config.sound[0].property[0]
              ),
              :$.config.sound[0].property[0] (
                :attributes (
                  :$.config.sound[0].property[0]['@name'] (
                    #{'@name=\'volume\''},
                    %if (
                      %check:exists (
                        ~>$.config.sound[0].property[0]['@name']
                      ),
                      %match:equality (
                        'volume',
                        %xml:value (
                          ~>$.config.sound[0].property[0]['@name']
                        ),
                        NULL,
                        BOOL(false)
                      )
                    )
                  ),
                  :$.config.sound[0].property[0]['@value'] (
                    #{'@value=\'11\''},
                    %if (
                      %check:exists (
                        ~>$.config.sound[0].property[0]['@value']
                      ),
                      %match:equality (
                        '11',
                        %xml:value (
                          ~>$.config.sound[0].property[0]['@value']
                        ),
                        NULL,
                        BOOL(false)
                      )
                    )
                  ),
                  %expect:entries (
                    ['name', 'value'],
                    %xml:attributes (
                      ~>$.config.sound[0].property[0]
                    ),
                    %join (
                      'The following expected attributes were missing: ',
                      %join-with (
                        ', ',
                        ** (
                          %apply ()
                        )
                      )
                    )
                  ),
                  %expect:only-entries (
                    ['name', 'value'],
                    %xml:attributes (
                      ~>$.config.sound[0].property[0]
                    )
                  )
                ),
                :#text (
                  %expect:empty (
                    %to-string (
                      ~>$.config.sound[0].property[0]['#text']
                    )
                  )
                ),
                %expect:empty (
                  ~>$.config.sound[0].property[0]
                )
              ),
              %error (
                'Was expecting an XML element /config/sound/0/property/0 but it was missing'
              )
            ),
            %if (
              %check:exists (
                ~>$.config.sound[0].property[1]
              ),
              :$.config.sound[0].property[1] (
                :attributes (
                  :$.config.sound[0].property[1]['@name'] (
                    #{'@name=\'mixer\''},
                    %if (
                      %check:exists (
                        ~>$.config.sound[0].property[1]['@name']
                      ),
                      %match:equality (
                        'mixer',
                        %xml:value (
                          ~>$.config.sound[0].property[1]['@name']
                        ),
                        NULL,
                        BOOL(false)
                      )
                    )
                  ),
                  :$.config.sound[0].property[1]['@value'] (
                    #{'@value=\'standard\''},
                    %if (
                      %check:exists (
                        ~>$.config.sound[0].property[1]['@value']
                      ),
                      %match:equality (
                        'standard',
                        %xml:value (
                          ~>$.config.sound[0].property[1]['@value']
                        ),
                        NULL,
                        BOOL(false)
                      )
                    )
                  ),
                  %expect:entries (
                    ['name', 'value'],
                    %xml:attributes (
                      ~>$.config.sound[0].property[1]
                    ),
                    %join (
                      'The following expected attributes were missing: ',
                      %join-with (
                        ', ',
                        ** (
                          %apply ()
                        )
                      )
                    )
                  ),
                  %expect:only-entries (
                    ['name', 'value'],
                    %xml:attributes (
                      ~>$.config.sound[0].property[1]
                    )
                  )
                ),
                :#text (
                  %expect:empty (
                    %to-string (
                      ~>$.config.sound[0].property[1]['#text']
                    )
                  )
                ),
                %expect:empty (
                  ~>$.config.sound[0].property[1]
                )
              ),
              %error (
                'Was expecting an XML element /config/sound/0/property/1 but it was missing'
              )
            )
          ),
          %error (
            'Was expecting an XML element /config/sound/0 but it was missing'
          )
        )
      ),
      %error (
        'Was expecting an XML element /config but it was missing'
      )
    )
  )
)"#, buffer);
  }

  #[test_log::test]
  fn matching_rule_on_element_text() {
    let builder = XMLPlanBuilder::new();
    let matching_rules = matchingrules! {
      "body" => { "$.values.value" => [ MatchingRule::Regex("\\d+".to_string()) ] }
    };
    let context = PlanMatchingContext {
      matching_rules: matching_rules.rules_for_category("body").unwrap_or_default(),
      .. PlanMatchingContext::default()
    };
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?> <values><value>100</value></values>"#;
    let content = Bytes::copy_from_slice(xml.as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);

    assert_eq!(r#"%tee (
  %xml:parse (
    $.body
  ),
  :$ (
    %if (
      %check:exists (
        ~>$.values
      ),
      :$.values (
        :#text (
          %expect:empty (
            %to-string (
              ~>$.values['#text']
            )
          )
        ),
        %expect:only-entries (
          ['value'],
          ~>$.values
        ),
        %expect:count (
          UINT(1),
          ~>$.values.value,
          %join (
            'Expected 1 <value> child element but there were ',
            %length (
              ~>$.values.value
            )
          )
        ),
        %if (
          %check:exists (
            ~>$.values.value[0]
          ),
          :$.values.value[0] (
            :#text (
              #{'#text must match the regular expression /\\d+/'},
              %match:regex (
                '100',
                %to-string (
                  ~>$.values.value[0]['#text']
                ),
                json:{"regex":"\\d+"},
                BOOL(false)
              )
            ),
            %expect:empty (
              ~>$.values.value[0]
            )
          ),
          %error (
            'Was expecting an XML element /values/value/0 but it was missing'
          )
        )
      ),
      %error (
        'Was expecting an XML element /values but it was missing'
      )
    )
  )
)"#, buffer);
  }

  #[test_log::test]
  fn matching_rule_on_attribute() {
    let builder = XMLPlanBuilder::new();
    let matching_rules = matchingrules! {
      "body" => { "$.value.@id" => [ MatchingRule::Regex("\\d+".to_string()) ] }
    };
    let context = PlanMatchingContext {
      matching_rules: matching_rules.rules_for_category("body").unwrap_or_default(),
      .. PlanMatchingContext::default()
    };
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?> <value id="100"/>"#;
    let content = Bytes::copy_from_slice(xml.as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);

    assert_eq!(r#"%tee (
  %xml:parse (
    $.body
  ),
  :$ (
    %if (
      %check:exists (
        ~>$.value
      ),
      :$.value (
        :attributes (
          :$.value['@id'] (
            #{'@id must match the regular expression /\\d+/'},
            %if (
              %check:exists (
                ~>$.value['@id']
              ),
              %match:regex (
                '100',
                %xml:value (
                  ~>$.value['@id']
                ),
                json:{"regex":"\\d+"},
                BOOL(false)
              )
            )
          ),
          %expect:entries (
            ['id'],
            %xml:attributes (
              ~>$.value
            ),
            %join (
              'The following expected attributes were missing: ',
              %join-with (
                ', ',
                ** (
                  %apply ()
                )
              )
            )
          ),
          %expect:only-entries (
            ['id'],
            %xml:attributes (
              ~>$.value
            )
          )
        ),
        :#text (
          %expect:empty (
            %to-string (
              ~>$.value['#text']
            )
          )
        ),
        %expect:empty (
          ~>$.value
        )
      ),
      %error (
        'Was expecting an XML element /value but it was missing'
      )
    )
  )
)"#, buffer);
  }

  #[test_log::test]
  fn type_matching_rule_on_element() {
    let builder = XMLPlanBuilder::new();
    let matching_rules = matchingrules! {
      "body" => { "$.values" => [ MatchingRule::MinType(2) ] }
    };
    let context = PlanMatchingContext {
      matching_rules: matching_rules.rules_for_category("body").unwrap_or_default(),
      .. PlanMatchingContext::default()
    };
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?> <values><value>100</value><value>300</value></values>"#;
    let content = Bytes::copy_from_slice(xml.as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);

    assert_eq!(r#"%tee (
  %xml:parse (
    $.body
  ),
  :$ (
    %if (
      %check:exists (
        ~>$.values
      ),
      :$.values (
        :#text (),
        %expect:only-entries (
          ['value'],
          ~>$.values
        ),
        #{'value must match by type and have at least 2 items'},
        %match:min-type (
          xml:'<value>100</value>',
          ~>$.values.value,
          json:{"min":2},
          BOOL(false)
        ),
        %for-each (
          'value*',
          ~>$.values.value,
          %if (
            %check:exists (
              ~>$.values['value*']
            ),
            :$.values['value*'] (
              :#text (),
              %expect:empty (
                ~>$.values['value*']
              )
            ),
            %error (
              'Was expecting an XML element /values/value* but it was missing'
            )
          )
        )
      ),
      %error (
        'Was expecting an XML element /values but it was missing'
      )
    )
  )
)"#, buffer);
  }
}
