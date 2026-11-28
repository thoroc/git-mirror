# Changelog

## [0.5.0](https://github.com/thoroc/git-mirror/compare/git-mirror-v0.4.6...git-mirror-v0.5.0) (2026-11-28)


### Features

* add automatic Copilot review workflow ([d070d04](https://github.com/thoroc/git-mirror/commit/d070d04dab2515465484a383337d9ca34aac8e31))
* add migrate command and unify remote parsing ([#49](https://github.com/thoroc/git-mirror/issues/49)) ([7a97529](https://github.com/thoroc/git-mirror/commit/7a97529f7b46958ef142c016b48ae8c4bcd313b4))
* add version to artifact filenames for releases ([#22](https://github.com/thoroc/git-mirror/issues/22)) ([c69b8ae](https://github.com/thoroc/git-mirror/commit/c69b8ae74a62e7b0a65352475c5579c4aa27434c))
* allow git+https:// ([61b9233](https://github.com/thoroc/git-mirror/commit/61b923368ea6becd7a9a3a396ad2ce25bf5900de))
* Complete Rust conversion and remove Deno runtime dependency ([c4eebfa](https://github.com/thoroc/git-mirror/commit/c4eebfacf8a7f13fae0ba1a1ac873f6068340753))
* emit cloned repo path for git alias integration ([01c65f5](https://github.com/thoroc/git-mirror/commit/01c65f541b85e1ab98157d1e4af26f7cff8c3052))
* emit cloned repo path for git alias integration ([10a4611](https://github.com/thoroc/git-mirror/commit/10a46111d5a4b424570471b23513e542edf03d27))
* enhance GitHub workflows and add automated installation ([fe599ea](https://github.com/thoroc/git-mirror/commit/fe599ea348bf2d01e2d152d421fe4d1005118da5))
* enhance GitHub workflows and add automated installation script ([f516493](https://github.com/thoroc/git-mirror/commit/f51649303234893ebb55f21041a1a80d4091cfe2))
* link to latest release ([0ee5503](https://github.com/thoroc/git-mirror/commit/0ee55037254fcace1d49cf13034d8e7d4a913ff0))
* **migrate:** score duplicate checkouts before choosing a keeper ([#53](https://github.com/thoroc/git-mirror/issues/53)) ([8b70db9](https://github.com/thoroc/git-mirror/commit/8b70db9ff67ebb1b49b032864227e18536cde520))
* **rust:** implement clone_repo and local path builder ([968c77a](https://github.com/thoroc/git-mirror/commit/968c77ae215932b37181fa11bdfb3f819af889a9))


### Bug Fixes

* add GH_TOKEN to upload release artifacts step ([#11](https://github.com/thoroc/git-mirror/issues/11)) ([e55c24e](https://github.com/thoroc/git-mirror/commit/e55c24eea41a4c140cc62d4c9bbdfc4bb0234ccf))
* add shell: bash to version extraction steps ([#23](https://github.com/thoroc/git-mirror/issues/23)) ([01a749d](https://github.com/thoroc/git-mirror/commit/01a749d1e6f19837a04437d895174267b36590ab))
* address PR review comments for CI/CD security and cross-platform compatibility ([f75cec3](https://github.com/thoroc/git-mirror/commit/f75cec3b0071a0683b49273e25d66f9838909635))
* **ci:** correct release-please output variable name ([2675413](https://github.com/thoroc/git-mirror/commit/267541380c12004f2f2dd34010d84268259b877b))
* **ci:** correct release-please output variable name ([7978c97](https://github.com/thoroc/git-mirror/commit/7978c9722f64d7544a0ca5ac0337778661e0f3f9))
* **ci:** publish releases through release-please drafts ([#50](https://github.com/thoroc/git-mirror/issues/50)) ([d12a7dd](https://github.com/thoroc/git-mirror/commit/d12a7dd7191752528021051155f37d9361d97786))
* correct output reference and tag construction ([#20](https://github.com/thoroc/git-mirror/issues/20)) ([b94e502](https://github.com/thoroc/git-mirror/commit/b94e502eb6e4a55d61341e7f7381d7800c1925d8))
* correct variable order in install.sh ([#28](https://github.com/thoroc/git-mirror/issues/28)) ([82e1b4a](https://github.com/thoroc/git-mirror/commit/82e1b4a01786f5e9928f07aa0e147c9f7a9dcf75))
* create release after building assets for immutability ([#17](https://github.com/thoroc/git-mirror/issues/17)) ([a58f791](https://github.com/thoroc/git-mirror/commit/a58f79188e31693dbb249b097dd70cf37f779644))
* **deps:** update rust crate dirs to v6 ([#35](https://github.com/thoroc/git-mirror/issues/35)) ([dbf04a7](https://github.com/thoroc/git-mirror/commit/dbf04a7f365bede3da05787a1109c477b66912ba))
* **deps:** update rust crate dirs to v7 ([#44](https://github.com/thoroc/git-mirror/issues/44)) ([8fe01ce](https://github.com/thoroc/git-mirror/commit/8fe01ce3dec0fa86fb6d7b6d63eeed449b09e71e))
* **deps:** update rust crate owo-colors to v4 ([#45](https://github.com/thoroc/git-mirror/issues/45)) ([27bc7f5](https://github.com/thoroc/git-mirror/commit/27bc7f5a3656baf7427d214206c2149ce2399028))
* **deps:** update rust crate which to v8 ([#34](https://github.com/thoroc/git-mirror/issues/34)) ([4757840](https://github.com/thoroc/git-mirror/commit/47578405429eaf3ddcc1577b8e4b944bde66e540))
* extract version from Cargo.toml for immutable releases ([#21](https://github.com/thoroc/git-mirror/issues/21)) ([f1c9536](https://github.com/thoroc/git-mirror/commit/f1c9536e6fcbe9989344b2b7b8cb6f800256665e))
* handle existing releases when creating with artifacts ([#24](https://github.com/thoroc/git-mirror/issues/24)) ([8a47164](https://github.com/thoroc/git-mirror/commit/8a47164fd12696846c2efbea31dcfa44b381ca3e))
* include version in archive name for install script ([#27](https://github.com/thoroc/git-mirror/issues/27)) ([7853447](https://github.com/thoroc/git-mirror/commit/7853447d575991221db8000c4ce367eca60e1cdc))
* make releases immutable by removing --clobber flag ([#13](https://github.com/thoroc/git-mirror/issues/13)) ([232e3a1](https://github.com/thoroc/git-mirror/commit/232e3a1cc97caba2d74b3046c9f49b02745a6964))
* missed the extraction of VERSION ([fa34609](https://github.com/thoroc/git-mirror/commit/fa34609c1dfca92d6ad49764fccc61f9f404795d))
* move which(code) check after dry_run in open_in_vscode ([35f8df2](https://github.com/thoroc/git-mirror/commit/35f8df2317f1771e66dade58d700a0725eff7dad))
* not building for windows for now ([6dd66f2](https://github.com/thoroc/git-mirror/commit/6dd66f288507ff0d22d8b907f091f8f4faa515d3))
* remove duplicate output in fetch_repo function ([aad901b](https://github.com/thoroc/git-mirror/commit/aad901babafe54b7715a37cc91b5a251f21eb95a))
* remove redundant archive name construction before fetching latest version ([#29](https://github.com/thoroc/git-mirror/issues/29)) ([1104cd1](https://github.com/thoroc/git-mirror/commit/1104cd153c32893ea32f7cdf17532861b3c46344))
* **rust:** handle scp-style repo paths before slash-based URLs ([39a787a](https://github.com/thoroc/git-mirror/commit/39a787abbd533f61b83ab250c241853b170b0768))
* skip release creation if immutable release already exists ([#25](https://github.com/thoroc/git-mirror/issues/25)) ([eadbeed](https://github.com/thoroc/git-mirror/commit/eadbeed6d908e3016a885ff20091e5ee0ce06a63))
* updating release link on README ([a2d8678](https://github.com/thoroc/git-mirror/commit/a2d8678d5b6536d67a1d5fcd6123b671d5ad45fc))
* use paths_released and version outputs for immutable releases ([#19](https://github.com/thoroc/git-mirror/issues/19)) ([c119026](https://github.com/thoroc/git-mirror/commit/c119026fcf2422f1f4cfb0ce1ca94a263892fce5))
* use release_created (singular) output for release-please v4 ([#15](https://github.com/thoroc/git-mirror/issues/15)) ([8b182f4](https://github.com/thoroc/git-mirror/commit/8b182f4f5ee9cfefb0bf8a17de1eeb913847525f))
* wrong path for the version ([189bdcb](https://github.com/thoroc/git-mirror/commit/189bdcbec632862459280895ec7c5a1b1a005dbf))

## [0.4.6](https://github.com/thoroc/git-mirror/compare/v0.4.5...v0.4.6) (2026-02-07)


### Bug Fixes

* skip release creation if immutable release already exists ([#25](https://github.com/thoroc/git-mirror/issues/25))
* handle existing releases when creating with artifacts ([#24](https://github.com/thoroc/git-mirror/issues/24))
* add shell: bash to version extraction steps ([#23](https://github.com/thoroc/git-mirror/issues/23))
* extract version from Cargo.toml for immutable releases ([#21](https://github.com/thoroc/git-mirror/issues/21))

## [0.4.5](https://github.com/thoroc/git-mirror/compare/v0.4.4...v0.4.5) (2026-02-06)


### Bug Fixes

* create release after building assets for immutability ([#17](https://github.com/thoroc/git-mirror/issues/17)) ([a58f791](https://github.com/thoroc/git-mirror/commit/a58f79188e31693dbb249b097dd70cf37f779644))

## [0.4.4](https://github.com/thoroc/git-mirror/compare/v0.4.3...v0.4.4) (2026-02-06)


### Bug Fixes

* use release_created (singular) output for release-please v4 ([#15](https://github.com/thoroc/git-mirror/issues/15)) ([8b182f4](https://github.com/thoroc/git-mirror/commit/8b182f4f5ee9cfefb0bf8a17de1eeb913847525f))

## [0.4.3](https://github.com/thoroc/git-mirror/compare/v0.4.2...v0.4.3) (2026-02-06)


### Bug Fixes

* make releases immutable by removing --clobber flag ([#13](https://github.com/thoroc/git-mirror/issues/13)) ([232e3a1](https://github.com/thoroc/git-mirror/commit/232e3a1cc97caba2d74b3046c9f49b02745a6964))

## [0.4.2](https://github.com/thoroc/git-mirror/compare/v0.4.1...v0.4.2) (2026-02-06)


### Bug Fixes

* add GH_TOKEN to upload release artifacts step ([#11](https://github.com/thoroc/git-mirror/issues/11)) ([e55c24e](https://github.com/thoroc/git-mirror/commit/e55c24eea41a4c140cc62d4c9bbdfc4bb0234ccf))

## [0.4.1](https://github.com/thoroc/git-mirror/compare/v0.4.0...v0.4.1) (2026-02-06)


### Bug Fixes

* **ci:** correct release-please output variable name ([2675413](https://github.com/thoroc/git-mirror/commit/267541380c12004f2f2dd34010d84268259b877b))
* **ci:** correct release-please output variable name ([7978c97](https://github.com/thoroc/git-mirror/commit/7978c9722f64d7544a0ca5ac0337778661e0f3f9))

## [0.4.0](https://github.com/thoroc/git-mirror/compare/v0.3.0...v0.4.0) (2026-02-06)


### Features

* emit cloned repo path for git alias integration ([01c65f5](https://github.com/thoroc/git-mirror/commit/01c65f541b85e1ab98157d1e4af26f7cff8c3052))
* emit cloned repo path for git alias integration ([10a4611](https://github.com/thoroc/git-mirror/commit/10a46111d5a4b424570471b23513e542edf03d27))

## [0.3.0](https://github.com/thoroc/git-mirror/compare/v0.2.0...v0.3.0) (2026-02-06)


### Features

* add automatic Copilot review workflow ([d070d04](https://github.com/thoroc/git-mirror/commit/d070d04dab2515465484a383337d9ca34aac8e31))
* enhance GitHub workflows and add automated installation ([fe599ea](https://github.com/thoroc/git-mirror/commit/fe599ea348bf2d01e2d152d421fe4d1005118da5))
* enhance GitHub workflows and add automated installation script ([f516493](https://github.com/thoroc/git-mirror/commit/f51649303234893ebb55f21041a1a80d4091cfe2))


### Bug Fixes

* address PR review comments for CI/CD security and cross-platform compatibility ([f75cec3](https://github.com/thoroc/git-mirror/commit/f75cec3b0071a0683b49273e25d66f9838909635))

## [0.2.0](https://github.com/thoroc/git-mirror/compare/v0.1.10...v0.2.0) (2026-02-06)


### Features

* Complete Rust conversion and remove Deno runtime dependency ([c4eebfa](https://github.com/thoroc/git-mirror/commit/c4eebfacf8a7f13fae0ba1a1ac873f6068340753))
* **rust:** implement clone_repo and local path builder ([968c77a](https://github.com/thoroc/git-mirror/commit/968c77ae215932b37181fa11bdfb3f819af889a9))


### Bug Fixes

* move which(code) check after dry_run in open_in_vscode ([35f8df2](https://github.com/thoroc/git-mirror/commit/35f8df2317f1771e66dade58d700a0725eff7dad))
* remove duplicate output in fetch_repo function ([aad901b](https://github.com/thoroc/git-mirror/commit/aad901babafe54b7715a37cc91b5a251f21eb95a))
* **rust:** handle scp-style repo paths before slash-based URLs ([39a787a](https://github.com/thoroc/git-mirror/commit/39a787abbd533f61b83ab250c241853b170b0768))
