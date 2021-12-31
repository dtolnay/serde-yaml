use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use serde_yaml::yamlformat::YamlFormat;

// Numbers in different bases, static comments.
#[derive(Serialize, YamlFormat)]
struct Coordinate {
    #[yaml(format=hex, comment="X-coordinate")]
    x: u32,
    #[yaml(format=dec, comment="Y-coordinate")]
    y: u32,
    #[yaml(format=oct, comment="Z-coordinate")]
    z: u32,
}

// Text blocks, comments a fields within the struct.
#[derive(Serialize, YamlFormat)]
struct Gettysburg {
    author: &'static str,

    #[yaml(format=block, comment=_prelude)]
    prelude: &'static str,
    #[serde(skip)]
    _prelude: &'static str,

    #[yaml(format=block, comment=_middle)]
    middle: &'static str,
    #[serde(skip)]
    _middle: &'static str,

    #[yaml(format=block, comment=_end)]
    end: &'static str,
    #[serde(skip)]
    _end: &'static str,
}

#[test]
fn test_coordinate() -> Result<()> {
    let foo = Coordinate { x: 16, y: 10, z: 8 };
    let s = serde_yaml::to_string(&foo)?;
    println!("{}", s);
    Ok(())
}

#[test]
fn test_gettysburg() -> Result<()> {
    let foo = Gettysburg {
        // Note: trailing newline should cause a "|+" block.
        prelude: r#"Four score and seven years ago our fathers brought forth on this
continent, a new nation, conceived in Liberty, and dedicated to the
proposition that all men are created equal.
"#,
        _prelude: "Bliss copy",

        // Note: trailing newline should cause a "|+" block.
        middle: r#"Now we are engaged in a great civil war, testing whether that nation,
or any nation so conceived, and so dedicated, can long endure. We are met
on a great battle field of that war. We come to dedicate a portion of it,
as a final resting place for those who died here, that the nation might
live. This we may, in all propriety do.
"#,
        _middle: "Nicolay Copy",

        // Note: NO trailing newline should cause a "|-" block.
        end: r#"But in a larger sense, we can not dedicate we can not consecrate we can
not hallow this ground. The brave men, living and dead, who struggled
here, have consecrated it far above our poor power to add or detract. The
world will little note, nor long remember, what we say here, but can
never forget what they did here.

It is for us, the living, rather to be dedicated here to the unfinished
work which they have, thus far, so nobly carried on. It is rather for
us to be here dedicated to the great task remaining before us that from
these honored dead we take increased devotion to that cause for which
they gave the last full measure of devotion that we here highly resolve
that these dead shall not have died in vain; that this nation shall
have a new birth of freedom; and that this government of the people,
by the people, for the people, shall not perish from the earth."#,
        _end: "Hay Copy",

        author: "Abraham Lincoln",
    };

    let s = serde_yaml::to_string(&foo)?;
    println!("{}", s);
    Ok(())
}

// A containing struct which does not implement YamlFormat.
#[derive(Serialize)]
struct Sfdp {
    header: SfdpHeader,
}

// Numbers in different bases, comments from functions within the impl.
#[derive(Serialize, YamlFormat)]
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

#[test]
fn test_sfdp() -> Result<()> {
    let foo = Sfdp {
        header: SfdpHeader {
            signature: 0x50444653,
            minor: 6,
            major: 1,
            nph: 2,
            reserved: 255,
        },
    };
    println!("foo.header={:p}", &foo.header);
    let s = serde_yaml::to_string(&foo)?;
    println!("{}", s);
    Ok(())
}
