// Wire normalisation for Nexus
use crate::chip::*;
use regex::Regex;

lazy_static! {
    //  - General wire name format
    static ref WIRE_RE: Regex = Regex::new(r"^R(\d+)C(\d+)_(.+)$").unwrap();
    //  - Global clock distribution levels
    // Horizontal branches
    static ref GLB_HBRANCH_RE: Regex = Regex::new(r"^HPBX(\d{2})00$").unwrap();
    // Vertical spine
    static ref GLB_SPINE_RE: Regex = Regex::new(r"^VPSX(\d{2})00$").unwrap();
    // Horizontal rows
    static ref GLB_HROW_RE: Regex = Regex::new(r"^HPRX(\d{2})00$").unwrap();
    // Horizontal row drivers
    static ref GLB_HROWD_RE: Regex = Regex::new(r"^([LR])HPRX(\d+)$").unwrap();
    // Central clock signals
    static ref GLB_CMUXI_RE: Regex = Regex::new(r"^J([HV])F([NESW])(\d+)_(DCSMUX|CMUX)_CORE_(DCSMUX|CMUX)(\d)$").unwrap();
    // Perimeter clock signals
    static ref GLB_MIDMUX_RE: Regex = Regex::new(r"^(.*)(.)MID_CORE_(.)MIDMUX$").unwrap();
    // Edge clock signals
    static ref ECLK_RE: Regex = Regex::new(r"^JECLKOUT(\d)_ECLKCASMUX_CORE_ECLKCASMUX(\d+)$").unwrap();
    // Edge clock sources
    static ref ECLK_MUXIN_RE: Regex = Regex::new(r"^JMUXIN(\d+)_ECLKBANK_CORE_ECLKBANK(\d+)$").unwrap();
    // DQS group shared signals
    static ref DQS_GROUP_RE: Regex = Regex::new(r"^J(WRPNTR\d|RDPNTR\d|DQSR90|DQSW270|DQSW)_DQSBUF_CORE_I_DQS_TOP$").unwrap();

    // - CIB (general routing) regex
    static ref GENERAL_ROUTE_RE: Regex = Regex::new(r"R\d+C\d+_[VH]\d{2}[NESWTLBR]\d{4}").unwrap();
    static ref CIB_SIG_RE: Regex = Regex::new(r"R\d+C\d+_J?(CIBMUXOUT|CIBMUXIN)?[ABCDMFQ]\d").unwrap();
    static ref CIB_CTRLSIG_RE: Regex = Regex::new(r"R\d+C\d+_J?(CIBMUXOUT|CIBMUXIN)?(CLK|LSR|CE)\d").unwrap();
    static ref CIB_BOUNCE_RE: Regex = Regex::new(r"R\d+C\d+_[NESW]BOUNCE").unwrap();

    static ref H_WIRE_RE: Regex = Regex::new(r"^H(\d{2})([EW])(\d{2})(\d{2})$").unwrap();
    static ref V_WIRE_RE: Regex = Regex::new(r"^V(\d{2})([NS])(\d{2})(\d{2})$").unwrap();

}

fn is_full_global_wn(wire: &str) -> bool {
    GLB_HROWD_RE.is_match(wire)
        || GLB_CMUXI_RE.is_match(wire)
        || GLB_MIDMUX_RE.is_match(wire)
        || ECLK_RE.is_match(wire)
        || ECLK_MUXIN_RE.is_match(wire)
}

