use crate::de::{Event, Progress};
use crate::error::{self, Error, ErrorImpl, Result};
use crate::libyaml::error::Mark;
use crate::libyaml::parser::{Event as YamlEvent, Parser};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) struct Loader<'input> {
    parser: Option<Parser<'input>>,
}

pub(crate) struct Document {
    pub events: Vec<(Event, Mark)>,
    pub error: Option<Arc<ErrorImpl>>,
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

    pub fn next_document(&mut self) -> Option<Document> {
        let parser = match &mut self.parser {
            Some(parser) => parser,
            None => return None,
        };

        let mut anchors = BTreeMap::new();
        let mut document = Document {
            events: Vec::new(),
            error: None,
            aliases: BTreeMap::new(),
        };

        loop {
            let (event, mark) = match parser.next() {
                Ok((event, mark)) => (event, mark),
                Err(err) => {
                    document.error = Some(Error::from(err).shared());
                    return Some(document);
                }
            };
            let event = match event {
                YamlEvent::StreamStart => continue,
                YamlEvent::StreamEnd => {
                    self.parser = None;
                    return None;
                }
                YamlEvent::DocumentStart => continue,
                YamlEvent::DocumentEnd => return Some(document),
                YamlEvent::Alias(alias) => match anchors.get(&alias) {
                    Some(id) => Event::Alias(*id),
                    None => {
                        document.error = Some(error::unknown_anchor(mark).shared());
                        return Some(document);
                    }
                },
                YamlEvent::Scalar(mut scalar) => {
                    if let Some(anchor) = scalar.anchor.take() {
                        let id = anchors.len();
                        anchors.insert(anchor, id);
                        document.aliases.insert(id, document.events.len());
                    }
                    Event::Scalar(scalar)
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
