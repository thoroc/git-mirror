# Changelog

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
