use crate::de::{Event, Input};
use crate::error::{self, Result};
use std::collections::BTreeMap;
use std::str;
use yaml_rust::parser::{Event as YamlEvent, MarkedEventReceiver, Parser};
use yaml_rust::scanner::Marker;

pub(crate) struct Loader {
    events: Vec<(Event, Marker)>,
    /// Map from alias id to index in events.
    aliases: BTreeMap<usize, usize>,
    current_document_start: usize,
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
        let mut loader = Loader {
            events: Vec::new(),
            aliases: BTreeMap::new(),
            current_document_start: 0,
            document_lengths: BTreeMap::new(),
        };
        parser.load(&mut loader, true).map_err(error::scanner)?;
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

impl MarkedEventReceiver for Loader {
    fn on_event(&mut self, event: YamlEvent, marker: Marker) {
        let event = match event {
            YamlEvent::Nothing | YamlEvent::StreamStart | YamlEvent::StreamEnd => return,
            YamlEvent::DocumentStart => {
                self.current_document_start = self.events.len();
                return;
            }
            YamlEvent::DocumentEnd => {
                let len = self.events.len() - self.current_document_start;
                self.document_lengths
                    .insert(self.current_document_start, len);
                self.current_document_start = self.events.len();
                return;
            }
            YamlEvent::Alias(id) => Event::Alias(id),
            YamlEvent::Scalar(value, style, id, tag) => {
                self.aliases.insert(id, self.events.len());
                Event::Scalar(value, style, tag)
            }
            YamlEvent::SequenceStart(id) => {
                self.aliases.insert(id, self.events.len());
                Event::SequenceStart
            }
            YamlEvent::SequenceEnd => Event::SequenceEnd,
            YamlEvent::MappingStart(id) => {
                self.aliases.insert(id, self.events.len());
                Event::MappingStart
            }
            YamlEvent::MappingEnd => Event::MappingEnd,
        };
        self.events.push((event, marker));
    }
}
