use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{alpha1, digit1, multispace0, one_of};
use nom::combinator::{opt, recognize};
use nom::multi::{many0, many1, many_till};
use nom::sequence::tuple;

use crate::dict::Dictionary;
use crate::translation_parse::format_rtf_to_plover;

use std::char::from_u32;

fn unsigned(input: &str) -> IResult<&str, u32> {
  let (input, num) = digit1(input)?;
  Ok((input, num.parse::<u32>().unwrap()))
}

fn integer(input: &str) -> IResult<&str, i32> {
  let (input, num) = recognize(tuple((tag("-"), digit1)))(input)?;
  Ok((input, num.parse::<i32>().unwrap()))
}

fn unicode(input: &str) -> IResult<&str, String> {
  let (input, (_, code, _)) = tuple((
    tag("\\u"), unsigned, opt(tag(" "))))(input)?;
  Ok((input, from_u32(code).unwrap().to_string()))
}

fn control_word(input: &str) -> IResult<&str, String> {
  let (input, cw) = recognize(tuple((
    tag("\\"), alpha1, opt(integer), opt(tag(" ")))))(input)?;
  Ok((input, cw.to_string()))
}

fn control_symbol(input: &str) -> IResult<&str, String> {
  let (input, cs) = recognize(tuple((
    tag("\\"),
    one_of("\\{}*_~"))))(input)?;
  Ok((input, cs.to_string()))
}

fn text(input: &str) -> IResult<&str, String> {
  let (input, text) = recognize(many1(is_not("\\{}")))(input)?;
  Ok((input, text.to_string()))
}

fn group(input: &str) -> IResult<&str, String> {
  let (input, (l, grp, r)) = tuple((
    tag("{"),
    many1(alt((group, unicode, control_word, control_symbol, text))),
    tag("}")))(input)?;
  Ok((input, format!("{}{}{}", l, grp.join(""), r)))
}

fn steno_group(input: &str) -> IResult<&str, String> {
  let (input, (steno, _)) = tuple((
    many1(alt((unicode, control_word, control_symbol, text))),
    tag("}")))(input)?;
  Ok((input, steno.join("")))
}

#[derive(Debug)]
enum TranslationItem {
  Comment(String),
  NotComment(String),
}

fn cxcomment(input: &str) -> IResult<&str, TranslationItem> {
  let (input, (_, comment, _)) = tuple((
    tag(r"{\*\cxcomment "),
    many0(alt((group, unicode, control_word, control_symbol, text))),
    tag("}")))(input)?;
  Ok((input, TranslationItem::Comment(comment.join(""))))
}

fn non_comment(input: &str) -> IResult<&str, TranslationItem> {
  let (input, item) =
    alt((group, unicode, control_word, control_symbol, text))(input)?;
  Ok((input, TranslationItem::NotComment(item)))
}

fn steno_entry(input: &str) -> IResult<&str, (String, String, Option<String>)> {
  let (input, (steno_group, (contents, _))) = tuple((
    steno_group,
    many_till(
      alt((cxcomment, non_comment)),
      tag(r"{\*\cxs "))))(input)?;
  let translation = contents.iter()
    .map(|obj| match obj { TranslationItem::NotComment(s) => s.as_str(), _ => "" })
    .collect::<Vec<&str>>().join("").trim().to_string();
  let comment = match contents.iter()
    .map(|obj| match obj { TranslationItem::Comment(s) => s.as_str(), _ => "" })
    .collect::<Vec<&str>>().join("").trim() { "" => None, s => Some(s.to_string()) };
  Ok((input, (steno_group, translation, comment)))
}

fn last_steno_entry(input: &str) -> IResult<&str, (String, String, Option<String>)> {
  let (input, (steno_group, (contents, _))) = tuple((
    steno_group,
    many_till(
      alt((cxcomment, non_comment)),
      tag(r"}"))))(input)?;
  let translation = contents.iter()
    .map(|obj| match obj { TranslationItem::NotComment(s) => s.as_str(), _ => "" })
    .collect::<Vec<&str>>().join("").trim().to_string();
  let comment = match contents.iter()
    .map(|obj| match obj { TranslationItem::Comment(s) => s.as_str(), _ => "" })
    .collect::<Vec<&str>>().join("").trim() { "" => None, s => Some(s.to_string()) };
  Ok((input, (steno_group, translation, comment)))
}

fn cxsystem(input: &str) -> IResult<&str, String> {
  let (input, (_, system, _)) = tuple((
    tag(r"{\*\cxsystem"),
    many0(alt((group, unicode, control_word, control_symbol, text))),
    tag("}")))(input)?;
  Ok((input, system.join("").trim().to_string()))
}

fn no_entries(input: &str) -> IResult<&str, Vec<(String, String, Option<String>)>> {
  let (input, _) = recognize(many_till(alt((group, unicode, control_word, control_symbol, text)), tag(r"}")))(input)?;
  Ok((input, vec![]))
}

fn some_entries(input: &str) -> IResult<&str, Vec<(String, String, Option<String>)>> {
  let (input, (_, mut entries, last_entry)) = tuple((
            recognize(many_till(alt((group, unicode, control_word, control_symbol, text)), tag(r"{\*\cxs "))),
    many0(steno_entry),
    last_steno_entry,
        ))(input)?;
  entries.push(last_entry);
  Ok((input, entries))
}

pub fn parse_file(input: &str) -> IResult<&str, Dictionary> {
  let (input, (_, cxsystem, _, entries)) = tuple((
    recognize(tuple((
      multispace0,
      tag("{"), multispace0, tag(r"\rtf1"), multispace0, tag(r"\ansi"), multispace0,
      tag(r"{\*\cxrev100}"), multispace0, tag(r"\cxdict"), multispace0))),
    cxsystem,
    recognize(tuple((
      multispace0,
      opt(tuple((
        tag(r"{\stylesheet"),
        many1(alt((group, unicode, control_word, control_symbol, text))),
        tag("}")))),
    ))),
    alt((
      some_entries,
      no_entries)),
    ))(input)?;

  let mut dict = Dictionary::new(&cxsystem);
  for (steno, translation, comment) in entries {
    dict.add_entry(steno, format_rtf_to_plover(translation.trim()), comment);
  }
  Ok((input, dict))
}

pub fn parse_rtf(input: &str) -> Option<Dictionary> {
  match parse_file(input.trim()) {
    Ok((_, dict)) => Some(dict),
    Err(_) => None,
  }
}
