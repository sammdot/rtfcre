use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_till, take_until, take_while};
use nom::character::complete::space1;
use nom::combinator::opt;
use nom::sequence::tuple;

use crate::translation_model::{Object, Case, ParagraphMode};

macro_rules! opt {
  ($i:expr) => {
    match $i {
      Some("") => None,
      Some(x) => Some(x.to_string()),
      None => None,
    }
  }
}

fn escaped(input: &str) -> IResult<&str, Object> {
  let (input, arg) = alt((tag("\\\\"), tag("\\{"), tag("\\}")))(input)?;
  Ok((input, Object::RawString(arg[1..].to_string())))
}

fn cancel(input: &str) -> IResult<&str, Object> {
  let (input, _) = tag("{}")(input)?;
  Ok((input, Object::Cancel))
}

fn noop(input: &str) -> IResult<&str, Object> {
  let (input, _) = tag("{#}")(input)?;
  Ok((input, Object::Noop))
}

fn spaces(input: &str) -> IResult<&str, Object> {
  let (input, (_, _, _)) = tuple((tag("{"), space1, tag("}")))(input)?;
  Ok((input, Object::Space))
}

fn argument(input: &str) -> IResult<&str, &str> {
  let (input, (_, arg)) = tuple((tag(":"), take_while(|c| c != '}')))(input)?;
  Ok((input, arg))
}

fn command(input: &str) -> IResult<&str, Object> {
  let (input, (_, cmd_name, cmd_arg, _)) = tuple((
    tag_no_case("{plover:"), take_till(|c| c == ':' || c == '}'), opt(argument), tag("}")))(input)?;
  Ok((input, Object::Command(cmd_name.to_lowercase(), opt!(cmd_arg))))
}

fn meta(input: &str) -> IResult<&str, Object> {
  let (input, (_, meta_name, meta_arg, _)) = tuple((
    tag("{:"), take_till(|c| c == ':' || c == '}'), opt(argument), tag("}")))(input)?;
  Ok((input, match meta_name.to_lowercase().as_str() {
    "glue" => Object::Fingerspell(opt!(meta_arg).unwrap()),
    "stop" | "comma" => Object::Punctuation(opt!(meta_arg).unwrap()),
    "key_combo" => Object::KeyCombo(opt!(meta_arg).unwrap()),
    "case" => match opt!(meta_arg).unwrap().as_str() {
      "cap_first_word" => Object::ForceCapitalize,
      "upper_first_word" => Object::ForceCapitalizeWord,
      "lower_first_char" => Object::ForceLowercase,
      arg @ _ => Object::Meta(meta_name.to_lowercase(), Some(arg.to_string())),
    },
    "attach" => match meta_arg {
      Some(x) if x == " " => Object::HardSpace,
      Some(x) if x.starts_with("^") => Object::AttachSuffix(x[1..].to_string()),
      Some(x) if x.ends_with("^") => Object::AttachPrefix(x[..x.len() - 1].to_string()),
      Some(x) => Object::AttachInfix(x.to_string()),
      None => Object::AttachRaw,
    },
    "carry_capitalize" => match meta_arg {
      Some(x) if x.starts_with("^") && x.ends_with("^") =>
        Object::CarryCapInfix(x[1..x.len() - 1].to_string()),
      Some(x) if x.starts_with("^") => Object::CarryCapSuffix(x[1..].to_string()),
      Some(x) if x.ends_with("^") => Object::CarryCapPrefix(x[..x.len() - 1].to_string()),
      Some(x) => Object::CarryCapRaw(x.to_string()),
      None => Object::CarryCapRaw("".to_string()),
    },
    "retro_case" => match opt!(meta_arg).unwrap().as_str() {
      "cap_first_word" => Object::RetroForceCapitalize,
      "upper_first_word" => Object::RetroForceCapitalizeWord,
      "lower_first_char" => Object::RetroForceLowercase,
      arg @ _ => Object::Meta(meta_name.to_lowercase(), Some(arg.to_string())),
    }
    "stitch" => {
      let arg = opt!(meta_arg).unwrap();
      Object::Stitch(
        if arg.contains(":") {
          // TODO: Implement support for delimiters if/when Plover
          // supports stitching natively
          let parts: Vec<&str> = arg.split(":").collect();
          parts[0].to_string()
        } else { arg })
    },
    "command" => {
      let arg = opt!(meta_arg).unwrap();
      if arg.contains(":") {
        let parts: Vec<&str> = arg.split(":").collect();
        Object::Command(parts[0].to_lowercase(), Some(parts[1].to_string()))
      } else {
        Object::Command(arg.to_lowercase(), None)
      }
    },
    "mode" => match opt!(meta_arg).unwrap().as_str() {
      "reset_case" => Object::CaseMode(Case::Sentence),
      "lower" => Object::CaseMode(Case::Lower),
      "title" => Object::CaseMode(Case::Title),
      "caps" => Object::CaseMode(Case::Upper),
      "camel" => Object::CaseMode(Case::Camel),
      "snake" => Object::CaseMode(Case::Snake),
      "reset_space" => Object::SpaceMode(None),
      "reset" => Object::ResetCaseAndSpace,
      x if x.starts_with("set_space:") =>
        Object::SpaceMode(Some(x["set_space:".len()..].to_string())),
      _ => Object::Meta(meta_name.to_lowercase(), opt!(meta_arg)),
    },
    _ => Object::Meta(meta_name.to_lowercase(), opt!(meta_arg)),
  }))
}

