use crate::header::QmagePixelType;

pub fn convert_565_to_888(w: u16) -> (u8, u8, u8) {
    let r5 = ((w >> 11) & 0x1F) as u8;
    let g6 = ((w >> 5) & 0x3F) as u8;
    let b5 = (w & 0x1F) as u8;

    (
        (r5 << 3) | (r5 >> 2),
        (g6 << 2) | (g6 >> 4),
        (b5 << 3) | (b5 >> 2),
    )
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Pixel {
    RGB565(u16),
    RGBA5658(u16, u8),
    RGBA8888(u8, u8, u8, u8),
}

impl Pixel {
    pub fn pixel_type(&self) -> QmagePixelType {
        match self {
            Self::RGB565(_) => QmagePixelType::RGB565,
            Self::RGBA5658(_, _) => QmagePixelType::RGBA5658,
            Self::RGBA8888(_, _, _, _) => QmagePixelType::RGBA8888,
        }
    }

    pub fn zero_pixel(type_: QmagePixelType) -> Self {
        match type_ {
            QmagePixelType::RGB565 => Self::RGB565(0),
            QmagePixelType::RGBA5658 => Self::RGBA5658(0, 0),
            QmagePixelType::RGBA8888 => Self::RGBA8888(0, 0, 0, 0),
        }
    }

    pub fn as_rgba8888(&self) -> Pixel {
        match self {
            Self::RGB565(w) => {
                let as_888 = convert_565_to_888(*w);
                Self::RGBA8888(as_888.0, as_888.1, as_888.2, 255)
            }
            Self::RGBA5658(w, a) => {
                let as_888 = convert_565_to_888(*w);
                Self::RGBA8888(as_888.0, as_888.1, as_888.2, *a)
            }
            Self::RGBA8888(_, _, _, _) => *self,
        }
    }

    pub fn convert_to_type(&self, new_type: QmagePixelType) -> Pixel {
        if new_type == self.pixel_type() {
            return *self;
        }
        let rgba8888 = self.as_rgba8888();
        let (r, g, b, a) = match rgba8888 {
            Self::RGBA8888(r, g, b, a) => (r, g, b, a),
            _ => unreachable!(),
        };

        let (r5, g6, b5) = ((r >> 3) as u16, (g >> 2) as u16, (b >> 3) as u16);
        let w = (r5 << 11) | (g6 << 5) | b5;

        match new_type {
            QmagePixelType::RGB565 => Pixel::RGB565(w),
            QmagePixelType::RGBA5658 => Pixel::RGBA5658(w, a),
            QmagePixelType::RGBA8888 => Pixel::RGBA8888(r, g, b, a),
        }
    }

    pub fn from_slice(slice: &[u8]) -> Option<Pixel> {
        match slice.len() {
            2 => Some(Pixel::RGB565((slice[0] as u16) << 8 | (slice[1] as u16))),
            3 => Some(Pixel::RGBA5658(
                (slice[0] as u16) << 8 | (slice[1] as u16),
                slice[2],
            )),
            4 => Some(Pixel::RGBA8888(slice[0], slice[1], slice[2], slice[3])),
            _ => None,
        }
    }

    pub fn write_to_slice(&self, slice: &mut [u8]) -> Option<()> {
        let size = self.pixel_type().bytes_per_pixel();
        if slice.len() < size {
            return None;
        }

        match self {
            Pixel::RGB565(w) => {
                slice[0] = (w >> 8) as u8;
                slice[1] = (w & 0xff) as u8;
            }
            Pixel::RGBA5658(w, a) => {
                slice[0] = (w >> 8) as u8;
                slice[1] = (w & 0xff) as u8;
                slice[2] = *a;
            }
            Pixel::RGBA8888(r, g, b, a) => {
                slice[0] = *r;
                slice[1] = *g;
                slice[2] = *b;
                slice[3] = *a;
            }
        };

        Some(())
    }
}

impl Into<Vec<u8>> for Pixel {
    fn into(self) -> Vec<u8> {
        let mut vector = vec![0u8; self.pixel_type().bytes_per_pixel()];
        self.write_to_slice(vector.as_mut_slice());
        vector
    }
}

#[derive(Clone)]
pub struct PixelArray {
    // invariants:
    // pixel_size * width == line_size
    // line_size * height == buffer.len()
    // pixel_size == pixel_type.bytes_per_pixel()
    pixel_type: QmagePixelType,
    pixel_size: usize,
    line_size: usize,
    width: usize,
    height: usize,
    buffer: Vec<u8>,
}

impl PixelArray {
    pub fn new(pixel_type: QmagePixelType, width: u16, height: u16) -> Self {
        let width = width as usize;
        let height = height as usize;
        let pixel_size = pixel_type.bytes_per_pixel();
        let line_size = pixel_size * width;
        let total_size = line_size * height;
        let buffer = vec![0; total_size];
        Self {
            pixel_type,
            pixel_size,
            line_size,
            width,
            height,
            buffer,
        }
    }

    #[inline]
    pub fn pixel_type(&self) -> QmagePixelType {
        self.pixel_type
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn buffer<'a>(&'a self) -> &'a [u8] {
        self.buffer.as_slice()
    }

    #[inline]
    pub fn buffer_mut<'a>(&'a mut self) -> &'a mut [u8] {
        self.buffer.as_mut_slice()
    }

    pub fn get_pixel_slice<'a>(&'a self, x: usize, y: usize) -> Option<&'a [u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(
            &self.buffer[x * self.pixel_size + y * self.line_size
                ..(x + 1) * self.pixel_size + y * self.line_size],
        )
    }

    pub fn get_pixel_mut_slice<'a>(&'a mut self, x: usize, y: usize) -> Option<&'a mut [u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(
            &mut self.buffer[x * self.pixel_size + y * self.line_size
                ..(x + 1) * self.pixel_size + y * self.line_size],
        )
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Pixel> {
        let maybe_slice = self.get_pixel_slice(x, y);
        match maybe_slice {
            Some(slice) => Pixel::from_slice(slice),
            None => None,
        }
    }

    pub fn get_pixel_zero(&self, x: usize, y: usize) -> Pixel {
        self.get_pixel(x, y)
            .unwrap_or(Pixel::zero_pixel(self.pixel_type))
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) -> Option<()> {
        if x >= self.width || y >= self.height || pixel.pixel_type() != self.pixel_type {
            return None;
        }

        let slice = &mut self.buffer[x * self.pixel_size + y * self.line_size
            ..(x + 1) * self.pixel_size + y * self.line_size];
        pixel.write_to_slice(slice);
        Some(())
    }

    pub fn convert_to_pixel_type(&self, new_type: QmagePixelType) -> PixelArray {
        if new_type == self.pixel_type {
            return self.clone();
        }
        let mut new_array = PixelArray::new(new_type, self.width as u16, self.height as u16);

        for y in 0..self.height {
            for x in 0..self.width {
                new_array.set_pixel(
                    x,
                    y,
                    self.get_pixel(x, y)
                        .unwrap_or_else(|| unreachable!())
                        .convert_to_type(new_type),
                );
            }
        }

        new_array
    }

    #[cfg(feature = "image-convert")]
    pub fn rgba8888_to_rgbaimage(self) -> Option<image::RgbaImage> {
        if self.pixel_type != QmagePixelType::RGBA8888 {
            return None;
        }
        let (width, height) = (self.width, self.height);
        image::RgbaImage::from_raw(width as u32, height as u32, self.buffer)
    }

    #[cfg(feature = "image-convert")]
    pub fn rgbaimage_to_rgba8888(image: image::RgbaImage) -> Self {
        let width = image.width() as usize;
        let height = image.height() as usize;
        let pixel_type = QmagePixelType::RGBA8888;
        let pixel_size = pixel_type.bytes_per_pixel();
        let line_size = pixel_size * width;
        let buffer = image.into_raw();
        Self {
            pixel_type,
            pixel_size,
            line_size,
            width,
            height,
            buffer,
        }
    }
}
