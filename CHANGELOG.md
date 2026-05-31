# Changelog

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
