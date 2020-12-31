use crate::dict::Dictionary;
use crate::rtf::parse_file as parse_file;

use std::collections::HashMap;
use std::io::Read;
use std::str::from_utf8;

use pyo3::prelude::*;
use pyo3::class::{PyMappingProtocol, PySequenceProtocol};
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::wrap_pyfunction;
use pyo3_file::PyFileLikeObject;

#[pyclass]
pub struct RtfDictionary {
  dict: Dictionary,
}

#[pymethods]
impl RtfDictionary {
  #[new]
  fn new() -> Self {
    Self { dict: Dictionary::new("") }
  }

  #[getter]
  /// The name of the system that prepared this RTF dictionary.
  fn get_cre_system(&self) -> PyResult<String> {
    Ok(self.dict.cre_system.clone())
  }

  #[setter]
  fn set_cre_system(&mut self, value: String) -> PyResult<()> {
    self.dict.cre_system = value;
    Ok(())
  }

  /// dump(self, file, /)
  /// --
  ///
  /// Write the contents of the dictionary to `file`, a file-like object.
  /// `file` should be opened in binary mode.
  fn dump(&self, file: PyObject) -> PyResult<()> {
    match PyFileLikeObject::with_requirements(file, true, true, true) {
      Ok(mut f) => {
        match self.dict.write(&mut f) {
          Ok(_) => Ok(()),
          Err(err) => Err(PyValueError::new_err(format!("failed to write RTF dictionary: {:?}", err))),
        }
      },
      Err(e) => Err(e),
    }
  }

  /// dumps(self, /)
  /// --
  ///
  /// Write the contents of the dictionary to a string and return the string.
  fn dumps(&self) -> PyResult<String> {
    let mut buf: Vec<u8> = vec![];
    match self.dict.write(&mut buf) {
      Ok(_) => Ok(from_utf8(&buf)?.to_string()),
      Err(err) => Err(PyValueError::new_err(format!("failed to write RTF dictionary: {:?}", err))),
    }
  }

  #[getter]
  /// The number of strokes of the longest stroke defined in the dictionary.
  fn longest_key(&self) -> PyResult<usize> {
    Ok(self.dict.longest_key)
  }

  /// lookup(self, steno, /)
  /// --
  ///
  /// Return a tuple of the translation and the comment for the given steno
  /// stroke, or None if not available.
  fn lookup(&self, steno: &str) -> PyResult<Option<(String, Option<String>)>> {
    match self.dict.entry(steno) {
      Some(entry) => Ok(Some((entry.translation.clone(), entry.comment()))),
      None => Ok(None),
    }
  }

  /// reverse_lookup(self, translation, /)
  /// --
  ///
  /// Return the list of steno strokes that translate to `translation`.
  fn reverse_lookup(&self, translation: &str) -> PyResult<Vec<String>> {
    match self.dict.rev_lookup(translation) {
      Some(strokes) => Ok(strokes),
      None => Ok(vec![]),
    }
  }

  /// add_comment(self, steno, comment, /)
  /// --
  ///
  /// Add a comment to the entry for the given steno stroke, or raise a
  /// KeyError if not available.
  fn add_comment(&mut self, steno: &str, comment: &str) -> PyResult<()> {
    match self.dict.entry_mut(steno) {
      Some(entry) => {
        entry.add_comment(comment);
        Ok(())
      },
      None => Err(PyKeyError::new_err(steno.to_string())),
    }
  }

  /// remove_comment(self, steno, /)
  /// --
  ///
  /// Remove the comment from the entry for the given steno stroke, or raise a
  /// KeyError if not available.
  fn remove_comment(&mut self, steno: &str) -> PyResult<()> {
    match self.dict.entry_mut(steno) {
      Some(entry) => {
        entry.remove_comment();
        Ok(())
      },
      None => Err(PyKeyError::new_err(steno.to_string())),
    }
  }

  #[getter]
  /// A dictionary mapping steno strokes to translations.
  fn stroke_to_translation(&self) -> PyResult<HashMap<String, String>> {
    Ok(self.dict.entries.iter()
      .map(|(k, v)| (k.clone(), v.translation.clone()))
      .collect::<HashMap<String, String>>())
  }

  #[getter]
  /// A dictionary mapping translations to the list of steno strokes that
  /// translate to them.
  fn translation_to_strokes(&self) -> PyResult<HashMap<String, Vec<String>>> {
    Ok(self.dict.reverse_entries.iter()
      .map(|(k, v)| (k.clone(), v.iter().cloned().collect::<Vec<String>>()))
      .collect::<HashMap<String, Vec<String>>>())
  }
}

