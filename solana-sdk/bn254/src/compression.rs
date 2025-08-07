pub mod prelude {
    pub use crate::compression::{
        alt_bn128_compression_size::*, consts::*, target_arch::*, AltBn128CompressionError,
    };
}

use thiserror::Error;

mod consts {
    pub const ALT_BN128_G1_COMPRESS: u64 = 0;
    pub const ALT_BN128_G1_DECOMPRESS: u64 = 1;
    pub const ALT_BN128_G2_COMPRESS: u64 = 2;
    pub const ALT_BN128_G2_DECOMPRESS: u64 = 3;
}

mod alt_bn128_compression_size {
    pub const G1: usize = 64;
    pub const G2: usize = 128;
    pub const G1_COMPRESSED: usize = 32;
    pub const G2_COMPRESSED: usize = 64;
}

// AltBn128CompressionError must be removed once the
// simplify_alt_bn128_syscall_error_codes feature gets activated
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AltBn128CompressionError {
    #[error("Unexpected error")]
    UnexpectedError,
    #[error("Failed to decompress g1")]
    G1DecompressionFailed,
    #[error("Failed to decompress g2")]
    G2DecompressionFailed,
    #[error("Failed to compress affine g1")]
    G1CompressionFailed,
    #[error("Failed to compress affine g2")]
    G2CompressionFailed,
    #[error("Invalid input size")]
    InvalidInputSize,
}

impl From<u64> for AltBn128CompressionError {
    fn from(v: u64) -> AltBn128CompressionError {
        match v {
            1 => AltBn128CompressionError::G1DecompressionFailed,
            2 => AltBn128CompressionError::G2DecompressionFailed,
            3 => AltBn128CompressionError::G1CompressionFailed,
            4 => AltBn128CompressionError::G2CompressionFailed,
            5 => AltBn128CompressionError::InvalidInputSize,
            _ => AltBn128CompressionError::UnexpectedError,
        }
    }
}

impl From<AltBn128CompressionError> for u64 {
    fn from(v: AltBn128CompressionError) -> u64 {
        // note: should never return 0, as it risks to be confused with syscall success
        match v {
            AltBn128CompressionError::G1DecompressionFailed => 1,
            AltBn128CompressionError::G2DecompressionFailed => 2,
            AltBn128CompressionError::G1CompressionFailed => 3,
            AltBn128CompressionError::G2CompressionFailed => 4,
            AltBn128CompressionError::InvalidInputSize => 5,
            AltBn128CompressionError::UnexpectedError => 6,
        }
    }
}

#[cfg(not(target_os = "solana"))]
mod target_arch {

