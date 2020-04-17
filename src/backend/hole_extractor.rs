use quick_xml::{Reader, Error};
use quick_xml::events::{Event, BytesStart};
use std::io::BufRead;
use std::path::Path;
use std::iter::repeat;
use std::collections::HashMap;

pub struct HoleExtractor {
    n_holes: usize,
    hole_offset: usize
}

impl HoleExtractor {
    pub fn new(n_holes: usize, hole_offset: usize) -> Self {
        HoleExtractor {
            n_holes: n_holes,
            hole_offset: hole_offset
        }
    }

    fn extract_hole_from_element(&self, holes: &mut Vec<Option<isize>>, element: &BytesStart) -> Option<()> {
        if element.name() != b"hole_value" { return None; }
        let attrs: HashMap<_, _>= element.attributes()
                    .into_iter().flatten()
                    .map(|attr| (attr.key, attr.value))
                    .collect();
        let e_type = attrs.get(b"type".as_ref())?;
        if e_type.as_ref() != b"int" { return None; }

        let e_name = std::str::from_utf8(attrs.get(b"name".as_ref())?).ok()?;
        let e_name_parts: Vec<_> = e_name.split("__").collect();
        if e_name_parts.get(0)? != &"H" { return None; }
        let hole_index = e_name_parts.get(1)?.parse::<usize>().ok()?;
        if hole_index < self.hole_offset { return None; }

        let e_value = std::str::from_utf8(attrs.get(b"value".as_ref())?).ok()?
                        .parse::<isize>().ok()?;
        
        *holes.get_mut(hole_index - self.hole_offset)? = Some(e_value);
        Some(())
    }

    pub fn read_holes<B: BufRead>(&self, r: &mut Reader<B>) -> Result<Vec<isize>, Error> {
        let mut buffer = Vec::new();
        let mut holes: Vec<Option<isize>> = repeat(None).take(self.n_holes).collect();
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
        Ok(holes.into_iter().collect::<Option<Vec<isize>>>().ok_or(quick_xml::Error::TextNotFound)?)
    }

    pub fn read_holes_from_str<S: AsRef<str>>(&self, xml: S) -> Result<Vec<isize>, Error> {
        let mut reader = Reader::from_str(xml.as_ref());
        self.read_holes(&mut reader)
    }

    pub fn read_holes_from_file<P: AsRef<Path>>(&self, file_name: P) -> Result<Vec<isize>, Error> {
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
            <hole_value line="45" col="-1"  name="H__6" type="int" value="1" />
            <hole_value line="9" col="-1"  name="H__0" type="array">
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
                <entry value="0" />
            </hole_value>
            <hole_value line="48" col="-1"  name="H__7" type="int" value="0" />
            <hole_value line="29" col="-1"  name="H__1" type="int" value="1" />
            <hole_value line="39" col="-1"  name="H__4" type="int" value="0" />
            <hole_value line="30" col="-1"  name="H__2" type="int" value="0" />
            <hole_value line="42" col="-1"  name="H__5" type="int" value="0" />
            <hole_value line="32" col="-1"  name="H__3" type="int" value="0" />
        </hole_values>
        "#;
        let extractor = HoleExtractor::new(7, 1);
        let holes = extractor.read_holes_from_str(&xml_str)?;
        assert_eq!(holes, vec![1, 0, 0, 0, 0, 1, 0]);
        Ok(())
    }

}