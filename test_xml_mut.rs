use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::Writer;
use std::io::Cursor;

fn main() {
    let xml = r#"<office:document><office:body><office:text><text:p>Old Text</text:p></office:text></office:body></office:document>"#;
    let new_inner = r#"<text:p>New Text</text:p>"#;
    
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    let mut skip_depth = 0;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"office:text" && skip_depth == 0 => {
                writer.write_event(Event::Start(e.clone())).unwrap();
                writer.get_mut().get_mut().extend_from_slice(new_inner.as_bytes());
                skip_depth = 1;
            },
            Ok(Event::Start(ref _e)) if skip_depth > 0 => {
                skip_depth += 1;
            },
            Ok(Event::Empty(ref _e)) if skip_depth > 0 => {},
            Ok(Event::End(ref e)) if skip_depth > 0 => {
                skip_depth -= 1;
                if skip_depth == 0 {
                    writer.write_event(Event::End(e.clone())).unwrap();
                }
            },
            Ok(Event::Eof) => break,
            Ok(e) => {
                if skip_depth == 0 {
                    writer.write_event(e).unwrap();
                }
            },
            Err(e) => panic!("Error: {}", e),
        }
        buf.clear();
    }
    
    let result = String::from_utf8(writer.into_inner().into_inner()).unwrap();
    println!("Result: {}", result);
}
