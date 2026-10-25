use super::*;

#[test]
fn tinychannels_registry_matches_scoped_runtime_release() {
    let record = find("tinychannels").expect("compiled TinyChannels module");
    assert_eq!(record.version, "0.1.42");
    assert_eq!(tinychannels_bus::CONTRACT_VERSION, 23);
    assert_eq!(
        tinychannels_bus::module_relay::MODULE_OPERATION_VERSION,
        (1, 23)
    );
    let expected = [
        (
            "archlinux-arm64",
            "tinychannels-module-0.1.42-archlinux-arm64.tar.gz",
            "7e1073c1eeb93b5c3d63ed2628b3ea3b52ec3e7de5b1419704be28f1d26a2209",
        ),
        (
            "archlinux-x86_64",
            "tinychannels-module-0.1.42-archlinux-x86_64.tar.gz",
            "f59564ec7d40a245cb5c2c4ff9d6a087701929307fa637c3ff555f3a78e9468d",
        ),
        (
            "fedora-43-arm64",
            "tinychannels-module-0.1.42-fedora-43-arm64.tar.gz",
            "6cb8308631035c03c638768626ba90f963e973eccc405e78924a4aaf55395f4e",
        ),
        (
            "fedora-43-x86_64",
            "tinychannels-module-0.1.42-fedora-43-x86_64.tar.gz",
            "6d0328d3535552b68e1aade4cdf2f319f5d7647f25c4462389d51704349407c9",
        ),
        (
            "fedora-44-arm64",
            "tinychannels-module-0.1.42-fedora-44-arm64.tar.gz",
            "379758e4ee298bb90358ad33a1c2ee112f7bfbd860e8457084e0361bab4e4b7d",
        ),
        (
            "fedora-44-x86_64",
            "tinychannels-module-0.1.42-fedora-44-x86_64.tar.gz",
            "f5970f8313992b7beb3b5c90ab2504a537dfd7703055b67ad4ea8197d2763a1d",
        ),
        (
            "macos-15-arm64",
            "tinychannels-module-0.1.42-macos-15-arm64.tar.gz",
            "b52a725650cb210d29c8a31838ecd92b0a0a7a398222289bea360c1992095096",
        ),
        (
            "macos-15-x86_64",
            "tinychannels-module-0.1.42-macos-15-x86_64.tar.gz",
            "db056c66f0d2b1c22ccec3b461239013909316af200e845e0747434112cbcd50",
        ),
        (
            "macos-26-arm64",
            "tinychannels-module-0.1.42-macos-26-arm64.tar.gz",
            "7bfcfaccbc5cd454899cc163bbe3d8fc0533a6e8fa9c0eedee9eabbbbe9aeb0f",
        ),
        (
            "macos-26-x86_64",
            "tinychannels-module-0.1.42-macos-26-x86_64.tar.gz",
            "0fdbaf108ed1b31824cd3c1ee2cd7ff9f1ccbe781a3a19d50e1885bc0b39c569",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinychannels-module-0.1.42-ubuntu-22.04-arm64.tar.gz",
            "d79d6cc8720c57f0fce36d17d505fe8c64e6b1d5c2568eb486e615140b04ca9b",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinychannels-module-0.1.42-ubuntu-22.04-x86_64.tar.gz",
            "8c737a5e2ba0ac277d1632179fa164608cded7c955923a808ee5f9c51ab8a10e",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinychannels-module-0.1.42-ubuntu-24.04-arm64.tar.gz",
            "b3c9deb3790337518542473bc8b779fa904d46654e10ae9834fcc72c74ede3cc",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinychannels-module-0.1.42-ubuntu-24.04-x86_64.tar.gz",
            "e01e8529a423bb44ac4b5ca46808fb1f8f38cbd682b5d28e7da907c8f1a68ad7",
        ),
        (
            "windows-11-arm64",
            "tinychannels-module-0.1.42-windows-11-arm64.zip",
            "66f3e658e395f4726bb6391e00d99a6c75f8da578f6c7bc692e43c9fa6be9f39",
        ),
        (
            "windows-2022-x86_64",
            "tinychannels-module-0.1.42-windows-2022-x86_64.zip",
            "d070816a13ae53e0ff2bfd2feee8b93e7a4c390e603bf40869da39f2d5e4c9ad",
        ),
        (
            "windows-2025-x86_64",
            "tinychannels-module-0.1.42-windows-2025-x86_64.zip",
            "659df880c0bc2e526efc9921c95bbc9da872a16b6959dca058c7004716eeb706",
        ),
    ];
    assert_eq!(record.assets.len(), expected.len());
    for (key, archive, sha256) in expected {
        let asset = record.asset_for(key).expect("supported platform asset");
        assert_eq!(asset.archive, archive);
        assert_eq!(asset.sha256, sha256);
    }
}

