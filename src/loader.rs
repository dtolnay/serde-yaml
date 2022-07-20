use crate::de::{Event, Progress};
use crate::error::{self, Result};
use crate::libyaml::error::Mark;
use crate::libyaml::parser::{Event as YamlEvent, Parser};
use std::borrow::Cow;
use std::collections::BTreeMap;

pub(crate) struct Loader<'input> {
    parser: Option<Parser<'input>>,
}

pub(crate) struct Document {
    pub events: Vec<(Event, Mark)>,
    /// Map from alias id to index in events.
    pub aliases: BTreeMap<usize, usize>,
}

impl<'input> Loader<'input> {
    pub fn new(progress: Progress<'input>) -> Result<Self> {
        let input = match progress {
            Progress::Str(s) => Cow::Borrowed(s.as_bytes()),
            Progress::Slice(bytes) => Cow::Borrowed(bytes),
            Progress::Read(mut rdr) => {
                let mut buffer = Vec::new();
                rdr.read_to_end(&mut buffer).map_err(error::io)?;
                Cow::Owned(buffer)
            }
            Progress::Iterable(_) | Progress::Document(_) => unreachable!(),
            Progress::Fail(err) => return Err(error::shared(err)),
        };

        Ok(Loader {
            parser: Some(Parser::new(input)),
        })
    }

    pub fn next_document(&mut self) -> Result<Option<Document>> {
        let parser = match &mut self.parser {
            Some(parser) => parser,
            None => return Ok(None),
        };

        let mut anchors = BTreeMap::new();
        let mut document = Document {
            events: Vec::new(),
            aliases: BTreeMap::new(),
        };

        loop {
            let (event, mark) = parser.next()?;
            let event = match event {
                YamlEvent::StreamStart => continue,
                YamlEvent::StreamEnd => {
                    self.parser = None;
                    return Ok(None);
                }
                YamlEvent::DocumentStart => continue,
                YamlEvent::DocumentEnd => return Ok(Some(document)),
                YamlEvent::Alias(alias) => match anchors.get(&alias) {
                    Some(id) => Event::Alias(*id),
                    None => return Err(error::unknown_anchor(mark)),
                },
                YamlEvent::Scalar(scalar) => {
                    if let Some(anchor) = scalar.anchor {
                        let id = anchors.len();
                        anchors.insert(anchor, id);
                        document.aliases.insert(id, document.events.len());
                    }
                    Event::Scalar(scalar.value, scalar.style, scalar.tag)
                }
                YamlEvent::SequenceStart(sequence_start) => {
                    if let Some(anchor) = sequence_start.anchor {
                        let id = anchors.len();
                        anchors.insert(anchor, id);
                        document.aliases.insert(id, document.events.len());
                    }
                    Event::SequenceStart
                }
                YamlEvent::SequenceEnd => Event::SequenceEnd,
                YamlEvent::MappingStart(mapping_start) => {
                    if let Some(anchor) = mapping_start.anchor {
                        let id = anchors.len();
                        anchors.insert(anchor, id);
                        document.aliases.insert(id, document.events.len());
                    }
                    Event::MappingStart
                }
                YamlEvent::MappingEnd => Event::MappingEnd,
            };
            document.events.push((event, mark));
        }
    }
}
