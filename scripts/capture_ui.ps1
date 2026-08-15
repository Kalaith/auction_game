<#
.SYNOPSIS
    Headless screenshot harness for Auction House Tycoon.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (AUCTION_GAME_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Named scenes cover the title, briefing, listings, research,
    live auction phases and outcomes, portfolio decisions, weekly ledgers,
    sales, and both campaign conclusions. Unknown scene names fail loudly.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
    ./scripts/capture_ui.ps1 -Scenes auction,portfolio -WindowWidth 1200 -WindowHeight 675
#>
param(
    [string[]]$Scenes = @("title", "dashboard", "auction"),
    [int]$Frames = 150,
    [int]$WindowWidth = 0,
    [int]$WindowHeight = 0,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -WindowWidth $WindowWidth -WindowHeight $WindowHeight -OutputDir $OutputDir -SkipBuild:$SkipBuild