pub fn handle_edge_name(
    max_x: u32,
    max_y: u32,
    tx: u32,
    ty: u32,
    wx: u32,
    wy: u32,
    wn: &str,
) -> (String, u32, u32) {
    /*
    At the edges of the device, canonical wire names do not follow normal naming conventions, as they
    would mean the nominal position of the wire would be outside the bounds of the chip. Before we add routing to the
    database, we must however normalise these names to what they would be if not near the edges, otherwise there is a
    risk of database conflicts, having multiple names for the same wire.

    Returns a tuple (netname, x, y)
    */
    if let Some(hm) = H_WIRE_RE.captures(wn) {
        match &hm[1] {
            "01" => {
                // H01xyy00 --> x+1, H01xyy01
                if tx == max_x - 1 {
                    assert_eq!(hm[4].to_string(), "00");
                    return (format!("H01{}{}01", &hm[2], &hm[3]), wx + 1, wy);
                }
            }
            "02" => {
                if tx == 1 {
                    // H02E0002 --> x-1, H02E0001
                    // H02W0000 --> x-1, H02W00001
                    if &hm[2] == "E" && wx == 1 && &hm[4] == "02" {
                        return (format!("H02E{}01", &hm[3]), wx - 1, wy);
                    } else if &hm[2] == "W" && wx == 1 && &hm[4] == "00" {
                        return (format!("H02W{}01", &hm[3]), wx - 1, wy);
                    }
                } else if tx == max_x - 1 {
                    // H02E0000 --> x+1, H02E0001
                    // H02W0002 --> x+1, H02W00001
                    if &hm[2] == "E" && wx == max_x - 1 && &hm[4] == "00" {
                        return (format!("H02E{}01", &hm[3]), wx + 1, wy);
                    } else if &hm[2] == "W" && wx == max_x - 1 && &hm[4] == "02" {
                        return (format!("H02W{}01", &hm[3]), wx + 1, wy);
                    }
                }
            }
            "06" => {
                if tx <= 5 {
                    // x-2, H06W0302 --> x-3, H06W0303
                    // x-2, H06E0004 --> x-3, H06E0003
                    // x-1, H06W0301 --> x-3, H06W0303
                    // x-1, H06E0305 --> x-3, H06E0303
                    match &hm[2] {
                        "W" => {
                            return (
                                format!("H06W{}03", &hm[3]),
                                wx - (3 - hm[4].parse::<u32>().unwrap()),
                                wy,
                            )
                        }
                        "E" => {
                            return (
                                format!("H06E{}03", &hm[3]),
                                wx - (hm[4].parse::<u32>().unwrap() - 3),
                                wy,
                            )
                        }
                        _ => panic!("unknown H06 wire {}", wn),
                    }
                } else if tx >= max_x - 5 {
                    match &hm[2] {
                        "W" => {
                            return (
                                format!("H06W{}03", &hm[3]),
                                wx + (hm[4].parse::<u32>().unwrap() - 3),
                                wy,
                            )
                        }
                        "E" => {
                            return (
                                format!("H06E{}03", &hm[3]),
                                wx + (3 - hm[4].parse::<u32>().unwrap()),
                                wy,
                            )
                        }
                        _ => panic!("unknown H06 wire {}", wn),
                    }
                }
            }
            _ => panic!("bad HWIRE {}", &wn),
        }
    }
    if let Some(vm) = V_WIRE_RE.captures(wn) {
        match &vm[1] {
            "01" => {
                if ty == 1 {
                    if wy == 1 && &vm[2] == "N" && &vm[4] == "00" {
                        return (format!("V01{}{}01", &vm[2], &vm[3]), wx, wy - 1);
                    }
                    if wy == 1 && &vm[2] == "S" && &vm[4] == "01" {
                        return (format!("V01{}{}01", &vm[2], &vm[3]), wx, wy - 1);
                    }
                }
            }
            "02" => {
                if ty == 1 {
                    if &vm[2] == "S" && wy == 1 && &vm[4] == "02" {
                        return (format!("V02S{}01", &vm[3]), wx, wy - 1);
                    }
                    if &vm[2] == "N" && wy == 1 && &vm[4] == "00" {
                        return (format!("V02N{}01", &vm[3]), wx, wy - 1);
                    }
                } else if ty == max_y - 1 {
                    if &vm[2] == "S" && wy == (max_y - 1) && &vm[4] == "00" {
                        return (format!("V02S{}01", &vm[3]), wx, wy + 1);
                    }
                    if &vm[2] == "N" && wy == (max_y - 1) && &vm[4] == "02" {
                        return (format!("V02N{}01", &vm[3]), wx, wy + 1);
                    }
                }
            }
            "06" => {
                if ty <= 5 {
                    // y-2, V06N0302 --> y-3, H06W0303
                    // y-2, V06S0004 --> y-3, V06S0003
                    // y-1, V06N0301 --> y-3, V06N0303
                    // y-1, V06S0005 --> y-3, V06S0003
                    match &vm[2] {
                        "N" => {
                            return (
                                format!("V06N{}03", &vm[3]),
                                wx,
                                wy - (3 - vm[4].parse::<u32>().unwrap()),
                            )
                        }
                        "S" => {
                            return (
                                format!("V06S{}03", &vm[3]),
                                wx,
                                wy - (vm[4].parse::<u32>().unwrap() - 3),
                            )
                        }
                        _ => panic!("unknown V06 wire {}", wn),
                    }
                } else if ty >= max_y - 5 {
                    // y+2, V06N0304 --> y+3, V06N0303
                    // y+2, V06S0302 --> x+3, V06S0303
                    match &vm[2] {
                        "N" => {
                            return (
                                format!("V06N{}03", &vm[3]),
                                wx,
                                wy + (vm[4].parse::<u32>().unwrap() - 3),
                            )
                        }
                        "S" => {
                            return (
                                format!("V06S{}03", &vm[3]),
                                wx,
                                wy + (3 - vm[4].parse::<u32>().unwrap()),
                            )
                        }
                        _ => panic!("unknown V06 wire {}", wn),
                    }
                }
            }
            _ => panic!("bad VWIRE {}", &wn),
        }
    }
    (wn.to_string(), wx, wy)
}

