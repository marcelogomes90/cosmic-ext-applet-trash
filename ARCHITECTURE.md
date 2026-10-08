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

## Opening the trash, and why it runs on the host

There is no single call that works everywhere, so `src/trash/open.rs` tries in order:

1. `cosmic-files --trash`, which is exactly what COSMIC Files' own trash applet runs. On COSMIC this
   is the right answer and the first attempt succeeds. It is only attempted after
   `command -v cosmic-files` says it exists, because a long-lived window gives us no useful exit
   code to fall back on.
2. `xdg-open trash:///`, the FreeDesktop URI, for desktops that register an
   `x-scheme-handler/trash` handler.
3. `xdg-open <trash>/files`, the directory itself. It shows the trashed files without restore, which
   is why it is last.

**Every one of those runs through `flatpak-spawn --host` when sandboxed**, and that is the whole
reason the applet asks for `--talk-name=org.freedesktop.Flatpak`. Two things break otherwise:
COSMIC Files does not exist inside the sandbox at all, and — worse — `xdg-open trash:///` inside the
sandbox goes to the OpenURI portal, which answers an unhandled scheme by showing the user an
application chooser and then **returning success**. A fallback chain cannot fall back through a
step that lies about having worked. On the host, the exit code means what it says.

Two things about that host spawn cost a round of debugging each, and neither is guessable:

- **`flatpak-spawn --host` hands the process no session environment.** `WAYLAND_DISPLAY` and
  `DISPLAY` are both empty there, so COSMIC Files panicked with *"neither WAYLAND_DISPLAY nor
  WAYLAND_SOCKET nor DISPLAY is set"* before drawing anything. `Launcher::detect` therefore asks the
  host to list its `XDG_RUNTIME_DIR` once, takes the first `wayland-N` socket, and passes it back as
  `--env=WAYLAND_DISPLAY=`. The applet's own `WAYLAND_DISPLAY` is no use: inside a panel it names
  cosmic-panel's nested compositor, not the session's.
- **Spawning successfully is not the same as launching successfully.** The first version treated a
  successful `spawn()` as the end of the chain, so the panic above looked exactly like an opened
  trash and nothing ever fell back. `Launcher::launch` now waits a settle window: still running
  afterwards counts, exiting non-zero within it does not. `Launcher::handled` is the opposite case —
  `xdg-open` is *supposed* to exit, so it is waited on in full.

The last resort is a plain in-sandbox `xdg-open` on the trash directory, for the case where
`flatpak-spawn` itself is refused. It is the only step that cannot fail for want of a permission.

## The panel button is hand-rolled

`core.applet.icon_button_from_handle` sizes the glyph from the handle but takes its padding from
`suggested_padding(true)` regardless. For a symbolic icon the two agree; for the dock's full-colour
icon they do not, and the button comes out thicker than the dock. `Trash::panel_button` threads
`handle.symbolic` through both `suggested_size` and `suggested_padding`, the way `cosmic-app-list`
does, so the button is exactly the panel's thickness in either case.

## What the applet asks about its host

One question, `is_dock`, and three answers hang off it:

- **The icon.** The dock shows applications as full-colour icons, the panel shows status as symbolic
  glyphs, and a trash applet is both depending on where it is put.
- **The tooltip.** The dock names every icon it holds on hover and an applet gets none of that for
  free; a panel names nothing, so a tooltip there would be noise.
- **The menu row's height.** The dock pins each row to `20 + 2 * space_xxs`, the figure
  `cosmic-app-list` pins its own context menu to; on a panel the row is whatever
  `cosmic::applet::menu_button` comes out as. Nothing else about the row differs.
`PanelType::Other`, which includes a user-created panel and the applet run outside a panel at all,
is not a dock.

**Frosting is not one of the questions, and it is not a dock-versus-panel difference either.** The
runtime keeps `Theme::transparent` as `blur_enabled && core.frosted(theme)`, where `frosted` reads
the theme's `frosted_applets` and `blur_enabled` is set from an `Action::BlurEnabled` the
**compositor** sends. One `cosmic-panel` process serves every space, and it offers the blur protocol
to all of its applet clients or to none of them — it gates the global on whether the host offers it
to the panel, not on which space the applet sits in. So both faces of this applet are frosted or
neither is, and the applet only has to read `background(theme.transparent)`, the same token
`core.applet.popup_container` uses. Deciding it here instead was tried, and it got the dock wrong.

## Left click acts, right click offers

The panel button is a `button::custom` wrapped in a `mouse_area`: the button's `on_press` opens the
trash, and the mouse area's `on_right_press` opens the popup. A `button` ignores right presses, so
the event reaches the wrapper without a fight.

That is why the desktop entry has **no** `X-CosmicHoverPopup`. Sweeping the pointer across the
applet while another popup is open would otherwise open a context menu nobody asked for — right for
a status applet whose popup *is* its content, wrong for a context menu.

The popup keeps `Open Trash` as its first row even though a left click already does it: a context
menu that offers only the destructive half of what the applet does reads as if opening were
unavailable.