fn mode_space(input: &str) -> IResult<&str, Object> {
  let (input, (_, space, _)) = tuple((
    tag_no_case("{mode:set_space:"), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::SpaceMode(match space {
    " " => None,
    _ => Some(space.to_string()),
  })))
}

fn mode(input: &str) -> IResult<&str, Object> {
  let (input, (_, mode, _)) = tuple((
    tag_no_case("{mode:"), take_until("}"), tag("}")))(input)?;
  Ok((input, match mode.to_lowercase().as_str() {
    "reset_case" => Object::CaseMode(Case::Sentence),
    "lower" => Object::CaseMode(Case::Lower),
    "title" => Object::CaseMode(Case::Title),
    "caps" => Object::CaseMode(Case::Upper),
    "camel" => Object::CaseMode(Case::Camel),
    "snake" => Object::CaseMode(Case::Snake),
    "reset_space" => Object::SpaceMode(None),
    "reset" => Object::ResetCaseAndSpace,
    _ => Object::RawString("".to_string()),
  }))
}

fn prefix_attach(input: &str) -> IResult<&str, Object> {
  let (input, (_, string, _)) = tuple((
    tag("{"), take_until("^"), tag("^}")))(input)?;
  Ok((input, Object::AttachPrefix(string.to_string())))
}

fn suffix_attach(input: &str) -> IResult<&str, Object> {
  let (input, (_, string, _)) = tuple((
    tag("{^"), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::AttachSuffix(string.to_string())))
}

fn infix_attach(input: &str) -> IResult<&str, Object> {
  let (input, (_, string, _)) = tuple((
    tag("{^"), take_until("^"), tag("^}")))(input)?;
  Ok((input, match string {
    " " => Object::HardSpace,
    _ => Object::AttachInfix(string.to_string()),
  }))
}

fn prefix_carry_cap(input: &str) -> IResult<&str, Object> {
  let (input, (_, string, _)) = tuple((
    tag("{~|"), take_until("^"), tag("^}")))(input)?;
  Ok((input, Object::CarryCapPrefix(string.to_string())))
}

fn suffix_carry_cap(input: &str) -> IResult<&str, Object> {
  let (input, (_, string, _)) = tuple((
    tag("{~|^"), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::CarryCapSuffix(string.to_string())))
}

fn infix_carry_cap(input: &str) -> IResult<&str, Object> {
  let (input, (_, string, _)) = tuple((
    tag("{~|^"), take_until("^"), tag("^}")))(input)?;
  Ok((input, Object::CarryCapInfix(string.to_string())))
}

fn raw_carry_cap(input: &str) -> IResult<&str, Object> {
  let (input, (_, string, _)) = tuple((
    tag("{~|"), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::CarryCapRaw(string.to_string())))
}

fn glue(input: &str) -> IResult<&str, Object> {
  let (input, (_, glue, _)) = tuple((
    tag("{&"), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::Fingerspell(glue.to_string())))
}

fn key_combo(input: &str) -> IResult<&str, Object> {
  let (input, (_, combo, _)) = tuple((
    tag("{#"), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::KeyCombo(combo.to_string())))
}

fn punctuation(input: &str) -> IResult<&str, Object> {
  let (input, (_, arg, _)) = tuple((
    tag("{"), alt(
      (tag("..."), tag("--"), tag("-"),
        tag("."), tag(","), tag(":"), tag(";"), tag("?"), tag("!"))), tag("}")))(input)?;
  Ok((input, Object::Punctuation(arg.to_string())))
}

