//! yaml formatting.

use once_cell::sync::OnceCell;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
pub use yamlformat_derive::*;

/// `MemberId` identifies struct fields and enum variants.
#[derive(Debug)]
pub enum MemberId<'a> {
    /// `Name` identifies a named field.
    Name(&'a str),
    /// `Index` identifies a tuple field.
    Index(u32),
    /// `Variant` identifies a variant wihin an enum.
    Variant,
}

/// `Format` describes how a field is to be formatted.
#[derive(Debug, PartialEq)]
pub enum Format {
    /// Format a string field using block formatting.
    Block,
    /// Format an integer field as binary.
    Binary,
    /// Format an integer field as decimal.
    Decimal,
    /// Format an integer field as Hexadecimal.
    Hex,
    /// Format an integer field as Octal.
    Octal,
    /// Format hashes and arrays on one line.
    Oneline,
}

/// YamlFormat is used by the serializer to choose the desired formatting.
pub trait YamlFormat {
    /// Returns the format for a given field.
    fn format(&self, variant: Option<&str>, field: &MemberId) -> Option<Format>;
    /// Returns the comment associated with a given field.
    fn comment(&self, variant: Option<&str>, field: &MemberId) -> Option<String>;
}

#[doc(hidden)]
pub type CastFn = unsafe fn(*const ()) -> &'static dyn YamlFormat;

#[doc(hidden)]
pub type IdFn = fn() -> usize;

#[doc(hidden)]
pub struct YamlFormatType {
    pub id: IdFn,
    pub reconstitute: CastFn,
}
inventory::collect!(YamlFormatType);

impl YamlFormatType {
    #[doc(hidden)]
    pub fn of<T>() -> usize
    where
        T: ?Sized,
    {
        // Just like https://github.com/rust-lang/rust/issues/41875#issuecomment-317292888
        // We monomorphize on T and then cast the function pointer address of
        // the monomorphized `YamlFormatType::of` function to an integer identifier.
        YamlFormatType::of::<T> as usize
    }

    #[doc(hidden)]
    pub unsafe fn cast<T>(ptr: *const ()) -> &'static dyn YamlFormat
    where
        T: 'static + YamlFormat,
    {
        // Cast a generic pointer back to a reference to T and return a dyn
        // reference to the YamlFormat trait.
        &*(ptr as *const T)
    }

    fn lookup(id: usize) -> Option<CastFn> {
        static TYPEMAP: OnceCell<Mutex<HashMap<usize, CastFn>>> = OnceCell::new();
        let typemap = TYPEMAP
            .get_or_init(|| {
                let mut types = HashMap::new();
                // Iterate over all registered YamlFormatTypes and store them in
                // a HashMap for fast access.
                for yf in inventory::iter::<YamlFormatType> {
                    types.insert((yf.id)(), yf.reconstitute);
                }
                Mutex::new(types)
            })
            .lock()
            .unwrap();
        typemap.get(&id).cloned()
    }

    #[doc(hidden)]
    pub fn get<'a, T>(object: &'a T) -> Option<&'a dyn YamlFormat>
    where
        T: ?Sized,
    {
        // Get the type-id of `object` and cast it back to a YamlFormat
        // if we can.
        let id = Self::of::<T>();
        Self::lookup(id).map(|reconstitute| unsafe {
            // Shorten the lifetime to 'a, as the dyn YamlFormat reference is
            // really a reinterpretation of `object`, which has lifetime 'a.
            std::mem::transmute::<&'static dyn YamlFormat, &'a dyn YamlFormat>(reconstitute(
                object as *const T as *const (),
            ))
        })
    }
}
