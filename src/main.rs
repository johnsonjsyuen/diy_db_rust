use std::fs::OpenOptions;
use std::path::PathBuf;

use byteorder::{ByteOrder, LittleEndian};
use memmap2::{Mmap, MmapMut};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

const HEADER: u8 = 4;
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

/*// pointers
func (node BNode) getPtr(idx uint16) uint64 { assert(idx < node.nkeys())
pos := HEADER + 8*idx
return binary.LittleEndian.Uint64(node.data[pos:])
}
func (node BNode) setPtr(idx uint16, val uint64) {
assert(idx < node.nkeys())
pos := HEADER + 8*idx
binary.LittleEndian.PutUint64(node.data[pos:], val)
}*/
impl BNode {
    fn btype(self) -> u16 {
        LittleEndian::read_u16(&self.data[0..1])
    }
    fn nkeys(self) -> u16 {
        LittleEndian::read_u16(&self.data[2..4])
    }
    fn setHeader(mut self, btype: u16, nkeys: u16) -> Result() {
        LittleEndian::write_u16(&mut self.data[0..1], btype);
        LittleEndian::write_u16(&mut self.data[2..4], nkeys);
        Ok(())
    }
    fn getPtr(self, idx: u16) -> u64 {
        assert!(idx < self.nkeys(), "Tried to get pointer beyond key offsets");
        let pos = HEADER + 8 * idx;
        LittleEndian::read_u64(&self.data[pos..pos + 8])
    }
    fn setPtr(self, idx: u16, val: u64) -> Result() {
        assert!(idx < self.nkeys(), "Tried to set pointer beyond key offsets");
        let pos = HEADER + 8 * idx;
        LittleEndian::write_u64(&self.data[pos..pos + 8], val);
        Ok(())
    }
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
    fn del(nodeID: u64) -> Result(()) { // deallocate a page
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

    Ok(())
}