    use {
        super::*,
        crate::compression::alt_bn128_compression_size,
        ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate},
    };

    type G1 = ark_bn254::g1::G1Affine;
    type G2 = ark_bn254::g2::G2Affine;

    pub fn alt_bn128_g1_decompress(
        g1_bytes: &[u8],
    ) -> Result<[u8; alt_bn128_compression_size::G1], AltBn128CompressionError> {
        let g1_bytes: [u8; alt_bn128_compression_size::G1_COMPRESSED] = g1_bytes
            .try_into()
            .map_err(|_| AltBn128CompressionError::InvalidInputSize)?;
        if g1_bytes == [0u8; alt_bn128_compression_size::G1_COMPRESSED] {
            return Ok([0u8; alt_bn128_compression_size::G1]);
        }
        let decompressed_g1 = G1::deserialize_with_mode(
            convert_endianness::<32, 32>(&g1_bytes).as_slice(),
            Compress::Yes,
            Validate::No,
        )
        .map_err(|_| AltBn128CompressionError::G1DecompressionFailed)?;
        let mut decompressed_g1_bytes = [0u8; alt_bn128_compression_size::G1];
        decompressed_g1
            .x
            .serialize_with_mode(&mut decompressed_g1_bytes[..32], Compress::No)
            .map_err(|_| AltBn128CompressionError::G1DecompressionFailed)?;
        decompressed_g1
            .y
            .serialize_with_mode(&mut decompressed_g1_bytes[32..], Compress::No)
            .map_err(|_| AltBn128CompressionError::G1DecompressionFailed)?;
        Ok(convert_endianness::<32, 64>(&decompressed_g1_bytes))
    }

    pub fn alt_bn128_g1_compress(
        g1_bytes: &[u8],
    ) -> Result<[u8; alt_bn128_compression_size::G1_COMPRESSED], AltBn128CompressionError> {
        let g1_bytes: [u8; alt_bn128_compression_size::G1] = g1_bytes
            .try_into()
            .map_err(|_| AltBn128CompressionError::InvalidInputSize)?;
        if g1_bytes == [0u8; alt_bn128_compression_size::G1] {
            return Ok([0u8; alt_bn128_compression_size::G1_COMPRESSED]);
        }
        let g1 = G1::deserialize_with_mode(
            convert_endianness::<32, 64>(&g1_bytes).as_slice(),
            Compress::No,
            Validate::No,
        )
        .map_err(|_| AltBn128CompressionError::G1CompressionFailed)?;
        let mut g1_bytes = [0u8; alt_bn128_compression_size::G1_COMPRESSED];
        G1::serialize_compressed(&g1, g1_bytes.as_mut_slice())
            .map_err(|_| AltBn128CompressionError::G1CompressionFailed)?;
        Ok(convert_endianness::<32, 32>(&g1_bytes))
    }

    pub fn alt_bn128_g2_decompress(
        g2_bytes: &[u8],
    ) -> Result<[u8; alt_bn128_compression_size::G2], AltBn128CompressionError> {
        let g2_bytes: [u8; alt_bn128_compression_size::G2_COMPRESSED] = g2_bytes
            .try_into()
            .map_err(|_| AltBn128CompressionError::InvalidInputSize)?;
        if g2_bytes == [0u8; alt_bn128_compression_size::G2_COMPRESSED] {
            return Ok([0u8; alt_bn128_compression_size::G2]);
        }
        let decompressed_g2 = G2::deserialize_with_mode(
            convert_endianness::<64, 64>(&g2_bytes).as_slice(),
            Compress::Yes,
            Validate::No,
        )
        .map_err(|_| AltBn128CompressionError::G2DecompressionFailed)?;
        let mut decompressed_g2_bytes = [0u8; alt_bn128_compression_size::G2];
        decompressed_g2
            .x
            .serialize_with_mode(&mut decompressed_g2_bytes[..64], Compress::No)
            .map_err(|_| AltBn128CompressionError::G2DecompressionFailed)?;
        decompressed_g2
            .y
            .serialize_with_mode(&mut decompressed_g2_bytes[64..128], Compress::No)
            .map_err(|_| AltBn128CompressionError::G2DecompressionFailed)?;
        Ok(convert_endianness::<64, 128>(&decompressed_g2_bytes))
    }

    pub fn alt_bn128_g2_compress(
        g2_bytes: &[u8],
    ) -> Result<[u8; alt_bn128_compression_size::G2_COMPRESSED], AltBn128CompressionError> {
        let g2_bytes: [u8; alt_bn128_compression_size::G2] = g2_bytes
            .try_into()
            .map_err(|_| AltBn128CompressionError::InvalidInputSize)?;
        if g2_bytes == [0u8; alt_bn128_compression_size::G2] {
            return Ok([0u8; alt_bn128_compression_size::G2_COMPRESSED]);
        }
        let g2 = G2::deserialize_with_mode(
            convert_endianness::<64, 128>(&g2_bytes).as_slice(),
            Compress::No,
            Validate::No,
        )
        .map_err(|_| AltBn128CompressionError::G2CompressionFailed)?;
        let mut g2_bytes = [0u8; alt_bn128_compression_size::G2_COMPRESSED];
        G2::serialize_compressed(&g2, g2_bytes.as_mut_slice())
            .map_err(|_| AltBn128CompressionError::G2CompressionFailed)?;
        Ok(convert_endianness::<64, 64>(&g2_bytes))
    }

    pub fn convert_endianness<const CHUNK_SIZE: usize, const ARRAY_SIZE: usize>(
        bytes: &[u8; ARRAY_SIZE],
    ) -> [u8; ARRAY_SIZE] {
        let reversed: [_; ARRAY_SIZE] = bytes
            .chunks_exact(CHUNK_SIZE)
            .flat_map(|chunk| chunk.iter().rev().copied())
            .enumerate()
            .fold([0u8; ARRAY_SIZE], |mut acc, (i, v)| {
                acc[i] = v;
                acc
            });
        reversed
    }
}

