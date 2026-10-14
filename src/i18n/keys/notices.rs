//! Translation keys of the notices area.

crate::i18n::keys::define_keys! {
    NoticesKey {
    // Toast buttons
    DismissHover,
    CopyDetails,
    RevealInFinder,
    OpenFolder,
    // Datasets and data loading
    NoVariablesFound,
    CouldntOpenDataset,
    UnsupportedFile,
    CouldntLoadData,
    CoordinatesUnavailable,
    CoordinatesFailedDetail,
    CoastlinesUnavailable,
    // GPU limits
    VolumeTooLargeForGpu,
    VolumeBufferTooLargeDetail,
    VolumeTextureTooLargeDetail,
    VolumeUploadSkipped,
    VolumeUploadMismatchDetail,
    DataTooLargeForGpuBuffer,
    BufferTooLargeDetail,
    BufferRangeTooLargeDetail,
    LinePlotTooLargeForGpu,
    LinePayloadTooLargeDetail,
    FontLoadFailed,
    FontLoadFailedDetail,
    DataFontMissing,
    DataFontMissingDetail,
    // OME-Zarr metadata
    OmeMetadataNonConforming,
    OmeMetadataNonConformingDetail,
    LabelIdsBeyond32Bits,
    }
}
