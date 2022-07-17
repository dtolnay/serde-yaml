use crate::de::{Event, Input};
use crate::error::{self, Result};
use std::collections::BTreeMap;
use std::str;
use yaml_rust::parser::{Event as YamlEvent, MarkedEventReceiver, Parser};
use yaml_rust::scanner::Marker;

pub(crate) struct Loader {
    pub events: Vec<(Event, Marker)>,
    /// Map from alias id to index in events.
    pub aliases: BTreeMap<usize, usize>,
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
            Input::Multidoc(_) => unreachable!(),
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
        };
        parser.load(&mut loader, true).map_err(error::scanner)?;
        Ok(loader)
    }
}

impl MarkedEventReceiver for Loader {
    fn on_event(&mut self, event: YamlEvent, marker: Marker) {
        let event = match event {
            YamlEvent::Nothing
            | YamlEvent::StreamStart
            | YamlEvent::StreamEnd
            | YamlEvent::DocumentStart
            | YamlEvent::DocumentEnd => return,

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
