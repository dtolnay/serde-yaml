use std::ops::Deref;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct Tag(pub(super) Box<[u8]>);

impl Tag {
    pub const NULL: &'static str = "tag:yaml.org,2002:null";
    pub const BOOL: &'static str = "tag:yaml.org,2002:bool";
    pub const INT: &'static str = "tag:yaml.org,2002:int";
    pub const FLOAT: &'static str = "tag:yaml.org,2002:float";
}

impl PartialEq<str> for Tag {
    fn eq(&self, other: &str) -> bool {
        *self.0 == *other.as_bytes()
    }
}

impl Deref for Tag {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
