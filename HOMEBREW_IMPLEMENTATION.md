# Homebrew Installation Implementation Summary

This PR implements support for Homebrew installation as requested in the issue.

## What Was Implemented

### 1. Documentation for Maintainers (`docs/homebrew-setup.md`)
- **Quick Start Guide**: Step-by-step instructions for setting up Homebrew distribution
- **Formula Templates**: Two options provided:
  - Binary distribution formula (using pre-built releases from GitHub)
  - Source distribution formula (builds from source using Cargo)
- **Automation Guide**: Instructions for automating formula updates
- **Resources**: Links to official Homebrew documentation

### 2. Updated README (`README.md`)
- Added Homebrew installation section under "Package manager"
- Installation command: `brew tap ShenMian/tap && brew install tracker`
- Added reference to Homebrew Setup Guide in documentation section

### 3. GitHub Actions Automation (`.github/workflows/homebrew.yml`)
- Automatically triggers on new releases
- Downloads release assets and computes SHA256 checksums
- Attempts to update Homebrew formula automatically (if token is configured)
- Falls back to providing manual update instructions if automation fails

## Next Steps for Maintainer

To complete the Homebrew setup, the repository owner needs to:

1. **Create Homebrew Tap Repository**
   ```bash
   # Create a new repository: ShenMian/homebrew-tap
   mkdir homebrew-tap
   cd homebrew-tap
   mkdir Formula
   ```

2. **Add Formula File**
   - Copy one of the formula templates from `docs/homebrew-setup.md`
   - Save it as `Formula/tracker.rb` in the homebrew-tap repository
   - Update version and checksums for the current release

3. **Optional: Configure Automation**
   - Create a Personal Access Token with `repo` scope
   - Add it as repository secret: `HOMEBREW_TAP_TOKEN`
   - The workflow will automatically update the formula on new releases

4. **Test Installation**
   ```bash
   brew tap ShenMian/tap
   brew install tracker
   tracker --version
   ```

## Benefits

- **Easy Installation**: Users can install via a simple `brew install` command
- **Cross-Platform**: Works on both macOS and Linux
- **Automated Updates**: Optional automation reduces manual work for releases
- **Well-Documented**: Comprehensive guides for both users and maintainers

## Resources Used

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [How to Create a Homebrew Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Rust Project Homebrew Example](https://federicoterzi.com/blog/how-to-publish-your-rust-project-on-homebrew/)

## File Changes

- `README.md`: Added Homebrew installation instructions
- `docs/homebrew-setup.md`: New comprehensive setup guide
- `.github/workflows/homebrew.yml`: New automation workflow

All changes follow minimal modification principles and don't affect existing functionality.
