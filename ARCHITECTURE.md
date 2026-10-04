# Architecture

Trash is about as small as a COSMIC applet gets: a button, a two-item menu, and a question before
anything is deleted. What follows is the handful of decisions that are not obvious from the code.

## Layout

```
src/main.rs            the Flatpak environment fix, tracing, i18n, then applet::run
src/lib.rs             APP_ID, init_tracing, the module list
src/i18n.rs            the Fluent loader and the fl!() macro
src/trash/mod.rs       the domain: status, empty, and where the trash lives
src/trash/open.rs      opening the trash in a file manager
src/trash/watch.rs     noticing that the trash changed
src/applet/mod.rs      the cosmic::Application impl, the popup state machine, the panel button
src/applet/message.rs  the Message enum
src/applet/popup.rs    surface geometry as pure functions
src/applet/style.rs    the popup surface and the confirmation's two buttons
src/applet/subscription.rs  the watcher, as a Subscription
src/applet/symbols.rs  icon names
src/applet/view.rs     the popup: two menu rows, or the question
src/bin/trash_dump.rs  the same domain, with no display server
```

`src/trash/` must not import `cosmic::` or `iced::`. `just layering` and a CI job enforce it, which
is what lets `just run-dump` and most of the tests run with no compositor in sight.

## The trash itself is not ours

Reading and emptying the trash is `trash::os_limited`, the same crate COSMIC Files uses. That buys
the whole FreeDesktop specification — the home trash, the per-volume `.Trash-$uid` directories, the
`.trashinfo` bookkeeping — instead of a second, subtly different implementation of it.

Two details are worth knowing:

- `is_empty()` only reads each bin's `files` directory and stops at the first entry. It is cheap
  enough to call on every filesystem event, which is why the panel icon can simply be derived from
  it rather than cached behind an invalidation scheme.
- `empty()` purges **one item at a time** and counts failures instead of aborting the batch. A
  single undeletable entry should not strand everything behind it. This is what COSMIC Files does
  too.

A failed probe reports `Status::Empty`, not an error. The panel then shows the quieter icon and
`Empty Trash` stays disabled. An applet that cannot read the trash has nothing useful to say about
it, and saying it loudly in the panel would be worse than saying nothing.

## Flatpak moves the trash

The `trash` crate resolves the home trash from `XDG_DATA_HOME`, as the specification says it
should. Inside a Flatpak sandbox that variable points at `~/.var/app/<id>/data`, which is the
app's private storage and not the trash the user sees.

`trash::adopt_host_data_home()` runs as the very first statement of `main`, before any thread
exists, and repoints `XDG_DATA_HOME` at `$HOME/.local/share` when `/.flatpak-info` is present.
`cosmic_config` reads `XDG_CONFIG_HOME` and the icon themes come from `XDG_DATA_DIRS`, so nothing
else in the process notices. The manifest grants `--filesystem=xdg-data/Trash:create` to match.

Per-volume trashes are not visible inside the sandbox. Making them visible would cost
`--filesystem=host`, which is far too much to ask for a button; outside the sandbox the crate finds
them as usual.

## Noticing that the trash changed

The panel icon has to stay honest without the popup being open, so a `notify` watcher feeds a
`Subscription`. Three choices in `src/trash/watch.rs` are deliberate:

- **Both the bin and its `files` directory are watched, non-recursively.** Watching the bin is what
  survives a `files` directory that does not exist yet; watching recursively would mean walking
  every trashed directory, which is slow and buys nothing.
- **The status is re-probed on every event, and a message is emitted only when it changed.** That is
  the debounce — cheaper than a dedicated debouncer, and it also means our own `empty()` does not
  echo back as a storm of updates.
- **The watcher keeps trying until every directory is watched.** On an account that has never used
  the trash there is nothing to watch at startup; `catch_up` re-runs the setup on each event until
  it succeeds, so the icon starts working the first time something is thrown away.

The popup also re-probes when it opens. The watcher does not follow volumes mounted after startup,
and the moment the user is looking is the moment it matters.

## Opening the trash

There is no single call that works everywhere, so `src/trash/open.rs` tries three, in order:

1. `cosmic-files --trash`, which is exactly what COSMIC Files' own trash applet runs. On COSMIC this
   is the right answer and the first attempt succeeds.
2. `xdg-open trash:///`, the FreeDesktop URI. This is what works on desktops that register an
   `x-scheme-handler/trash` handler, and it is what will start working on COSMIC the day it
   registers one.
3. `xdg-open <trash>/files`, the directory itself. It shows the trashed files without restore, which
   is why it is last — but it is the one that works inside the Flatpak sandbox, where neither of the
   first two can.

Only the `xdg-open` attempts are waited on. `cosmic-files` may become a long-lived window, so its
spawn succeeding is the whole signal we get.

## The panel button is hand-rolled

`core.applet.icon_button_from_handle` sizes the glyph from the handle but takes its padding from
`suggested_padding(true)` regardless. For a symbolic icon the two agree; for the dock's full-colour
icon they do not, and the button comes out thicker than the dock. `Trash::panel_button` threads
`handle.symbolic` through both `suggested_size` and `suggested_padding`, the way `cosmic-app-list`
does, so the button is exactly the panel's thickness in either case.

Which icon to use is `panel_type != Dock` — the only thing the applet asks about its host. The dock
shows applications as full-colour icons, the panel shows status as symbolic glyphs, and a trash
applet is both depending on where it is put. `PanelType::Other`, which includes a user-created panel
and the applet run outside a panel at all, takes the symbolic branch.

## The question replaces the menu

With exactly two actions there is no room for a confirmation card below them, and no reason to make
the popup grow. While the question is up it *is* the popup; cancelling puts the two rows back. That
also means there is no height budget to maintain, and the question cannot be left behind: closing
the popup resets it.

It is an inline question rather than a dialog because the popup is a grabbing Wayland popup, and a
dialog would need a surface of its own.
