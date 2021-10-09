use std::collections::HashMap;
use serde_json::{json, Value};

struct Palettes {
}

struct Palette {
    c0: Color,
    c1: Color,
    c2: Color,
    c3: Color,
    c4: Color,
}

#[derive(serde::Deserialize, Copy, Clone)]
struct Color([u8;3]);

#[derive(serde::Deserialize)]
struct JsonPalette {
    result: [Color;5]
}

fn parse_palette(json: Value) -> Palette {
    let colors : JsonPalette = serde_json::from_value(json).expect("bad data");
    Palette {
        c0: colors.result[0],
        c1: colors.result[1],
        c2: colors.result[2],
        c3: colors.result[3],
        c4: colors.result[4],
    }
}

macro_rules! palette {
    ($id: ident, $json: tt) => {
        fn $id() -> Palette {
            let item = json!($json);
            parse_palette(item)
        }
    }
}

impl Palettes {
    palette!(p1,{"result":[[186,179,203],[153,129,112],[169,54,68],[133,72,97],[62,44,55]]});
    palette!(p2,{"result":[[28,20,40],[58,84,131],[92,124,155],[118,133,109],[169,215,213]]});
    palette!(p3,{"result":[[180,130,76],[215,222,157],[134,175,158],[97,137,90],[43,43,37]]});
    palette!(p4,{"result":[[39,67,86],[88,100,120],[181,215,211],[248,247,243],[234,77,59]]});
    palette!(p5,{"result":[[191,54,63],[14,77,87],[145,149,120],[225,213,112],[240,231,193]]});
    palette!(p6,{"result":[[239,188,212],[123,136,97],[254,251,250],[73,151,167],[50,49,50]]});
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palettes_load_fine() {
        let p1 = Palettes::p1();
        let p2 = Palettes::p2();
        let p3 = Palettes::p2();
        let p4 = Palettes::p2();
        let p5 = Palettes::p2();
    }
}