Cancelling the question closes the popup rather than returning to the menu — the user who cancels is
done, not browsing.

The tooltip naming the applet is wrapped on **only** when `panel_type == Dock`. The dock names every
icon it holds on hover and an applet gets none of that for free; a panel names nothing, so a tooltip
there would be noise.

## One translucent layer, or the frosting is gone

COSMIC has **two** menu idioms. The **dock**'s app list fills each row with its own block of colour
— `button::custom` with `Button::MenuItem`, which resolves to `background(transparent).component` —
and **panel** applets do not: `cosmic::applet::menu_button` uses `Button::AppletMenu`, whose resting
fill is `text_button.base`, `#00000000`.

This applet takes the panel's, on both hosts, because the dock's cannot be frosted. The fills carry
the same alpha as the popup's own surface — 1.0 opaque, 0.761 frosted — so `MenuItem` over
`popup_container` is **two** translucent layers, and what reaches the eye from behind the popup is
`0.239 * 0.239`: **5.8%**, against 23.9% for the single layer a panel row leaves. The row colour is
nearly the whole popup, so the whole menu reads as if frosted glass were switched off, which is
exactly how it was reported. `AppletMenu` adds no layer of its own and the popup's one surface
frosts, so both faces now show the blur the theme asked for.

Taking the dock's idiom was tried, for one release, to make the menu look like the app list's beside
it. `cosmic-app-list` has the same two layers and the same near-opaque result; matching it is not
worth losing the effect the theme enables.

What stays per host is the row's **height** — see above — and nothing else. `question()` is one face
of the same popup and it also sits plain on that single surface: a card of its own behind it is a
second layer with the same arithmetic.

`AppletMenu` is taken for its colours and **not** for its corners: it pins `radius_0`, so its hover
fill is a rectangle and the first and last rows spill past the rounded border `popup_container` drew
at `radius_m`. `style::menu_row` is that class with the corners put back, and it is
`cosmic-status-hub`'s `menu_view` idiom part for part — `Edges`, `inner_radius`, `follow_surface`,
the same `Catalog as _` delegation — because the two applets sit on the same kind of surface and a
second answer to one question is a second thing to keep right.

**Only the corners that touch the surface are rounded**, and they take the surface's own
`radius_m` **less the 1 px border they sit inside**: the first row at the top, the last row at the
bottom, every other corner square so rows meet the divider flush. A uniform radius was tried and is
wrong in both directions — `radius_s` on all four corners still spills under the default `Round`
theme, where the surface is twice as round, and `radius_m` on all four rounds the inner corners that
nothing is curving around. `inner_radius` clamps at zero, so a square theme stays square.

`AppletMenu` discards the `on_disabled` colour `Catalog::disabled` works out, so a disabled row dims
its own label through `style::dimmed_text`, on either host.

The rest comes from `cosmic-app-list/src/app.rs` unchanged:
`core.applet.popup_container(container(content).padding(1))` paints the surface — hand-rolling an
equivalent drifted twice, and the helper is the only way to be sure the popup is whatever the native
ones are, including how it frosts. `.limits(..)` caps it at **300 px**, the figure app-list
overrides the helper's own 360 with. The separator is `divider::horizontal::light()`, whose
`FillMode::Padded(8)` *is* the inset; a `default()` divider in a padded container is the wrong way
to the same look and gets the amount wrong.

## The question is the popup's other face

COSMIC Files asks this same question, and the two should read as one question. It exposes no D-Bus
interface and no command-line flag to raise its own dialog, so the question has to be asked here.

`widget::dialog()` is the obvious way to ask it and is the wrong one. It paints its own card on the
**primary** layer, with a border and a drop shadow of its own, while the popup's surface is the
**background** layer. Side by side the two read as different windows, and the card covers the
frosted background the popup already has.

So `question()` reproduces the dialog's *contents* — `title3` heading, `space_xxs` gap, body text, a
right-aligned `suggested`/`standard` pair, the `space_l` and `space_m` spacing scale — and nothing
of its chrome.

**It is not a surface of its own, and that was tried.** An overlay layer surface with
`KeyboardInteractivity::Exclusive` is the other way to ask, and it is how `cosmic-applet-power` once
did it. Here it failed to draw — and a layer surface that holds the keyboard exclusively and then
draws nothing leaves the whole session unable to type or open any menu, with no way back short of
killing the applet. An applet must not be able to take the session down by getting its own geometry
wrong. A popup cannot do that, so the popup is where the question lives.

## The icons

The only drawn asset is `resources/<app-id>.svg`, for the desktop entry's `Icon=` and the Store
listing. There is deliberately **no** `<app-id>-symbolic.svg`: the sibling applets install one
because they paint it in the panel, and this one never does — both panel faces come from the icon
theme. A file nothing resolves is only a file to keep in step.

The glyph is centred on the canvas, not merely placed in it. The applet list draws the icon in a
fixed box, where a glyph whose extents are not symmetric about the centre reads as hanging low.

The menu rows carry no icons at all. Two actions do not need picture clues, and COSMIC's own applet
menus are text.
