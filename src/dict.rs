use std::collections::{HashMap, HashSet};
use std::fmt;
use std::vec::Vec;
use std::result::Result;
use std::io;

use crate::translation::format_plover_to_rtf;

use linked_hash_map::LinkedHashMap;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct EntryMetadata {
  comment: Option<String>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Entry {
  pub steno: String,
  pub translation: String,
  metadata: Option<EntryMetadata>,
}

impl Entry {
  #[allow(dead_code)]
  pub fn new(steno: &str, translation: &str, comment: Option<&str>) -> Self {
    Self {
      steno: String::from(steno),
      translation: String::from(translation),
      metadata: match comment {
        Some(comm) => Some(EntryMetadata { comment: Some(String::from(comm)) }),
        None => None,
      },
    }
  }

  pub fn comment(&self) -> Option<String> {
    Some(self.metadata.as_ref()?.comment.as_ref()?.clone())
  }

  pub fn write(&self, writer: &mut dyn io::Write) -> Result<(), io::Error> {
    write!(writer, "{{\\*\\cxs {}}}{}{}\n",
      self.steno, format_plover_to_rtf(&self.translation),
      match &self.metadata {
        Some(EntryMetadata { comment: Some(comment) }) =>
          format!("{{\\*\\cxcomment {}}}", comment),
        _ => String::from(""),
      })?;
    Ok(())
  }
}

pub struct Dictionary {
  pub cre_system: String,
  pub entries: LinkedHashMap<String, Entry>,
  pub reverse_entries: HashMap<String, HashSet<String>>,
  pub longest_key: usize,
}

static FILE_HEADER: &str = "{\\rtf1\\ansi{\\*\\cxrev100}\\cxdict";
static FILE_HEADER_END: &str = "{\\stylesheet{\\s0 Normal;\\s1 Contin;}}\n";
static FILE_FOOTER: &str = "}\n";

impl Dictionary {
  pub fn new(cre_system: &str) -> Self {
    Self {
      cre_system: String::from(cre_system),
      entries: LinkedHashMap::new(),
      reverse_entries: HashMap::new(),
      longest_key: 0,
    }
  }

  pub fn add_entry(&mut self, steno: String, translation: String, comment: Option<String>) {
    if let Some(entry) = self.entries.get(&steno) {
      // This outline is already defined and being overridden below, remove the
      // corresponding reverse entry so we don't inadvertently return that on
      // reverse lookups
      if let Some(rev_entry) = self.reverse_entries.get_mut(&entry.translation) {
        rev_entry.remove(&steno);
      }
    }

    self.entries.insert(
      steno.clone(),
      Entry {
        steno: steno.clone(),
        translation: translation.clone(),
        metadata: match comment {
          Some(comm) => Some(EntryMetadata { comment: Some(comm) }),
          None => None,
        },
      },
    );

    let key_length = steno.chars().filter(|c| *c == '/').count();
    if key_length > self.longest_key {
      self.longest_key = key_length;
    }

    if let Some(set) = self.reverse_entries.get_mut(&translation) {
      set.insert(steno);
    } else {
      let mut set = HashSet::new();
      set.insert(steno);
      self.reverse_entries.insert(translation, set);
    }
  }

  pub fn remove_entry(&mut self, steno: String) {
    if let Some(entry) = self.entries.get(&steno) {
      if let Some(rev_entry) = self.reverse_entries.get_mut(&entry.translation) {
        rev_entry.remove(&steno);

        if rev_entry.is_empty() {
          self.reverse_entries.remove(&entry.translation);
        }
      }
      self.entries.remove(&steno);
    }
  }

  pub fn contains_key(&self, steno: &str) -> bool {
    self.entries.contains_key(steno)
  }

  pub fn lookup(&self, steno: &str) -> Option<String> {
    Some(self.entries.get(steno)?.translation.clone())
  }

  pub fn rev_lookup(&self, translation: &str) -> Option<Vec<String>> {
    Some(self.reverse_entries.get(translation)?.into_iter().cloned().collect())
  }

  pub fn write(&self, writer: &mut dyn io::Write) -> Result<(), io::Error> {
    write!(writer, "{}", FILE_HEADER)?;
    write!(writer, "{{\\*\\cxsystem {}}}", &self.cre_system)?;
    write!(writer, "{}", FILE_HEADER_END)?;
    for (_, entry) in &self.entries {
      &entry.write(writer)?;
    }
    write!(writer, "{}", FILE_FOOTER)?;
    Ok(())
  }

  pub fn len(&self) -> usize {
    self.entries.len()
  }

  pub fn clear(&mut self) {
    self.entries = LinkedHashMap::new();
    self.reverse_entries = HashMap::new();
    self.longest_key = 0;
  }

  pub fn entry(&self, steno: &str) -> Option<&Entry> {
    Some(self.entries.get(steno)?)
  }
}

impl fmt::Debug for Dictionary {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.entries.values().cloned().collect::<Vec<_>>().fmt(f)
  }
}

#[macro_export]
macro_rules! add_entry {
  ($dict:expr, $outline:expr => $translation:expr) => {
    $dict.add_entry(String::from($outline), String::from($translation), None)
  };
  ($dict:expr, $outline:expr => $translation:expr, $comment:expr) => {
    $dict.add_entry(
      String::from($outline), String::from($translation),
      Some(String::from($comment)))
  };
}

#[macro_export]
macro_rules! remove_entry {
  ($dict:expr, $outline:expr) => {
    $dict.remove_entry(String::from($outline));
  };
}
