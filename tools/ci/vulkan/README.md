# Vulkan CI environment — staged adoption

Runenwerk's four required native/Vulkan jobs previously installed Mesa and
X11 packages from live Ubuntu archives at the start of every job. On 2026-11-11
both the Azure-first and official Ubuntu mirrors failed DNS in the GitHub-hosted
runner environment. The Rust baseline and documentation validation still passed.
See issues #1482 and #1484 for evidence and accepted migration boundaries.

## Image definition and tests

- Dockerfile: Ubuntu 24.04 Linux/amd64 image with an explicit historical
  Ubuntu archive snapshot (20261108T000000Z), native build prerequisites,
  Mesa Lavapipe, Vulkan tools, Xvfb/X11, and Rust stable. The declared
  RUST_VERSION is checked against the installed compiler and must be reviewed
  when stable moves.
- build-image.sh: resolves the upstream Ubuntu tag to a SHA-256 digest at
  image-build time. Records that immutable base reference in the logs and
  image label. It does not claim that the moving Ubuntu tag is itself pinned
  forever.
- smoke-image.sh: requires a real llvmpipe adapter from vulkaninfo and an
  actually responding Xvfb display.
- verify-image.sh: runs the image smoke and a real Runenwerk native/rendering
  smoke against the exact checked-out source revision, with Cargo locked.
- .github/workflows/vulkan-image.yml: read-only PR build and test. No PR
  receives credentials to publish an image. A separately privileged
  workflow_dispatch on accepted main rebuilds and verifies its own release
  image before publishing to GHCR. Only its publisher job has packages:write.

## Release and migration sequence

1. Review the complete image definition, upstream base-image digest printed
   by the job, package snapshot, Rust version, permissions and source checkout.
   Require an actual hosted image build, Vulkan adapter proof, Xvfb proof and
   native application smoke. A failed download is NOT an accepted test result.
2. Merge the image producer through protected main only after all currently
   required Rust, documentation and native/Vulkan checks succeed. Do not
   bypass merge queue or branch protection.
3. On accepted main, manually run the Vulkan CI image workflow. Require both
   the read-only build-and-test job and the main-only publication job to pass.
   Record the published ghcr.io/dornglut/runenwerk-vulkan-ci@sha256 digest.
4. Verify GHCR package permissions: if the image is private, the consuming
   GitHub Actions job must have read access. Do not assume its visibility.
5. Implement a separately reviewed migration PR for all four required Vulkan
   jobs using the exact published image digest, not a tag. Containerized
   GitHub Actions jobs require an explicit Bash shell because their default
   is sh. Preserve existing exact revision checkout, expected check names,
   stable Rust commands, Boids/native/Render Lab proofs, artifacts, and
   merge-group coverage. Prove real behavior rather than assuming host and
   container execution are equivalent.
6. Keep the previous approved digest available for rollback. Propose
   security and package updates through new image builds, independent tests,
   release, and a separate digest-rotation PR.

## Limitations and security boundary

The image removes live Ubuntu APT setup from each normal Vulkan job only
AFTER the consuming workflow is migrated. Its own build still needs access
to Ubuntu snapshots, Rust installation, the base registry, and Cargo source
registries, so today's DNS outage can also block image creation.

An image digest makes normal execution repeatable. It does not guarantee
byte-identical future rebuilds: the upstream base is resolved at build time
and the rustup bootstrap is fetched over HTTPS. Pin and record supply-chain
inputs and review provenance before treating an image release as accepted.
Third-party code is never given GHCR write credentials during PR validation.
Docker/GHCR outages remain separate infrastructure risks.

This stage intentionally leaves .github/workflows/ci.yml untouched until a
real GHCR digest and representative native evidence exist. Draft #1483's
mirror-rewrite experiment and draft #1481's change-aware CI refactor are
separately owned work and must be reconciled, not overwritten.
