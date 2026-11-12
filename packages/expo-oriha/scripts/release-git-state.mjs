// Verify local and remote release tag identity before publication.
export function checkGitState(version, {
  repoRoot, run, fail, warn, allowDirty, skipRemote, summarizeCommandFailure,
})  {
  const status = run("git", ["status", "--porcelain"], { cwd: repoRoot });
  if (!status.ok) {
    fail(`Could not read git status: ${summarizeCommandFailure(status)}`);
    return;
  }

  if (status.stdout.trim() && !allowDirty) {
    fail("Git working tree is dirty. Commit release changes before publishing.");
  }

  const head = run("git", ["rev-parse", "HEAD"], { cwd: repoRoot });
  if (!head.ok) {
    fail(`Could not resolve HEAD: ${summarizeCommandFailure(head)}`);
    return;
  }

  const headSha = head.stdout.trim();
  const commit = run("git", ["cat-file", "commit", headSha], { cwd: repoRoot });
  if (!commit.ok || !commit.stdout.split("\n\n", 1)[0].includes("gpgsig -----BEGIN PGP SIGNATURE-----")) {
    throw new Error("Release HEAD needs an OpenPGP-signed commit. Stop release work.");
  }
  const verifiedCommit = run("git", ["-c", "gpg.format=openpgp", "verify-commit", headSha], { cwd: repoRoot });
  if (!verifiedCommit.ok) {
    throw new Error(`OpenPGP commit verification failed. Stop release work: ${summarizeCommandFailure(verifiedCommit)}`);
  }
  const tagName = `v${version}`;
  const tagType = run("git", ["cat-file", "-t", `refs/tags/${tagName}`], {
    cwd: repoRoot,
  });

  if (!tagType.ok) {
    fail(`Missing local release tag ${tagName}.`);
    return;
  }

  if (tagType.stdout.trim() !== "tag") {
    fail(`Local release tag ${tagName} is not an annotated tag.`);
    return;
  }
  const verifiedTag = run("git", ["-c", "gpg.format=openpgp", "verify-tag", tagName], { cwd: repoRoot });
  if (!verifiedTag.ok) {
    throw new Error(`OpenPGP tag verification failed. Stop release work: ${summarizeCommandFailure(verifiedTag)}`);
  }

  const tagCommit = run("git", ["rev-parse", `refs/tags/${tagName}^{commit}`], {
    cwd: repoRoot,
  });

  if (!tagCommit.ok) {
    fail(`Could not resolve release tag ${tagName}: ${summarizeCommandFailure(tagCommit)}`);
    return;
  }

  if (tagCommit.stdout.trim() !== headSha) {
    fail(`Release tag ${tagName} does not point at HEAD.`);
  }

  if (skipRemote) {
    warn(`Skipped remote tag verification for ${tagName}.`);
    return;
  }

  const remote = run("git", ["ls-remote", "--tags", "origin", `${tagName}*`], {
    cwd: repoRoot,
    timeout: 30_000,
  });

  if (!remote.ok) {
    fail(`Could not verify remote release tag ${tagName}: ${summarizeCommandFailure(remote)}`);
    return;
  }

  const peeledTagLine = remote.stdout
    .split(/\r?\n/)
    .find((line) => line.endsWith(`refs/tags/${tagName}^{}`));
  const remoteCommit = peeledTagLine?.split(/\s+/)[0];

  if (!remoteCommit) {
    fail(`Remote release tag ${tagName} is missing or is not annotated.`);
    return;
  }

  if (remoteCommit !== headSha) {
    fail(`Remote release tag ${tagName} points at ${remoteCommit}, not HEAD ${headSha}.`);
  }
}
