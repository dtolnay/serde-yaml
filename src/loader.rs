use crate::de::{Event, Input};
use crate::error::{self, Result};
use std::collections::BTreeMap;
use std::str;
use yaml_rust::parser::{Event as YamlEvent, Parser};
use yaml_rust::scanner::Marker;

pub(crate) struct Loader {
    events: Vec<(Event, Marker)>,
    /// Map from alias id to index in events.
    aliases: BTreeMap<usize, usize>,
    /// Map from start index to number of events.
    document_lengths: BTreeMap<usize, usize>,
}

impl Loader {
    pub fn new(input: Input) -> Result<Self> {
        enum Input2<'a> {
            Str(&'a str),
            Slice(&'a [u8]),
        }

        let mut buffer;
        let input = match input {
            Input::Str(s) => Input2::Str(s),
            Input::Slice(bytes) => Input2::Slice(bytes),
            Input::Read(mut rdr) => {
                buffer = Vec::new();
                rdr.read_to_end(&mut buffer).map_err(error::io)?;
                Input2::Slice(&buffer)
            }
            Input::Iterable(_) | Input::Document(_) => unreachable!(),
            Input::Fail(err) => return Err(error::shared(err)),
        };

        let input = match input {
            Input2::Str(s) => s,
            Input2::Slice(bytes) => str::from_utf8(bytes).map_err(error::str_utf8)?,
        };

        let mut parser = Parser::new(input.chars());
        let mut current_document_start = 0;
        let mut loader = Loader {
            events: Vec::new(),
            aliases: BTreeMap::new(),
            document_lengths: BTreeMap::new(),
        };

        loop {
            let (event, marker) = parser.next().map_err(error::scanner)?;
            let event = match event {
                YamlEvent::Nothing => unreachable!(),
                YamlEvent::StreamStart => continue,
                YamlEvent::StreamEnd => break,
                YamlEvent::DocumentStart => {
                    current_document_start = loader.events.len();
                    continue;
                }
                YamlEvent::DocumentEnd => {
                    let len = loader.events.len() - current_document_start;
                    loader.document_lengths.insert(current_document_start, len);
                    current_document_start = loader.events.len();
                    continue;
                }
                YamlEvent::Alias(id) => Event::Alias(id),
                YamlEvent::Scalar(value, style, id, tag) => {
                    loader.aliases.insert(id, loader.events.len());
                    Event::Scalar(value, style, tag)
                }
                YamlEvent::SequenceStart(id) => {
                    loader.aliases.insert(id, loader.events.len());
                    Event::SequenceStart
                }
                YamlEvent::SequenceEnd => Event::SequenceEnd,
                YamlEvent::MappingStart(id) => {
                    loader.aliases.insert(id, loader.events.len());
                    Event::MappingStart
                }
                YamlEvent::MappingEnd => Event::MappingEnd,
            };
            loader.events.push((event, marker));
        }

        Ok(loader)
    }

    pub fn event(&self, pos: usize) -> Option<&(Event, Marker)> {
        self.events.get(pos)
    }

    pub fn alias(&self, id: usize) -> Option<usize> {
        self.aliases.get(&id).copied()
    }

    pub fn document_len(&self, start: usize) -> usize {
        *self.document_lengths.get(&start).unwrap_or(&0)
    }
}
