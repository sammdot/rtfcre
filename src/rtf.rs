use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};

use crate::dict::Dictionary;
use crate::translation_parse::format_rtf_to_plover;

fn header(input: &str) -> IResult<&str, &str> {
  let (input, (_, _, _, system_name, _, _)) = tuple((tag("{\\rtf1"), take_until("{\\*\\cxsystem "),
    tag("{\\*\\cxsystem "), take_until("}"), tag("}"), alt((take_until("{\\*\\cxs "), take_until("}")))))(input)?;
  Ok((input, system_name))
}

fn steno_group(input: &str) -> IResult<&str, &str> {
  Ok(delimited(tag("{\\*\\cxs "), is_not("}"), tag("}"))(input)?)
}

fn comment(input: &str) -> IResult<&str, &str> {
  let (input, (_, comment, _)) = tuple((tag("{\\*\\cxcomment "), take_until("}"), tag("}")))(input)?;
  Ok((input, comment))
}

fn non_steno(input: &str) -> IResult<&str, (String, Option<&str>)> {
  let (input, (left, comment, right)) = tuple((
    take_until("{\\*\\cx"), opt(comment), take_until("{\\*\\cxs ")))(input)?;
  Ok((input, (format!("{}{}", left, right), comment)))
}

fn last_non_steno_with_comment(input: &str) -> IResult<&str, (String, Option<&str>)> {
  let (input, (left, _, comment, _, right)) = tuple((
    take_until("{\\*\\cxcomment "), tag("{\\*\\cxcomment "), take_until("}"),
    tag("}"), take_until("}")))(input)?;
  Ok((input, (format!("{}{}", left, right), Some(comment))))
}

fn last_non_steno_without_comment(input: &str) -> IResult<&str, (String, Option<&str>)> {
  let (input, tl) = take_until("}")(input)?;
  Ok((input, (tl.to_string(), None)))
}

fn steno_entry(input: &str) -> IResult<&str, (&str, (String, Option<&str>))> {
  Ok(tuple((steno_group, non_steno))(input)?)
}

fn last_steno_entry(input: &str) -> IResult<&str, (&str, (String, Option<&str>))> {
  Ok(tuple((steno_group,
    alt((last_non_steno_with_comment, last_non_steno_without_comment))))(input)?)
}

pub fn parse_file(input: &str) -> IResult<&str, Dictionary> {
  let mut file = tuple((header, many0(steno_entry), last_steno_entry, tag("}")));
  let (input, (header, mut entries, last_entry, _)) = file(input)?;
  entries.push(last_entry);

  let mut dict = Dictionary::new(header);
  for (steno, (translation, comment)) in entries {
    dict.add_entry(steno.to_string(), format_rtf_to_plover(&translation.trim()),
      match comment { Some(c) => Some(c.to_string()), None => None });
  }

  Ok((input, dict))
}

pub fn parse_rtf(input: &str) -> Option<Dictionary> {
  match parse_file(input.trim()) {
    Ok((_, dict)) => Some(dict),
    Err(_) => None,
  }
}
