# Changelog

## [0.3.3](https://github.com/stmh/plotta-studio/compare/v0.3.2...v0.3.3) (2026-11-22)


### Bug Fixes

* **motion:** apply per-segment speed limit at straight junctions ([23a2441](https://github.com/stmh/plotta-studio/commit/23a2441f3be66e183627fa0dabafd5fd8155fa5c)), closes [#92](https://github.com/stmh/plotta-studio/issues/92)
* **motion:** report the adjusted duration of single SM moves ([68f002d](https://github.com/stmh/plotta-studio/commit/68f002d6d02cd5715b0ce4099b33dc2d19d7bf9c)), closes [#91](https://github.com/stmh/plotta-studio/issues/91)
* **plotter:** cap the constant-velocity LM step rate at 25 kHz ([89a2373](https://github.com/stmh/plotta-studio/commit/89a237344b542ab608536e39952d6582dd380ac4)), closes [#88](https://github.com/stmh/plotta-studio/issues/88)
* **plotter:** plan closing edges, validate speeds, cap LM rate, lift pen on error ([89a2373](https://github.com/stmh/plotta-studio/commit/89a237344b542ab608536e39952d6582dd380ac4))
* **plotter:** plan the closing edge of closed strokes ([89a2373](https://github.com/stmh/plotta-studio/commit/89a237344b542ab608536e39952d6582dd380ac4)), closes [#86](https://github.com/stmh/plotta-studio/issues/86)
* **plotter:** raise the pen and release motors when plotting fails ([89a2373](https://github.com/stmh/plotta-studio/commit/89a237344b542ab608536e39952d6582dd380ac4)), closes [#89](https://github.com/stmh/plotta-studio/issues/89)
* **plotter:** reject zero or negative speeds and accelerations ([89a2373](https://github.com/stmh/plotta-studio/commit/89a237344b542ab608536e39952d6582dd380ac4)), closes [#87](https://github.com/stmh/plotta-studio/issues/87)
* **stats:** estimate closed strokes and travel like the plotter ([977615b](https://github.com/stmh/plotta-studio/commit/977615b9900ed8b31729daa16609d9d2b1e6482e)), closes [#90](https://github.com/stmh/plotta-studio/issues/90)

## [0.3.2](https://github.com/stmh/plotta-studio/compare/v0.3.1...v0.3.2) (2026-11-22)


### Bug Fixes

* **cli:** correct pen raise rate default in help text ([1ab0b5c](https://github.com/stmh/plotta-studio/commit/1ab0b5ca31c3d700a008dc28b98ccd057cdaa752)), closes [#79](https://github.com/stmh/plotta-studio/issues/79)
* **cli:** drain final plot events after the plot thread exits ([8aeaaba](https://github.com/stmh/plotta-studio/commit/8aeaaba27c4cfc14de52d9f43ca43074cf4b5cfb)), closes [#70](https://github.com/stmh/plotta-studio/issues/70)
* **cli:** drop double tilde from initial remaining time ([23e1976](https://github.com/stmh/plotta-studio/commit/23e197649b323f72d3e0cb0f2f2a573c9ddb1a6f)), closes [#80](https://github.com/stmh/plotta-studio/issues/80)
* **cli:** name both stroke counts when the stroke limit is exceeded ([602a172](https://github.com/stmh/plotta-studio/commit/602a172461cc12685553af14e0d6bb60bcf20c6e)), closes [#76](https://github.com/stmh/plotta-studio/issues/76)
* **clip:** draw content unclipped for inverted clip without closed region ([c96d24f](https://github.com/stmh/plotta-studio/commit/c96d24ff9c7d4b3480f746598c2e08e01811d449)), closes [#69](https://github.com/stmh/plotta-studio/issues/69)
* **clip:** keep the inside part of strokes starting a rounding error outside ([0481fd1](https://github.com/stmh/plotta-studio/commit/0481fd1dd0771328d982366f3caa6666c2521c45)), closes [#68](https://github.com/stmh/plotta-studio/issues/68)
* **cli:** require a terminal before plot spawns the plot thread ([8906f8b](https://github.com/stmh/plotta-studio/commit/8906f8b2f474f64ffe9c03155ec45adbcf83f3c1)), closes [#71](https://github.com/stmh/plotta-studio/issues/71)
* **cli:** validate --port path in plot before connecting ([0420812](https://github.com/stmh/plotta-studio/commit/0420812f88a5256c333031580f69e96052fb96c7)), closes [#75](https://github.com/stmh/plotta-studio/issues/75)
* **motion:** carry step rounding across segments of a planned stroke ([77e72c8](https://github.com/stmh/plotta-studio/commit/77e72c8ee39644fe38a12fbe403929735fd4848f)), closes [#67](https://github.com/stmh/plotta-studio/issues/67)
* **motion:** limit segment entry speed by previous segment's reachable exit ([0b1aa6b](https://github.com/stmh/plotta-studio/commit/0b1aa6bb0ef4464b52b5765bfc8fef131189dc94)), closes [#66](https://github.com/stmh/plotta-studio/issues/66)
* **plotter:** export generate_stroke_sm_commands ([05cabdd](https://github.com/stmh/plotta-studio/commit/05cabdd5f28f7a07838c266d6d848ec2fc9aae93))
* **plotter:** merge strokes in plot, plot_with_events and plot_in_background ([80eeda6](https://github.com/stmh/plotta-studio/commit/80eeda607cc6a59bc4754768f5b8853c4b74fae6)), closes [#77](https://github.com/stmh/plotta-studio/issues/77)
* **plotter:** report Cancelled when cancel arrives during the last stroke ([64f3227](https://github.com/stmh/plotta-studio/commit/64f3227effe3aaed30390ad03e96479912bbddc1)), closes [#72](https://github.com/stmh/plotta-studio/issues/72)
* **sketch-runner:** adapt rendering to vello 0.9 / wgpu 29 API ([1f64e68](https://github.com/stmh/plotta-studio/commit/1f64e68e12305711c31eb6806ddfd7d77aef0ef0))
* **stats:** include the final travel home in distance and time estimates ([52cc4a9](https://github.com/stmh/plotta-studio/commit/52cc4a9ad662a1c5435ab0c3e4bc6b71b29953c0)), closes [#85](https://github.com/stmh/plotta-studio/issues/85)
* **svg:** warn when &lt;text&gt; elements are dropped on import ([3907d45](https://github.com/stmh/plotta-studio/commit/3907d45b63ba8419c12d9e7b5d2cc36c9236d26c)), closes [#74](https://github.com/stmh/plotta-studio/issues/74)
* **svg:** warn when gradient or pattern strokes fall back to the default colour ([b16364e](https://github.com/stmh/plotta-studio/commit/b16364e87f74b86b9b98b87ddfb521b3f2faa1be)), closes [#73](https://github.com/stmh/plotta-studio/issues/73)
* **text:** reject Hershey directory loads up front ([c06fd1b](https://github.com/stmh/plotta-studio/commit/c06fd1b1ba7dafce861ea83a7b11eb7baab386a3)), closes [#78](https://github.com/stmh/plotta-studio/issues/78)
* **text:** warn about a missing space glyph once per font ([ab55edb](https://github.com/stmh/plotta-studio/commit/ab55edb873e7adb76d994ed38c2e210aa50aa7df)), closes [#82](https://github.com/stmh/plotta-studio/issues/82)
* **vsf-convert:** name the file when minf input can't be read ([e116076](https://github.com/stmh/plotta-studio/commit/e116076a748325277c1b340b1cc81cd721a54403)), closes [#84](https://github.com/stmh/plotta-studio/issues/84)

## [0.3.1](https://github.com/stmh/plotta-studio/compare/v0.3.0...v0.3.1) (2026-05-31)


### Bug Fixes

* **drawing-core:** clip lines to boundary instead of dropping them ([e3ddf00](https://github.com/stmh/plotta-studio/commit/e3ddf00e89c9c2287945e0ebcf3c11d4c05f05b0)), closes [#19](https://github.com/stmh/plotta-studio/issues/19)

## [0.3.0](https://github.com/stmh/plotta-studio/compare/v0.2.0...v0.3.0) (2026-05-31)


### Features

* **drawing-plotter:** merge adjacent strokes and bridge close-stroke gaps ([11cd62b](https://github.com/stmh/plotta-studio/commit/11cd62b3cc9693bcb157fb2b53436c029274ea9c))
* **drawing-viewer:** add JSON viewer crate with cycling between files ([0c65a4c](https://github.com/stmh/plotta-studio/commit/0c65a4c7b7152cdba38408cdb124ee45a858db6d))
* **plotta-cli:** expose stroke-merge optimization flags ([d05ec2e](https://github.com/stmh/plotta-studio/commit/d05ec2e12b09ada95523f479c23ca0f2ad2af074))
* **sketch-runner:** add Sketch::base_filename for custom export names ([fb3d257](https://github.com/stmh/plotta-studio/commit/fb3d257a412c1b3cfe83b8e816a3198b52a123d2))


### Bug Fixes

* **drawing-core:** keep strokes touching the clip boundary ([4e3c2cb](https://github.com/stmh/plotta-studio/commit/4e3c2cb52859b4de3dfdda0d97ada76f6955bc4c)), closes [#19](https://github.com/stmh/plotta-studio/issues/19)

## [0.2.0](https://github.com/stmh/plotta-studio/compare/v0.1.0...v0.2.0) (2026-05-14)


### Features

* add hello-world sketch and standardize rotated-squares to A6 ([c1cd905](https://github.com/stmh/plotta-studio/commit/c1cd9059abf1bbf8b42d5b89bb2bf99a1b9471ab))
* add invert option to clip and randomness to hatch algorithm ([9c7ed00](https://github.com/stmh/plotta-studio/commit/9c7ed00b3f934e9e53ac470e3939122a883a8739))
* add rotated squares sketch with hidden line removal ([5179f3e](https://github.com/stmh/plotta-studio/commit/5179f3e4dfdf305f379b81a467760049c4c73779))
* add sketch-009-altoetting city map visualization ([64cc4c7](https://github.com/stmh/plotta-studio/commit/64cc4c76a95fca6b65f3b1925ecc474b2208bb5e))
* add sketch-010-schnellstrasse landscape city map ([b30fcf1](https://github.com/stmh/plotta-studio/commit/b30fcf1aa1be4de043aa9b24a2acd7169fe66569))
* add wool ball bezier curve sketch ([8475c1a](https://github.com/stmh/plotta-studio/commit/8475c1ab80c5c889db999e66b11f2ef296063f94))
* **examples:** add plotter calibration test sheet ([c3d9aeb](https://github.com/stmh/plotta-studio/commit/c3d9aeb8535b4b4123ffa4064fb2676382f58cdb))
* **optimize:** implement R*-tree spatial indexing for 83x faster stroke optimization ([c3bd1d1](https://github.com/stmh/plotta-studio/commit/c3bd1d13ba0d4141dd3868e4044e27107a0acb18))
* **plotta-cli:** add terminal-based plotter setup diagram ([8ab03f5](https://github.com/stmh/plotta-studio/commit/8ab03f574464802e261af04340dc850d085a868a))
* **plotta-cli:** implement CLI for AxiDraw plotter control ([ff9413c](https://github.com/stmh/plotta-studio/commit/ff9413c121ebac29ff7ccababe4d1b2e95586bd9))
* refactor motion planning to use SM commands with time-slice interpolation ([d5bd38f](https://github.com/stmh/plotta-studio/commit/d5bd38f7023deb5acadb1098b07235548160c1fd))
* replace hardcoded curve flattening with tolerance-based subdivision ([7b390c3](https://github.com/stmh/plotta-studio/commit/7b390c31eba37ab2163bcbb326029c7fe58ecc23))


### Bug Fixes

* **ci:** prevent duplicate workflow runs on push+PR ([d1b7724](https://github.com/stmh/plotta-studio/commit/d1b77242fe2c6ca82502abf0fac921baec5745f9))
* **ci:** resolve pre-existing clippy::collapsible_match in sketch-runner ([baa40ca](https://github.com/stmh/plotta-studio/commit/baa40cac6880c97efb3838d553262b18cc99a02b))
* correct junction velocity formula to match GRBL/Python driver ([7210992](https://github.com/stmh/plotta-studio/commit/7210992ef2900ae51756db3a7fd265af0ef6d321))
* **flatten:** stop simplify_points from eating curve detail ([40e3988](https://github.com/stmh/plotta-studio/commit/40e3988b227ba09e2906a3129995f25e386b908a))
* **placeholder:** render at half target_height so xxx is unobtrusive ([4b12e42](https://github.com/stmh/plotta-studio/commit/4b12e4278ad6f11eac45ecff5ead85864c2ffa63))
* **plotter:** correct CoreXY underspeed handling and SM endpoint positioning ([059fc87](https://github.com/stmh/plotta-studio/commit/059fc87d4445a7ca9bedb6e46979e02b3bc5e5a3))
* track actual stepper position to prevent plotter drift ([cf52a38](https://github.com/stmh/plotta-studio/commit/cf52a38fb097c9b9ce2b18f483f9eefebf1827fa))
* use &Path instead of &PathBuf in convert_embedded ([07a6c9b](https://github.com/stmh/plotta-studio/commit/07a6c9b9d9811a029536736a64cb9b014063fe67))

## Changelog

All notable changes to this project will be documented in this file.

This file is maintained by [release-please](https://github.com/googleapis/release-please)
based on [Conventional Commits](https://www.conventionalcommits.org/) messages
on the `main` branch.
