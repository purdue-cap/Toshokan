use quick_xml::{Reader, Error};
use quick_xml::events::{Event, BytesStart};
use std::io::BufRead;
use std::path::Path;
use regex::Regex;
use std::collections::HashMap;

pub struct HoleExtractor {
    hole_offset: usize
}

impl HoleExtractor {
    pub fn new(hole_offset: usize) -> Self {
        HoleExtractor {
            hole_offset: hole_offset
        }
    }

    fn map_hole_name<S: AsRef<str>>(&self, name: S) -> Option<String> {
        let hole_regex = Regex::new(r"H__(\d+)([\d_]*)").expect("Hard coded regex should not fail");
        let caps = hole_regex.captures(name.as_ref())?;
        let hole_index = caps.get(1)?.as_str().parse::<usize>().ok()?;
        if hole_index < self.hole_offset {
            None
        } else {
            Some(format!("H__{}{}", hole_index - self.hole_offset, caps.get(2)?.as_str()))
        }

    }

    fn extract_hole_from_element(&self, holes: &mut HashMap<String, isize>, element: &BytesStart) -> Option<()> {
        if element.name() != b"hole_value" { return None; }
        let attrs: HashMap<_, _>= element.attributes()
                    .into_iter().flatten()
                    .map(|attr| (attr.key, attr.value))
                    .collect();
        let e_type = attrs.get(b"type".as_ref())?;
        if e_type.as_ref() != b"int" { return None; }

        let e_name = std::str::from_utf8(attrs.get(b"name".as_ref())?).ok()?;
        let h_name = self.map_hole_name(&e_name)?;

        let e_value = std::str::from_utf8(attrs.get(b"value".as_ref())?).ok()?
                        .parse::<isize>().ok()?;
        
        holes.insert(h_name, e_value);
        Some(())
    }

    pub fn read_holes<B: BufRead>(&self, r: &mut Reader<B>) -> Result<HashMap<String, isize>, Error> {
        let mut buffer = Vec::new();
        let mut holes = HashMap::<String, isize>::new();
        loop {
            match r.read_event(&mut buffer)? {
                Event::Start(ref element) | Event::Empty(ref element) => {
                    self.extract_hole_from_element(&mut holes, element);
                },
                Event::Eof => break,
                _ => ()
            }
        }
        buffer.clear();
        Ok(holes)
    }

    pub fn read_holes_from_str<S: AsRef<str>>(&self, xml: S) -> Result<HashMap<String, isize>, Error> {
        let mut reader = Reader::from_str(xml.as_ref());
        self.read_holes(&mut reader)
    }

