//! Pure Rust Zstd codec plugin for `zarrs` WebAssembly data decoding using `ruzstd`.

use std::borrow::Cow;
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use zarrs::array::{ArrayBytesRaw, BytesRepresentation, CodecError};
use zarrs::metadata::Configuration;
use zarrs::metadata::v3::MetadataV3;
use zarrs::plugin::PluginCreateError;
use zarrs_codec::{
    BytesToBytesCodecTraits, Codec, CodecMetadataOptions, CodecOptions, CodecTraits, CodecTraitsV2,
    CodecTraitsV3, PartialDecoderCapability, PartialEncoderCapability, RecommendedConcurrency,
};
#[cfg(target_arch = "wasm32")]
use zarrs_codec::{CodecPluginV2, CodecPluginV3};
use zarrs_metadata::v2::MetadataV2;
use zarrs_plugin::{
    ExtensionAliases, ExtensionAliasesConfig, ExtensionNameStatic, ZarrVersion, ZarrVersion2,
    ZarrVersion3,
};

pub const IDENTIFIER: &str = "zstd";

static ALIASES_V3: OnceLock<RwLock<ExtensionAliasesConfig>> = OnceLock::new();
static ALIASES_V2: OnceLock<RwLock<ExtensionAliasesConfig>> = OnceLock::new();

fn make_aliases_config() -> RwLock<ExtensionAliasesConfig> {
    RwLock::new(ExtensionAliasesConfig::new(
        IDENTIFIER,
        vec![Cow::Borrowed("numcodecs.zstd")],
        Vec::new(),
    ))
}

fn get_aliases_v3() -> &'static RwLock<ExtensionAliasesConfig> {
    ALIASES_V3.get_or_init(make_aliases_config)
}

fn get_aliases_v2() -> &'static RwLock<ExtensionAliasesConfig> {
    ALIASES_V2.get_or_init(make_aliases_config)
}

/// Pure Rust Zstandard codec using `ruzstd`.
#[derive(Clone, Debug, Default)]
pub struct RuzstdCodec;

impl RuzstdCodec {
    pub fn new() -> Self {
        Self
    }
}

impl ExtensionNameStatic for RuzstdCodec {
    const DEFAULT_NAME_FN: fn(ZarrVersion) -> Option<Cow<'static, str>> =
        |_| Some(Cow::Borrowed(IDENTIFIER));
}

impl ExtensionAliases<ZarrVersion3> for RuzstdCodec {
    fn aliases() -> RwLockReadGuard<'static, ExtensionAliasesConfig> {
        get_aliases_v3().read().unwrap_or_else(|p| p.into_inner())
    }

    fn aliases_mut() -> RwLockWriteGuard<'static, ExtensionAliasesConfig> {
        get_aliases_v3().write().unwrap_or_else(|p| p.into_inner())
    }
}

impl ExtensionAliases<ZarrVersion2> for RuzstdCodec {
    fn aliases() -> RwLockReadGuard<'static, ExtensionAliasesConfig> {
        get_aliases_v2().read().unwrap_or_else(|p| p.into_inner())
    }

    fn aliases_mut() -> RwLockWriteGuard<'static, ExtensionAliasesConfig> {
        get_aliases_v2().write().unwrap_or_else(|p| p.into_inner())
    }
}

impl CodecTraits for RuzstdCodec {
    fn as_any(&self) -> &(dyn std::any::Any + 'static) {
        self
    }

    fn configuration(
        &self,
        _version: ZarrVersion,
        _options: &CodecMetadataOptions,
    ) -> Option<Configuration> {
        None
    }

    fn partial_decoder_capability(&self) -> PartialDecoderCapability {
        PartialDecoderCapability {
            partial_read: false,
            partial_decode: false,
        }
    }

    fn partial_encoder_capability(&self) -> PartialEncoderCapability {
        PartialEncoderCapability {
            partial_encode: false,
        }
    }
}

impl CodecTraitsV3 for RuzstdCodec {
    fn create(_metadata: &MetadataV3) -> Result<Codec, PluginCreateError> {
        Ok(Codec::BytesToBytes(Arc::new(RuzstdCodec::new())))
    }
}

impl CodecTraitsV2 for RuzstdCodec {
    fn create(_metadata: &MetadataV2) -> Result<Codec, PluginCreateError> {
        Ok(Codec::BytesToBytes(Arc::new(RuzstdCodec::new())))
    }
}

impl BytesToBytesCodecTraits for RuzstdCodec {
    fn into_dyn(self: Arc<Self>) -> Arc<dyn BytesToBytesCodecTraits + 'static> {
        self
    }

    fn recommended_concurrency(
        &self,
        _decoded_representation: &BytesRepresentation,
    ) -> Result<RecommendedConcurrency, CodecError> {
        Ok(RecommendedConcurrency::new_minimum(1))
    }

    fn encoded_representation(
        &self,
        decoded_representation: &BytesRepresentation,
    ) -> BytesRepresentation {
        *decoded_representation
    }

    fn decode<'a>(
        &self,
        value: ArrayBytesRaw<'a>,
        decoded_representation: &BytesRepresentation,
        _options: &CodecOptions,
    ) -> Result<ArrayBytesRaw<'a>, CodecError> {
        let decoder = ruzstd::decoding::StreamingDecoder::new(&value[..]).map_err(|e| {
            CodecError::Other(format!("Ruzstd streaming decoder init failed: {e:?}"))
        })?;
        // Never more than the chunk's own size: a small chunk expanding to
        // gigabytes would run the page out of memory.
        let (cap, reserve) = (
            super::capped::cap_for(decoded_representation),
            super::capped::reserve_for(decoded_representation),
        );
        let out = super::capped::read_capped(decoder, cap, reserve)
            .map_err(|e| CodecError::Other(format!("Ruzstd decompression failed: {e}")))?;
        Ok(out.into())
    }

    fn encode<'a>(
        &self,
        _value: ArrayBytesRaw<'a>,
        _options: &CodecOptions,
    ) -> Result<ArrayBytesRaw<'a>, CodecError> {
        Err(CodecError::Other(
            "Encoding is not supported for WASM zstd codec".to_string(),
        ))
    }
}

#[cfg(target_arch = "wasm32")]
inventory::submit! {
    CodecPluginV3::new::<RuzstdCodec>()
}

#[cfg(target_arch = "wasm32")]
inventory::submit! {
    CodecPluginV2::new::<RuzstdCodec>()
}
