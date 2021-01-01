use crate::dict::Dictionary;
use crate::rtf::parse_rtf;

lazy_static! {
  static ref RTF: String = r#"
  {\rtf1\ansi{\*\cxrev100}\cxdict{\*\cxsystem Test}
    {\*\cxs TEFT}test
    {\*\cxs TEFTS}tests
    {\*\cxs TEFTD}tested
    {\*\cxs TEFGT}testing
    {\*\cxs TEFT/-G}testing
  }"#.to_string();

  static ref RTF_WITH_COMMENTS: String = r#"
  {\rtf1\ansi{\*\cxrev100}\cxdict{\*\cxsystem Test}
    {\*\cxs TEFT}test
    {\*\cxs TEFTS}tests
    {\*\cxs TEFTD}tested
    {\*\cxs TEFGT}testing{\*\cxcomment inversion}
    {\*\cxs TEFT/-G}testing{\*\cxcomment two strokes}
  }"#.to_string();

  static ref RTF_WITH_COMMANDS: String = r#"
  {\rtf1\ansi{\*\cxrev100}\cxdict{\*\cxsystem Test}
    {\*\cxs PHROLG}{\*\cxplvrcmd lookup}
    {\*\cxs PHREUT}{\*\cxplvrcmd quit}
  }"#.to_string();

  static ref RTF_WITH_NON_ASCII: String = r#"
  {\rtf1\ansi{\*\cxrev100}\cxdict{\*\cxsystem Test}
    {\*\cxs \u12615\u12636\u12593\u12599}\u50864\u47532\u44032
  }"#.to_string();
}

macro_rules! check_rtf {
  ($rtf:expr, $func:expr) => {
    match parse_rtf($rtf) {
      Some(dict) => { $func(dict); },
      None => panic!("RTF parsing failed"),
    }
  }
}

#[test]
fn test_parse_rtf() {
  check_rtf!(&RTF, |dict: Dictionary| {
    assert_eq!(dict.len(), 5);
    assert_eq!(dict.cre_system, "Test");

    assert_eq!(dict.lookup("TEFGT"), Some("testing".to_string()));
  })
}

#[test]
fn test_parse_rtf_with_comments() {
  check_rtf!(&RTF_WITH_COMMENTS, |dict: Dictionary| {
    assert_eq!(dict.len(), 5);

    if let Some(entry) = dict.entry("TEFGT") {
      assert_eq!(entry.comment(), Some("inversion".to_string()));
    } else {
      panic!("Entry not found");
    }
  })
}

#[test]
fn test_parse_rtf_with_commands() {
  check_rtf!(&RTF_WITH_COMMANDS, |dict: Dictionary| {
    assert_eq!(dict.lookup("PHROLG"), Some("{plover:lookup}".to_string()));
  })
}

#[test]
fn test_parse_rtf_with_non_ascii() {
  check_rtf!(&RTF_WITH_NON_ASCII, |dict: Dictionary| {
    assert_eq!(dict.lookup("ㅇㅜㄱㄷ"), Some("우리가".to_string()));
  })
}
