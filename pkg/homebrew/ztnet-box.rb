# typed: false
# frozen_string_literal: true

# Homebrew formula for ztnet-box.
# Auto-updated by the release workflow; do not edit manually.
class ZtnetBox < Formula
  desc "Local web UI for ZeroTier management"
  homepage "https://github.com/CleoWixom/ztnet-box"
  version "0.6.3"

  on_macos do
    on_arm do
      url "https://github.com/CleoWixom/ztnet-box/releases/download/v#{version}/ztnet-box-#{version}-aarch64-apple-darwin.tar.gz"
      # sha256 is updated by the release workflow
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/CleoWixom/ztnet-box/releases/download/v#{version}/ztnet-box-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/CleoWixom/ztnet-box/releases/download/v#{version}/ztnet-box-#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/CleoWixom/ztnet-box/releases/download/v#{version}/ztnet-box-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "ztnet-box"
    (etc/"ztnet-box").mkpath
    (etc/"ztnet-box"/"config.yml.example").write(
      resource("config.yml.example").path.read
    ) unless (etc/"ztnet-box"/"config.yml").exist?
  end

  service do
    run [opt_bin/"ztnet-box"]
    keep_alive true
    log_path var/"log/ztnet-box.log"
    error_log_path var/"log/ztnet-box.log"
  end

  test do
    system "#{bin}/ztnet-box", "--version"
  end
end
