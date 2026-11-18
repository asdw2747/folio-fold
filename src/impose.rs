//! Binding modes and page-order matrices for PDF imposition.
//!
//! The rest of the application consumes an [`ImpositionPlan`]: a flat list of
//! physical sheets whose front and back sides contain numbered page
//! placements. Page-order mathematics lives here so the PDF writer and the
//! preview always agree.

/// Signature sizes offered in the UI for multi-signature books, in pages.
/// Each entry is an even number of leaves: `pages / 2`.
pub const SIGNATURE_SIZES: [u32; 8] = [4, 8, 12, 16, 20, 24, 28, 32];

/// The corresponding leaves-per-signature values shown in the UI.
pub const SIGNATURE_LEAVES: [u32; 8] = [2, 4, 6, 8, 10, 12, 14, 16];

pub const SIGNATURE_MIN_PAGES: u32 = 4;
pub const SIGNATURE_MAX_PAGES: u32 = 64;

pub const fn pages_from_leaves(leaves: u32) -> u32 {
    leaves * 2
}

pub const fn leaves_from_pages(pages: u32) -> u32 {
    pages / 2
}

pub const fn sheets_from_pages(pages: u32) -> u32 {
    pages / 4
}

/// How source pages are assembled into folded sheets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BindingMode {
    /// One nested saddle-stitch signature.
    #[default]
    Booklet,
    /// Multiple nested signatures for Smyth sewing or case binding.
    Signatures,
    /// Four pages per sheet side, folded twice.
    Quarto,
    /// Eight pages per sheet side, folded three times.
    Octavo,
}

impl BindingMode {
    pub const ALL: [Self; 4] = [Self::Booklet, Self::Signatures, Self::Quarto, Self::Octavo];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Booklet => "Booklet",
            Self::Signatures => "Signatures",
            Self::Quarto => "Quarto",
            Self::Octavo => "Octavo",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Booklet => "Saddle stitch",
            Self::Signatures => "Multi-signature",
            Self::Quarto => "Quarto",
            Self::Octavo => "Octavo",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Booklet => {
                "One nested saddle-stitch signature, 2-up duplex, padded to a multiple of 4."
            }
            Self::Signatures => "Consecutive nested signatures for Smyth sewing or case binding.",
            Self::Quarto => {
                "4 pages per side, folded twice. Each physical sheet carries 8 book pages."
            }
            Self::Octavo => {
                "8 pages per side, folded three times. Each physical sheet carries 16 book pages."
            }
        }
    }

    pub const fn slots_per_side(self) -> usize {
        match self {
            Self::Booklet | Self::Signatures => 2,
            Self::Quarto => 4,
            Self::Octavo => 8,
        }
    }

    pub const fn pages_per_sheet(self) -> u32 {
        (self.slots_per_side() as u32) * 2
    }

    pub const fn is_two_up(self) -> bool {
        matches!(self, Self::Booklet | Self::Signatures)
    }

    pub const fn prefers_landscape(self) -> bool {
        self.is_two_up()
    }

    pub const fn default_signature_pages(self) -> u32 {
        match self {
            Self::Booklet => 4,
            Self::Signatures => 16,
            Self::Quarto => 8,
            Self::Octavo => 16,
        }
    }
}

/// Which edge a duplex printer uses to turn the sheet over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DuplexFlip {
    /// Tumble / short-edge: the back side is printed upright relative to the
    /// logical back layout.
    #[default]
    ShortEdge,
    /// Standard / long-edge: back-side page content is rotated 180 degrees.
    LongEdge,
}

impl DuplexFlip {
    pub const ALL: [Self; 2] = [Self::ShortEdge, Self::LongEdge];

    pub const fn label(self) -> &'static str {
        match self {
            Self::ShortEdge => "Tumble",
            Self::LongEdge => "Standard",
        }
    }

    pub const fn detail(self) -> &'static str {
        match self {
            Self::ShortEdge => "Short-edge flip",
            Self::LongEdge => "Long-edge flip",
        }
    }

    pub const fn helper(self) -> &'static str {
        match self {
            Self::ShortEdge => {
                "Back sheet stays upright. Default for landscape 2-up and tumble printers."
            }
            Self::LongEdge => {
                "Back pages rotate 180 degrees for standard long-edge duplex printers."
            }
        }
    }

    pub const fn back_rotation(self) -> u16 {
        match self {
            Self::ShortEdge => 0,
            Self::LongEdge => 180,
        }
    }
}

