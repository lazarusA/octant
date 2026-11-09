//! What the pure Rust bytes-to-bytes codecs (Blosc, zstd) share: their
//! `zarrs` extension, alias and codec traits and their plugin registration.

/// Implements the `zarrs` codec traits for `$codec`, named `$name` (alias
/// `$alias`), reporting `$config` as its configuration: zarrs' sharded reader
/// rebuilds codecs from their metadata and drops one without it. Decoding and
/// encoding go to `$codec::decode_bytes` and `$codec::encode_bytes`.
macro_rules! bytes_codec_plugin {
    ($codec:ident, $name:expr, $alias:expr, $config:tt) => {
        use std::borrow::Cow;
        use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
        use zarrs::metadata::Configuration;
        use zarrs::metadata::v3::MetadataV3;
        use zarrs::plugin::PluginCreateError;
        use zarrs_codec::{
            BytesToBytesCodecTraits, Codec, CodecMetadataOptions, CodecOptions, CodecTraits,
            CodecTraitsV2, CodecTraitsV3, PartialDecoderCapability, PartialEncoderCapability,
            RecommendedConcurrency,
        };
        use zarrs_metadata::v2::MetadataV2;
        use zarrs_plugin::{
            ExtensionAliases, ExtensionAliasesConfig, ExtensionNameStatic, ZarrVersion,
            ZarrVersion2, ZarrVersion3,
        };

        static ALIASES_V3: OnceLock<RwLock<ExtensionAliasesConfig>> = OnceLock::new();
        static ALIASES_V2: OnceLock<RwLock<ExtensionAliasesConfig>> = OnceLock::new();

        fn aliases_config() -> RwLock<ExtensionAliasesConfig> {
            RwLock::new(ExtensionAliasesConfig::new(
                $name,
                vec![Cow::Borrowed($alias)],
                Vec::new(),
            ))
        }

        impl ExtensionNameStatic for $codec {
            const DEFAULT_NAME_FN: fn(ZarrVersion) -> Option<Cow<'static, str>> =
                |_| Some(Cow::Borrowed($name));
        }

        impl ExtensionAliases<ZarrVersion3> for $codec {
            fn aliases() -> RwLockReadGuard<'static, ExtensionAliasesConfig> {
                let lock = ALIASES_V3.get_or_init(aliases_config);
                lock.read().unwrap_or_else(|p| p.into_inner())
            }

            fn aliases_mut() -> RwLockWriteGuard<'static, ExtensionAliasesConfig> {
                let lock = ALIASES_V3.get_or_init(aliases_config);
                lock.write().unwrap_or_else(|p| p.into_inner())
            }
        }

        impl ExtensionAliases<ZarrVersion2> for $codec {
            fn aliases() -> RwLockReadGuard<'static, ExtensionAliasesConfig> {
                let lock = ALIASES_V2.get_or_init(aliases_config);
                lock.read().unwrap_or_else(|p| p.into_inner())
            }

            fn aliases_mut() -> RwLockWriteGuard<'static, ExtensionAliasesConfig> {
                let lock = ALIASES_V2.get_or_init(aliases_config);
                lock.write().unwrap_or_else(|p| p.into_inner())
            }
        }

        impl CodecTraits for $codec {
            fn as_any(&self) -> &(dyn std::any::Any + 'static) {
                self
            }

            fn configuration(
                &self,
                _version: ZarrVersion,
                _options: &CodecMetadataOptions,
            ) -> Option<Configuration> {
                let serde_json::Value::Object(map) = serde_json::json!($config) else {
                    return None;
                };
                Some(map.into())
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

        impl CodecTraitsV3 for $codec {
            fn create(_metadata: &MetadataV3) -> Result<Codec, PluginCreateError> {
                Ok(Codec::BytesToBytes(Arc::new($codec::new())))
            }
        }

        impl CodecTraitsV2 for $codec {
            fn create(_metadata: &MetadataV2) -> Result<Codec, PluginCreateError> {
                Ok(Codec::BytesToBytes(Arc::new($codec::new())))
            }
        }

        impl BytesToBytesCodecTraits for $codec {
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
                $codec::decode_bytes(value, decoded_representation)
            }

            fn encode<'a>(
                &self,
                value: ArrayBytesRaw<'a>,
                _options: &CodecOptions,
            ) -> Result<ArrayBytesRaw<'a>, CodecError> {
                $codec::encode_bytes(value)
            }
        }

        #[cfg(target_arch = "wasm32")]
        inventory::submit! {
            zarrs_codec::CodecPluginV3::new::<$codec>()
        }

        #[cfg(target_arch = "wasm32")]
        inventory::submit! {
            zarrs_codec::CodecPluginV2::new::<$codec>()
        }
    };
}

pub(super) use bytes_codec_plugin;
