# rtfcre

[![pypi](https://img.shields.io/pypi/v/rtfcre)](https://pypi.org/project/rtfcre)
![python](https://img.shields.io/pypi/pyversions/rtfcre)
![build](https://github.com/sammdot/rtfcre/workflows/build/badge.svg)

`rtfcre` is a Python library for reading and writing steno dictionaries in the
[RTF/CRE](http://www.legalxml.org/workgroups/substantive/transcripts/cre-spec.htm)
(Rich Text Format with Court Reporting Extensions) format. The library provides
an API similar to that of the `json` module for reading and writing dictionaries.

## Features

* **Speed**: The parsing logic is written in Rust using parser combinators,
  making it much faster than practically any pure-Python implementation.

* **Comments**: Rather than just exposing translations, `rtfcre` also reads the
  comments embedded in each entry (`{\*\cxcomment like this}`).

* **Unicode**: Full Unicode support -- while the dictionary files are not
  encoded in UTF-8, Unicode characters in translations are still fully
  supported. Translations can be in any language and they will seamlessly be
  converted to escapes when writing.

* **Plover support**: Translations are converted automatically to Plover's
  native syntax (e.g. fingerspelling is represented with `{&a}` rather than
  `{\cxfing a}`) and converted back when writing.

## Installation

```
pip install rtfcre
```

## Usage

To read an RTF dictionary:

```python
import rtfcre

# Reading directly from a file (make sure to open binary)
with open("dict.rtf", "rb") as file:
  dic = rtfcre.load(file)

# Reading from a string
rtf = r"""
{\rtf1\ansi{\*\cxrev100}\cxdict{\*\cxsystem KittyCAT}
{\*\cxs KAT}cat
{\*\cxs KOU}cow
}
""".lstrip()
dic = rtfcre.loads(rtf)
```

To write the RTF dictionary:

```python
# Writing to a file (make sure to open binary)
with open("dict.rtf", "wb") as file:
  dic.dump(file)

# Writing to a string
rtf = dic.dumps()
```

The dictionary object itself also supports the standard `dict` API:

```python
dic["KAT"] = "cat"

"KAT" in dic  # True
dic["KAT"]  # "cat"

del dic["KAT"]

dic["TKOG"]  # KeyError
dic["TKOG"] = "dog"
dic["TKOG"]  # "dog"
```

as well as a reverse lookup API for mapping from translations to steno strokes:

```python
dic.reverse_lookup("cat")  # ["KAT"]
```

To access comments:

```python
dic.lookup("TKOG")  # ("dog", None)

dic.add_comment("TKOG", "TK means D")
dic.lookup("TKOG")  # ("dog", "TK means D")

dic.remove_comment("TKOG")
```

