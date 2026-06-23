use std::io::SeekFrom;

use binrw::binrw;

use crate::{
    array::{EXGeoAbsArray, EXGeoCommonArrayElement, EXGeoHashArray, EXRelArray},
    common::{
        EXGeoAnimHeader, EXGeoAnimModeHeader, EXGeoAnimSetHeader, EXGeoAnimSkinHeader,
        EXGeoEntityHeader, EXGeoSpreadSheetHeader,
    },
    texture::EXGeoTextureHeader,
    // versions::{EDB_VERSION_BOND, EDB_VERSION_GFORCE, EDB_VERSION_ICEAGE3},
};

pub type EXGeoMapHeader = EXGeoCommonArrayElement;
pub type EXGeoParticleHeader = EXGeoCommonArrayElement;
pub type EXGeoRefPointerHeader = EXGeoCommonArrayElement;
pub type EXGeoAnimScriptHeader = EXGeoCommonArrayElement;
pub type EXGeoSwooshHeader = EXGeoCommonArrayElement;
pub type EXGeoFontHeader = EXGeoCommonArrayElement;
pub type EXGeoMaterialHeader = EXGeoCommonArrayElement;

#[binrw]
#[brw(magic = 0x47454F4Du32)]
#[derive(Debug, Clone)]
pub struct EXGeoHeader {
    pub hashcode: u32,

    // ! Disney Universe (and presumably later EngineXT titles) is the first
    // ! seen title where this slot isn't a plain u32 - the upper 16 bits are
    // ! a separate field (seen value: 2, meaning unknown) and the actual
    // ! version is only the lower 16 bits. Every older title happens to have
    // ! 0 in the upper half, so reading the whole thing as one u32 silently
    // ! worked for them by coincidence.
    pub version_unk: u16,

    #[brw(assert(version.ge(&182) || version.le(&335), "Unsupported version {version}"))]
    pub version: u16,

    pub flags: u32,
    pub time: u32,
    pub file_size: u32,
    pub base_file_size: u32,

    pub platform_versions: [u32; 6], // 0x1C

    #[brw(seek_before = SeekFrom::Start(if version.lt(&248) { 0x54 } else { 0x40 } ))]
    pub section_list: EXGeoHashArray<()>, // 0x40
    pub refpointer_list: EXGeoHashArray<EXGeoRefPointerHeader>,
    #[br(args(version as u32))]
    pub entity_list: EXGeoHashArray<EXGeoEntityHeader>, // 0x50
    pub anim_list: EXGeoHashArray<EXGeoAnimHeader>,
    #[br(args(version as u32))]
    pub animskin_list: EXGeoHashArray<EXGeoAnimSkinHeader>, // 0x60
    pub animscript_list: EXGeoHashArray<EXGeoAnimScriptHeader>,
    pub map_list: EXGeoHashArray<EXGeoMapHeader>, // 0x70
    pub animmode_list: EXGeoHashArray<EXGeoAnimModeHeader>,
    pub animset_list: EXGeoHashArray<EXGeoAnimSetHeader>, // 0x80
    pub particle_list: EXGeoHashArray<EXGeoParticleHeader>,
    pub swoosh_list: EXGeoHashArray<EXGeoSwooshHeader>, // 0x90
    pub spreadsheet_list: EXGeoHashArray<EXGeoSpreadSheetHeader>,
    pub font_list: EXGeoHashArray<EXGeoFontHeader>, // 0xa0

    #[brw(if(version.ge(&248)))]
    pub forcefeedback_list: EXGeoHashArray<()>,
    #[brw(if(version.ge(&248)))]
    pub material_list: EXGeoHashArray<EXGeoMaterialHeader>, // 0xb0

    // ! Spyro Hack
    #[brw(if(version.eq(&240)))]
    _spyrohack: u64,

    pub texture_list: EXGeoHashArray<EXGeoTextureHeader>,

    pub unk_c0: EXRelArray<()>,
}

