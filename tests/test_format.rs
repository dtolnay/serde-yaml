use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use serde_yaml::yamlformat::YamlFormat;

// Numbers in different bases, static comments.
#[derive(Serialize, Deserialize, YamlFormat, Debug, PartialEq)]
struct Coordinate {
    #[yaml(format=hex, comment="X-coordinate")]
    x: u32,
    #[yaml(format=dec, comment="Y-coordinate")]
    y: u32,
    #[yaml(format=oct, comment="Z-coordinate")]
    z: u32,
}

const COORDINATE: &str = r#"---
# X-coordinate
x: 0x00000010
# Y-coordinate
y: 10
# Z-coordinate
z: 0o00000000010
"#;

#[test]
fn test_coordinate() -> Result<()> {
    let val = Coordinate { x: 16, y: 10, z: 8 };
    let s = serde_yaml::to_string(&val)?;
    let d = serde_yaml::from_str::<Coordinate>(&s)?;
    assert_eq!(s, COORDINATE);
    assert_eq!(d, val);
    Ok(())
}

// Text blocks, comments a fields within the struct.
#[derive(Serialize, Deserialize, YamlFormat, Debug, PartialEq)]
struct Gettysburg {
    author: String,

    #[yaml(format=block, comment=_prelude)]
    prelude: String,
    #[serde(skip)]
    _prelude: String,

    #[yaml(format=block, comment=_middle)]
    middle: String,
    #[serde(skip)]
    _middle: String,

    #[yaml(format=block, comment=_end)]
    end: String,
    #[serde(skip)]
    _end: String,
}

const GETTYSBURG: &str = r#"---
author: Abraham Lincoln
# Hay copy
prelude: |+
  Four score and seven years ago our fathers brought forth, upon this
  continent, a new nation, conceived in Liberty, and dedicated to the
  proposition that all men are created equal.
# Nicolay Copy
middle: |+
  Now we are engaged in a great civil war, testing whether that nation,
  or any nation so conceived, and so dedicated, can long endure. We are met
  on a great battle field of that war. We come to dedicate a portion of it,
  as a final resting place for those who died here, that the nation might
  live. This we may, in all propriety do.
# Bliss Copy
end: |-
  But, in a larger sense, we can not dedicate -- we can not consecrate --
  we can not hallow -- this ground. The brave men, living and dead, who
  struggled here, have consecrated it, far above our poor power to add or
  detract. The world will little note, nor long remember what we say here,
  but it can never forget what they did here. It is for us the living,
  rather, to be dedicated here to the unfinished work which they who
  fought here have thus far so nobly advanced. It is rather for us to be
  here dedicated to the great task remaining before us -- that from these
  honored dead we take increased devotion to that cause for which they gave
  the last full measure of devotion -- that we here highly resolve that
  these dead shall not have died in vain -- that this nation, under God,
  shall have a new birth of freedom -- and that government of the people,
  by the people, for the people, shall not perish from the earth.
"#;

#[test]
fn test_gettysburg() -> Result<()> {
    let mut val = Gettysburg {
        // Note: trailing newline should cause a "|+" block.
        prelude: r#"Four score and seven years ago our fathers brought forth, upon this
continent, a new nation, conceived in Liberty, and dedicated to the
proposition that all men are created equal.
"#.to_string(),
        _prelude: "Hay copy".to_string(),

        // Note: trailing newline should cause a "|+" block.
        middle: r#"Now we are engaged in a great civil war, testing whether that nation,
or any nation so conceived, and so dedicated, can long endure. We are met
on a great battle field of that war. We come to dedicate a portion of it,
as a final resting place for those who died here, that the nation might
live. This we may, in all propriety do.
"#.to_string(),
        _middle: "Nicolay Copy".to_string(),

        // Note: NO trailing newline should cause a "|-" block.
        end: r#"But, in a larger sense, we can not dedicate -- we can not consecrate --
we can not hallow -- this ground. The brave men, living and dead, who
struggled here, have consecrated it, far above our poor power to add or
detract. The world will little note, nor long remember what we say here,
but it can never forget what they did here. It is for us the living,
rather, to be dedicated here to the unfinished work which they who
fought here have thus far so nobly advanced. It is rather for us to be
here dedicated to the great task remaining before us -- that from these
honored dead we take increased devotion to that cause for which they gave
the last full measure of devotion -- that we here highly resolve that
these dead shall not have died in vain -- that this nation, under God,
shall have a new birth of freedom -- and that government of the people,
by the people, for the people, shall not perish from the earth."#.to_string(),
        _end: "Bliss Copy".to_string(),

        author: "Abraham Lincoln".to_string(),
    };

    let s = serde_yaml::to_string(&val)?;
    let d = serde_yaml::from_str::<Gettysburg>(&s)?;
    assert_eq!(s, GETTYSBURG);

    // The deserialized struct will have empty strings in the comment fields.
    val._prelude = String::default();
    val._middle = String::default();
    val._end = String::default();
    assert_eq!(d, val);
    Ok(())
}

