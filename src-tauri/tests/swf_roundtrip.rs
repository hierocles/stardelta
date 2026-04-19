//! Parse → emit → parse smoke tests for the vendored `swf-emitter` + `swf-parser` stack.

use swf_emitter::emit_swf;
use swf_parser::parse_swf;
use swf_types::fixed::Ufixed8P8;
use swf_types::{CompressionMethod, Header, Movie, Rect, Tag};

fn minimal_movie() -> Movie {
    Movie {
        header: Header {
            swf_version: 19,
            frame_size: Rect {
                x_min: 0,
                x_max: 8000,
                y_min: 0,
                y_max: 6000,
            },
            frame_rate: Ufixed8P8::from_value(12.0f32),
            frame_count: 1,
        },
        tags: vec![Tag::ShowFrame],
    }
}

#[test]
fn minimal_swf_round_trips() {
    let m = minimal_movie();
    let bytes = emit_swf(&m, CompressionMethod::None).expect("emit");
    let parsed = parse_swf(&bytes).expect("parse");
    assert_eq!(parsed.header.swf_version, m.header.swf_version);
    assert_eq!(parsed.tags.len(), m.tags.len());
}
