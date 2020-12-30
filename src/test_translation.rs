use crate::translation::format_plover_to_rtf;

macro_rules! check_tl {
  ($translation: literal, $formatted: literal) => {
    assert_eq!(
      format_plover_to_rtf(&String::from($translation)),
      String::from($formatted));
  }
}

// check_tl!(<Plover syntax>, <RTF syntax>)

#[test]
fn test_raw() {
  check_tl!("mooo", "mooo");
}

#[test]
fn test_escapes() {
  check_tl!("{ }", " ");
  check_tl!("-", "\\_");
  check_tl!("\\{", "\\{");
  check_tl!("\\}", "\\}");
  check_tl!("\\\\", "\\\\");
}

#[test]
fn test_unicode() {
  check_tl!("你好!", "\\u20320 \\u22909 !");
}

#[test]
fn test_cancel() {
  // \cxplvrcancel: CANCEL formatting of the next word
  check_tl!("{}", "{\\*\\cxplvrcancel}");
}

#[test]
fn test_noop() {
  // \cxplvrnop: do nothing (No-OP)
  check_tl!("{#}", "{\\*\\cxplvrnop}");
}

#[test]
fn test_meta() {
  // \cxplvrmeta: run META
  check_tl!("{:test_meta}", "{\\*\\cxplvrmeta test_meta}");
  check_tl!("{:test_meta:arg}", "{\\*\\cxplvrmeta test_meta:arg}");
}

#[test]
fn test_undo() {
  // \cxdstroke: Deletes the previous STROKE
  check_tl!("=undo", "\\cxdstroke ");
}

#[test]
fn test_retro() {
  // \cxplvrast: toggle ASTerisk
  // \cxplvrrpt: RePeaT last stroke
  // \cxplvrrtisp: ReTroactive Insert SPace
  // \cxplvrrtdsp: ReTroactive Delete SPace
  check_tl!("{*}", "{\\*\\cxplvrast}");
  check_tl!("{*+}", "{\\*\\cxplvrrpt}");
  check_tl!("{*?}", "{\\*\\cxplvrrtisp}");
  check_tl!("{*!}", "{\\*\\cxplvrrtdsp}");

  check_tl!("=retrospective_toggle_asterisk", "{\\*\\cxplvrast}");
  check_tl!("=repeat_last_stroke", "{\\*\\cxplvrrpt}");
  check_tl!("=retrospective_insert_space", "{\\*\\cxplvrrtisp}");
  check_tl!("=retrospective_delete_space", "{\\*\\cxplvrrtdsp}");
}

#[test]
fn test_macro() {
  // \cxplvrmac: run MACro
  check_tl!("=test_macro", "{\\*\\cxplvrmac test_macro}");
  check_tl!("=test_macro:arg", "{\\*\\cxplvrmac test_macro:arg}");
}

#[test]
fn test_command() {
  // \cxplvrcmd: run CoMmanD
  check_tl!("{PLOVER:LOOKUP}", "{\\*\\cxplvrcmd lookup}");
  check_tl!("{PLOVER:SWITCH_SYSTEM:Test}", "{\\*\\cxplvrcmd switch_system:Test}");

  check_tl!("{:command:lookup}", "{\\*\\cxplvrcmd lookup}");
}

#[test]
fn test_mode() {
  // \cxplvrcase: change CASE mode
  //    0 - Sentence case, 1 - lower case, 2 - UPPER CASE, 3 - Title Case, 4 - camelCase
  // \cxplvrspc: chance SPaCe mode
  //    0 - default space, "" - no space
  check_tl!("{MODE:LOWER}", "{\\*\\cxplvrcase1}");
  check_tl!("{MODE:RESET}", "{\\*\\cxplvrcase0\\cxplvrspc0}");
  check_tl!("{MODE:RESET_CASE}", "{\\*\\cxplvrcase0}");
  check_tl!("{MODE:RESET_SPACE}", "{\\*\\cxplvrspc0}");
  check_tl!("{MODE:SET_SPACE:a}", "{\\*\\cxplvrspc a}");

  check_tl!("{:mode:lower}", "{\\*\\cxplvrcase1}");
  check_tl!("{:mode:caps}", "{\\*\\cxplvrcase2}");
  check_tl!("{:mode:title}", "{\\*\\cxplvrcase3}");
  check_tl!("{:mode:camel}", "{\\*\\cxplvrcase4\\cxplvrspc}");
  check_tl!("{:mode:snake}", "{\\*\\cxplvrcase0\\cxplvrspc _}");
}

#[test]
fn test_par() {
  check_tl!("{#return}{#return}", "\\par\\s0 ");
  check_tl!("{#return}{#return}    ", "\\par\\s1 ");
}

