use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::{alpha1, digit1};
use nom::combinator::{opt, recognize};
use nom::multi::separated_list1;
use nom::sequence::tuple;

use std::char::from_u32;
use regex::Regex;
use lazy_static::lazy_static;

use crate::translation_model::{Case, ParagraphMode, Object};

fn number(input: &str) -> IResult<&str, i32> {
  let (input, num) = recognize(tuple((opt(tag("-")), digit1)))(input)?;
  Ok((input, num.parse::<i32>().unwrap()))
}

fn positive_number(input: &str) -> IResult<&str, u32> {
  let (input, num) = digit1(input)?;
  Ok((input, num.parse::<u32>().unwrap()))
}

fn long_group(input: &str) -> IResult<&str, Object> {
  let (input, arg) = alt((
    tag("{\\*\\cxplvrcase0\\cxplvrspc0}"),
    tag("{\\*\\cxplvrcase4\\cxplvrspc}"),
    tag("{\\*\\cxplvrcase0\\cxplvrspc _}")))(input)?;
  Ok((input, match arg {
    "{\\*\\cxplvrcase0\\cxplvrspc0}" => Object::ResetCaseAndSpace,
    "{\\*\\cxplvrcase4\\cxplvrspc}" => Object::CaseMode(Case::Camel),
    "{\\*\\cxplvrcase0\\cxplvrspc _}" => Object::CaseMode(Case::Snake),
    _ => Object::RawString("".to_string()),
  }))
}

fn no_arg_group(input: &str) -> IResult<&str, Object> {
  let (input, (_, label, number, _)) = tuple((
    tag("{\\*\\"), recognize(tuple((tag("cxplvr"), alpha1))), opt(number), tag("}")))(input)?;
  Ok((input, match label {
    "cxplvrnop" => Object::Noop,
    "cxplvrcancel" => Object::Cancel,
    "cxplvrast" => Object::RetroToggleStar,
    "cxplvrrpt" => Object::RepeatLastStroke,
    "cxplvrrtisp" => Object::RetroInsertSpace,
    "cxplvrrtdsp" => Object::RetroDeleteSpace,
    "cxplvrcase" => Object::CaseMode(match number {
      Some(0) => Case::Sentence,
      Some(1) => Case::Lower,
      Some(2) => Case::Upper,
      Some(3) => Case::Title,
      _ => Case::Sentence,
    }),
    "cxplvrccap" => Object::CarryCapRaw("".to_string()),
    "cxplvrrtfc" => Object::RetroForceCapitalize,
    "cxplvrrtfl" => Object::RetroForceLowercase,
    "cxplvrfcw" => Object::ForceCapitalizeWord,
    "cxplvrrtfcw" => Object::RetroForceCapitalizeWord,
    "cxplvrspc" => Object::SpaceMode(None),
    "cxplvrortho" => Object::OrthoAttach,
    _ => Object::RawString("".to_string()),
  }))
}

fn arg_group(input: &str) -> IResult<&str, Object> {
  let (input, (_, label, _, _, arg, _)) = tuple((
    tag("{\\*\\"), recognize(tuple((tag("cxplvr"), alpha1))), opt(number), tag(" "), take_until("}"), tag("}")))(input)?;

  macro_rules! object {
    ($type:expr) => {
      if arg.contains(":") {
        let parts = arg.splitn(2, |c| c == ':').collect::<Vec<&str>>();
        $type(parts[0].to_string(), Some(parts[1].to_string()))
      } else { $type(arg.to_string(), None) }
    }
  }

  Ok((input, match label {
    "cxplvrmeta" => object!(Object::Meta),
    "cxplvrmac" => object!(Object::Macro),
    "cxplvrcmd" => object!(Object::Command),
    "cxplvrspc" => Object::SpaceMode(Some(arg.to_string())),
    "cxplvrkey" => Object::KeyCombo(arg.to_string()),
    "cxplvrcurr" => {
      if arg.contains("c") {
        let parts = arg.splitn(2, |c| c == 'c').collect::<Vec<&str>>();
        Object::Currency(
          match parts[0] { x if x == "" => None, x => Some(x.to_string()) },
          match parts[1] { x if x == "" => None, x => Some(x.to_string()) })
      } else { Object::RawString("".to_string()) }
    },
    _ => Object::RawString("".to_string()),
  }))
}

fn punc_group(input: &str) -> IResult<&str, Object> {
  let (input, (_, _, punct, _)) = tuple((
    tag("{\\cxp"), opt(tag(" ")), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::Punctuation(punct[..punct.len() - 1].to_string())))
}

