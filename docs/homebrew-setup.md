# Homebrew Installation Setup

This document provides instructions for setting up Homebrew distribution for tracker.

## Quick Start

The fastest way to set up Homebrew distribution:

1. **Create tap repository**: Create a new GitHub repository named `homebrew-tap` (e.g., `ShenMian/homebrew-tap`)
2. **Add formula**: Copy one of the formula templates below to `Formula/tracker.rb` in your tap repository
3. **Set up automation** (optional): Configure the included GitHub Actions workflow to automatically update checksums
4. **Users install via**: `brew tap ShenMian/tap && brew install tracker`

## Overview

To make tracker available via Homebrew, you'll need to:
1. Create a Homebrew tap repository
2. Add a formula file
3. Automate formula updates on releases

## Step 1: Create Homebrew Tap Repository

Create a new GitHub repository named `homebrew-tap` under your organization/user account (e.g., `ShenMian/homebrew-tap`).

Repository structure:
```
homebrew-tap/
└── Formula/
    └── tracker.rb
```

## Step 2: Create Formula File

Create `Formula/tracker.rb` in your homebrew-tap repository:

```ruby
class Tracker < Formula
  desc "Terminal-based real-time satellite tracking and orbit prediction application"
  homepage "https://github.com/ShenMian/tracker"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/ShenMian/tracker/releases/download/v0.1.15/tracker-macos-aarch64.tar.gz"
      sha256 "" # TODO: Add SHA256 checksum of the release asset
    else
      url "https://github.com/ShenMian/tracker/releases/download/v0.1.15/tracker-macos-x86_64.tar.gz"
      sha256 "" # TODO: Add SHA256 checksum of the release asset
    end
  end

  on_linux do
    url "https://github.com/ShenMian/tracker/releases/download/v0.1.15/tracker-linux-x86_64.tar.gz"
    sha256 "" # TODO: Add SHA256 checksum of the release asset
  end

  def install
    bin.install "tracker"
  end

  test do
    system "#{bin}/tracker", "--version"
  end
end
```

## Step 3: Generate SHA256 Checksums

After creating a release, generate SHA256 checksums for each platform:

```bash
# For macOS ARM64
shasum -a 256 tracker-macos-aarch64.tar.gz

# For macOS x86_64
shasum -a 256 tracker-macos-x86_64.tar.gz

# For Linux x86_64
shasum -a 256 tracker-linux-x86_64.tar.gz
```

Update the `sha256` values in the formula file with these checksums.

## Step 4: Update Formula on New Releases

When creating a new release:

1. Update version numbers in the formula URLs
2. Download the new release assets
3. Generate new SHA256 checksums
4. Update the formula file
5. Commit and push to the homebrew-tap repository

## Alternative: Using Source Installation

If you prefer users to build from source, use this simpler formula:

```ruby
class Tracker < Formula
  desc "Terminal-based real-time satellite tracking and orbit prediction application"
  homepage "https://github.com/ShenMian/tracker"
  url "https://github.com/ShenMian/tracker/archive/refs/tags/v0.1.15.tar.gz"
  sha256 "" # TODO: Add SHA256 checksum of source tarball
  license "Apache-2.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/tracker", "--version"
  end
end
```

To get the source tarball SHA256:
```bash
curl -L https://github.com/ShenMian/tracker/archive/refs/tags/v0.1.15.tar.gz | shasum -a 256
```

## Automation (Optional)

### Option 1: Using GitHub Actions (Recommended)

This repository includes a GitHub Actions workflow (`.github/workflows/homebrew.yml`) that automatically computes checksums when a new release is published.

To enable full automation:

1. Create a Personal Access Token (PAT) with `repo` scope for the homebrew-tap repository
2. Add it as a repository secret named `HOMEBREW_TAP_TOKEN`
3. The workflow will use [bump-homebrew-formula-action](https://github.com/mislav/bump-homebrew-formula-action) to update the formula

If the automation fails or the token is not set, the workflow will output manual update instructions with all the necessary checksums.

### Option 2: Using homebrew-releaser Action

Alternatively, consider using GitHub's official [homebrew-releaser](https://github.com/Homebrew/homebrew-releaser) action.

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [How to Create a Homebrew Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