    pub fn read_holes_from_file<P: AsRef<Path>>(&self, file_name: P) -> Result<HashMap<String, isize>, Error> {
        let mut reader = Reader::from_file(file_name)?;
        self.read_holes(&mut reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn reads_holes_from_str() -> Result<(), Box<dyn Error>> {
        let xml_str = r#"
        <?xml version="1.0"?>
        <hole_values>
            <hole_value line="10" col="-1"  name="H__3_1_0_0_0_0_1_0_0_0_0" type="int" value="0" />
            <hole_value line="4" col="-1"  name="H__1_1" type="int" value="0" />
            <hole_value line="4" col="-1"  name="H__1_1_0_0_0" type="int" value="1" />
            <hole_value line="8" col="-1"  name="H__5_1" type="int" value="1" />
            <hole_value line="7" col="-1"  name="H__2_1_0" type="int" value="1" />
            <hole_value line="7" col="-1"  name="H__2_1_0_0_0" type="int" value="1" />
            <hole_value line="8" col="-1"  name="H__5_1_0_0_0" type="int" value="1" />
            <hole_value line="5" col="-1"  name="H__4_1_0_0_0" type="int" value="1" />
            <hole_value line="8" col="-1"  name="H__5_1_0_0_0_0" type="int" value="0" />
            <hole_value line="4" col="-1"  name="H__1_1_0_0_0_0" type="int" value="1" />
            <hole_value line="7" col="-1"  name="H__2_1" type="int" value="0" />
            <hole_value line="10" col="-1"  name="H__3_1" type="int" value="1" />
            <hole_value line="5" col="-1"  name="H__4_1_0" type="int" value="1" />
            <hole_value line="7" col="-1"  name="H__2_1_0_0" type="int" value="1" />
            <hole_value line="8" col="-1"  name="H__5_1_0" type="int" value="0" />
            <hole_value line="10" col="-1"  name="H__3_1_0_0" type="int" value="1" />
            <hole_value line="4" col="-1"  name="H__1_1_0_0" type="int" value="1" />
            <hole_value line="5" col="-1"  name="H__4_1" type="int" value="0" />
            <hole_value line="5" col="-1"  name="H__4_1_0_0" type="int" value="0" />
            <hole_value line="4" col="-1"  name="H__1_1_0" type="int" value="1" />
            <hole_value line="10" col="-1"  name="H__3_1_0_0_0" type="int" value="1" />
            <hole_value line="1" col="-1"  name="H__0" type="array">
                <entry value="0" />
                <entry value="0" />
                <entry value="2" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
            </hole_value>
            <hole_value line="5" col="-1"  name="H__4_1_0_0_0_0" type="int" value="0" />
            <hole_value line="10" col="-1"  name="H__3_1_0" type="int" value="1" />
            <hole_value line="8" col="-1"  name="H__5_1_0_0" type="int" value="1" />
            <hole_value line="7" col="-1"  name="H__2_1_0_0_0_0" type="int" value="0" />
        </hole_values>
        "#;
        let extractor = HoleExtractor::new(1);
        let holes = extractor.read_holes_from_str(&xml_str)?;
        let mut holes_fixture = HashMap::<String, isize>::new();
        holes_fixture.insert("H__2_1_0_0_0_0_1_0_0_0_0".to_string(), 0);
        holes_fixture.insert("H__0_1".to_string(), 0);
        holes_fixture.insert("H__0_1_0_0_0".to_string(), 1);
        holes_fixture.insert("H__4_1".to_string(), 1);
        holes_fixture.insert("H__1_1_0".to_string(), 1);
        holes_fixture.insert("H__1_1_0_0_0".to_string(), 1);
        holes_fixture.insert("H__4_1_0_0_0".to_string(), 1);
        holes_fixture.insert("H__3_1_0_0_0".to_string(), 1);
        holes_fixture.insert("H__4_1_0_0_0_0".to_string(), 0);
        holes_fixture.insert("H__0_1_0_0_0_0".to_string(), 1);
        holes_fixture.insert("H__1_1".to_string(), 0);
        holes_fixture.insert("H__2_1".to_string(), 1);
        holes_fixture.insert("H__3_1_0".to_string(), 1);
        holes_fixture.insert("H__1_1_0_0".to_string(), 1);
        holes_fixture.insert("H__4_1_0".to_string(), 0);
        holes_fixture.insert("H__2_1_0_0".to_string(), 1);
        holes_fixture.insert("H__0_1_0_0".to_string(), 1);
        holes_fixture.insert("H__3_1".to_string(), 0);
        holes_fixture.insert("H__3_1_0_0".to_string(), 0);
        holes_fixture.insert("H__0_1_0".to_string(), 1);
        holes_fixture.insert("H__2_1_0_0_0".to_string(), 1);
        holes_fixture.insert("H__3_1_0_0_0_0".to_string(), 0);
        holes_fixture.insert("H__2_1_0".to_string(), 1);
        holes_fixture.insert("H__4_1_0_0".to_string(), 1);
        holes_fixture.insert("H__1_1_0_0_0_0".to_string(), 0);
        assert_eq!(holes, holes_fixture);
        Ok(())
    }

}