// A containing struct which does not implement YamlFormat.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Sfdp {
    header: SfdpHeader,
}

// Numbers in different bases, comments from functions within the impl.
#[derive(Serialize, Deserialize, YamlFormat, Debug, PartialEq)]
struct SfdpHeader {
    #[yaml(format=hex, comment=_signature())]
    signature: u32,
    #[yaml(comment = "SFDP version")]
    minor: u8,
    major: u8,
    #[yaml(comment = "Number of parameter headers minus 1")]
    nph: u8,
    #[yaml(format=bin, comment="Reserved field should be all ones")]
    reserved: u8,
}

impl SfdpHeader {
    fn _signature(&self) -> Option<String> {
        Some(format!(
            "Signature value='{}' (should be 'SFDP')",
            self.signature
                .to_le_bytes()
                .map(|b| char::from(b).to_string())
                .join("")
        ))
    }
}


const SFDP: &str  = r#"---
header:
  # Signature value='SFDP' (should be 'SFDP')
  signature: 0x50444653
  # SFDP version
  minor: 6
  major: 1
  # Number of parameter headers minus 1
  nph: 2
  # Reserved field should be all ones
  reserved: 0b11111111
"#;

#[test]
fn test_sfdp() -> Result<()> {
    let val = Sfdp {
        header: SfdpHeader {
            signature: 0x50444653,
            minor: 6,
            major: 1,
            nph: 2,
            reserved: 255,
        },
    };
    let s = serde_yaml::to_string(&val)?;
    let d = serde_yaml::from_str::<Sfdp>(&s)?;
    assert_eq!(s, SFDP);
    assert_eq!(d, val);
    Ok(())
}

#[derive(Serialize, Deserialize, YamlFormat, Debug, PartialEq)]
struct IntelWord(
    #[yaml(format=hex, comment="MSB")] u8,
    #[yaml(format=hex, comment="LSB")] u8,
);

const INTEL_WORD: &str = r#"---
# MSB
- 0x80
# LSB
- 0x01
"#;

#[test]
fn test_intel_word() -> Result<()> {
    let val = IntelWord(0x80, 0x01);
    let s = serde_yaml::to_string(&val)?;
    let d = serde_yaml::from_str::<IntelWord>(&s)?;
    assert_eq!(s, INTEL_WORD);
    assert_eq!(d, val);
    Ok(())
}

#[derive(Serialize, Deserialize, YamlFormat, Debug, PartialEq)]
enum NesAddress {
    #[yaml(format=oneline, comment="NES file offset")]
    File(u32),
    #[yaml(format=oneline, comment="NES PRG bank:address")]
    Prg(#[yaml(format=hex)] u8, #[yaml(format=hex)] u16),
    #[yaml(format=oneline)]
    Chr(#[yaml(format=hex)] u8, #[yaml(format=hex)] u16),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Addresses {
    a: NesAddress,
    b: NesAddress,
    c: NesAddress,
}

// Note: there is an extra space after the `a` and `b` key fields.
const ADDRESSES: &str = r#"---
a: 
  # NES file offset
  {"File": 16400}
b: 
  # NES PRG bank:address
  {"Prg": [0x01, 0x8000]}
c: {"Chr": [0x00, 0x1000]}
"#;

#[test]
fn test_nes_address() -> Result<()> {
    let val = Addresses {
        a: NesAddress::File(0x4010),
        b: NesAddress::Prg(1, 0x8000),
        c: NesAddress::Chr(0, 0x1000),
    };
    let s = serde_yaml::to_string(&val)?;
    let d = serde_yaml::from_str::<Addresses>(&s)?;
    assert_eq!(s, ADDRESSES);
    assert_eq!(d, val);
    Ok(())
}
