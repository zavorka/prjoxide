use crate::bba::idstring::*;
use crate::bels::*;
use crate::database::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum BranchSide {
    Left,
    Right,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Neighbour {
    RelXY { rel_x: i32, rel_y: i32 },
    Branch,
    BranchDriver { side: BranchSide },
    Spine,
    HRow,
    Global,
    DQSGroup,
}

impl Neighbour {
    pub fn parse_wire(s: &str) -> (Option<Neighbour>, &str) {
        let sep_pos = s.find(':');
        match sep_pos {
            Some(p) => {
                let prefix = &s[0..p];
                (
                    Some(match prefix {
                        "BRANCH" => Neighbour::Branch,
                        "BRANCH_L" => Neighbour::BranchDriver {
                            side: BranchSide::Left,
                        },
                        "BRANCH_R" => Neighbour::BranchDriver {
                            side: BranchSide::Right,
                        },
                        "SPINE" => Neighbour::Spine,
                        "HROW" => Neighbour::HRow,
                        "G" => Neighbour::Global,
                        "DQSG" => Neighbour::DQSGroup,
                        _ => {
                            let mut rel_x = 0;
                            let mut rel_y = 0;
                            let mut tokens = Vec::new();
                            let mut last = 0;
                            for (index, _) in prefix
                                .match_indices(|c| c == 'N' || c == 'E' || c == 'S' || c == 'W')
                            {
                                if last != index {
                                    tokens.push(&prefix[last..index]);
                                }
                                last = index;
                            }
                            tokens.push(&prefix[last..]);
                            for tok in tokens {
                                match tok.chars().nth(0).unwrap() {
                                    'N' => rel_y = -tok[1..].parse::<i32>().unwrap(),
                                    'S' => rel_y = tok[1..].parse::<i32>().unwrap(),
                                    'E' => rel_x = tok[1..].parse::<i32>().unwrap(),
                                    'W' => rel_x = -tok[1..].parse::<i32>().unwrap(),
                                    _ => panic!("bad pos token {}", tok),
                                }
                            }
                            Neighbour::RelXY { rel_x, rel_y }
                        }
                    }),
                    &s[p + 1..],
                )
            }
            None => (None, s),
        }
    }
}

pub struct TileType {
    data: TileBitsDatabase,
    pub wires: BTreeSet<String>,
    pub wire_ids: BTreeSet<IdString>,
    pub driven_wire_ids: BTreeSet<IdString>,
    pub neighbour_wire_ids: BTreeMap<Neighbour, Vec<IdString>>,
    pub neighbours: BTreeSet<Neighbour>,
    pub bels: Vec<Bel>,
}

impl TileType {
    pub fn new(db: &mut Database, ids: &mut IdStringDB, fam: &str, tt: &str) -> TileType {
        let mut tt = TileType {
            data: db.tile_bitdb(fam, tt).db.clone(),
            wires: BTreeSet::new(),
            wire_ids: BTreeSet::new(),
            driven_wire_ids: BTreeSet::new(),
            neighbour_wire_ids: BTreeMap::new(),
            neighbours: BTreeSet::new(),
            bels: get_tile_bels(tt),
        };
        // Add wires from pips
        for (to_wire, wire_pips) in tt.data.pips.iter() {
            tt.wires.insert(to_wire.to_string());
            let to_wire_id = ids.id(to_wire);
            tt.wire_ids.insert(to_wire_id);
            tt.driven_wire_ids.insert(to_wire_id);
            for from_wire in wire_pips.iter().map(|x| &x.from_wire) {
                tt.wires.insert(from_wire.to_string());
                tt.wire_ids.insert(ids.id(from_wire));
            }
        }
        // Add wires from fixed connections
        for (to_wire, wire_conns) in tt.data.conns.iter() {
            tt.wires.insert(to_wire.to_string());
            let to_wire_id = ids.id(to_wire);
            tt.wire_ids.insert(to_wire_id);
            tt.driven_wire_ids.insert(to_wire_id);
            for from_wire in wire_conns.iter().map(|x| &x.from_wire) {
                tt.wires.insert(from_wire.to_string());
                tt.wire_ids.insert(ids.id(from_wire));
            }
        }
        // Add wires from bel pins
        for bel in tt.bels.iter() {
            for pin in bel.pins.iter() {
                let wire_name = pin.wire.rel_name();
                let wire_id = ids.id(&wire_name);
                tt.wire_ids.insert(wire_id);
                if pin.dir == PinDir::OUTPUT {
                    tt.driven_wire_ids.insert(wire_id);
                }
                tt.wires.insert(wire_name);
            }
        }
        // Determine which wires are neighbour wires, and the neighbour they are in
        for wire in tt.wires.iter() {
            let (maybe_n, basewire) = Neighbour::parse_wire(wire);
            if let Some(n) = maybe_n {
                tt.neighbour_wire_ids
                    .entry(n)
                    .or_insert(Vec::new())
                    .push(ids.id(basewire));
            }
        }
        tt.neighbours = tt.neighbour_wire_ids.keys().cloned().collect();
        return tt;
    }

    pub fn has_routing(&self) -> bool {
        !self.data.pips.is_empty() || !self.data.conns.is_empty()
    }
}

pub struct TileTypes {
    types: HashMap<String, TileType>,
}

impl TileTypes {
    pub fn new(db: &mut Database, ids: &mut IdStringDB, fam: &str, dev: &str) -> TileTypes {
        let tg = db.device_tilegrid(fam, dev);
        let unique_tiletypes: BTreeSet<String> =
            tg.tiles.iter().map(|t| t.1.tiletype.to_string()).collect();
        let mut types: HashMap<String, TileType> = HashMap::new();
        for tt in unique_tiletypes.iter() {
            types.insert(tt.to_string(), TileType::new(db, ids, fam, tt));
        }
        TileTypes { types }
    }
    pub fn get(&self, tt: &str) -> Option<&TileType> {
        self.types.get(tt)
    }
}