#[cfg(target_os = "solana")]
mod target_arch {
    use {
        super::*,
        alt_bn128_compression_size::{G1, G1_COMPRESSED, G2, G2_COMPRESSED},
        prelude::*,
        solana_define_syscall::definitions as syscalls,
    };

    pub fn alt_bn128_g1_compress(
        input: &[u8],
    ) -> Result<[u8; G1_COMPRESSED], AltBn128CompressionError> {
        let mut result_buffer = [0; G1_COMPRESSED];
        let result = unsafe {
            syscalls::sol_alt_bn128_compression(
                ALT_BN128_G1_COMPRESS,
                input as *const _ as *const u8,
                input.len() as u64,
                &mut result_buffer as *mut _ as *mut u8,
            )
        };

        match result {
            0 => Ok(result_buffer),
            _ => Err(AltBn128CompressionError::UnexpectedError),
        }
    }

    pub fn alt_bn128_g1_decompress(input: &[u8]) -> Result<[u8; G1], AltBn128CompressionError> {
        let mut result_buffer = [0; G1];
        let result = unsafe {
            syscalls::sol_alt_bn128_compression(
                ALT_BN128_G1_DECOMPRESS,
                input as *const _ as *const u8,
                input.len() as u64,
                &mut result_buffer as *mut _ as *mut u8,
            )
        };

        match result {
            0 => Ok(result_buffer),
            _ => Err(AltBn128CompressionError::UnexpectedError),
        }
    }

    pub fn alt_bn128_g2_compress(
        input: &[u8],
    ) -> Result<[u8; G2_COMPRESSED], AltBn128CompressionError> {
        let mut result_buffer = [0; G2_COMPRESSED];
        let result = unsafe {
            syscalls::sol_alt_bn128_compression(
                ALT_BN128_G2_COMPRESS,
                input as *const _ as *const u8,
                input.len() as u64,
                &mut result_buffer as *mut _ as *mut u8,
            )
        };

        match result {
            0 => Ok(result_buffer),
            _ => Err(AltBn128CompressionError::UnexpectedError),
        }
    }

    pub fn alt_bn128_g2_decompress(
        input: &[u8; G2_COMPRESSED],
    ) -> Result<[u8; G2], AltBn128CompressionError> {
        let mut result_buffer = [0; G2];
        let result = unsafe {
            syscalls::sol_alt_bn128_compression(
                ALT_BN128_G2_DECOMPRESS,
                input as *const _ as *const u8,
                input.len() as u64,
                &mut result_buffer as *mut _ as *mut u8,
            )
        };

        match result {
            0 => Ok(result_buffer),
            _ => Err(AltBn128CompressionError::UnexpectedError),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::compression::target_arch::convert_endianness,
        ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate},
        std::ops::Neg,
        target_arch::{
            alt_bn128_g1_compress, alt_bn128_g1_decompress, alt_bn128_g2_compress,
            alt_bn128_g2_decompress,
        },
    };
    type G1 = ark_bn254::g1::G1Affine;
    type G2 = ark_bn254::g2::G2Affine;

