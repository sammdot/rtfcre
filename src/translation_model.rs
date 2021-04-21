#[derive(Debug)]
pub enum Case {
  Sentence,
  Lower,
  Upper,
  Title,
  Camel,
  Snake,
}

#[derive(Debug)]
pub enum ParagraphMode {
  Default,
  Contin,
}

#[derive(Debug)]
pub enum Object {
  Cancel,
  Noop,
  DeleteStroke,
  RepeatLastStroke,
  RetroToggleStar,
  RetroInsertSpace,
  RetroDeleteSpace,
  Space,
  HardSpace,
  Paragraph(ParagraphMode),
  RawString(String),
  Fingerspell(String),
  Stitch(String),
  Command(String, Option<String>),
  Meta(String, Option<String>),
  Macro(String, Option<String>),
  Currency(Option<String>, Option<String>),
  Punctuation(String),
  KeyCombo(String),
  ResetCaseAndSpace,
  CaseMode(Case),
  SpaceMode(Option<String>),
  AttachRaw,
  OrthoAttach,
  AttachPrefix(String),
  AttachSuffix(String),
  AttachInfix(String),
  CarryCapRaw(String),
  CarryCapPrefix(String),
  CarryCapSuffix(String),
  CarryCapInfix(String),
  ForceCapitalize,
  ForceLowercase,
  RetroForceCapitalize,
  RetroForceLowercase,
  ForceCapitalizeWord,
  RetroForceCapitalizeWord,
}