/// One slot on one side of a physical sheet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PagePlacement {
    /// Row-major slot index on the sheet side.
    pub slot: usize,
    /// 1-based source page number, or `None` for a blank flyleaf.
    pub page: Option<u32>,
    /// Fold/flip rotation in degrees, always a multiple of 180 for current modes.
    pub rotation: u16,
}

impl PagePlacement {
    pub fn blank(slot: usize, rotation: u16) -> Self {
        Self {
            slot,
            page: None,
            rotation,
        }
    }

    pub fn page(slot: usize, page: u32, rotation: u16) -> Self {
        Self {
            slot,
            page: Some(page),
            rotation,
        }
    }

    pub fn is_blank(&self) -> bool {
        self.page.is_none()
    }
}

/// One physical sheet of paper, identified within its signature and the book.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PhysicalSheet {
    /// 1-based global physical-sheet number.
    pub number: u32,
    /// 1-based signature number.
    pub signature: u32,
    /// 1-based sheet number inside the signature.
    pub sheet_in_signature: u32,
    /// Total sheets in this signature.
    pub sheets_in_signature: u32,
    /// Pages on the front side, in row-major slot order.
    pub front: Vec<PagePlacement>,
    /// Pages on the back side, in row-major slot order.
    pub back: Vec<PagePlacement>,
}

impl PhysicalSheet {
    pub fn side(&self, side: Side) -> &[PagePlacement] {
        match side {
            Side::Front => &self.front,
            Side::Back => &self.back,
        }
    }

    pub fn side_rotation(&self, side: Side, duplex: DuplexFlip) -> u16 {
        match side {
            Side::Front => 0,
            Side::Back => duplex.back_rotation(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Front,
    Back,
}

impl Side {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Front => "Front",
            Self::Back => "Back",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Front => "F",
            Self::Back => "B",
        }
    }
}

/// Complete, deterministic imposition plan.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImpositionPlan {
    pub mode: BindingMode,
    /// Effective pages per signature or folded section.
    pub signature_pages: u32,
    pub source_pages: u32,
    pub padded_pages: u32,
    pub blank_pages: u32,
    pub signatures: u32,
    pub sheets: Vec<PhysicalSheet>,
    pub warnings: Vec<String>,
}

impl ImpositionPlan {
    pub fn physical_sheet_count(&self) -> usize {
        self.sheets.len()
    }

    pub fn output_pdf_pages(&self) -> usize {
        self.sheets.len() * 2
    }

    pub fn sheet(&self, zero_based: usize) -> Option<&PhysicalSheet> {
        self.sheets.get(zero_based)
    }

    pub fn page_count_label(&self) -> String {
        format!("{} pages", self.source_pages)
    }
}

/// Clamp a requested signature size to a supported value.
pub fn signature_size(value: u32) -> u32 {
    let clamped = value.clamp(SIGNATURE_MIN_PAGES, SIGNATURE_MAX_PAGES);
    clamped.div_ceil(4) * 4
}

fn round_up(value: u32, multiple: u32) -> u32 {
    if multiple == 0 {
        return value;
    }
    value.div_ceil(multiple) * multiple
}

