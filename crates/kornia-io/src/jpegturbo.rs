use std::sync::{Arc, Mutex};
use turbojpeg;

use kornia_image::{Image, ImageError, ImageSize};

/// Error types for the JPEG module.
#[derive(thiserror::Error, Debug)]
pub enum JpegTurboError {
    /// Error when the JPEG compressor cannot be created.
    #[error("Something went wrong with the JPEG compressor")]
    TurboJpegError(#[from] turbojpeg::Error),

    /// Error when the image data is not contiguous.
    #[error("Image data is not contiguous")]
    ImageDataNotContiguous,

    /// Error to create the image.
    #[error("Failed to create image")]
    ImageCreationError(#[from] ImageError),
}

/// A JPEG decoder using the turbojpeg library.
pub struct JpegTurboDecoder {
    /// The turbojpeg decompressor.
    pub decompressor: Arc<Mutex<turbojpeg::Decompressor>>,
}

/// A JPEG encoder using the turbojpeg library.
pub struct JpegTurboEncoder {
    /// The turbojpeg compressor.
    pub compressor: Arc<Mutex<turbojpeg::Compressor>>,
}

impl Default for JpegTurboDecoder {
    fn default() -> Self {
        match Self::new() {
            Ok(decoder) => decoder,
            Err(e) => panic!("Failed to create ImageDecoder: {}", e),
        }
    }
}

impl Default for JpegTurboEncoder {
    fn default() -> Self {
        match Self::new() {
            Ok(encoder) => encoder,
            Err(e) => panic!("Failed to create ImageEncoder: {}", e),
        }
    }
}

/// Implementation of the ImageEncoder struct.
impl JpegTurboEncoder {
    /// Creates a new `ImageEncoder`.
    ///
    /// # Returns
    ///
    /// A new `ImageEncoder` instance.
    ///
    /// # Panics
    ///
    /// Panics if the compressor cannot be created.
    pub fn new() -> Result<Self, JpegTurboError> {
        let compressor = turbojpeg::Compressor::new()?;
        Ok(Self {
            compressor: Arc::new(Mutex::new(compressor)),
        })
    }

    /// Encodes the given RGB8 image into a JPEG image.
    ///
    /// # Arguments
    ///
    /// * `image` - The image to encode.
    ///
    /// # Returns
    ///
    /// The encoded data as `Vec<u8>`.
    pub fn encode_rgb8(&mut self, image: &Image<u8, 3>) -> Result<Vec<u8>, JpegTurboError> {
        // get the image data
        let image_data = image.as_slice();

        // create a turbojpeg image
        let buf = turbojpeg::Image {
            pixels: image_data,
            width: image.width(),
            pitch: 3 * image.width(),
            height: image.height(),
            format: turbojpeg::PixelFormat::RGB,
        };

        // encode the image
        Ok(self
            .compressor
            .lock()
            .expect("Failed to lock the compressor")
            .compress_to_vec(buf)?)
    }

    /// Encodes the given grayscale (Gray8) image into a JPEG image.
    ///
    /// # Arguments
    ///
    /// * `image` - The grayscale image to encode.
    ///
    /// # Returns
    ///
    /// The encoded data as `Vec<u8>`.
    pub fn encode_gray8(&mut self, image: &Image<u8, 1>) -> Result<Vec<u8>, JpegTurboError> {
        // get the image data
        let image_data = image.as_slice();

        // create a turbojpeg image
        let buf = turbojpeg::Image {
            pixels: image_data,
            width: image.width(),
            pitch: image.width(), // 1 byte per pixel for grayscale
            height: image.height(),
            format: turbojpeg::PixelFormat::GRAY,
        };

        // encode the image
        Ok(self
            .compressor
            .lock()
            .expect("Failed to lock the compressor")
            .compress_to_vec(buf)?)
    }

    /// Sets the quality of the encoder.
    ///
    /// # Arguments
    ///
    /// * `quality` - The quality to set.
    pub fn set_quality(&mut self, quality: i32) -> Result<(), JpegTurboError> {
        Ok(self
            .compressor
            .lock()
            .expect("Failed to lock the compressor")
            .set_quality(quality)?)
    }
}

/// Implementation of the ImageDecoder struct.
impl JpegTurboDecoder {
    /// Creates a new `ImageDecoder`.
    ///
    /// # Returns
    ///
    /// A new `ImageDecoder` instance.
    pub fn new() -> Result<Self, JpegTurboError> {
        let decompressor = turbojpeg::Decompressor::new()?;
        Ok(JpegTurboDecoder {
            decompressor: Arc::new(Mutex::new(decompressor)),
        })
    }

    /// Reads the header of a JPEG image.
    ///
    /// # Arguments
    ///
    /// * `jpeg_data` - The JPEG data to read the header from.
    ///
    /// # Returns
    ///
    /// The image size.
    ///
    /// # Panics
    ///
    /// Panics if the header cannot be read.
    pub fn read_header(&mut self, jpeg_data: &[u8]) -> Result<ImageSize, JpegTurboError> {
        // read the JPEG header with image size
        let header = self
            .decompressor
            .lock()
            .expect("Failed to lock the decompressor")
            .read_header(jpeg_data)?;

        Ok(ImageSize {
            width: header.width,
            height: header.height,
        })
    }

    /// Decodes the given JPEG data as RGB8 image.
    ///
    /// # Arguments
    ///
    /// * `jpeg_data` - The JPEG data to decode.
    ///
    /// # Returns
    ///
    /// The decoded data as Image<u8, 3>.
    pub fn decode_rgb8(&mut self, jpeg_data: &[u8]) -> Result<Image<u8, 3>, JpegTurboError> {
        // get the image size to allocate th data storage
        let image_size = self.read_header(jpeg_data)?;

        // prepare a storage for the raw pixel data
        let mut pixels = vec![0u8; image_size.height * image_size.width * 3];

        // allocate image container
        let buf = turbojpeg::Image {
            pixels: pixels.as_mut_slice(),
            width: image_size.width,
            pitch: 3 * image_size.width, // we use no padding between rows
            height: image_size.height,
            format: turbojpeg::PixelFormat::RGB,
        };

        // decompress the JPEG data
        self.decompressor
            .lock()
            .expect("Failed to lock the decompressor")
            .decompress(jpeg_data, buf)?;

        Ok(Image::new(image_size, pixels)?)
    }

    /// Decodes the given JPEG data as grayscale (Gray8) image.
    ///
    /// # Arguments
    ///
    /// * `jpeg_data` - The JPEG data to decode.
    ///
    /// # Returns
    ///
    /// The decoded data as Image<u8, 1>.
    pub fn decode_gray8(&mut self, jpeg_data: &[u8]) -> Result<Image<u8, 1>, JpegTurboError> {
        // get the image size to allocate th data storage
        let image_size = self.read_header(jpeg_data)?;

        // prepare a storage for the raw pixel data
        let mut pixels = vec![0u8; image_size.height * image_size.width * 1]; // 1 byte per pixel

        // allocate image container
        let buf = turbojpeg::Image {
            pixels: pixels.as_mut_slice(),
            width: image_size.width,
            pitch: image_size.width, // 1 byte per pixel, no padding
            height: image_size.height,
            format: turbojpeg::PixelFormat::GRAY,
        };

        // decompress the JPEG data
        self.decompressor
            .lock()
            .expect("Failed to lock the decompressor")
            .decompress(jpeg_data, buf)?;

        Ok(Image::new(image_size, pixels)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::jpegturbo::{JpegTurboDecoder, JpegTurboEncoder, JpegTurboError};
    use kornia_image::{Image, ImageSize};

    #[test]
    fn image_decoder() -> Result<(), JpegTurboError> {
        let jpeg_data = std::fs::read("../../tests/data/dog.jpeg").unwrap();
        // read the header
        let image_size = JpegTurboDecoder::new()?.read_header(&jpeg_data)?;
        assert_eq!(image_size.width, 258);
        assert_eq!(image_size.height, 195);
        // load the image as file and decode it
        let image = JpegTurboDecoder::new()?.decode_rgb8(&jpeg_data)?;
        assert_eq!(image.cols(), 258);
        assert_eq!(image.rows(), 195);
        assert_eq!(image.num_channels(), 3);
        Ok(())
    }

    #[test]
    fn image_encoder() -> Result<(), Box<dyn std::error::Error>> {
        let jpeg_data_fs = std::fs::read("../../tests/data/dog.jpeg")?;
        let image = JpegTurboDecoder::new()?.decode_rgb8(&jpeg_data_fs)?;
        let jpeg_data = JpegTurboEncoder::new()?.encode_rgb8(&image)?;
        let image_back = JpegTurboDecoder::new()?.decode_rgb8(&jpeg_data)?;
        assert_eq!(image_back.cols(), 258);
        assert_eq!(image_back.rows(), 195);
        assert_eq!(image_back.num_channels(), 3);
        Ok(())
    }

    #[test]
    fn image_encoder_decoder_gray() -> Result<(), Box<dyn std::error::Error>> {
        // Create a simple grayscale test image
        let image_size = ImageSize {
            width: 4,
            height: 4,
        };
        
        // Create a gradient pattern (0, 85, 170, 255) repeated for each row
        let pixel_data = vec![
            0, 85, 170, 255,
            0, 85, 170, 255,
            0, 85, 170, 255,
            0, 85, 170, 255,
        ];
        
        let image = Image::<u8, 1>::new(image_size, pixel_data)?;
        
        // Encode to JPEG
        let jpeg_data = JpegTurboEncoder::new()?.encode_gray8(&image)?;
        
        // Decode back and verify
        let image_back = JpegTurboDecoder::new()?.decode_gray8(&jpeg_data)?;
        
        assert_eq!(image_back.cols(), 4);
        assert_eq!(image_back.rows(), 4);
        assert_eq!(image_back.num_channels(), 1);
        
        // Note: We don't check exact pixel values because JPEG is lossy
        // But we can check dimensions and general structure
         for row in 0..4 {
            let row_start = row * 4;
            let row_data = &image_back.as_slice()[row_start..row_start + 4];
            
            // Check that values increase from left to right (with some tolerance for JPEG artifacts)
            assert!(row_data[0] < row_data[1], "Row {}: Left-to-right pattern broken at pos 0-1: {:?}", row, row_data);
            assert!(row_data[1] < row_data[2], "Row {}: Left-to-right pattern broken at pos 1-2: {:?}", row, row_data);
            assert!(row_data[2] < row_data[3], "Row {}: Left-to-right pattern broken at pos 2-3: {:?}", row, row_data);
            
            // Check the range - first pixel should be relatively dark, last pixel relatively bright
            assert!(row_data[0] < 50, "First pixel should be dark, got: {}", row_data[0]);
            assert!(row_data[3] > 200, "Last pixel should be bright, got: {}", row_data[3]);
        }
        
        // Check overall brightness is preserved
        let original_sum: u32 = image.as_slice().iter().map(|&p| p as u32).sum();
        let decoded_sum: u32 = image_back.as_slice().iter().map(|&p| p as u32).sum();
        
        // Allow for up to 10% difference in overall brightness
        let ratio = (decoded_sum as f64) / (original_sum as f64);
        assert!(ratio > 0.9 && ratio < 1.1, 
                "Brightness differs too much: original sum={}, decoded sum={}, ratio={:.2}", 
                original_sum, decoded_sum, ratio);
        Ok(())
    }
}
