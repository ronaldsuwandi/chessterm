# Changelog

## [v0.1.0](https://github.com/ronaldsuwandi/chessterm/releases/tag/v0.1.0) Initial release (2025-02-04)

First public release of chessterm

### Features
- PGN-only input
- Bitboard based engine
- Board flipping
- Terminal UI using `ratatui` and `ratatui-image`
- Historical moves list

### Compatibility Notice
- Only tested on Mac
- Best experience: Kitty (renders perfectly)
- Works with issues: iTerm2 (flickering may occur). Use `--halfblocks` mode to reduce flickering
- Not supported: Mac's default Terminal.app

### Known Limitations
- No AI Opponent – chessterm is strictly designed for human-vs-human games
- No Threefold Repetition Rule – The engine does not track repeated positions for automatic draws 
- No 50-Move Rule – The game does not enforce automatic draws after 50 moves without a pawn move or capture

