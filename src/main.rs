use std::fs::OpenOptions;
use std::path::PathBuf;

use byteorder::{ByteOrder, LittleEndian};
use memmap2::{Mmap, MmapMut};
use rocksdb::{ColumnFamilyDescriptor, DB, Options};
use zerocopy::{AsBytes, FromBytes, FromZeroes};


const HEADER: usize = 4;
const BTREE_PAGE_SIZE: u16 = 4096;
const BTREE_MAX_KEY_SIZE: u16 = 1000;
const BTREE_MAX_VAL_SIZE: u16 = 3000;

enum NodeType {
    BNODE_NODE = 1,
    // internal nodes without values
    BNODE_LEAF = 2, // leaf nodes with values
}

struct BNode {
    data: [u8; 64 * 1024],
}

impl BNode {
    fn btype(&self) -> u16 {
        LittleEndian::read_u16(&self.data[0..1])
    }
    fn nkeys(&self) -> u16 {
        LittleEndian::read_u16(&self.data[2..4])
    }
    fn setHeader(&mut self, btype: u16, nkeys: u16) -> Result<(), anyhow::Error> {
        LittleEndian::write_u16(&mut self.data[0..1], btype);
        LittleEndian::write_u16(&mut self.data[2..4], nkeys);
        Ok(())
    }
    fn getPtr(&self, idx: u16) -> u64 {
        assert!(idx < self.nkeys(), "Tried to get pointer beyond key offsets");
        let pos = HEADER + (8 * idx) as usize;
        LittleEndian::read_u64(&self.data[pos..pos + 8])
    }
    fn setPtr(&mut self, idx: u16, val: u64) -> Result<(), anyhow::Error> {
        assert!(idx < self.nkeys(), "Tried to set pointer beyond key offsets");
        let pos = HEADER + (8 * idx) as usize;
        LittleEndian::write_u64(&mut self.data[pos..pos + 8], val);
        Ok(())
    }
    fn offsetPos(node: &BNode, idx: u16) -> usize {
        assert!(1 <= idx && idx <= node.nkeys(), "Tried to access pointer beyond range");
        HEADER + (8 * node.nkeys() + 2 * (idx - 1)) as usize
    }

    fn getOffset(&self, idx: u16) -> u16 {
        if idx == 0 {
            0
        } else {
            let pos = BNode::offsetPos(self, idx);
            LittleEndian::read_u16(&self.data[pos..pos+8])
        }
    }

    fn setOffset(&mut self, idx: u16, offset: u16) {
        let pos = BNode::offsetPos(self, idx);
        LittleEndian::write_u16(&mut self.data[pos..pos+8], offset);
    }
    /*
    // key-values
func (node BNode) kvPos(idx uint16) uint16 {
assert(idx <= node.nkeys())
return HEADER + 8*node.nkeys() + 2*node.nkeys() + node.getOffset(idx)
}
func (node BNode) getKey(idx uint16) []byte {
assert(idx < node.nkeys())
pos := node.kvPos(idx)
klen := binary.LittleEndian.Uint16(node.data[pos:]) return node.data[pos+4:][:klen]
}
func (node BNode) getVal(idx uint16) []byte {
assert(idx < node.nkeys())
pos := node.kvPos(idx)
klen := binary.LittleEndian.Uint16(node.data[pos+0:]) vlen := binary.LittleEndian.Uint16(node.data[pos+2:]) return node.data[pos+4+klen:][:vlen]
}
     */

}

struct BTree {
    // pointer (a nonzero page number)
    root: u64,
}

impl BTree {
    fn get(nodeID: u64) -> BNode { // dereference a pointer
        todo!()
    }
    fn new(node: BNode) -> u64 {  // allocate a new page
        todo!()
    }
    fn del(nodeID: u64) -> Result<(), anyhow::Error> { // deallocate a page
        todo!()
    }
}


#[derive(AsBytes, FromBytes, FromZeroes)]
#[repr(C)]
struct DatabaseHeader {
    src_port: [u8; 2],
    dst_port: [u8; 2],
    length: [u8; 2],
    checksum: [u8; 2],
}

fn main() -> Result<(), anyhow::Error> {
    // bytes
    println!("Hello, world!");
    use bytes::{BufMut, BytesMut};

    let mut buf = BytesMut::with_capacity(1024);
    buf.put(&b"hello world"[..]);
    buf.put_u16(1234);

    let a = buf.split();
    assert_eq!(a, b"hello world\x04\xD2"[..]);

    buf.put(&b"goodbye world"[..]);

    let b = buf.split();
    assert_eq!(b, b"goodbye world"[..]);

    assert_eq!(buf.capacity(), 998);

    // zero-copy
    let header = DatabaseHeader {
        src_port: [0, 1],
        dst_port: [2, 3],
        length: [4, 5],
        checksum: [6, 7],
    };

    let mut bytes = [0, 0, 0, 0, 0, 0, 0, 0];

    {
        header.write_to(&mut bytes[..]);
    }

    assert_eq!(bytes, [0, 1, 2, 3, 4, 5, 6, 7]);

    // mmap

    let path: PathBuf = PathBuf::from("dbfile");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?;
    file.set_len(bytes.len() as u64)?;

    let mut mmap = unsafe { MmapMut::map_mut(&file)? };

    mmap.copy_from_slice(&bytes);

    let mmap2 = unsafe { Mmap::map(&file)? };

    let header = DatabaseHeader::ref_from(&mmap2[..]).unwrap();

    assert_eq!(header.src_port, [0, 1]);
    assert_eq!(header.dst_port, [2, 3]);
    assert_eq!(header.length, [4, 5]);
    assert_eq!(header.checksum, [6, 7]);

    // rocks db
    let path = "rocksdbfile";
    let mut cf_opts = Options::default();
    cf_opts.set_max_write_buffer_number(16);
    let cf = ColumnFamilyDescriptor::new("cf1", cf_opts);

    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);
    {
        let db = DB::open_cf_descriptors(&db_opts, path, vec![cf]).unwrap();
        db.put(b"santi","dog");
        db.put(b"pippen","dog");
        db.put(b"snowy","dog");
        match db.get(b"santi") {
            Ok(Some(value)) => println!("retrieved value {}", String::from_utf8(value).unwrap()),
            Ok(None) => println!("value not found"),
            Err(e) => println!("operational problem encountered: {}", e),
        }
        //db.iterator(IteratorMode::Start).for_each(|Result{(k,v),e}|println!("retrieved value {}", String::from_utf8(t).unwrap()))
    }
    let _ = DB::destroy(&db_opts, path);
    Ok(())
}