#[pyproto]
impl PyMappingProtocol for RtfDictionary {
  fn __len__(&self) -> usize {
    self.dict.len()
  }

  fn __getitem__(&self, steno: &str) -> PyResult<String> {
    match self.dict.lookup(steno) {
      Some(translation) => Ok(translation),
      None => Err(PyKeyError::new_err(steno.to_string())),
    }
  }

  fn __setitem__(&mut self, steno: &str, translation: &str) -> PyResult<()> {
    Ok(self.dict.add_entry(steno.to_string(), translation.to_string(), None))
  }

  fn __delitem__(&mut self, steno: &str) -> PyResult<()> {
    Ok(self.dict.remove_entry(steno.to_string()))
  }
}

#[pyproto]
impl PySequenceProtocol for RtfDictionary {
  fn __contains__(&self, steno: &str) -> PyResult<bool> {
    Ok(self.dict.contains_key(steno))
  }
}

#[pyfunction]
/// load(file, /)
/// --
///
/// Read the contents of `file`, a file-like object containing an RTF
/// dictionary, into a Python object. `file` should be opened in binary mode.
fn load(file: PyObject) -> PyResult<RtfDictionary> {
  match PyFileLikeObject::with_requirements(file, true, false, true) {
    Ok(mut f) => {
      let mut contents = String::new();
      let _ = f.read_to_string(&mut contents);
      match parse_file(&contents) {
        Ok((_, dict)) => Ok(RtfDictionary { dict }),
        Err(err) => Err(PyValueError::new_err(format!("failed to read RTF dictionary: {:?}", err))),
      }
    },
    Err(e) => Err(e),
  }
}

#[pyfunction]
/// loads(string, /)
/// --
///
/// Read the contents of `string`, a string or string-like containing an RTF
/// dictionary, into a Python object.
fn loads(string: &str) -> PyResult<RtfDictionary> {
  match parse_file(&string) {
    Ok((_, dict)) => Ok(RtfDictionary { dict }),
    Err(err) => Err(PyValueError::new_err(format!("failed to read RTF dictionary: {:?}", err))),
  }
}

#[pymodule]
/// RTF/CRE (Rich Text Format with Court Reporting Extensions) is an application
/// of Microsoft's Rich Text Format in court reporting and related professions.
/// This library provides utilities to read and write RTF/CRE dictionaries,
/// for translating from steno strokes to written text (other types of RTF/CRE
/// documents are not currently supported).
///
/// The rtfcre module provides an API similar to the standard json and pickle
/// modules for reading and writing dictionaries, and one based on the builtin
/// `dict` type for using them.
///
/// Reading dictionaries:
///
///     >>> import rtfcre
///     >>> with open("dict.rtf", "rb") as file:
///     ...     dict = rtfcre.load(file)
///
///     >>> rtf = r"""
///     ... {\rtf1\ansi{\*\cxrev100}\cxdict{\*\cxsystem KittyCAT}
///     ... {\*\cxs KAT}cat
///     ... {\*\cxs TKOG}dog
///     ... }
///     ... """.lstrip()
///     >>> dict = rtfcre.loads(rtf)
///
/// Writing dictionaries:
///
///     >>> with open("dict.rtf", "wb") as file:
///     >>>     dict.dump(file)
///     >>> dict.dumps()
///     "{\rtf1\ansi{\*\cxrev100}\cxdict[...]"
///
/// Using dictionaries:
///
///     >>> dict["KAT"]
///     "cat"
///     >>> dict["KOU"] = "cow"
///     >>> dict["KOU"]
///     "cow"
///     >>> del dict["KOU"]
///     >>> "KOU" in dict
///     False
///     >>> dict.reverse_lookup("cat")
///     ["KAT"]
///
/// Accessing entry comments:
///
///     >>> dict.add_comment("TKOG", "TK means D")
///     >>> translation, comment = dict.lookup("TKOG")
///     >>> comment
///     "TK means D"
///     >>> dict.remove_comment("TKOG")
///
fn rtfcre(_: Python, m: &PyModule) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(load, m)?)?;
  m.add_function(wrap_pyfunction!(loads, m)?)?;
  m.add_class::<RtfDictionary>()?;

  Ok(())
}
