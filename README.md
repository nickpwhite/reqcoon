# Reqcoon
Reqcoon is a TUI http client, useful for testing APIs and making HTTP requests.

## Getting Started
1. To install, download the provided binary and put it in your PATH
2. To run, run `reqcoon -f request.http` from a terminal
3. Navigate between panes with `Ctrl+h`/`Ctrl+j`/`Ctrl+k`/`Ctrl+l`, full keybindings below
4. Enter a URL to make a request to

## Keybindings

| Mode   | Pane         | Keys | Action                         |
| ------ | ------------ | ---- | ------------------------------ |
| Normal | all          | \^h  | Move to pane left              |
| Normal | all          | \^j  | Move to pane down              |
| Normal | all          | \^k  | Move to pane up                |
| Normal | all          | \^l  | Move to pane right             |
| Normal | all          | i    | Enter insert mode              |
| Normal | all          | a    | Enter insert mode, appending   |
| Normal | all          | ↵    | Send specified request         |
| Normal | Method       | j    | Select next method             |
| Normal | Method       | k    | Select previous method         |
| Normal | Headers/Body | ⇧→   | Switch to next input type      |
| Normal | Headers/Body | ⇧←   | Switch to previous input type  |
| Normal | Headers/Body | ↹    | Switch to next input field     |
| Normal | Headers/Body | ⇧↹   | Switch to previous input field |
| Normal | Body         | \^⇧→ | Switch to next body format     |
| Normal | Body         | \^⇧← | Switch to previous body format |
| Normal | text fields  | h    | Move cursor left               |
| Normal | text fields  | j    | Move cursor down               |
| Normal | text fields  | k    | Move cursor up                 |
| Normal | text fields  | l    | Move cursor right              |
| Normal | text fields  | b    | Move cursor to previous word   |
| Normal | text fields  | w    | Move cursor to next word       |
| Normal | text fields  | ^    | Move cursor to start of line   |
| Normal | text fields  | $    | Move cursor to end of line     |
| Insert | all          | ⎋    | Enter normal mode              |
| Insert | all          | \^c  | Exit the application           |

## Issues

Please file a Github issue for bugs you encounter, feature requests, etc.
