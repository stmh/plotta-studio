---
# plotta-studio-vpqe
title: Add additional Hershey font variants
status: completed
type: feature
priority: normal
created_at: 2025-12-26T12:33:38Z
updated_at: 2025-12-26T14:07:51Z
parent: plotta-studio-ah5h
---

Add more Hershey font variants beyond the current Simplex font.

## Available Hershey Fonts

From the hershey-fonts collection:
- **Roman**: Simplex (current), Duplex, Complex, Triplex
- **Gothic**: German, English, Italian
- **Script**: Simplex, Complex
- **Italic**: Simplex, Complex, Triplex
- **Greek**: Simplex, Complex
- **Cyrillic**: Complex

## Implementation

1. Download .jhf files from kamalmostafa/hershey-fonts repository
2. Add files to fonts/hershey/ directory
3. Create loader functions in hershey.rs module
4. Add to FontManager as built-in options

## Checklist

- [ ] Research and download additional .jhf files
- [ ] Add Gothic font files (gothgbt.jhf, gothgrt.jhf, gothitt.jhf)
- [ ] Add Script font files (scripts.jhf, scriptc.jhf)
- [ ] Add Italic font files (italicc.jhf, italiccs.jhf, italict.jhf)
- [ ] Add Complex/Triplex variants (rowmand.jhf, rowmant.jhf)
- [ ] Create loader functions for each variant
- [ ] Add LICENSE information for each font
- [ ] Update sketch-003-text to demo multiple fonts