#[test]
fn test_key_combo() {
  // \cxplvrkey: send KEY combo
  check_tl!("{#Left}", "{\\*\\cxplvrkey Left}");
  check_tl!("{# ctrl_l(tab)}", "{\\*\\cxplvrkey ctrl_l(tab)}");
}

#[test]
fn test_punct() {
  // \cxp: Punctuation
  check_tl!("{.}", "{\\cxp. }");
  check_tl!("{,}", "{\\cxp, }");
  check_tl!("{:stop:.}", "{\\cxp. }");
  check_tl!("{:comma:,}", "{\\cxp, }");
}

#[test]
fn test_attach() {
  // \cxds: Delete Space
  check_tl!("{^}", "\\cxds ");
  check_tl!("{^ing}", "\\cxds ing");
  check_tl!("{pre^}", "pre\\cxds ");
  check_tl!("{^...^}", "\\cxds ...\\cxds ");

  check_tl!("{:attach}", "\\cxds ");
  check_tl!("{:attach:^ing}", "\\cxds ing");
  check_tl!("{:attach:pre^}", "pre\\cxds ");
  check_tl!("{:attach:...}", "\\cxds ...\\cxds ");

  check_tl!("{^ ^}", "\\~");
}

#[test]
fn test_glue() {
  // \cxfing: FINGerspell
  check_tl!("{&a}", "{\\cxfing a}");
  check_tl!("{&th}", "{\\cxfing th}");

  check_tl!("{:glue:a}", "{\\cxfing a}");
  check_tl!("{:glue:th}", "{\\cxfing th}");
}

#[test]
fn test_stitch() {
  // \cxstit: STITch
  check_tl!("{:stitch:a}", "{\\cxstit a}");
  check_tl!("{:stitch:L:-}", "{\\cxstit L}");
}

#[test]
fn test_force_cap() {
  // \cxfc: Force Capitalize
  // \cxfl: Force Lowercase
  // \cxplvrrtfc: ReTroactive Force Capitalize
  // \cxplvrrtfl: ReTroactive Force Lowercase
  // \cxplvrfcw: Force Capitalize next Word
  // \cxplvrrtfcw: ReTroactive Force Capitalize last Word
  check_tl!("{-|}", "\\cxfc ");
  check_tl!("{*-|}", "{\\*\\cxplvrrtfc}");
  check_tl!("{<}", "{\\*\\cxplvrfcw}");
  check_tl!("{*<}", "{\\*\\cxplvrrtfcw}");
  check_tl!("{>}", "\\cxfl ");
  check_tl!("{*>}", "{\\*\\cxplvrrtfl}");

  check_tl!("{:case:cap_first_word}", "\\cxfc ");
  check_tl!("{:retro_case:cap_first_word}", "{\\*\\cxplvrrtfc}");
  check_tl!("{:case:upper_first_word}", "{\\*\\cxplvrfcw}");
  check_tl!("{:retro_case:upper_first_word}", "{\\*\\cxplvrrtfcw}");
  check_tl!("{:case:lower_first_char}", "\\cxfl ");
  check_tl!("{:retro_case:lower_first_char}", "{\\*\\cxplvrrtfl}");
}

#[test]
fn test_carry_cap() {
  // \cxplvrccap: Carry CAPitalization
  check_tl!("{~|^-^}", "{\\*\\cxplvrccap}\\cxds -\\cxds ");
  check_tl!("{~|^-esque}", "{\\*\\cxplvrccap}\\cxds -esque");
  check_tl!("{~|un-^}", "{\\*\\cxplvrccap}un-\\cxds ");
  check_tl!("{~|5}", "{\\*\\cxplvrccap}5");

  check_tl!("{:carry_capitalize:^-^}", "{\\*\\cxplvrccap}\\cxds -\\cxds ");
  check_tl!("{:carry_capitalize:^-esque}", "{\\*\\cxplvrccap}\\cxds -esque");
  check_tl!("{:carry_capitalize:un-^}", "{\\*\\cxplvrccap}un-\\cxds ");
  check_tl!("{:carry_capitalize:5}", "{\\*\\cxplvrccap}5");
}

#[test]
fn test_currency() {
  // \cxplvrcurr: format CURRency
  check_tl!("{*($c)}", "{\\*\\cxplvrcurr $c}");
  check_tl!("{:retro_currency:$c}", "{\\*\\cxplvrcurr $c}");
  check_tl!("{*(c $)}", "{\\*\\cxplvrcurr c $}");
  check_tl!("{:retro_currency:c $}", "{\\*\\cxplvrcurr c $}");
}

#[test]
fn test_newline() {
  check_tl!("{^\\n^}", "\\cxds \\n\\cxds ");
  check_tl!("{^\\t^}", "\\cxds \\t\\cxds ");
}

#[test]
fn test_concat() {
  check_tl!("mooo啦!{#}test{*}", "mooo\\u21862 !{\\*\\cxplvrnop}test{\\*\\cxplvrast}");
}