/// Build the page-order plan for the given mode and source page count.
///
/// The plan is pure: it does not read files. `signature_pages` is only used by
/// [`BindingMode::Signatures`]; Quarto and Octavo use their fixed 8- and
/// 16-page sections, and Booklet uses one signature padded to 4.
pub fn build_plan(
    mode: BindingMode,
    signature_pages: u32,
    source_pages: u32,
    duplex: DuplexFlip,
) -> ImpositionPlan {
    let (effective_signature, padded_pages, signatures) = match mode {
        BindingMode::Booklet => {
            let padded = round_up(source_pages, 4);
            (padded.max(4), padded, u32::from(padded != 0))
        }
        BindingMode::Signatures => {
            let size = signature_size(signature_pages);
            let padded = round_up(source_pages, size);
            (size, padded, padded / size)
        }
        BindingMode::Quarto => {
            let padded = round_up(source_pages, 8);
            (8, padded, padded / 8)
        }
        BindingMode::Octavo => {
            let padded = round_up(source_pages, 16);
            (16, padded, padded / 16)
        }
    };

    if source_pages == 0 {
        return ImpositionPlan {
            mode,
            signature_pages: effective_signature,
            source_pages: 0,
            padded_pages: 0,
            blank_pages: 0,
            signatures: 0,
            sheets: Vec::new(),
            warnings: Vec::new(),
        };
    }

    let blank_pages = padded_pages.saturating_sub(source_pages);
    let mut sheets = Vec::new();

    match mode {
        BindingMode::Booklet => {
            let sheets_per_signature = padded_pages / 4;
            for sheet_index in 0..sheets_per_signature {
                let front = nested_two_up_front(padded_pages, sheet_index)
                    .into_iter()
                    .enumerate()
                    .map(|(slot, page)| {
                        placement_from_page(slot, page, source_pages, duplex, Side::Front)
                    })
                    .collect();
                let back = nested_two_up_back(padded_pages, sheet_index)
                    .into_iter()
                    .enumerate()
                    .map(|(slot, page)| {
                        placement_from_page(slot, page, source_pages, duplex, Side::Back)
                    })
                    .collect();
                sheets.push(PhysicalSheet {
                    number: sheet_index + 1,
                    signature: 1,
                    sheet_in_signature: sheet_index + 1,
                    sheets_in_signature: sheets_per_signature,
                    front,
                    back,
                });
            }
        }
        BindingMode::Signatures => {
            let sheets_per_signature = effective_signature / 4;
            for signature in 0..signatures {
                let start = signature * effective_signature;
                for sheet_index in 0..sheets_per_signature {
                    let front_local = nested_two_up_front(effective_signature, sheet_index);
                    let back_local = nested_two_up_back(effective_signature, sheet_index);
                    let front = front_local
                        .into_iter()
                        .enumerate()
                        .map(|(slot, local)| {
                            placement_from_local(
                                slot,
                                local,
                                start,
                                source_pages,
                                duplex,
                                Side::Front,
                            )
                        })
                        .collect();
                    let back = back_local
                        .into_iter()
                        .enumerate()
                        .map(|(slot, local)| {
                            placement_from_local(
                                slot,
                                local,
                                start,
                                source_pages,
                                duplex,
                                Side::Back,
                            )
                        })
                        .collect();
                    sheets.push(PhysicalSheet {
                        number: sheets.len() as u32 + 1,
                        signature: signature + 1,
                        sheet_in_signature: sheet_index + 1,
                        sheets_in_signature: sheets_per_signature,
                        front,
                        back,
                    });
                }
            }
        }
        BindingMode::Quarto | BindingMode::Octavo => {
            let section_pages = effective_signature;
            for section in 0..signatures {
                let start = section * section_pages;
                let front_local: &[u32] = if mode == BindingMode::Quarto {
                    &[5, 4, 8, 1]
                } else {
                    &[5, 12, 9, 8, 4, 13, 16, 1]
                };
                let back_local: &[u32] = if mode == BindingMode::Quarto {
                    &[3, 6, 2, 7]
                } else {
                    &[7, 10, 11, 6, 2, 15, 14, 3]
                };
                let front_rotations: &[u16] = if mode == BindingMode::Quarto {
                    &[180, 180, 0, 0]
                } else {
                    &[180, 180, 180, 180, 0, 0, 0, 0]
                };
                let back_rotations: &[u16] = if mode == BindingMode::Quarto {
                    &[180, 180, 0, 0]
                } else {
                    &[180, 180, 180, 180, 0, 0, 0, 0]
                };
                let mut front = Vec::with_capacity(front_local.len());
                let mut back = Vec::with_capacity(back_local.len());
                for (slot, local) in front_local.iter().copied().enumerate() {
                    front.push(placement_from_local(
                        slot,
                        local,
                        start,
                        source_pages,
                        duplex,
                        Side::Front,
                    ));
                    let extra = front_rotations.get(slot).copied().unwrap_or(0);
                    front[slot].rotation = (front[slot].rotation + extra) % 360;
                }
                for (slot, local) in back_local.iter().copied().enumerate() {
                    back.push(placement_from_local(
                        slot,
                        local,
                        start,
                        source_pages,
                        duplex,
                        Side::Back,
                    ));
                    let extra = back_rotations.get(slot).copied().unwrap_or(0);
                    back[slot].rotation = (back[slot].rotation + extra) % 360;
                }
                sheets.push(PhysicalSheet {
                    number: section + 1,
                    signature: section + 1,
                    sheet_in_signature: 1,
                    sheets_in_signature: 1,
                    front,
                    back,
                });
            }
        }
    }

    let mut warnings = Vec::new();
    if blank_pages > 0 {
        warnings.push(format!(
            "Added {blank_pages} blank page{} to complete the final folded section.",
            if blank_pages == 1 { "" } else { "s" }
        ));
    }
    if mode == BindingMode::Booklet && source_pages > 0 && source_pages < 4 {
        warnings.push("The booklet is padded to the minimum four-page fold.".to_string());
    }

    ImpositionPlan {
        mode,
        signature_pages: effective_signature,
        source_pages,
        padded_pages,
        blank_pages,
        signatures,
        sheets,
        warnings,
    }
}

