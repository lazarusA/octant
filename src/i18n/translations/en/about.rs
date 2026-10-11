use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> &'static str {
    match key {
        AboutKey::WindowTitle => "About Octant",
        AboutKey::IconsWindowTitle => "Native Vector Icons",
        AboutKey::IconsTab => "Vector Icons",
        AboutKey::Subtitle => "N-Dimensional Data Explorer",
        AboutKey::Description => {
            "Octant is an interactive viewer for N-dimensional datasets with native support for local and cloud object storage, Zarr (v2/v3), and Icechunk. Built in Rust with GPU-accelerated rendering via WGPU. Octant runs natively on macOS, Linux, and Windows."
        }
        AboutKey::HyperslabTitle => "Hyperslab Slicing",
        AboutKey::HyperslabDesc => "Async LRU chunk cache & multi-resolution pyramids.",
        AboutKey::StorageTitle => "Zarr & Icechunk Native",
        AboutKey::StorageDesc => "Local, S3, GCS, Azure, and HTTP streaming backends.",
        AboutKey::ShadersTitle => "Hardware-Accelerated WGPU Shaders",
        AboutKey::ShadersDesc => {
            "2D Flatmaps, 3D Spheres, Elevation Surfaces, Volumes & 1D Profiles."
        }
        AboutKey::LinksHeading => "Links & Resources",
        AboutKey::DocsLink => "Octant documentation",
        AboutKey::GoodFaithNotice => {
            "Octant credits third-party work in good faith. If you believe something is missing or incorrectly attributed, please let us know and we will correct or remove it."
        }
        AboutKey::BrowseIcons => "Browse Native Vector Icons",
        AboutKey::BrowseIconsHover => "Explore procedural vector icons in popover",
        AboutKey::Licensed => "Licensed under MIT or Apache-2.0",
        AboutKey::OctantTeaser => {
            "The signs of x, y and z (+/-, +/-, +/-) divide space into eight octants. Explore your data the same way: pick the axes, cut a slab, and look at one region at a time."
        }
        AboutKey::OctantHover => "Click to visit the next octant",
        AboutKey::ColormapCredits => "Colormap credits",
        AboutKey::ColormapCreditsNote => "Bundled colormaps keep the licenses of their authors.",
        AboutKey::LicenseTexts => "License texts",
        AboutKey::ThirdPartyLicenses => "Third-party licenses",
        AboutKey::ThirdPartyNote => {
            "Octant includes open-source crates, fonts and C libraries (netCDF, HDF5), each under its own license."
        }
        AboutKey::ViewOnGithub => "View on GitHub",
        AboutKey::IconsSearchHint => "Search icons...",
        AboutKey::IconsAll => "All",
        AboutKey::IconsCopied => "Copied {}",
        AboutKey::IconsNoMatch => "No matching vector icons found.",
        AboutKey::IconsCategory => "Category: {}",
        AboutKey::IconsClickToCopy => "Click to copy enum identifier",
        AboutKey::IconsSizesTones => "Sizes & tones",
        AboutKey::ClearSearchHover => "Clear search",
        AboutKey::FontLicensesNote => {
            "Noto Sans SC, JP and KR (Chinese, Japanese and Korean text):"
        }
    }
}
