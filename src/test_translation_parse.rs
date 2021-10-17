use crate::translation_parse::format_rtf_to_plover;

macro_rules! check_tl {
  ($translation: literal, $formatted: literal) => {
    assert_eq!(
      format_rtf_to_plover(&String::from($translation)),
      String::from($formatted));
  }
}

// check_tl!(<RTF syntax>, <Plover syntax>)

#[test]
fn test_raw() {
  check_tl!("mooo", "mooo");
}

#[test]
fn test_escapes() {
  check_tl!("\\_", "-");
  check_tl!("\\~", "{^ ^}");
  check_tl!("\\{", "\\{");
  check_tl!("\\}", "\\}");
  check_tl!("\\\\", "\\\\");
}

#[test]
fn test_unicode() {
  check_tl!("\\u20320 \\u22909 !", "你好!");
}

#[test]
fn test_cancel() {
  check_tl!("{\\*\\cxplvrcancel}", "{}");
}

#[test]
fn test_noop() {
  check_tl!("{\\*\\cxplvrnop}", "{#}");
}

#[test]
fn test_meta() {
  check_tl!("{\\*\\cxplvrmeta test_meta}", "{:test_meta}");
  check_tl!("{\\*\\cxplvrmeta test_meta:arg}", "{:test_meta:arg}");
}

#[test]
fn test_undo() {
  check_tl!("\\cxdstroke ", "=undo");
}

#[test]
fn test_retro() {
  check_tl!("{\\*\\cxplvrast}", "{*}");
  check_tl!("{\\*\\cxplvrrpt}", "{*+}");
  check_tl!("{\\*\\cxplvrrtisp}", "{*?}");
  check_tl!("{\\*\\cxplvrrtdsp}", "{*!}");
}

#[test]
fn test_macro() {
  check_tl!("{\\*\\cxplvrmac test_macro}", "=test_macro");
  check_tl!("{\\*\\cxplvrmac test_macro:arg}", "=test_macro:arg");
}

#[test]
fn test_command() {
  check_tl!("{\\*\\cxplvrcmd lookup}", "{plover:lookup}");
  check_tl!("{\\*\\cxplvrcmd switch_system:Test}", "{plover:switch_system:Test}");
}

#[test]
fn test_mode() {
  check_tl!("{\\*\\cxplvrcase1}", "{mode:lower}");
  check_tl!("{\\*\\cxplvrcase0\\cxplvrspc0}", "{mode:reset}");
  check_tl!("{\\*\\cxplvrcase0}", "{mode:reset_case}");
  check_tl!("{\\*\\cxplvrcase1}", "{mode:lower}");
  check_tl!("{\\*\\cxplvrcase2}", "{mode:caps}");
  check_tl!("{\\*\\cxplvrcase3}", "{mode:title}");
  check_tl!("{\\*\\cxplvrcase4\\cxplvrspc}", "{mode:camel}");
  check_tl!("{\\*\\cxplvrspc0}", "{mode:reset_space}");
  check_tl!("{\\*\\cxplvrspc a}", "{mode:set_space:a}");
  check_tl!("{\\*\\cxplvrcase0\\cxplvrspc _}", "{mode:snake}");
}

#[test]
fn test_par() {
  check_tl!("\\par\\s0 ", "{#return}{#return}");
  check_tl!("\\par\\s1 ", "{#return}{#return}    ");
}

#[test]
fn test_key_combo() {
  check_tl!("{\\*\\cxplvrkey Left}", "{#Left}");
  check_tl!("{\\*\\cxplvrkey ctrl_l(tab)}", "{#ctrl_l(tab)}");
}

#[test]
fn test_auto() {
  check_tl!("{\\cxa Q. }", "Q. ");
}

#[test]
fn test_punct() {
  check_tl!("{\\cxp. }", "{.}");
  check_tl!("{\\cxp, }", "{,}");
}

#[test]
fn test_attach() {
  check_tl!("\\cxds ", "{^}");
  check_tl!("\\cxds ing", "{^}ing");
  check_tl!("pre\\cxds ", "pre{^}");
  check_tl!("\\cxds ...\\cxds ", "{^}...{^}");

  check_tl!("{\\*\\cxplvrortho}\\cxds ing", "{^ing}");
  check_tl!("{\\*\\cxplvrortho}pre\\cxds ", "{pre^}");
  check_tl!("{\\*\\cxplvrortho}\\cxds ...\\cxds ", "{^...^}");
}

#[test]
fn test_glue() {
  check_tl!("{\\cxfing a}", "{&a}");
  check_tl!("{\\cxfing th}", "{&th}");
}

#[test]
fn test_stitch() {
  check_tl!("{\\cxstit a}", "{:stitch:a}");
}

#[test]
fn test_conflict() {
  check_tl!("{\\cxconf [{\\cxc first}|{\\cxc second}|{\\cxc last}]}", "last");
}

#[test]
fn test_force_cap() {
  check_tl!("\\cxfc ", "{-|}");
  check_tl!("{\\*\\cxplvrrtfc}", "{*-|}");
  check_tl!("{\\*\\cxplvrfcw}", "{<}");
  check_tl!("{\\*\\cxplvrrtfcw}", "{*<}");
  check_tl!("\\cxfl ", "{>}");
  check_tl!("{\\*\\cxplvrrtfl}", "{*>}");
}

#[test]
fn test_carry_cap() {
  check_tl!("{\\*\\cxplvrccap}{\\*\\cxplvrortho}\\cxds -\\cxds ", "{~|^-^}");
  check_tl!("{\\*\\cxplvrccap}{\\*\\cxplvrortho}\\cxds -esque", "{~|^-esque}");
  check_tl!("{\\*\\cxplvrccap}{\\*\\cxplvrortho}un-\\cxds ", "{~|un-^}");

  check_tl!("{\\*\\cxplvrccap}\\cxds -\\cxds ", "{~|}{^}-{^}");
  check_tl!("{\\*\\cxplvrccap}\\cxds -esque", "{~|}{^}-esque");
  check_tl!("{\\*\\cxplvrccap}un-\\cxds ", "{~|}un-{^}");
  check_tl!("{\\*\\cxplvrccap}5", "{~|}5");
}

#[test]
fn test_currency() {
  check_tl!("{\\*\\cxplvrcurr $c}", "{*($c)}");
  check_tl!("{\\*\\cxplvrcurr c $}", "{*(c $)}");
}

#[test]
fn test_newline() {
  check_tl!("{\\*\\cxplvrortho}\\cxds \\n\\cxds ", "{^\\n^}");
  check_tl!("{\\*\\cxplvrortho}\\cxds \\t\\cxds ", "{^\\t^}");
}

#[test]
fn test_concat() {
  check_tl!("mooo\\u21862 !{\\*\\cxplvrnop}test{\\*\\cxplvrast}", "mooo啦!{#}test{*}");
}
