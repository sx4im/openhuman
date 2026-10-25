//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinybox` module, loaded on demand.
#[cfg(feature = "modules")]
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox execution and lifecycle operations through TinyBox",
    bus_name: tinybox_bus::INTERFACE,
    object_path: tinybox_bus::OBJECT_PATH,
    version: "0.2.1",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.2.1",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinybox-0.2.1-archlinux-arm64.tar.gz",
            sha256: "ff2404f24840632a00d8107d02ea4ac258cc56b77e650653b6c440e769da282a",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinybox-0.2.1-archlinux-x86_64.tar.gz",
            sha256: "3b8f14a175fb0e73578047192f5aeefe3f76014d5a1baf3fb85d63f300ee48fc",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinybox-0.2.1-fedora-43-arm64.tar.gz",
            sha256: "8b53c7f60cfddfadda3f0269aee15676069d727c1fc8c10f7aa8211baa55292a",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinybox-0.2.1-fedora-43-x86_64.tar.gz",
            sha256: "aaafcd8b33a844e44362f7566d4c384c6a541abfd5888b0e415158c8d5bc0ea4",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinybox-0.2.1-fedora-44-arm64.tar.gz",
            sha256: "5441fda12064b029e745229e0c9fd2fd64dcc8e30f3416f5cdbe877583524295",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinybox-0.2.1-fedora-44-x86_64.tar.gz",
            sha256: "1d48b6b734bba68dc3160b85f0cb9e8f7e96e36acab18afd30f71a20e002df2b",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.2.1-macos-15-arm64.tar.gz",
            sha256: "8d59ecb80220913770a25ab32116c6ff8e55cdb6fac1db634eab3b8767e22690",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.2.1-macos-15-x86_64.tar.gz",
            sha256: "4c1f22592913ca5c57a840cca65a638493cb76b95cee4d88b7367d762a5afa01",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.2.1-macos-26-arm64.tar.gz",
            sha256: "daaec8cbb2a8060783c53621794ae6c03b62c9f5295753c5040fe0bbdf75b98e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.2.1-macos-26-x86_64.tar.gz",
            sha256: "de5c07812e54b5ae421bd28a0071566c36d124c07f87dde2eef6e0df83745350",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.2.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "488082b3619698a82dfe14d13547ffed90aabc9278f64384f39e7b2f3b8ec020",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.2.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "d6dc0320b2944ebc3d1972d80d4a47e459c5ebb24d0c41ef0f64e5f442f90685",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.2.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "ff1f9e0b9658c463a4b8f1e6aeb36cc7f9bc8b97117e8d17fd8986245a8e5b97",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.2.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "87f9017e0384babe52a43cc9002a23d13558cdbab2d1807caf6d48152684871a",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.2.1-windows-11-arm64.zip",
            sha256: "86ab6d28269b9e7eb59b54f6534fdc3e9bff67ebfcf66311a0ce6fdd16eaf1d1",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.2.1-windows-2022-x86_64.zip",
            sha256: "68b7089dd4141434d3dfc91b32fae688cae549a297d0a933797a3b2b43e92795",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.2.1-windows-2025-x86_64.zip",
            sha256: "059e7fbc993627d866c6b57d59286fe80eefc4ebe6542d4535d0e0f7d08e22a9",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinybox",
        manifest: "crates/tinybox-module/Cargo.toml",
        package: "tinybox-module",
        lib_name: "tinybox",
        features: &[],
    },
};

/// The `tinychannels` module, loaded on demand.
pub(crate) const TINYCHANNELS: ModuleRecord = ModuleRecord {
    id: "tinychannels",
    description: "Channel provider lifecycle and message transport",
    bus_name: tinychannels_bus::names::BUS_NAME,
    object_path: tinychannels_bus::names::OBJECT_PATH,
    version: "0.1.42",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.42",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinychannels-module-0.1.42-archlinux-arm64.tar.gz",
            sha256: "7e1073c1eeb93b5c3d63ed2628b3ea3b52ec3e7de5b1419704be28f1d26a2209",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinychannels-module-0.1.42-archlinux-x86_64.tar.gz",
            sha256: "f59564ec7d40a245cb5c2c4ff9d6a087701929307fa637c3ff555f3a78e9468d",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinychannels-module-0.1.42-fedora-43-arm64.tar.gz",
            sha256: "6cb8308631035c03c638768626ba90f963e973eccc405e78924a4aaf55395f4e",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinychannels-module-0.1.42-fedora-43-x86_64.tar.gz",
            sha256: "6d0328d3535552b68e1aade4cdf2f319f5d7647f25c4462389d51704349407c9",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinychannels-module-0.1.42-fedora-44-arm64.tar.gz",
            sha256: "379758e4ee298bb90358ad33a1c2ee112f7bfbd860e8457084e0361bab4e4b7d",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinychannels-module-0.1.42-fedora-44-x86_64.tar.gz",
            sha256: "f5970f8313992b7beb3b5c90ab2504a537dfd7703055b67ad4ea8197d2763a1d",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.42-macos-15-arm64.tar.gz",
            sha256: "b52a725650cb210d29c8a31838ecd92b0a0a7a398222289bea360c1992095096",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.42-macos-15-x86_64.tar.gz",
            sha256: "db056c66f0d2b1c22ccec3b461239013909316af200e845e0747434112cbcd50",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.42-macos-26-arm64.tar.gz",
            sha256: "7bfcfaccbc5cd454899cc163bbe3d8fc0533a6e8fa9c0eedee9eabbbbe9aeb0f",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.42-macos-26-x86_64.tar.gz",
            sha256: "0fdbaf108ed1b31824cd3c1ee2cd7ff9f1ccbe781a3a19d50e1885bc0b39c569",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.42-ubuntu-22.04-arm64.tar.gz",
            sha256: "d79d6cc8720c57f0fce36d17d505fe8c64e6b1d5c2568eb486e615140b04ca9b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.42-ubuntu-22.04-x86_64.tar.gz",
            sha256: "8c737a5e2ba0ac277d1632179fa164608cded7c955923a808ee5f9c51ab8a10e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.42-ubuntu-24.04-arm64.tar.gz",
            sha256: "b3c9deb3790337518542473bc8b779fa904d46654e10ae9834fcc72c74ede3cc",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.42-ubuntu-24.04-x86_64.tar.gz",
            sha256: "e01e8529a423bb44ac4b5ca46808fb1f8f38cbd682b5d28e7da907c8f1a68ad7",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.42-windows-11-arm64.zip",
            sha256: "66f3e658e395f4726bb6391e00d99a6c75f8da578f6c7bc692e43c9fa6be9f39",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.42-windows-2022-x86_64.zip",
            sha256: "d070816a13ae53e0ff2bfd2feee8b93e7a4c390e603bf40869da39f2d5e4c9ad",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.42-windows-2025-x86_64.zip",
            sha256: "659df880c0bc2e526efc9921c95bbc9da872a16b6959dca058c7004716eeb706",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinychannels",
        manifest: "crates/tinychannels-module/Cargo.toml",
        package: "tinychannels-module",
        lib_name: "tinychannels_module",
        features: &[],
    },
};