fn operator(input: &str) -> IResult<&str, Object> {
  let (input, (_, oper, _)) = tuple((
    tag("{"), alt((
      tag("^"), tag("-|"),
      tag("*-|"), tag("*+"), tag("*?"), tag("*!"),
      tag("*<"), tag("*>"), tag("*"), tag("<"), tag(">"),
      tag("|"), tag("'"), tag("l+"), tag("l-"),
      )), tag("}")))(input)?;
  Ok((input, match oper {
    "^" => Object::AttachRaw,
    "*" => Object::RetroToggleStar,
    "*+" => Object::RepeatLastStroke,
    "*?" => Object::RetroInsertSpace,
    "*!" => Object::RetroDeleteSpace,
    "-|" | "l-" => Object::ForceCapitalize,
    "*-|" => Object::RetroForceCapitalize,
    "<" => Object::ForceCapitalizeWord,
    "*<" => Object::RetroForceCapitalizeWord,
    ">" | "l+" => Object::ForceLowercase,
    "*>" => Object::RetroForceLowercase,
    _ => Object::RawString(oper.to_string()),
  }))
}

fn par(input: &str) -> IResult<&str, Object> {
  let (input, (_, space)) = tuple((tag_no_case("{#return}{#return}"), opt(tag("    "))))(input)?;
  Ok((input, match space {
    None => Object::Paragraph(ParagraphMode::Default),
    _ => Object::Paragraph(ParagraphMode::Contin),
  }))
}

fn currency_meta(input: &str) -> IResult<&str, Object> {
  let (input, (_, pre_symbols, _, post_symbols, _)) = tuple((
    tag("{:retro_currency:"), take_until("c"), tag("c"), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::Currency(
    match pre_symbols {
      "" => None,
      x => Some(x.to_string()),
    },
    match post_symbols {
      "" => None,
      x => Some(x.to_string()),
    })))
}

fn currency(input: &str) -> IResult<&str, Object> {
  let (input, (_, pre_symbols, _, post_symbols, _)) = tuple((
    tag("{*("), take_until("c"), tag("c"), take_until(")"), tag(")}")))(input)?;
  Ok((input, Object::Currency(
    match pre_symbols {
      "" => None,
      x => Some(x.to_string()),
    },
    match post_symbols {
      "" => None,
      x => Some(x.to_string()),
    })))
}

fn anything_between_braces(input: &str) -> IResult<&str, Object> {
  let (input, (_, string, _)) = tuple((
    tag("{"), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::RawString(string.to_string())))
}

fn raw(input: &str) -> IResult<&str, Object> {
  let (input, string) = take_till(|c| c == '{' || c == '\\')(input)?;
  Ok((input, Object::RawString(string.to_string())))
}

fn macro_(input: &str) -> IResult<&str, Vec<Object>> {
  let (input, (_, macro_name, macro_arg)) = tuple((
    tag("="), take_till(|c| c == ':'), opt(argument)))(input)?;
  Ok((input, vec![match macro_name.to_lowercase().as_str() {
    "undo" => Object::DeleteStroke,
    "repeat_last_stroke" => Object::RepeatLastStroke,
    "retrospective_toggle_asterisk" => Object::RetroToggleStar,
    "retrospective_insert_space" => Object::RetroInsertSpace,
    "retrospective_delete_space" => Object::RetroDeleteSpace,
    _ => Object::Macro(macro_name.to_lowercase(), opt!(macro_arg)),
  }]))
}

fn rest(input: &str) -> IResult<&str, Vec<Object>> {
  if input == "" {
    return Ok(("", vec![]));
  }

  let parsers = (
    escaped,
    spaces,
    cancel,
    noop,
    par,
    command,
    mode_space,
    mode,
    glue,
    currency,
    currency_meta,
    key_combo,
    punctuation,
    operator,
    meta,
    alt((infix_carry_cap, prefix_carry_cap, suffix_carry_cap, raw_carry_cap)),
    alt((infix_attach, suffix_attach, prefix_attach)),
    anything_between_braces,
    raw,
  );
  let (input, (first, rest)) = tuple((alt(parsers), opt(rest)))(input)?;
  let mut items = vec![first];
  if let Some(mut r) = rest {
    items.append(&mut r);
  }
  Ok((input, items))
}

fn parse_translation(input: &str) -> Vec<Object> {
  match alt((macro_, rest))(input) { Ok((_, a)) => a, _ => vec![] }
}

