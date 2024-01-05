use zerocopy::{AsBytes, FromBytes, FromZeroes};
    use std::fs::OpenOptions;
    use std::path::PathBuf;

    use memmap2::{Mmap, MmapMut};
struct BNode {
    data: [u8; 64 * 1024]
}

enum NodeType {
    BNODE_NODE = 1, // internal nodes without values
    BNODE_LEAF = 2, // leaf nodes with values
}

struct BTree {
    // pointer (a nonzero page number)
    root: u64
}

impl BTree {
    fn get(nodeID: u64)->BNode{
        todo!()
    }
}

/*type BTree struct {
    // pointer (a nonzero page number)
    root uint64
    // callbacks for managing on-disk pages
    get func(uint64) BNode // dereference a pointer new func(BNode) uint64 // allocate a new page del func(uint64) // deallocate a page
}*/
#[derive(AsBytes, FromBytes, FromZeroes)]
#[repr(C)]
struct PacketHeader {
    src_port: [u8; 2],
    dst_port: [u8; 2],
    length: [u8; 2],
    checksum: [u8; 2],
}

fn main()-> Result<(), anyhow::Error>  {
    // bytes
    println!("Hello, world!");
    use bytes::{BytesMut, BufMut};

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
    let header = PacketHeader {
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

    let mmap2 = unsafe { Mmap::map(&file)?};

    let header = PacketHeader::ref_from(&mmap2[..]).unwrap();

    assert_eq!(header.src_port, [0, 1]);
    assert_eq!(header.dst_port, [2, 3]);
    assert_eq!(header.length, [4, 5]);
    assert_eq!(header.checksum, [6, 7]);

    Ok(())
}
