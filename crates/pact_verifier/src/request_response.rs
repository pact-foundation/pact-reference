use ansi_term::Colour::*;
use ansi_term::Style;

use pact_matching::Mismatch;
use pact_models::sync_interaction::RequestResponseInteraction;

use crate::{generate_display_for_result, MismatchResult};

pub fn process_request_response_result(
  interaction: &RequestResponseInteraction,
  match_result: &Result<Option<String>, MismatchResult>,
  output: &mut Vec<String>,
  coloured: bool) {
  let plain = Style::new();
  match match_result {
    Ok(_) => {
      generate_display_for_result(
        interaction.response.status,
        if coloured { Green.paint("OK") } else { plain.paint("OK") },
        interaction.response.headers.clone().map(|h| h.iter().map(|(k, v)| {
          (k.clone(), v.join(", "), if coloured { Green.paint("OK") } else { plain.paint("OK") })
        }).collect()), if coloured { Green.paint("OK") } else { plain.paint("OK") },
        output,
        coloured
      );
    },
    Err(err) => match err {
      MismatchResult::Error(err_des, _) => {
        if coloured {
          output.push(format!("      {}", Red.paint(format!("Request Failed - {}", err_des))));
        } else {
          output.push(format!("      {}", format!("Request Failed - {}", err_des)));
        }
      },
      MismatchResult::Mismatches { mismatches, .. } => {
        let status_result = if mismatches.iter().any(|m| m.mismatch_type() == "StatusMismatch") {
          if coloured { Red.paint("FAILED") } else { plain.paint("FAILED") }
        } else {
            if coloured { Green.paint("OK") } else { plain.paint("OK") }
        };
        let header_results = match interaction.response.headers {
          Some(ref h) => Some(h.iter().map(|(k, v)| {
            (k.clone(), v.join(", "), if mismatches.iter().any(|m| {
              match *m {
                Mismatch::HeaderMismatch { ref key, .. } => k == key,
                _ => false
              }
            }) {
              if coloured { Red.paint("FAILED") } else { plain.paint("FAILED") }
            } else {
              if coloured { Green.paint("OK") } else { plain.paint("OK") }
            })
          }).collect()),
          None => None
        };
        let body_result = if mismatches.iter().any(|m| m.mismatch_type() == "BodyMismatch" ||
          m.mismatch_type() == "BodyTypeMismatch") {
          if coloured { Red.paint("FAILED") } else { plain.paint("FAILED") }
        } else {
            if coloured { Green.paint("OK") } else { plain.paint("OK") }
        };

        generate_display_for_result(interaction.response.status, status_result, header_results,
                                    body_result, output, coloured);
      }
    }
  }
}
