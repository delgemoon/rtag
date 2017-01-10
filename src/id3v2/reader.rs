use id3v2;
use scanner;
use std::io;

const HEADER_LEN: usize = 10;
const EXTENDED_HEADER_SIZE_LEN: usize = 4;

pub trait FrameIterator {
    fn has_next_frame(&mut self) -> bool;
    fn next_frame(&mut self) -> io::Result<id3v2::tag::frame::Frame>;
}

pub struct FrameReader<'a> {
    reader: Reader<'a>
}

impl<'a> FrameReader<'a> {
    pub fn new(scanner: &'a mut scanner::Scanner) -> io::Result<Self> {
        let mut reader = try!(Reader::new(scanner));
        // skip extended header
        reader.get_extended_header();

        Ok(FrameReader {
            reader: reader
        })
    }
}

impl<'a> FrameIterator for FrameReader<'a> {
    fn has_next_frame(&mut self) -> bool {
        self.reader.has_next_frame()
    }

    fn next_frame(&mut self) -> io::Result<id3v2::tag::frame::Frame> {
        self.reader.next_frame()
    }
}

pub struct Reader<'a> {
    header: id3v2::tag::header::Header,
    scanner: &'a mut scanner::Scanner
}

impl<'a> Reader<'a> {
    pub fn new(scanner: &'a mut scanner::Scanner) -> io::Result<Self> {
        let bytes = try!(scanner.read_as_bytes(HEADER_LEN));
        let header = id3v2::tag::header::Header::new(bytes);

        Ok(Reader {
            header: header,
            scanner: scanner
        })
    }

    pub fn get_extended_header(&mut self) -> Option<id3v2::tag::header::ExtendedHeader> {
        if !self.header.has_flag(id3v2::tag::header::HeaderFlag::ExtendedHeader) {
            return None
        }

        if let Ok(bytes) = self.scanner.read_as_bytes(self::EXTENDED_HEADER_SIZE_LEN) {
            let size = match self.header.get_version() {
                // Did not explained for whether big-endian or synchsafe in "http://id3.org/id3v2.3.0".
                3 => id3v2::bytes::to_u32(&bytes),
                // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
                // see "http://id3.org/id3v2.4.0-structure".
                _ => id3v2::bytes::to_synchsafe(&bytes),
            };

            if let Ok(bytes) = self.scanner.read_as_bytes(size as usize) {
                return Some(id3v2::tag::header::ExtendedHeader::new(size, &bytes));
            }
        }
        None
    }
}

impl<'a> FrameIterator for Reader<'a> {
    fn has_next_frame(&mut self) -> bool {
        id3v2::tag::frame::Frame::has_next_frame(self.scanner, &self.header)
    }

    fn next_frame(&mut self) -> io::Result<id3v2::tag::frame::Frame> {
        id3v2::tag::frame::Frame::new(self.scanner)
    }
}