#[cfg(feature = "voice")]
#[test]
fn voice_capture_release_pins_summary_contract_and_every_platform_digest() {
    let record = find("tinyvoice").unwrap();
    assert_eq!(record.version, "0.4.1");
    assert_eq!(tinyvoice_bus::CONTRACT_VERSION, (1, 12));
    let expected = [
        (
            "archlinux-arm64",
            "tinyvoice-module-0.4.1-archlinux-arm64.tar.gz",
            "b6bb17e6097f1b089f014c2392e028a481b0c6f8543652d7fc8ae890c86777a7",
        ),
        (
            "archlinux-x86_64",
            "tinyvoice-module-0.4.1-archlinux-x86_64.tar.gz",
            "fde049b4112de018799a91215cbc84572a9b4bd598bb44cfa61d8fcd1b96c9f9",
        ),
        (
            "fedora-43-arm64",
            "tinyvoice-module-0.4.1-fedora-43-arm64.tar.gz",
            "58a9bd07922770610b7f42e6f3708940ca501f430d27cd3ebbb6115402fdb9c8",
        ),
        (
            "fedora-43-x86_64",
            "tinyvoice-module-0.4.1-fedora-43-x86_64.tar.gz",
            "e8c95d1d85f4d89a1b8f8fa1740f8e174c835dab3ad856a99184749ca846bca7",
        ),
        (
            "fedora-44-arm64",
            "tinyvoice-module-0.4.1-fedora-44-arm64.tar.gz",
            "15370fb697a4ffdafe9264728b45efbbe155a91cf00aeba55471b0daa85c4768",
        ),
        (
            "fedora-44-x86_64",
            "tinyvoice-module-0.4.1-fedora-44-x86_64.tar.gz",
            "c7ee0327548bd05079ba30af0108385bea723bb8080f846c4def31ddbab57fcd",
        ),
        (
            "macos-15-arm64",
            "tinyvoice-module-0.4.1-macos-15-arm64.tar.gz",
            "77332a814521ac6ae0a5fd36671088ab6596b981544130b852431fe1d08e9f49",
        ),
        (
            "macos-15-x86_64",
            "tinyvoice-module-0.4.1-macos-15-x86_64.tar.gz",
            "2b4556461e3a30cb06fa5b548cc7e9ee8aee63475de05d16d4809140c44d9f4c",
        ),
        (
            "macos-26-arm64",
            "tinyvoice-module-0.4.1-macos-26-arm64.tar.gz",
            "17c1a6a80c1ae7a46fb19ce129e4b0dffb3d6391747bf9d1a5df0fab6ac75c48",
        ),
        (
            "macos-26-x86_64",
            "tinyvoice-module-0.4.1-macos-26-x86_64.tar.gz",
            "1cf0fbd9f20ffc449a9df938cdc15be4d9c2d4077a839ddeba4423ed8ec2f175",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinyvoice-module-0.4.1-ubuntu-22.04-arm64.tar.gz",
            "001b8b76b920dcab616a1f504efb2893c0ce87a251513dfd8e52c6414b239c7f",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinyvoice-module-0.4.1-ubuntu-22.04-x86_64.tar.gz",
            "c6f6a764747324a2789d9a6cd3764b6b5a5dc08a4a1ca305e157f65df762ae0d",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinyvoice-module-0.4.1-ubuntu-24.04-arm64.tar.gz",
            "8cb7d71e60213896326b85a551dfdd8af2ff1ee8b1a6d6fab27a53ed0d457f57",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinyvoice-module-0.4.1-ubuntu-24.04-x86_64.tar.gz",
            "7755ee1652735ac081bedbdc6dd6a80f8171639cd480f3a0d044230f5da2b0f3",
        ),
        (
            "windows-11-arm64",
            "tinyvoice-module-0.4.1-windows-11-arm64.zip",
            "ea575671e6d119886ec09ea43eed5bd35a7c9445aa8d54fa4621e26100856c6c",
        ),
        (
            "windows-2022-x86_64",
            "tinyvoice-module-0.4.1-windows-2022-x86_64.zip",
            "b472b4ef877bca793b45fca87bb5edbc43f119748cff85a3a25ea6d83815e254",
        ),
        (
            "windows-2025-x86_64",
            "tinyvoice-module-0.4.1-windows-2025-x86_64.zip",
            "c8c4cd75139c09a31082747d9ad4c1b656d9f6d4f8572370ee8017779c3fc1c0",
        ),
    ];
    assert_eq!(record.assets.len(), expected.len());
    for (key, archive, digest) in expected {
        let asset = record.asset_for(key).unwrap();
        assert_eq!(asset.archive, archive);
        assert_eq!(asset.sha256, digest);
    }
}
