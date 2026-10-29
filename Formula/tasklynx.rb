class Tasklynx < Formula
  desc "Developer-first local task manager with Git-style workflows"
  homepage "https://peekstore.github.io/tasklynx"
  version "0.0.1-beta-2"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/peekstore/tasklynx/releases/download/v#{version}/tasklynx-darwin-amd64.tar.gz"
    else
      url "https://github.com/peekstore/tasklynx/releases/download/v#{version}/tasklynx-darwin-arm64.tar.gz"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/peekstore/tasklynx/releases/download/v#{version}/tasklynx-linux-amd64.tar.gz"
    else
      url "https://github.com/peekstore/tasklynx/releases/download/v#{version}/tasklynx-linux-arm64.tar.gz"
    end
  end

  def install
    bin_name = Dir["tasklynx-*"].first || "tasklynx"
    bin.install bin_name => "tasklynx"
    bin.install_symlink bin/"tasklynx" => "tx"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tasklynx --version")
    assert_match version.to_s, shell_output("#{bin}/tx --version")
  end
end
