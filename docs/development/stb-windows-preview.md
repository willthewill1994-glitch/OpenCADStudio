# Windows STB preview package

The corrected source is on `feat/native-stb-plot-styles` (STB implementation
commit `1c81a5f42`). The local Linux tests and real-file export do **not**
constitute a Windows build or Windows runtime validation.

`.github/workflows/stb-windows-preview.yml` prepares a portable x64 executable
for Windows 11 Intel/AMD computers. It runs on a Windows GitHub Actions runner
when the corrected branch is pushed to an authorized repository, or by manual
dispatch once that workflow is available there. It does not create a release,
open a pull request, change the upstream repository, or upload user drawings.
The workflow has read-only repository permissions.

It uses the repository's existing pinned actions and native MSVC build target,
runs the six new STB checks in the integration runner, builds the release
executable, and checks `--help` startup. A failing test/build prevents upload.
These checks do not validate interactive graphics drivers or a physical printer.

After a successful run, download `OpenCADStudio-STB-Windows11-x64-unsigned`
from the run's **Artifacts** section and extract the ZIP. It contains the
portable `.exe`, GPL license, exact application source archive, technical
notes, commit identification and executable SHA-256. It is an experimental,
unsigned build; it is not the project's official signed MSI installer.
The package does not include DWG files, the user's STB or exported PDFs.

## Current delivery blocker

At preparation time, GitHub CLI authentication in this environment is invalid
(`gh auth status`). No branch has been pushed and no Windows runner has been
started. The workflow has only been checked locally for YAML structure and
expected build/test/package steps. There is no Windows executable to download
yet. An authorized GitHub connection and a destination repository are needed
before pushing the branch and obtaining the downloadable build artifact.

## First Windows run and packaging correction

Run `44179877664` successfully compiled and passed all six STB tests on
Windows (78m 28s for the first release/test build). The following standalone
`cargo build` unnecessarily recompiled the app with a different dependency
feature graph; the 90-minute job timeout cancelled it before ZIP publication.
No downloadable package was produced by that run.

The workflow now reuses the application executable Cargo already builds for
integration tests and checks its startup directly. Absence of that executable
fails packaging explicitly. The preview uses optimization level 1 and 64
codegen units with two jobs to reduce compilation time; no STB source or test
assertions have changed. The timeout is 180 minutes to leave enough margin
for the initial Windows build. This is a preview configuration, not the
project's default optimization-level-3 official release configuration.
