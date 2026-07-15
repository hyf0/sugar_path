# Tested release workflow

## Event and test gate

Crates.io trusted publishing accepts GitHub Actions identity tokens from `push`, `release`, and `workflow_dispatch` events, but rejects `workflow_run`. Release-plz therefore remains triggered by a `main` push and waits for the repository's `Test` workflow to report success for the exact same repository, `push` event, `main` branch, and commit SHA before either publishing or updating the release PR. The rejected `workflow_run` path was observed in [run 29387021998](https://github.com/hyfdev/sugar_path/actions/runs/29387021998), after its upstream [Test run 29386965193](https://github.com/hyfdev/sugar_path/actions/runs/29386965193) had succeeded for merge commit `c387d729e5d865b7b5d432d310f21e011cfb0d2a`.

The first attempt for each `main` push shares one concurrency group, so a newer push requests cancellation of validation and release work for the superseded commit; cancellation cannot undo an external publish that has already completed. Manual reruns use a SHA-specific group, so rerunning an old release cannot cancel the current first attempt. After the exact Test run succeeds, each Release-plz job checks out an attached local `main`, verifies its local commit against the push SHA, and reconfirms the remote `main` SHA immediately before its write action. A failed or timed-out Test run fails the release gate and does not request a crates.io token. If Test is rerun successfully after that failure, rerun the Release-plz workflow for the same current `main` push; never rerun an older release as a substitute.

The trusted-publishing job and every action with repository write access use full action commit SHAs, with the human-facing release line retained in comments. This prevents a movable action tag or branch from changing the reviewed publishing code between the exact-SHA Test and the release job. The pre-transfer success in run 29319137656 proves that the `push` event is accepted for trusted publishing, but it does not prove the current `hyfdev` owner configuration. The first post-merge `main` run must complete the crates.io token exchange; if crates.io rejects the current repository identity, rebuild the trusted publisher configuration for the transferred repository before retrying the same release.

## Durable evidence

- [Release workflow](../../.github/workflows/release.yml)
- [Main test workflow](../../.github/workflows/test.yaml)
- [Pre-transfer evidence that a push event can obtain a trusted-publishing token](https://github.com/hyfdev/sugar_path/actions/runs/29319137656)