fn fing_group(input: &str) -> IResult<&str, Object> {
  let (input, (_, letters, _)) = tuple((
    tag("{\\cxfing "), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::Fingerspell(letters.to_string())))
}

fn cxc_group(input: &str) -> IResult<&str, &str> {
  let (input, (_, letters, _)) = tuple((
    tag("{\\cxc "), take_until("}"), tag("}")))(input)?;
  Ok((input, letters))
}

fn conf_group(input: &str) -> IResult<&str, Object> {
  let (input, (_, groups, _)) = tuple((
    tag("{\\cxconf ["), separated_list1(tag("|"), cxc_group), tag("]}")))(input)?;
  Ok((input, Object::RawString(groups[groups.len() - 1].to_string())))
}

fn stit_group(input: &str) -> IResult<&str, Object> {
  let (input, (_, letters, _)) = tuple((
    tag("{\\cxstit "), take_until("}"), tag("}")))(input)?;
  Ok((input, Object::Stitch(letters.to_string())))
}

fn par(input: &str) -> IResult<&str, Object> {
  let (input, (_, style, _)) = tuple((tag("\\par\\s"), number, opt(tag(" "))))(input)?;
  Ok((input, Object::Paragraph(match style {
    1 => ParagraphMode::Contin,
    _ => ParagraphMode::Default,
  })))
}

fn dstroke(input: &str) -> IResult<&str, Object> {
  let (input, _) = tuple((tag("\\cxdstroke"), opt(tag(" "))))(input)?;
  Ok((input, Object::DeleteStroke))
}

fn fl_fc(input: &str) -> IResult<&str, Object> {
  let (input, (_, mode, _)) = tuple((tag("\\cxf"),
    alt((tag("l"), tag("c"))), opt(tag(" "))))(input)?;
  Ok((input, match mode {
    "l" => Object::ForceLowercase,
    "c" => Object::ForceCapitalize,
    _ => Object::RawString("".to_string()),
  }))
}

fn dspace(input: &str) -> IResult<&str, Object> {
  let (input, _) = tuple((tag("\\cxds"), opt(tag(" "))))(input)?;
  Ok((input, Object::AttachRaw))
}

fn unicode(input: &str) -> IResult<&str, Object> {
  let (input, (_, unicode, _)) = tuple((tag("\\u"), positive_number, opt(tag(" "))))(input)?;
  Ok((input, Object::RawString(from_u32(unicode).unwrap().to_string())))
}

fn hyphen(input: &str) -> IResult<&str, Object> {
  let (input, _) = tag("\\_")(input)?;
  Ok((input, Object::RawString("-".to_string())))
}

fn escaped(input: &str) -> IResult<&str, Object> {
  let (input, text) = recognize(tuple((tag("\\"),
    alt((tag("\\"), tag("{"), tag("}"), tag("~"), tag("_"))))))(input)?;
  Ok((input, match text {
    "\\~" => Object::HardSpace,
    "\\_" => Object::RawString("-".to_string()),
    _ => Object::RawString(text.to_string()),
  }))
}

fn newline(input: &str) -> IResult<&str, Object> {
  let (input, text) = recognize(tuple((tag("\\"), alt((tag("n"), tag("t"))))))(input)?;
  Ok((input, Object::RawString(text.to_string())))
}

fn plain_text(input: &str) -> IResult<&str, Object> {
  let (input, text) = take_while(|c| c != '\\' && c != '{')(input)?;
  Ok((input, Object::RawString(text.to_string())))
}

fn objects(input: &str) -> IResult<&str, Vec<Object>> {
  if input == "" {
    return Ok(("", vec![]));
  }

  let parsers = (
    long_group,
    arg_group,
    no_arg_group,
    punc_group,
    fing_group,
    stit_group,
    conf_group,
    par,
    dstroke,
    fl_fc,
    dspace,
    escaped,
    hyphen,
    unicode,
    newline,
    plain_text,
  );
  let (input, (first, rest)) = tuple((alt(parsers), opt(objects)))(input)?;
  let mut items = vec![first];
  if let Some(mut r) = rest {
    items.append(&mut r);
  }
  Ok((input, items))
}

fn parse_translation(input: &str) -> Vec<Object> {
  match objects(input) { Ok((_, a)) => a, _ => vec![] }
}

