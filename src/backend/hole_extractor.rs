use quick_xml::{Reader, Error};
use quick_xml::events::{Event, BytesStart};
use std::io::BufRead;
use std::path::Path;
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub struct HoleExtractor {
    hole_offset: usize,
    hole_names: HashSet<String>,
    hole_name_mapping: Option<HashMap<String, String>>
}

impl HoleExtractor {
    pub fn new(hole_offset: usize, h_names: HashSet<String>) -> Self {
        HoleExtractor {
            hole_offset: hole_offset,
            hole_names: h_names,
            hole_name_mapping: None
        }
    }

    fn map_hole_name<S: AsRef<str>>(&self, name: S) -> Option<String> {
        let hole_regex = Regex::new(r"H__(\d+)([\d_]*)").expect("Hard coded regex should not fail");
        let caps = hole_regex.captures(name.as_ref())?;
        let hole_index = caps.get(1)?.as_str().parse::<usize>().ok()?;
        if hole_index < self.hole_offset {
            None
        } else {
            let plain_name = format!("H__{}{}", hole_index - self.hole_offset, caps.get(2)?.as_str());
            if let Some(ref mapping) = self.hole_name_mapping {
                mapping.get(&plain_name).map(|s| s.clone()).or(Some(plain_name))
            } else {
                Some(plain_name)
            }
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

    pub fn read_holes<B: BufRead>(&mut self, r: &mut Reader<B>) -> Result<HashMap<String, isize>, Error> {
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
        // If name mapping is not initialized, initialize it here
        if self.hole_name_mapping.is_none() {
            let map_to_version_vec = |s: &String| {
                s.trim_start_matches("H__").split("_").map(|number| number.parse::<usize>().ok()).collect::<Option<Vec<_>>>()
            };
            let mut h_name_to = self.hole_names.iter()
                .map(|name| map_to_version_vec(name).map(|version_vec| (name, version_vec)))
                .collect::<Option<Vec<_>>>()
                .ok_or(quick_xml::Error::UnexpectedToken("Hole name parsing failed".to_string()))?;
            let mut h_name_from = holes.keys()
                .map(|name| map_to_version_vec(name).map(|version_vec| (name, version_vec)))
                .collect::<Option<Vec<_>>>()
                .ok_or(quick_xml::Error::UnexpectedToken("Hole name parsing failed".to_string()))?;
            h_name_to.sort_by_cached_key(|t| t.1.clone());
            h_name_from.sort_by_cached_key(|t| t.1.clone());
            self.hole_name_mapping = Some(
                h_name_from.into_iter().zip(h_name_to.into_iter())
                    .map(|(n_from, n_to)| (n_from.0.clone(), n_to.0.clone())).collect()
            );
            holes.into_iter()
                .map(|(k, v)| self.hole_name_mapping.as_ref().and_then(
                    |mapping| mapping.get(&k).map(|mapped_name| (mapped_name.clone(), v))
                )).collect::<Option<HashMap<_, _>>>()
                .ok_or(quick_xml::Error::UnexpectedToken("Hole name mapping failed".to_string()))
        } else {
            Ok(holes)
        }
    }

    pub fn read_holes_from_str<S: AsRef<str>>(&mut self, xml: S) -> Result<HashMap<String, isize>, Error> {
        let mut reader = Reader::from_str(xml.as_ref());
        self.read_holes(&mut reader)
    }

    pub fn read_holes_from_file<P: AsRef<Path>>(&mut self, file_name: P) -> Result<HashMap<String, isize>, Error> {
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
        let hole_names = vec![
            "H__2_1_0_0_0_0_1_0_0_0_0",
            "H__0_1",
            "H__0_1_0_0_0",
            "H__4_1",
            "H__1_1_0",
            "H__1_1_0_0_0",
            "H__4_1_0_0_0",
            "H__3_1_0_0_0",
            "H__4_1_0_0_0_0",
            "H__0_1_0_0_0_0",
            "H__1_1",
            "H__2_1",
            "H__3_1_0",
            "H__1_1_0_0",
            "H__4_1_0",
            "H__2_1_0_0",
            "H__0_1_0_0",
            "H__3_1",
            "H__3_1_0_0",
            "H__0_1_0",
            "H__2_1_0_0_0",
            "H__3_1_0_0_0_0",
            "H__2_1_0",
            "H__4_1_0_0",
            "H__1_1_0_0_0_0"].into_iter().map(|s| s.to_string()).collect();
        let mut extractor = HoleExtractor::new(1, hole_names);
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

    #[test]
    fn reads_holes_from_str_that_needs_remapping() -> Result<(), Box<dyn Error>> {
        let xml_str = r#"
        <?xml version="1.0"?>
        <hole_values>
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
        </hole_values>
        "#;
        let hole_names = vec![
            "H__0_2_0_0",
            "H__3_2", "H__3_2_1_1",
            "H__0_2_0", "H__2_2_3_3_3",
            "H__3_2_1_1_1_1", "H__2_2_3"].into_iter().map(|s| s.to_string()).collect();
        let mut extractor = HoleExtractor::new(1, hole_names);
        let holes = extractor.read_holes_from_str(&xml_str)?;
        let mut holes_fixture = HashMap::<String, isize>::new();
        holes_fixture.insert("H__0_2_0_0".to_string(), 1);
        holes_fixture.insert("H__3_2".to_string(), 0);
        holes_fixture.insert("H__3_2_1_1".to_string(), 0);
        holes_fixture.insert("H__0_2_0".to_string(), 1);
        holes_fixture.insert("H__2_2_3_3_3".to_string(), 1);
        holes_fixture.insert("H__3_2_1_1_1_1".to_string(), 0);
        holes_fixture.insert("H__2_2_3".to_string(), 1);
        assert_eq!(holes, holes_fixture);
        let xml_str = r#"
        <?xml version="1.0"?>
        <hole_values>
            <hole_value line="4" col="-1"  name="H__1_1_0_0" type="int" value="2" />
            <hole_value line="5" col="-1"  name="H__4_1" type="int" value="3" />
            <hole_value line="5" col="-1"  name="H__4_1_0_0" type="int" value="1" />
            <hole_value line="4" col="-1"  name="H__1_1_0" type="int" value="0" />
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
            <hole_value line="5" col="-1"  name="H__4_1_0_0_0_0" type="int" value="1" />
            <hole_value line="10" col="-1"  name="H__3_1_0" type="int" value="1" />
        </hole_values>
        "#;
        let holes = extractor.read_holes_from_str(&xml_str)?;
        let mut holes_fixture = HashMap::<String, isize>::new();
        holes_fixture.insert("H__0_2_0_0".to_string(), 2);
        holes_fixture.insert("H__3_2".to_string(), 3);
        holes_fixture.insert("H__3_2_1_1".to_string(), 1);
        holes_fixture.insert("H__0_2_0".to_string(), 0);
        holes_fixture.insert("H__2_2_3_3_3".to_string(), 1);
        holes_fixture.insert("H__3_2_1_1_1_1".to_string(), 1);
        holes_fixture.insert("H__2_2_3".to_string(), 1);
        assert_eq!(holes, holes_fixture);
        Ok(())
    }
}