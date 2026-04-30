use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[cfg(feature = "junit")] use junit_report::{ReportBuilder, TestCaseBuilder, TestSuiteBuilder};
#[cfg(feature = "junit")] use strip_ansi_escapes;
use serde_json::Value;
use tracing::debug;

#[cfg(feature = "junit")] use pact_verifier::{interaction_mismatch_output, MismatchResult};
use pact_verifier::verification_result::VerificationExecutionResult;

mod xml;

pub(crate) fn write_json_report(result: &VerificationExecutionResult, file_name: &str) -> anyhow::Result<()> {
  debug!("Writing JSON result of the verification to '{file_name}'");
  let mut f = File::create(file_name)?;
  let json: Value = result.into();
  f.write_all(json.to_string().as_bytes())?;
  Ok(())
}

#[cfg(feature = "junit")]
pub(crate) fn write_junit_report(result: &VerificationExecutionResult, file_name: &str, provider: &String) -> anyhow::Result<()> {
  debug!("Writing JUnit result of the verification to '{file_name}'");
  let mut f = File::create(file_name)?;

  let mut test_suite = TestSuiteBuilder::new(provider);
  let stripped_system_out = strip_ansi_escapes::strip_str(result.output.join("\n"));
  test_suite.set_system_out(&stripped_system_out);
  for interaction_result in &result.interaction_results {
    let duration = time::Duration::try_from(interaction_result.duration).unwrap_or_default();
    let test_case = match &interaction_result.result {
      Ok(_) => TestCaseBuilder::success(interaction_result.description.as_str(), duration),
      Err(result) => {
        if interaction_result.pending {
          TestCaseBuilder::skipped(interaction_result.description.as_str())
        } else {
          match result {
            MismatchResult::Mismatches { mismatches, expected, actual, .. } => {
              let mut output_buffer = vec![];
              interaction_mismatch_output(&mut output_buffer, false, 1, &interaction_result.description,
                &mismatches, expected.as_ref(), actual.as_ref());
              let mut builder = TestCaseBuilder::failure(
                interaction_result.description.as_str(),
                duration,
                "",
                "Verification for interaction failed"
              );
              let stripped_output = strip_ansi_escapes::strip_str(output_buffer.join("\n"));
              builder.set_system_out(&stripped_output);
              builder
            },
            MismatchResult::Error(error, _) => TestCaseBuilder::error(
              interaction_result.description.as_str(),
              duration,
              "",
              error.as_str()
            )
          }
        }
      }
    };
    test_suite.add_testcase(test_case.build());
  }

  let mut report_builder = ReportBuilder::new();
  report_builder.add_testsuite(test_suite.build());
  let report = report_builder.build();
  report.write_xml(&mut f)?;
  Ok(())
}

pub(crate) fn write_html_report(
  result: &VerificationExecutionResult,
  file_name: &str,
  provider: &str
) -> anyhow::Result<()> {
  let path = PathBuf::from(file_name);
  let parent = if let Some(parent) = path.parent() {
    parent
  } else {
    return Err(anyhow::anyhow!("No parent directory found for {}", file_name));
  };
  fs::create_dir_all(parent)?;

  let filename = if let Some(filename) = path.file_name() {
    filename
  } else {
    return Err(anyhow::anyhow!("Failed to get file name of '{}'", path.display()));
  };

  let xml_path = if let Some(_extension) = path.extension() {
    path.with_extension("xml")
  } else {
    parent.join(filename).with_extension("xml")
  };
  debug!("Writing XML report of the verification to '{}'", xml_path.display());
  let mut xml_file = File::create(&xml_path)?;
  xml::write_xml_report(&mut xml_file, result, provider)?;

  debug!("Writing HTML report of the verification to '{file_name}'");

  Ok(())
}