fn nested_two_up_front(signature_pages: u32, sheet_index: u32) -> [u32; 2] {
    [signature_pages - 2 * sheet_index, 1 + 2 * sheet_index]
}

fn nested_two_up_back(signature_pages: u32, sheet_index: u32) -> [u32; 2] {
    [2 + 2 * sheet_index, signature_pages - 1 - 2 * sheet_index]
}

fn placement_from_page(
    slot: usize,
    page: u32,
    source_pages: u32,
    duplex: DuplexFlip,
    side: Side,
) -> PagePlacement {
    let rotation = if side == Side::Back {
        duplex.back_rotation()
    } else {
        0
    };
    if page > source_pages {
        PagePlacement::blank(slot, rotation)
    } else {
        PagePlacement::page(slot, page, rotation)
    }
}

fn placement_from_local(
    slot: usize,
    local: u32,
    signature_start: u32,
    source_pages: u32,
    duplex: DuplexFlip,
    side: Side,
) -> PagePlacement {
    let absolute = signature_start + local;
    placement_from_page(slot, absolute, source_pages, duplex, side)
}

/// Effective pages per folded section for the current settings.
pub fn effective_signature_pages(mode: BindingMode, selected_signature: u32) -> u32 {
    match mode {
        BindingMode::Booklet => 4,
        BindingMode::Signatures => signature_size(selected_signature),
        BindingMode::Quarto => 8,
        BindingMode::Octavo => 16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbers(placements: &[PagePlacement]) -> Vec<Option<u32>> {
        placements.iter().map(|placement| placement.page).collect()
    }

    fn rotations(placements: &[PagePlacement]) -> Vec<u16> {
        placements
            .iter()
            .map(|placement| placement.rotation)
            .collect()
    }

    #[test]
    fn saddle_stitch_eight_pages() {
        let plan = build_plan(BindingMode::Booklet, 4, 8, DuplexFlip::ShortEdge);
        assert_eq!(plan.signatures, 1);
        assert_eq!(plan.blank_pages, 0);
        assert_eq!(plan.sheets.len(), 2);
        assert_eq!(numbers(&plan.sheets[0].front), vec![Some(8), Some(1)]);
        assert_eq!(numbers(&plan.sheets[0].back), vec![Some(2), Some(7)]);
        assert_eq!(numbers(&plan.sheets[1].front), vec![Some(6), Some(3)]);
        assert_eq!(numbers(&plan.sheets[1].back), vec![Some(4), Some(5)]);
    }

    #[test]
    fn saddle_stitch_pads_to_four() {
        let plan = build_plan(BindingMode::Booklet, 4, 5, DuplexFlip::ShortEdge);
        assert_eq!(plan.padded_pages, 8);
        assert_eq!(plan.blank_pages, 3);
        assert_eq!(plan.sheets.len(), 2);
        assert_eq!(numbers(&plan.sheets[0].front), vec![None, Some(1)]);
        assert_eq!(numbers(&plan.sheets[1].back), vec![Some(4), Some(5)]);
    }

    #[test]
    fn multi_signature_eight_page_blocks() {
        let plan = build_plan(BindingMode::Signatures, 8, 20, DuplexFlip::ShortEdge);
        assert_eq!(plan.signatures, 3);
        assert_eq!(plan.padded_pages, 24);
        assert_eq!(plan.blank_pages, 4);
        assert_eq!(plan.sheets.len(), 6);
        assert_eq!(numbers(&plan.sheets[2].front), vec![Some(16), Some(9)]);
        assert_eq!(numbers(&plan.sheets[2].back), vec![Some(10), Some(15)]);
        assert_eq!(numbers(&plan.sheets[3].front), vec![Some(14), Some(11)]);
        assert_eq!(numbers(&plan.sheets[3].back), vec![Some(12), Some(13)]);
        assert_eq!(numbers(&plan.sheets[4].front), vec![None, Some(17)]);
        assert_eq!(numbers(&plan.sheets[4].back), vec![Some(18), None]);
        assert!(plan
            .sheets
            .iter()
            .flat_map(|sheet| sheet.front.iter().chain(sheet.back.iter()))
            .all(|placement| placement.page.is_none_or(|page| page <= 20)));
    }

    #[test]
    fn multi_signature_sixteen_page_blocks() {
        let plan = build_plan(BindingMode::Signatures, 16, 40, DuplexFlip::ShortEdge);
        assert_eq!(plan.signatures, 3);
        assert_eq!(plan.sheets.len(), 12);
        assert_eq!(plan.blank_pages, 8);
        assert_eq!(numbers(&plan.sheets[0].front), vec![Some(16), Some(1)]);
        assert_eq!(numbers(&plan.sheets[3].front), vec![Some(10), Some(7)]);
        assert_eq!(numbers(&plan.sheets[4].front), vec![Some(32), Some(17)]);
        assert_eq!(numbers(&plan.sheets[11].back), vec![Some(40), None]);
    }

    #[test]
    fn quarto_matrix_and_rotations() {
        let plan = build_plan(BindingMode::Quarto, 16, 8, DuplexFlip::ShortEdge);
        assert_eq!(plan.signatures, 1);
        assert_eq!(plan.sheets.len(), 1);
        let sheet = &plan.sheets[0];
        assert_eq!(
            numbers(&sheet.front),
            vec![Some(5), Some(4), Some(8), Some(1)]
        );
        assert_eq!(rotations(&sheet.front), vec![180, 180, 0, 0]);
        assert_eq!(
            numbers(&sheet.back),
            vec![Some(3), Some(6), Some(2), Some(7)]
        );
        assert_eq!(rotations(&sheet.back), vec![180, 180, 0, 0]);
        let mut pages: Vec<u32> = numbers(&sheet.front)
            .into_iter()
            .chain(numbers(&sheet.back))
            .flatten()
            .collect();
        pages.sort_unstable();
        assert_eq!(pages, (1..=8).collect::<Vec<_>>());
    }

    #[test]
    fn octavo_matrix_and_rotations() {
        let plan = build_plan(BindingMode::Octavo, 8, 16, DuplexFlip::ShortEdge);
        assert_eq!(plan.signatures, 1);
        assert_eq!(plan.sheets.len(), 1);
        let sheet = &plan.sheets[0];
        assert_eq!(
            numbers(&sheet.front),
            vec![
                Some(5),
                Some(12),
                Some(9),
                Some(8),
                Some(4),
                Some(13),
                Some(16),
                Some(1)
            ]
        );
        assert_eq!(
            rotations(&sheet.front),
            vec![180, 180, 180, 180, 0, 0, 0, 0]
        );
        assert_eq!(
            numbers(&sheet.back),
            vec![
                Some(7),
                Some(10),
                Some(11),
                Some(6),
                Some(2),
                Some(15),
                Some(14),
                Some(3)
            ]
        );
        let mut pages: Vec<u32> = numbers(&sheet.front)
            .into_iter()
            .chain(numbers(&sheet.back))
            .flatten()
            .collect();
        pages.sort_unstable();
        assert_eq!(pages, (1..=16).collect::<Vec<_>>());
    }

    #[test]
    fn quarto_and_octavo_pad_final_section() {
        let quarto = build_plan(BindingMode::Quarto, 8, 9, DuplexFlip::ShortEdge);
        assert_eq!(quarto.padded_pages, 16);
        assert_eq!(quarto.blank_pages, 7);
        assert_eq!(quarto.sheets.len(), 2);

        let octavo = build_plan(BindingMode::Octavo, 16, 17, DuplexFlip::ShortEdge);
        assert_eq!(octavo.padded_pages, 32);
        assert_eq!(octavo.blank_pages, 15);
        assert_eq!(octavo.sheets.len(), 2);
    }

    #[test]
    fn long_edge_duplex_rotates_only_back_sides() {
        let plan = build_plan(BindingMode::Booklet, 4, 8, DuplexFlip::LongEdge);
        for sheet in &plan.sheets {
            assert!(sheet.front.iter().all(|placement| placement.rotation == 0));
            assert!(sheet.back.iter().all(|placement| placement.rotation == 180));
        }

        let quarto = build_plan(BindingMode::Quarto, 8, 8, DuplexFlip::LongEdge);
        assert_eq!(rotations(&quarto.sheets[0].back), vec![0, 0, 180, 180]);
    }

    #[test]
    fn every_signature_size_covers_real_pages_once() {
        for size in SIGNATURE_SIZES {
            let source_pages = size + 3;
            let plan = build_plan(
                BindingMode::Signatures,
                size,
                source_pages,
                DuplexFlip::ShortEdge,
            );
            let padded = source_pages.div_ceil(size) * size;
            assert_eq!(plan.padded_pages, padded, "size {size}");
            assert_eq!(plan.blank_pages, padded - source_pages, "size {size}");
            assert_eq!(plan.sheets.len() as u32, padded / 4, "size {size}");
            let mut pages: Vec<u32> = plan
                .sheets
                .iter()
                .flat_map(|sheet| sheet.front.iter().chain(sheet.back.iter()))
                .filter_map(|placement| placement.page)
                .collect();
            pages.sort_unstable();
            assert_eq!(pages, (1..=source_pages).collect::<Vec<_>>(), "size {size}");
        }
    }

    #[test]
    fn custom_signature_multiple_of_four_is_supported() {
        let plan = build_plan(BindingMode::Signatures, 36, 80, DuplexFlip::ShortEdge);
        assert_eq!(plan.signature_pages, 36);
        assert_eq!(plan.signatures, 3);
        assert_eq!(plan.padded_pages, 108);
        assert_eq!(plan.blank_pages, 28);
        assert_eq!(plan.sheets.len(), 27);
        let mut pages: Vec<u32> = plan
            .sheets
            .iter()
            .flat_map(|sheet| sheet.front.iter().chain(sheet.back.iter()))
            .filter_map(|placement| placement.page)
            .collect();
        pages.sort_unstable();
        assert_eq!(pages, (1..=80).collect::<Vec<_>>());
    }

    #[test]
    fn four_page_signature_supports_one_sheet() {
        let plan = build_plan(BindingMode::Signatures, 4, 5, DuplexFlip::ShortEdge);
        assert_eq!(plan.signatures, 2);
        assert_eq!(plan.sheets.len(), 2);
        assert_eq!(numbers(&plan.sheets[0].front), vec![Some(4), Some(1)]);
        assert_eq!(numbers(&plan.sheets[1].front), vec![None, Some(5)]);
    }

    #[test]
    fn zero_pages_produces_empty_plan() {
        let plan = build_plan(BindingMode::Booklet, 8, 0, DuplexFlip::ShortEdge);
        assert!(plan.sheets.is_empty());
        assert_eq!(plan.signatures, 0);
        assert_eq!(plan.blank_pages, 0);
        assert_eq!(plan.output_pdf_pages(), 0);
    }

    #[test]
    fn signature_size_clamps_to_supported_values() {
        assert_eq!(signature_size(4), 4);
        assert_eq!(signature_size(6), 8);
        assert_eq!(signature_size(9), 12);
        assert_eq!(signature_size(28), 28);
        assert_eq!(signature_size(99), 64);
    }

    #[test]
    fn leaf_and_sheet_conversions() {
        assert_eq!(pages_from_leaves(8), 16);
        assert_eq!(leaves_from_pages(28), 14);
        assert_eq!(sheets_from_pages(28), 7);
        assert_eq!(SIGNATURE_LEAVES.len(), SIGNATURE_SIZES.len());
        for (leaves, pages) in SIGNATURE_LEAVES.iter().zip(SIGNATURE_SIZES.iter()) {
            assert_eq!(pages_from_leaves(*leaves), *pages);
            assert_eq!(*pages / 4, sheets_from_pages(*pages));
        }
        assert!(SIGNATURE_SIZES.contains(&28));
    }
}