// structure_size_tests!(EXGeoHeader = 936);

/// EngineXT (Disney Universe and later - confirmed shipped version 244, and
/// version 300 seen in an Apr 2010 prototype build) GEOM chunk header. This is
/// a different, later redesign from EXGeoHeader above - not just a few extra
/// fields, a genuinely different list set and a different (and simpler) list
/// descriptor shape (plain count+absolute-offset, no self-relative pointer
/// encoding, no hash_size field).
///
/// Recovered by decompiling CBasePlatform::WriteMainGeoHeader from
/// ResourceBuilderXT.exe (the real Eurocom devkit tool that writes these
/// files) with its matching PDB, and validated byte-for-byte against a real
/// sample .edb file with zero discrepancies (filelen field matched the
/// sample's actual file size exactly).
#[binrw]
#[brw(magic = 0x47454F4Du32)]
#[derive(Debug, Clone)]
pub struct EXGeoHeaderXT {
    pub hashcode: u32,
    /// Same packed-u16 quirk as `EXGeoHeader.version_unk`/`version` above --
    /// the upper 16 bits are a separate field (unknown purpose, seen value
    /// 2), the real version is the lower 16 bits only.
    pub version_unk: u16,
    pub version: u16,
    pub flags: u32,
    pub time: u32,
    /// Real total file size - matches the on-disk file length exactly.
    pub file_size: u32,
    pub base_file_size: u32,
    /// Literal ASCII "NUID" tag, always present as a constant, not per-file data.
    pub nuid_tag: u32,
    pub unk_checksum_related: u32,
    pub checksum: u32,
    /// Absolute offset to the debug section, deferred-patched at build time.
    pub debug_section_offset: u32,
    /// Duplicate of file_size.
    pub file_end: u32,
    /// Always 0 on disk - runtime-only pointer placeholder.
    pub debug_file_ptr: u32,
    /// Always 0 on disk - runtime-only pointer placeholder (the in-memory EXGeoFile object).
    pub geo_file_ptr: u32,

    pub auto_include_list: EXGeoAbsArray<()>,
    pub group_section_list: EXGeoAbsArray<()>,
    pub custom_attribute_template_list: EXGeoAbsArray<()>,
    pub shader_list: EXGeoAbsArray<()>,
    pub texture_list: EXGeoAbsArray<()>,
    pub material_list: EXGeoAbsArray<()>,
    pub entity_list: EXGeoAbsArray<()>,
    pub skeleton_list: EXGeoAbsArray<()>,
    pub anim_sequence_list: EXGeoAbsArray<()>,
    pub rigid_body_list: EXGeoAbsArray<()>,
    pub swoosh_list: EXGeoAbsArray<()>,
    pub particle_system_list: EXGeoAbsArray<()>,
    pub timeline_list: EXGeoAbsArray<()>,
    pub zone_list: EXGeoAbsArray<()>,
    pub map_list: EXGeoAbsArray<()>,
    // Fixed header ends here, 0xb0 (176) bytes from the start of the chunk.
}

impl EXGeoHeaderXT {
    /// Reads an `EXGeoHeaderXT` from a full GEOM chunk buffer, decrypting
    /// the list region (see [`crate::geoxt_crypto`]) first since version
    /// 244 (and presumably later) EngineXT chunks store it encrypted.
    ///
    /// Returns both the parsed header and a decrypted copy of the whole
    /// buffer, so callers can go on to read list contents (textures,
    /// materials, entities, etc.) at the offsets the header reports without
    /// re-decrypting.
    ///
    /// Not every chunk turns out to need this -- some (so far seen only for
    /// `CustomAttributeTemplate`-bearing data/config chunks) are already
    /// plaintext, and applying the cipher would corrupt them. If the
    /// decrypted header's list descriptors don't look at all sane (wildly
    /// implausible counts), try [`Self::read_plain`] instead.
    pub fn read_decrypted(data: &[u8]) -> binrw::BinResult<(Self, Vec<u8>)> {
        // file_size lives in the unencrypted part of the header (offset 0x10),
        // peek it before decrypting to know exactly where the cipher region ends.
        let file_size = if data.len() >= 0x14 {
            Some(u32::from_le_bytes(data[0x10..0x14].try_into().unwrap()) as usize)
        } else {
            None
        };
        let decrypted = crate::geoxt_crypto::decrypt_geoxt_region_copy(data, file_size);
        let header = Self::read_plain(&decrypted)?;
        Ok((header, decrypted))
    }