fn fix_attach(translation: String) -> String {
  macro_rules! regex {
    ($re:literal) => { Regex::new($re).unwrap() };
  }
  lazy_static! {
    static ref INFIX: Regex = regex!(r"^\{\^\}([^\{]+?)\{\^\}$");
    static ref PREFIX: Regex = regex!(r"^([^\{]+?)\{\^\}$");
    static ref SUFFIX: Regex = regex!(r"^\{\^\}([^\{]+?)$");
    static ref CARRY_CAP_INFIX: Regex = regex!(r"^\{~\|\}\{\^\}([^\{]+?)\{\^\}$");
    static ref CARRY_CAP_PREFIX: Regex = regex!(r"^\{~\|\}([^\{]+?)\{\^\}$");
    static ref CARRY_CAP_SUFFIX: Regex = regex!(r"^\{~\|\}\{\^\}([^\{]+?)$");
    static ref CARRY_CAP: Regex = regex!(r"^\{~\|\}([^\{]+?)$");
  }

  macro_rules! sub {
    ($regex:expr, $replacement:expr, $tl:expr) => {
      $regex.replace(&$tl, $replacement)
    };
  }

  sub!(CARRY_CAP, "{~|$1}",
    sub!(CARRY_CAP_SUFFIX, "{~|^$1}",
      sub!(CARRY_CAP_PREFIX, "{~|$1^}",
        sub!(CARRY_CAP_INFIX, "{~|^$1^}",
          sub!(SUFFIX, "{^$1}",
            sub!(PREFIX, "{$1^}",
              sub!(INFIX, "{^$1^}",
                translation))))))).to_string()
}

pub fn format_rtf_to_plover(tl: &str) -> String {
  let mut ortho_attach = false;

  let items = parse_translation(tl).iter()
    .map(|obj| {
      match obj {
        Object::Paragraph(mode) => format!("{{#return}}{{#return}}{}",
          match mode { ParagraphMode::Default => "", ParagraphMode::Contin => "    " }),
        Object::Fingerspell(letters) => format!("{{&{}}}", letters),
        Object::Stitch(letters) => format!("{{:stitch:{}}}", letters),
        Object::Command(cmd, Some(arg)) => format!("{{plover:{}:{}}}", cmd, arg),
        Object::Command(cmd, None) => format!("{{plover:{}}}", cmd),
        Object::Meta(mac, Some(arg)) => format!("{{:{}:{}}}", mac, arg),
        Object::Meta(mac, None) => format!("{{:{}}}", mac),
        Object::Macro(mac, Some(arg)) => format!("={}:{}", mac, arg),
        Object::Macro(mac, None) => format!("={}", mac),
        Object::Currency(left, right) => format!("{{*({}c{})}}",
          match left { Some(x) => x, None => "" },
          match right { Some(x) => x, None => "" }),
        Object::Punctuation(punct) => format!("{{{}}}", punct),
        Object::KeyCombo(key) => format!("{{#{}}}", key),
        Object::CaseMode(mode) => format!("{{mode:{}}}", match mode {
          Case::Sentence => "reset_case",
          Case::Lower => "lower",
          Case::Upper => "caps",
          Case::Title => "title",
          Case::Camel => "camel",
          Case::Snake => "snake",
        }),
        Object::SpaceMode(Some(space)) => format!("{{mode:set_space:{}}}", space),
        _ => match obj {
          Object::Cancel => "{}",
          Object::Noop => "{#}",
          Object::DeleteStroke => "=undo",
          Object::RepeatLastStroke => "{*+}",
          Object::RetroToggleStar => "{*}",
          Object::RetroInsertSpace => "{*?}",
          Object::RetroDeleteSpace => "{*!}",
          Object::HardSpace => "{^ ^}",
          Object::RawString(string) => string,
          Object::ResetCaseAndSpace => "{mode:reset}",
          Object::SpaceMode(None) => "{mode:reset_space}",
          Object::AttachRaw => "{^}",
          Object::OrthoAttach => {
            ortho_attach = true;
            ""
          },
          Object::CarryCapRaw(_) => "{~|}",
          Object::ForceCapitalize => "{-|}",
          Object::ForceLowercase => "{>}",
          Object::RetroForceCapitalize => "{*-|}",
          Object::RetroForceLowercase => "{*>}",
          Object::ForceCapitalizeWord => "{<}",
          Object::RetroForceCapitalizeWord => "{*<}",
          _ => "",
        }.to_string(),
      }
    })
    .collect::<Vec<String>>()
    .join("");

  if ortho_attach {
    fix_attach(items)
  } else {
    items
  }
}
