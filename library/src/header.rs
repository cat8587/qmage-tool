use std::fmt::Debug;
use thiserror::Error;

pub const QMAGE_MAGIC_1: u16 = 0x4751;
pub const QMAGE_MAGIC_2: u16 = 0x4d51;
pub const QMAGE_MAGIC_VERSION: u8 = 11;
pub const MIN_HEADER_SIZE: usize = 12;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct QmageHeader {
    // offset: 0x0, little endian
    pub magic: u16,
    // offset: 0x2
    pub version: u8,
    // offset: 0x3
    pub image_type: QmagePixelType,

    // offset: 0x4, mask: 0x80
    pub is_multiframe: bool,
    // offset: 0x4, mask: 0x40
    pub use_chroma_key: bool,
    // offset: 0x4, mask: 0x20, inverted
    pub is_compressed: bool,
    // offset: 0x4, mask: 0x1f
    pub qp: u8,

    // offset: 0x5, mask: 0x8f
    pub unknown2: u8,
    // offset: 0x5, mask: 0x40
    // 16 bits if set, 8 bits otherwise
    pub bit_depth: bool,
    // offset: 0x5, mask: 0x20,
    // 16 bits if set, 8 bits otherwise
    pub alpha_depth: bool,
    // offset: 0x5, mask: 0x10
    pub custom_lookup_table: bool,

    // offset: 0x5, mask: 0x0f (above MAGIC_VERSION)
    // offset: 0x5, mask: 0x07 (at MAGIC_VERSION)
    // always 0                (below MAGIC_VERSION)
    pub codec: QmageCodec,

    // offset: 0x6, little endian
    pub width: u16,
    // offset: 0x8, little endian
    pub height: u16,

    // offset: 0xa, mask: 0xbf
    pub unknown3: u8,
    // offset: 0xa, mask: 0x40
    pub near_lossless_compression: bool,

    // offset: 0xb, mask: 0x43
    pub unknown4: u8,
    // offset: 0xb, mask: 0x80
    // https://en.wikipedia.org/wiki/9-slice_scaling
    pub is_9_patched: bool,
    // offset: 0xb, mask: 0x20
    pub is_opaque: bool,
    // offset: 0xb, mask: 0x10
    pub pre_multiplied: bool,
    // offset: 0xb, mask: 0x08
    pub use_indexed_color: bool,
    // offset: 0xb, mask: 0x04
    pub is_grayscale: bool,

    // offset: 0xc, length: 0xb
    pub multiframe_data: Option<MultiframeData>,

    // offset: 0xc, length: 0x4
    // probably unused
    pub unknown7: Option<[u8; 4]>,

    pub header_size: usize,
}

impl QmageHeader {
    pub fn read_header(header: &[u8]) -> Result<Self, QmageHeaderError> {
        if header.len() < 12 {
            return Err(QmageHeaderError::BufferTooSmall(header.len(), 12));
        }

        let magic = u16::from_le_bytes([header[0], header[1]]);
        if (magic != QMAGE_MAGIC_1) && (magic != QMAGE_MAGIC_2) {
            return Err(QmageHeaderError::UnknownMagicNumber(magic));
        }
        let version = header[2];
        let image_type = QmagePixelType::try_from(header[3])?;
        let is_multiframe = (header[4] & 0x80) > 0;
        let header_size = if is_multiframe {
            24
        } else {
            if image_type.has_transparency() {
                16
            } else {
                12
            }
        };

        if header.len() < header_size {
            return Err(QmageHeaderError::BufferTooSmall(header.len(), header_size));
        }

        let use_chroma_key = (header[4] & 0x40) > 0;
        let is_compressed = (header[4] & 0x20) == 0;
        let qp = header[4] & 0x1f;

        let bit_depth = (header[5] & 0x40) > 0;
        let alpha_depth = (header[5] & 0x20) > 0;
        let custom_lookup_table = (header[5] & 0x10) > 0;
        let unknown2 = header[5] & 0x8f;
        let codec_mask = if version > QMAGE_MAGIC_VERSION {
            0x07
        } else if version == QMAGE_MAGIC_VERSION {
            0x0f
        } else {
            0x00
        };
        let codec = QmageCodec::try_from(header[5] & codec_mask)?;

        let width = u16::from_le_bytes([header[6], header[7]]);
        let height = u16::from_le_bytes([header[8], header[9]]);

        let near_lossless_compression = (header[10] & 0x40) > 0;
        let unknown3 = header[10] & 0xbf;

        let is_9_patched = (header[11] & 0x80) > 0;
        let is_opaque = (header[11] & 0x20) > 0;
        let pre_multiplied = (header[11] & 0x10) > 0;
        let use_indexed_color = (header[11] & 0x08) > 0;
        let is_grayscale = (header[11] & 0x04) > 0;
        let unknown4 = header[11] & 0x43;

        let multiframe_data = if is_multiframe {
            let total_length = u16::from_le_bytes([header[12], header[13]]);
            let current_frame = u16::from_le_bytes([header[14], header[15]]);
            let frame_time = u16::from_le_bytes([header[16], header[17]]);
            let unknown5 = header[18] & 0xfe;
            let animation_repeats = (header[18] & 0x01) == 0;
            let unknown6 = [header[19], header[20], header[21], header[22], header[23]];
            Some(MultiframeData {
                total_length,
                current_frame,
                frame_time,
                unknown5,
                animation_repeats,
                unknown6,
            })
        } else {
            None
        };

        let unknown7 = if image_type.has_transparency() {
            Some([header[12], header[13], header[14], header[15]])
        } else {
            None
        };

        Ok(Self {
            magic,
            version,
            image_type,
            is_multiframe,
            use_chroma_key,
            is_compressed,
            qp,
            bit_depth,
            alpha_depth,
            custom_lookup_table,
            unknown2,
            codec,
            width,
            height,
            near_lossless_compression,
            unknown3,
            is_9_patched,
            is_opaque,
            pre_multiplied,
            use_indexed_color,
            is_grayscale,
            unknown4,
            multiframe_data,
            unknown7,
            header_size,
        })
    }