#[cfg(feature = "hosting")]
/// The `tinyhosts` module, loaded on demand.
pub(crate) const TINYHOSTS: ModuleRecord = ModuleRecord {
    id: "tinyhosts",
    description: "Hosting provider operations",
    bus_name: tinyhosts_bus::BUS_NAME,
    object_path: tinyhosts_bus::OBJECT_PATH,
    version: "0.4.3",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.4.3",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyhosts-0.4.3-archlinux-arm64.tar.gz",
            sha256: "97388fd17b72b92f0384397378fece2e7d0806004676edcdfc2b1798669b67a8",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyhosts-0.4.3-archlinux-x86_64.tar.gz",
            sha256: "93e0500e30b77aca6d05de69636c4afc014a3856809e3f781868635d626ef9c0",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyhosts-0.4.3-fedora-43-arm64.tar.gz",
            sha256: "fef318d960bbaea005a2f1f8bd1c64ff0bce09a3e4cef4922e7a75019dadd29f",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyhosts-0.4.3-fedora-43-x86_64.tar.gz",
            sha256: "cef9120fc178bf5b40f06e23727d47901746952aab4129eecbc3babd2f516421",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyhosts-0.4.3-fedora-44-arm64.tar.gz",
            sha256: "be1a4c1c2da9e8745fd18a3d5c050cc99fc6655f6eba2e914e191086e7b0ec6d",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyhosts-0.4.3-fedora-44-x86_64.tar.gz",
            sha256: "ed5a96104a1db21f3852e0d3824cb493c5091318d36f44de97122abc8f7969ef",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.4.3-macos-15-arm64.tar.gz",
            sha256: "79a3fa3c643f50d7e46fec58bfbd3117590853ab76792175b62b4325846377cf",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.4.3-macos-15-x86_64.tar.gz",
            sha256: "4ac3e43e3d48210f40361b199760df738ed3ab01c248f2d45495a8e4c1d50509",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.4.3-macos-26-arm64.tar.gz",
            sha256: "efff57af745179807ef8eb41a1c83df6cce1cde23ae664ac8e2e67065dae06b7",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.4.3-macos-26-x86_64.tar.gz",
            sha256: "6c7f70714c7d2246436487ce986c58e1962a393d1d9f3db6d1634c7ce3f75dad",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.4.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "d554e1a524040b0ae023ff6aec0c0129d9e01526077ca5d6ba1249ce5cd440f0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.4.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "c6d7cf38c8b3e062e1a8af7785cf8c902c6e7d2d1be51a502407537369210b50",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.4.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "9eec6afb328b22dad77e96ca802c39a83656f0d57faea9c2d2d2017547e03c0b",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.4.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "70da542ec03e90485c84b8ef1f75867b15f09292ac941d3a457be8bec410f6bc",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.4.3-windows-11-arm64.zip",
            sha256: "28e10831884064238ebf897d49eb7050f98e5550f7552061015d018fe4c778d3",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.4.3-windows-2022-x86_64.zip",
            sha256: "19e525b143662e3580d90dcba003b661a97b5bcd8ed4e00872081766cdd2b59d",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.4.3-windows-2025-x86_64.zip",
            sha256: "83483a4e5eaf8fd0f02c7bc6575d506f443bf0938408288e4cc4930f092fac89",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinyhosts",
        manifest: "Cargo.toml",
        package: "tinyhosts",
        lib_name: "tinyhosts",
        features: &[],
    },
};
