# TODO — Auction House Tycoon

Last reviewed: 23 August 2026

The core research, auction, portfolio, and campaign loop is implemented. The
items below are the remaining path from the current **Playable** build to a
full release, ordered by release impact.

## P0 — Release blockers

- **Create the tracked itch.io release configuration.** `publish-itch.ps1
  -Status` currently fails because `itch.json` is missing. Add the real itch
  project/channel metadata, then require successful `-Status` and `-DryRun`
  checks before uploading a release candidate.

- **Explain failed title-screen loads.** The title screen currently discards
  the error from `apply_saved_game()`, so a missing or corrupt quicksave makes
  **Load Game** appear to do nothing. Surface a clear status there and keep
  the player on the title screen; cover both missing and invalid saves.

## P1 — Release validation

- **Run and record a full browser and Windows playthrough.** Complete one
  campaign win and one week-24 failure from a new game, including research,
  an auction win, a pass-in decision, repair or renovation, leasing, rent
  review, refinance or sale, save/load, and the final ledger. Refresh the
  matching captures in `docs/verification/` when a verified screen differs.

- **Add deterministic campaign-level regression coverage.** The existing tests
  protect simulation rules, but none drives the app's new-game, auction,
  settlement, week-advance, and campaign-outcome flow together. Add a
  non-rendering replay or equivalent action-level test that proves an authored
  winning route remains possible and that a saved live auction resumes without
  changing its outcome.

- **Perform touch and layout QA at release sizes.** The UI is drawn on a fixed
  1200×675 virtual canvas and uses several fixed-position controls. Verify in
  the browser at the catalog size and common desktop widths that every required
  action remains visible, readable, and tappable without a keyboard; fix any
  clipping, overlap, or undersized target found.

## P2 — Release-quality polish

- **Add game feedback and accessible settings.** The project has no audio
  assets or playback, and Settings currently exposes only fullscreen. Add
  restrained auction/UI sound feedback with persisted volume controls, plus a
  persisted UI-scale or readability option using the shared toolkit settings.

- **Finish player-facing release materials.** Move `game_page.json` from
  **Playable** to its final release state only after the gates above pass;
  publish release notes, credits/attribution for shipped art and audio, and
  the chosen license/support details alongside the release candidate.
