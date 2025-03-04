/*!

This crate answers the question *"Is the terminal dark or light?"*.

It provides

* the background color, either as RGB or ANSI
* the background color's luma, which varies from 0 (black) to 1 (white)

A use case in a TUI is to determine what set of colors would be most suitable depending on the terminal's background:

```
let should_use_light_skin = terminal_light::luma()
    .map_or(false, |luma| luma > 0.6);
```

If you have very specialized skins, you may choose a more precise switch:

```
match terminal_light::luma() {
    Ok(luma) if luma > 0.85 => {
        // Use a "light mode" skin.
    }
    Ok(luma) if luma < 0.2 => {
        // Use a "dark mode" skin.
    }
    _ => {
        // Either we couldn't determine the mode or it's kind of medium.
        // We should use an itermediate skin, or one defining the background.
    }
}
```

# Strategies

## `$COLORFGBG` strategy

This environment variable is set by some terminals, like konsole or the rxvt family.
It can also be set by users.
Its value is like `15;0` where the second number is the ANSI code for the background color.

Bonus:

* querying an env variable is a fast operation

Malus:

* this env variable isn't always immediately updated when you change the color of the terminal
* the value isn't precise: `0` is "dark" and `15` is "light" but the real RGB color is uncertain as the low ANSI codes are often modified by the user

## "Dynamic colors" OSC escape sequence strategy

Modern terminals implement this xterm extension: a query making it possible to know the background color as RGB.

Terminal-light sends the query to `stdout`, then sends a more widely understood vt100 query (Status Report), waits for the answer on `stdin` with a default timeout of 100ms, then analyses this answer.

If Status Report is responded to before the background color query, we know it is not supported.

Bonus:

* this works well on all tested linux terminals
* the value is precise (RGB)
* the value is up to date when it's available
* even most terminals that don't support the xterm extension (PuTTY's pterm, for example) don't
  need to wait for the timeout

Malus:

* waiting for stdin with a timeout isn't implemented on Windows in this crate (help welcome)
* this isn't instant, a delay of 10 ms to get the answer isn't unusual
* if a terminal doesn't support the vt100 Status Report, we're waiting for 100ms
* it may fail on some terminal multiplexers
* if the timeout expires (because we are running over a slow ssh connection, for example),
  user-visible nonsense will be spewed to the terminal.

## Global strategy used by Terminal-light

1. if we're on a unix-like platform, we try the escape sequence strategy
2. if it failed or we're not on unix, we try the `$COLORFGBG` strategy
3. without a solution, we return a `TlError::Unsupported` error

*/

pub mod env;
mod error;

#[cfg(unix)]
mod xterm;

pub use {coolor::*, error::*};

/// Try to determine the background color of the terminal.
///
/// The result may come as Ansi or Rgb, depending on where
/// the information has been found.
///
/// If you want it as RGB:
///
/// ```
/// let backround_color_rgb = terminal_light::background_color()
///     .map(|c| c.rgb()); // may be an error
/// ```
pub fn background_color() -> Result<Color, TlError> {
    #[cfg(unix)]
    {
        let xterm_color = xterm::query_bg_color();
        if let Ok(xterm_color) = xterm_color {
            return Ok(Color::Rgb(xterm_color));
        }
    }
    let env_color = env::bg_color();
    if let Ok(env_color) = env_color {
        return Ok(Color::Ansi(env_color));
    }
    Err(TlError::Unsupported)
}

/// Try to return the "luma" value of the terminal's background, characterizing
/// the "light" of the color, going from 0 (black) to 1 (white).
///
/// You can say a terminal is "dark" when the luma is below 0.2 and
/// "light" when it's over 0.9. If you need to choose a pivot between
/// "rather dark" and "rather light" then 0.6 should do.
pub fn luma() -> Result<f32, TlError> {
    background_color().map(|c| c.luma())
}
