class Tasklynx < Formula
  desc "Developer-first local task manager with Git-style workflows"
  homepage "https://peekstore.github.io/tasklynx"
  version "0.0.1-beta-4"
  license "MIT"

  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/peekstore/tasklynx/releases/download/v#{version}/tasklynx-darwin-amd64.tar.gz"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/peekstore/tasklynx/releases/download/v#{version}/tasklynx-darwin-arm64.tar.gz"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/peekstore/tasklynx/releases/download/v#{version}/tasklynx-linux-amd64.tar.gz"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/peekstore/tasklynx/releases/download/v#{version}/tasklynx-linux-arm64.tar.gz"
  end

  def install
    bin.install Dir["tasklynx-*"].first => "tasklynx"
    bin.install_symlink bin/"tasklynx" => "tx"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tasklynx --version")
  end
end