    pub fn is_valid(&self) -> Result<(), QmageHeaderError> {
        let expected_size = if self.is_multiframe {
            24
        } else {
            if self.image_type.has_transparency() {
                16
            } else {
                12
            }
        };

        if self.header_size != expected_size {
            return Err(QmageHeaderError::InvalidSize(
                self.header_size,
                expected_size,
            ));
        }

        Ok(())
    }

    #[inline]
    pub fn get_record_size(&self, is_first_frame: bool) -> Option<usize> {
        if let Some(m) = self.multiframe_data.as_ref() {
            if self.version == QMAGE_MAGIC_VERSION || (!is_first_frame) {
                Some(m.total_length as usize)
            } else {
                Some(4 * (m.total_length as usize))
            }
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct MultiframeData {
    // offset: 0xc, little endian
    // for first frame:
    // total_length * 4 = length of the frame and header
    // for every other frame:
    // total_length = length of frame and header
    pub total_length: u16,

    // offset: 0xe, little endian
    pub current_frame: u16,

    // offset: 0x10, little endian
    // probably the time it's on screen in milliseconds
    pub frame_time: u16,

    // offset: 0x12, mask: 0xfe
    // probably unused
    pub unknown5: u8,

    // offset: 0x12, mask: 0x01, inverted
    pub animation_repeats: bool,

    // offset: 0x13
    // probably also unused
    pub unknown6: [u8; 5],
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum QmagePixelType {
    // value: 0x00
    RGB565,
    // value: 0x03
    RGBA5658,
    // value: 0x06
    RGBA8888,
}

impl QmagePixelType {
    pub fn has_transparency(&self) -> bool {
        match self {
            Self::RGB565 => false,
            Self::RGBA5658 => true,
            Self::RGBA8888 => true,
        }
    }

    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::RGB565 => 2,
            Self::RGBA5658 => 3,
            Self::RGBA8888 => 4,
        }
    }
}

impl TryFrom<u8> for QmagePixelType {
    type Error = QmageHeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::RGB565),
            3 => Ok(Self::RGBA5658),
            6 => Ok(Self::RGBA8888),
            other => Err(Self::Error::UnknownImageType(other)),
        }
    }
}

impl Into<u8> for QmagePixelType {
    fn into(self) -> u8 {
        match self {
            Self::RGB565 => 0,
            Self::RGBA5658 => 3,
            Self::RGBA8888 => 6,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum QmageCodec {
    // value: 0x00
    V16ShortIndex,
    // value: 0x01
    W2Pass,
}

impl TryFrom<u8> for QmageCodec {
    type Error = QmageHeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::V16ShortIndex),
            1 => Ok(Self::W2Pass),
            other => Err(Self::Error::UnknownCodec(other)),
        }
    }
}

impl Into<u8> for QmageCodec {
    fn into(self) -> u8 {
        match self {
            Self::V16ShortIndex => 0,
            Self::W2Pass => 1,
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum QmageHeaderError {
    #[error("Unknown magic number: {0:#04x}")]
    UnknownMagicNumber(u16),

    #[error("Header buffer too small at {0} bytes, should be at least {1} bytes")]
    BufferTooSmall(usize, usize),

    #[error("Impossible size of {0}, must be 12, 16 or 24")]
    ImpossibleSize(usize),

    #[error("Invalid header size of {0}, expected {1} bytes")]
    InvalidSize(usize, usize),

    #[error("Unknown image type {0}, should be 0, 3 or 6")]
    UnknownImageType(u8),

    #[error("Unknown codec byte {0}, should be 0 or 1")]
    UnknownCodec(u8),

    #[error("multiframe_data provided when it shouldn't be")]
    UnexpectedMultiframeData,

    #[error("multiframe_data not provided when it should be")]
    MissingMultiframeData,

    #[error("unknown7 provided when it shouldn't be")]
    UnexpectedUnknown7,

    #[error("unknown7 not provided when it should be")]
    MissingUnkown7,
}