pub fn normalize_wire(chip: &Chip, tile: &Tile, wire: &str) -> String {
    /*
    Wire name normalisation for tile wires and fuzzing
    All net names that we have access too are canonical, global names
    These are thus no good for building up a database that is the same for all tiles
    of a given type, as the names will be different in each location.

    Lattice names are of the form R{r}C{c}_{WIRENAME}

    Hence, we normalise names in the following way:
     - Global wires have the prefix "G:" added
     - Wires where (r, c) correspond to the current tile have their prefix removed
     - Wires to the left (in TAPs) are given the prefix BRANCH_L:, and wires to the right
       are given the prefix BRANCH_R:
     - Wires corresponding to the global network branch, spine or HROWs are given
       BRANCH:, SPINE:, or HROW: prefixes accordingly
     - Wires within a DQS group are given the prefix DQSG:
     - Other wires are given a relative position prefix using the syntax
       ([NS]\d+)?([EW]\d+)?:
       so a wire whose nominal location is 6 tiles up would be given a prefix N6:
       a wire whose nominal location is 2 tiles down and 1 tile right would be given a prefix
       S2E1:

    N.B. the ':' symbol is not legal in some contexts such as FASM. In these cases it is to be replaced by a
    '__' token.

    This is more complicated at the edges of the device, where irregular names are used to keep the row and column
    of the nominal position in bounds. Extra logic is be needed to catch and regularise these cases.

    Returns the normalised netname
    */
    let spw = WIRE_RE
        .captures(wire)
        .expect(&format!("invalid wire name '{}'", wire));
    let (mut wy, mut wx, mut wn) = (
        spw[1].parse::<u32>().unwrap(),
        spw[2].parse::<u32>().unwrap(),
        &spw[3],
    );
    if wn.ends_with("VCCHPRX") || wn.ends_with("VCCHPBX") || wn.ends_with("VCC") {
        return "G:VCC".to_string();
    }
    if tile.name.contains("TAP") && wn.starts_with("H") {
        if wx < tile.x {
            return format!("BRANCH_L:{}", wn);
        } else if wx > tile.x {
            return format!("BRANCH_R:{}", wn);
        } else {
            panic!("unable to determine TAP side of {} in {}", wire, tile.name);
        }
    }
    if GLB_HBRANCH_RE.is_match(wn) {
        return format!("BRANCH:{}", wn);
    } else if GLB_SPINE_RE.is_match(wn) {
        return format!("SPINE:{}", wn);
    } else if GLB_HROW_RE.is_match(wn) {
        return format!("HROW:{}", wn);
    } else if is_full_global_wn(wn) {
        return format!("G:{}", wn);
    } else if DQS_GROUP_RE.is_match(wn) {
        return format!("DQSG:{:}", wn);
    }
    let en = handle_edge_name(
        chip.data.max_col,
        chip.data.max_row,
        tile.x,
        tile.y,
        wx,
        wy,
        wn,
    );
    wn = &en.0;
    wx = en.1;
    wy = en.2;
    if wx == tile.x && wy == tile.y {
        return wn.to_string();
    } else {
        let mut prefix = String::new();
        if wy < tile.y {
            prefix.push_str(&format!("N{}", tile.y - wy));
        }
        if wy > tile.y {
            prefix.push_str(&format!("S{}", wy - tile.y));
        }
        if wx > tile.x {
            prefix.push_str(&format!("E{}", wx - tile.x));
        }
        if wx < tile.x {
            prefix.push_str(&format!("W{}", tile.x - wx));
        }
        return format!("{}:{}", prefix, wn);
    }
}