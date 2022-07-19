use crate::de::{Event, Input};
use crate::error::{self, Result};
use crate::libyaml::error::Mark;
use crate::libyaml::parser::{Event as YamlEvent, Parser};
use std::borrow::Cow;
use std::collections::BTreeMap;

pub(crate) struct Loader {
    events: Vec<(Event, Mark)>,
    /// Map from alias id to index in events.
    aliases: BTreeMap<usize, usize>,
    /// Map from start index to number of events.
    document_lengths: BTreeMap<usize, usize>,
}

impl Loader {
    pub fn new(input: Input) -> Result<Self> {
        let mut buffer;
        let input = match input {
            Input::Str(s) => s.as_bytes(),
            Input::Slice(bytes) => bytes,
            Input::Read(mut rdr) => {
                buffer = Vec::new();
                rdr.read_to_end(&mut buffer).map_err(error::io)?;
                &buffer
            }
            Input::Iterable(_) | Input::Document(_) => unreachable!(),
            Input::Fail(err) => return Err(error::shared(err)),
        };

        let mut parser = Parser::new(Cow::Borrowed(input));
        let mut current_document_start = 0;
        let mut anchors = BTreeMap::new();
        let mut loader = Loader {
            events: Vec::new(),
            aliases: BTreeMap::new(),
            document_lengths: BTreeMap::new(),
        };

        loop {
            let (event, mark) = parser.next()?;
            let event = match event {
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
                YamlEvent::Alias(alias) => match anchors.get(&alias) {
                    Some(id) => Event::Alias(*id),
                    None => return Err(error::unknown_anchor(mark)),
                },
                YamlEvent::Scalar(scalar) => {
                    if let Some(anchor) = scalar.anchor {
                        let id = anchors.len();
                        anchors.insert(anchor, id);
                        loader.aliases.insert(id, loader.events.len());
                    }
                    Event::Scalar(scalar.value, scalar.style, scalar.tag)
                }
                YamlEvent::SequenceStart(sequence_start) => {
                    if let Some(anchor) = sequence_start.anchor {
                        let id = anchors.len();
                        anchors.insert(anchor, id);
                        loader.aliases.insert(id, loader.events.len());
                    }
                    Event::SequenceStart
                }
                YamlEvent::SequenceEnd => Event::SequenceEnd,
                YamlEvent::MappingStart(mapping_start) => {
                    if let Some(anchor) = mapping_start.anchor {
                        let id = anchors.len();
                        anchors.insert(anchor, id);
                        loader.aliases.insert(id, loader.events.len());
                    }
                    Event::MappingStart
                }
                YamlEvent::MappingEnd => Event::MappingEnd,
            };
            loader.events.push((event, mark));
        }

        Ok(loader)
    }

    pub fn event(&self, pos: usize) -> Option<(&Event, Mark)> {
        self.events.get(pos).map(|(event, mark)| (event, *mark))
    }

    pub fn alias(&self, id: usize) -> Option<usize> {
        self.aliases.get(&id).copied()
    }

    pub fn document_len(&self, start: usize) -> usize {
        *self.document_lengths.get(&start).unwrap_or(&0)
    }
}
