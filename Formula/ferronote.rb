class Ferronote < Formula
  desc "Blazing-fast terminal note-taking app inspired by Notational Velocity"
  homepage "https://github.com/tareqmy/ferronote"
  url "https://github.com/tareqmy/ferronote/archive/refs/tags/v1.0.5.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "ferronote", shell_output("#{bin}/ferronote --version")
  end
end
