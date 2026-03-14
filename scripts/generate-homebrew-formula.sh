#!/usr/bin/env bash
set -euo pipefail

dist_dir="${1:?usage: generate-homebrew-formula.sh DIST_DIR VERSION [OUTPUT_PATH]}"
version="${2:?usage: generate-homebrew-formula.sh DIST_DIR VERSION [OUTPUT_PATH]}"
output_path="${3:-}"

read_checksum() {
  local sha_path="${1:?sha path required}"
  awk '{print $1; exit}' "${sha_path}"
}

arm_sha="$(read_checksum "${dist_dir}/magellan-aarch64-apple-darwin.tar.xz.sha256")"
intel_mac_sha="$(read_checksum "${dist_dir}/magellan-x86_64-apple-darwin.tar.xz.sha256")"
intel_linux_sha="$(read_checksum "${dist_dir}/magellan-x86_64-unknown-linux-gnu.tar.xz.sha256")"

formula_contents="$(cat <<EOF
class Magellan < Formula
  desc "Deterministic presentation engine for AI-generated technical walkthroughs"
  homepage "https://github.com/nclandrei/magellan"
  version "${version}"
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/nclandrei/magellan/releases/download/v${version}/magellan-aarch64-apple-darwin.tar.xz"
      sha256 "${arm_sha}"
    end
    if Hardware::CPU.intel?
      url "https://github.com/nclandrei/magellan/releases/download/v${version}/magellan-x86_64-apple-darwin.tar.xz"
      sha256 "${intel_mac_sha}"
    end
  end
  if OS.linux?
    if Hardware::CPU.intel?
      url "https://github.com/nclandrei/magellan/releases/download/v${version}/magellan-x86_64-unknown-linux-gnu.tar.xz"
      sha256 "${intel_linux_sha}"
    end
  end
  license "MIT"

  def install
    bin.install "magellan"

    doc_files = Dir["README.*", "readme.*", "LICENSE", "LICENSE.*", "CHANGELOG.*"]
    leftover_contents = Dir["*"] - doc_files - ["magellan"]
    pkgshare.install(*leftover_contents) unless leftover_contents.empty?
  end

  test do
    assert_match "magellan", shell_output("#{bin}/magellan --help")
  end
end
EOF
)"

if [[ -n "${output_path}" ]]; then
  printf '%s\n' "${formula_contents}" > "${output_path}"
else
  printf '%s\n' "${formula_contents}"
fi