    /// Reads an `EXGeoHeaderXT` from a buffer assumed to already be
    /// plaintext (no decryption applied). Endianness is auto-detected from
    /// the magic the same way the rest of eurochef does (big-endian console
    /// builds store the magic byte-reversed from PC's little-endian form).
    pub fn read_plain(data: &[u8]) -> binrw::BinResult<Self> {
        use binrw::BinReaderExt;
        let mut cursor = std::io::Cursor::new(data);
        let endian = if data.first() == Some(&0x47) {
            binrw::Endian::Big
        } else {
            binrw::Endian::Little
        };
        cursor.read_type(endian)
    }
}

#[cfg(test)]
mod xt_tests {
    use super::*;

    fn load(path: &str) -> Vec<u8> {
        std::fs::read(path).expect("test fixture missing -- only runs against this project's own checkout")
    }

    /// A static prop model with known texture/material/entity counts,
    /// independently verified in Python (tools/test_geom_decrypt.py) against
    /// the same real Xbox 360 Nightmare Before Christmas DLC file.
    #[test]
    fn gravestone_decrypts_to_known_counts() {
        let data = load(r"G:\Projects\DisneyUniverseDLCPorting\tools\chunk_tests\nightmare_named\io_nbc_glo_gravestone.edb");
        let (header, decrypted) = EXGeoHeaderXT::read_decrypted(&data).expect("parse failed");

        assert_eq!(header.hashcode, 0x40100eda);
        assert_eq!(header.version, 244);

        assert_eq!(header.group_section_list.count, 1);
        assert_eq!(header.custom_attribute_template_list.count, 1);
        assert_eq!(header.shader_list.count, 0);
        assert_eq!(header.texture_list.count, 7);
        assert_eq!(header.material_list.count, 7);
        assert_eq!(header.entity_list.count, 4);
        assert_eq!(header.skeleton_list.count, 0);
        assert_eq!(header.timeline_list.count, 1);

        // NOTE: texture_list's per-entry inner format has an extra
        // indirection level not yet fully mapped -- each 8-byte entry at
        // texture_list.offset_absolute is *not* a direct 4-byte hashcode,
        // it's some kind of pointer/index (multiple entries can resolve to
        // the same target, ruling out a simple self-relative-offset
        // theory). The real per-texture hashcode data lives further into
        // the chunk (confirmed manually in tools/test_geom_decrypt.py /
        // memory notes) but the exact entry layout connecting the two is a
        // follow-up task, not blocking this decryption validation.
        let _ = decrypted; // decrypted buffer available for that follow-up work
    }

    #[test]
    fn content_basepack_decrypts_to_known_counts() {
        // A pure data/manifest file with no visual assets -- everything
        // except group_section_list and custom_attribute_template_list
        // should be empty.
        let data = load(r"G:\Projects\DisneyUniverseDLCPorting\tools\chunk_tests\nightmare_named\nbc_content_basepack.edb");
        let (header, _) = EXGeoHeaderXT::read_decrypted(&data).expect("parse failed");

        assert_eq!(header.group_section_list.count, 1);
        assert_eq!(header.custom_attribute_template_list.count, 17);
        assert_eq!(header.texture_list.count, 0);
        assert_eq!(header.material_list.count, 0);
        assert_eq!(header.entity_list.count, 0);
    }
}