pub fn format_plover_to_rtf(tl: &str) -> String {
  parse_translation(tl).iter()
    .map(|obj| {
      match obj {
        Object::RawString(string) =>
          string.chars().map(|c|
            match c {
              '{' | '}' | '\\' => format!("\\{}", c),
              '-' => "\\_".to_string(),
              '\n' => "\\n".to_string(),
              '\t' => "\\t".to_string(),
              c if (c as u32) > 255 => format!("\\u{} ", (c as u32)),
              c => c.to_string(),
            }
          ).collect::<Vec<String>>().join(""),
        Object::Command(name, None) => format!("{{\\*\\cxplvrcmd {}}}", name),
        Object::Command(name, Some(arg)) => format!("{{\\*\\cxplvrcmd {}:{}}}", name, arg),
        Object::Meta(name, None) => format!("{{\\*\\cxplvrmeta {}}}", name),
        Object::Meta(name, Some(arg)) => format!("{{\\*\\cxplvrmeta {}:{}}}", name, arg),
        Object::Macro(name, None) => format!("{{\\*\\cxplvrmac {}}}", name),
        Object::Macro(name, Some(arg)) => format!("{{\\*\\cxplvrmac {}:{}}}", name, arg),
        Object::Punctuation(punct) => format!("{{\\cxp{} }}", punct),
        Object::SpaceMode(Some(x)) if x.as_str() != " " => format!("{{\\*\\cxplvrspc {}}}", x),
        Object::KeyCombo(keys) => format!("{{\\*\\cxplvrkey {}}}", keys.trim()),
        Object::Fingerspell(string) => format!("{{\\cxfing {}}}", string),
        Object::Stitch(string) => format!("{{\\cxstit {}}}", string),
        Object::AttachSuffix(string) => format!("{{\\*\\cxplvrortho}}\\cxds {}", string),
        Object::AttachPrefix(string) => format!("{{\\*\\cxplvrortho}}{}\\cxds ", string),
        Object::AttachInfix(string) => format!("{{\\*\\cxplvrortho}}\\cxds {}\\cxds ", string),
        Object::CarryCapRaw(string) => format!("{{\\*\\cxplvrccap}}{}", string),
        Object::CarryCapSuffix(string) => format!("{{\\*\\cxplvrccap}}\\cxds {}", string),
        Object::CarryCapPrefix(string) => format!("{{\\*\\cxplvrccap}}{}\\cxds ", string),
        Object::CarryCapInfix(string) => format!("{{\\*\\cxplvrccap}}\\cxds {}\\cxds ", string),
        Object::Currency(left, right) =>
          format!("{{\\*\\cxplvrcurr {}c{}}}",
            match left { Some(x) => x, None => "" },
            match right { Some(x) => x, None => "" }),
        _ => match obj {
          Object::Noop => "{\\*\\cxplvrnop}",
          Object::Cancel => "{\\*\\cxplvrcancel}",
          Object::DeleteStroke => "\\cxdstroke ",
          Object::RepeatLastStroke => "{\\*\\cxplvrrpt}",
          Object::RetroToggleStar => "{\\*\\cxplvrast}",
          Object::RetroInsertSpace => "{\\*\\cxplvrrtisp}",
          Object::RetroDeleteSpace => "{\\*\\cxplvrrtdsp}",
          Object::ForceCapitalize => "\\cxfc ",
          Object::RetroForceCapitalize => "{\\*\\cxplvrrtfc}",
          Object::ForceCapitalizeWord => "{\\*\\cxplvrfcw}",
          Object::RetroForceCapitalizeWord => "{\\*\\cxplvrrtfcw}",
          Object::ForceLowercase => "\\cxfl ",
          Object::RetroForceLowercase => "{\\*\\cxplvrrtfl}",
          Object::CaseMode(case @ _) => match case {
            Case::Sentence => "{\\*\\cxplvrcase0}",
            Case::Lower => "{\\*\\cxplvrcase1}",
            Case::Upper => "{\\*\\cxplvrcase2}",
            Case::Title => "{\\*\\cxplvrcase3}",
            Case::Camel => "{\\*\\cxplvrcase4\\cxplvrspc}",
            Case::Snake => "{\\*\\cxplvrcase0\\cxplvrspc _}",
          }
          Object::SpaceMode(Some(x)) if x.as_str() == " " => "{{\\*\\cxplvrspc0}",
          Object::SpaceMode(None) => "{\\*\\cxplvrspc0}",
          Object::ResetCaseAndSpace => "{\\*\\cxplvrcase0\\cxplvrspc0}",
          Object::AttachRaw => "\\cxds ",
          Object::Paragraph(ParagraphMode::Default) => "\\par\\s0 ",
          Object::Paragraph(ParagraphMode::Contin) => "\\par\\s1 ",
          Object::Space => " ",
          Object::HardSpace => "\\~",
          _ => "",
        }.to_string()
      }
    })
    .collect::<Vec<String>>()
    .join("")
}