    #[test]
    fn alt_bn128_g1_compression() {
        let g1_be = [
            45, 206, 255, 166, 152, 55, 128, 138, 79, 217, 145, 164, 25, 74, 120, 234, 234, 217,
            68, 149, 162, 44, 133, 120, 184, 205, 12, 44, 175, 98, 168, 172, 20, 24, 216, 15, 209,
            175, 106, 75, 147, 236, 90, 101, 123, 219, 245, 151, 209, 202, 218, 104, 148, 8, 32,
            254, 243, 191, 218, 122, 42, 81, 193, 84,
        ];
        let g1_le = convert_endianness::<32, 64>(&g1_be);
        let g1: G1 =
            G1::deserialize_with_mode(g1_le.as_slice(), Compress::No, Validate::No).unwrap();

        let g1_neg = g1.neg();
        let mut g1_neg_be = [0u8; 64];
        g1_neg
            .x
            .serialize_with_mode(&mut g1_neg_be[..32], Compress::No)
            .unwrap();
        g1_neg
            .y
            .serialize_with_mode(&mut g1_neg_be[32..64], Compress::No)
            .unwrap();
        let g1_neg_be: [u8; 64] = convert_endianness::<32, 64>(&g1_neg_be);

        let points = [(g1, g1_be), (g1_neg, g1_neg_be)];

        for (point, g1_be) in &points {
            let mut compressed_ref = [0u8; 32];
            G1::serialize_with_mode(point, compressed_ref.as_mut_slice(), Compress::Yes).unwrap();
            let compressed_ref: [u8; 32] = convert_endianness::<32, 32>(&compressed_ref);

            let decompressed = alt_bn128_g1_decompress(compressed_ref.as_slice()).unwrap();

            assert_eq!(
                alt_bn128_g1_compress(&decompressed).unwrap(),
                compressed_ref
            );
            assert_eq!(decompressed, *g1_be);
        }
    }

    #[test]
    fn alt_bn128_g2_compression() {
        let g2_be = [
            40, 57, 233, 205, 180, 46, 35, 111, 215, 5, 23, 93, 12, 71, 118, 225, 7, 46, 247, 147,
            47, 130, 106, 189, 184, 80, 146, 103, 141, 52, 242, 25, 0, 203, 124, 176, 110, 34, 151,
            212, 66, 180, 238, 151, 236, 189, 133, 209, 17, 137, 205, 183, 168, 196, 92, 159, 75,
            174, 81, 168, 18, 86, 176, 56, 16, 26, 210, 20, 18, 81, 122, 142, 104, 62, 251, 169,
            98, 141, 21, 253, 50, 130, 182, 15, 33, 109, 228, 31, 79, 183, 88, 147, 174, 108, 4,
            22, 14, 129, 168, 6, 80, 246, 254, 100, 218, 131, 94, 49, 247, 211, 3, 245, 22, 200,
            177, 91, 60, 144, 147, 174, 90, 17, 19, 189, 62, 147, 152, 18,
        ];
        let g2_le = convert_endianness::<64, 128>(&g2_be);
        let g2: G2 =
            G2::deserialize_with_mode(g2_le.as_slice(), Compress::No, Validate::No).unwrap();

        let g2_neg = g2.neg();
        let mut g2_neg_be = [0u8; 128];
        g2_neg
            .x
            .serialize_with_mode(&mut g2_neg_be[..64], Compress::No)
            .unwrap();
        g2_neg
            .y
            .serialize_with_mode(&mut g2_neg_be[64..128], Compress::No)
            .unwrap();
        let g2_neg_be: [u8; 128] = convert_endianness::<64, 128>(&g2_neg_be);

        let points = [(g2, g2_be), (g2_neg, g2_neg_be)];

        for (point, g2_be) in &points {
            let mut compressed_ref = [0u8; 64];
            G2::serialize_with_mode(point, compressed_ref.as_mut_slice(), Compress::Yes).unwrap();
            let compressed_ref: [u8; 64] = convert_endianness::<64, 64>(&compressed_ref);

            let decompressed = alt_bn128_g2_decompress(compressed_ref.as_slice()).unwrap();

            assert_eq!(
                alt_bn128_g2_compress(&decompressed).unwrap(),
                compressed_ref
            );
            assert_eq!(decompressed, *g2_be);
        }
    }

    #[test]
    fn alt_bn128_compression_g1_point_of_infitity() {
        let g1_bytes = vec![0u8; 64];
        let g1_compressed = alt_bn128_g1_compress(&g1_bytes).unwrap();
        let g1_decompressed = alt_bn128_g1_decompress(&g1_compressed).unwrap();
        assert_eq!(g1_bytes, g1_decompressed);
    }

    #[test]
    fn alt_bn128_compression_g2_point_of_infitity() {
        let g1_bytes = vec![0u8; 128];
        let g1_compressed = alt_bn128_g2_compress(&g1_bytes).unwrap();
        let g1_decompressed = alt_bn128_g2_decompress(&g1_compressed).unwrap();
        assert_eq!(g1_bytes, g1_decompressed);
    }
}
