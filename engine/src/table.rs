use std::mem::size_of;

use chess::{EncodedMove, Zobrist};

// 16 MB of ram for the hashtables
const MEM_SIZE: usize = 16777216;

//#################################################################################################
//
//                                         enum NodeType
//
//#################################################################################################

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum NodeType {
    Alpha = 0,
    Beta  = 1,
    Exact = 2,
}

impl NodeType {
    const NODE_TYPES: [NodeType; 3] = [
        NodeType::Alpha,
        NodeType::Beta,
        NodeType::Exact,
    ];
}

//#################################################################################################
//
//                                        struct NodeInfo
//
//#################################################################################################

#[repr(transparent)]
#[derive(Clone, Copy)]
struct NodeInfo(u16);

impl NodeInfo {
    const ZERO: NodeInfo = NodeInfo(0);

    fn new(age: u16, flag: NodeType) -> NodeInfo {
        NodeInfo(age | (flag as u16).wrapping_shl(14))
    }

    fn age(&self) -> u16 {
        self.0 & 0x3fff
    }

    fn flag(&self) -> NodeType {
        NodeType::from(NodeType::NODE_TYPES[self.0.wrapping_shr(14) as usize])
    }

    fn get_raw(&self) -> u32 {
        self.0 as u32
    }
}

//#################################################################################################
//
//                                        struct Node
//
//#################################################################################################

#[repr(packed)]
struct Entry {
    zobrist: Zobrist,
    mv: EncodedMove,
    eval: i16,
    info: NodeInfo,
    depth: u8,
    checksum: u32,
}

impl Entry {
    const ZERO: Entry = Entry {
        zobrist: Zobrist::ZERO,
        mv: EncodedMove::ZERO,
        eval: 0,
        info: NodeInfo::ZERO,
        depth: 0,
        checksum: 0,
    };

    fn checksum(zobrist: Zobrist, mv: EncodedMove, eval: i16, info: NodeInfo, depth: u8) -> u32 {
        zobrist.get_lower() ^ mv.get_raw() ^ (eval as u32) ^ info.get_raw() ^ (depth as u32)
    }
}

//#################################################################################################
//
//                                        struct Table
//
//#################################################################################################

const SIZE: usize = MEM_SIZE / size_of::<Entry>();

pub struct Table {
    entries: [Entry; SIZE],
}

static mut TABLE: Table = Table {
    entries: [Entry::ZERO; SIZE],
};

impl Table {
    pub fn put(zobrist: Zobrist, mv: EncodedMove, eval: i16, age: i16, flag: NodeType, depth: u8, ) {
        let i = zobrist.index(SIZE);

        let entry = unsafe {&mut TABLE.entries[i]};

        let local_checksum = Entry::checksum(entry.zobrist, entry.mv, entry.eval, entry.info, entry.depth);
        if entry.checksum == local_checksum {
            // Always replace older entries
            // Prefr depth, exact flag gets 4 points and beta flag 2

            let mut put_score = depth as i8 - entry.depth as i8;
        }
    
    }
}