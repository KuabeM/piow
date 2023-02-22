# Put Icons On Workspaces

[![Build](https://img.shields.io/github/actions/workflow/status/KuabeM/piow/build_master.yml?branch=master)](https://github.com/KuabeM/piow/actions?query=workflow%3Abuild-master)
[![Crates.io](https://img.shields.io/crates/v/piow.svg)](https://crates.io/crates/piow)
[![AUR](https://img.shields.io/aur/version/piow-bin)](https://aur.archlinux.org/packages/piow-bin)

Rename [sway] workspaces with icons according to the applications running on them. It constructs the
name based on a format string from the config. Duplicate icons are stripped from the list.

With a space as separator and waybar, it looks like this:

![](piow.png)

## Installation & Usage

Build and install from source with cargo. Then simply run the executable. See `piow --help` for
supported cli options.

```bash
# install
cargo install piow
# run
piow
# increase log level for troubleshooting, possible values: Trace, Debug, Info, Warn, Error
RUST_LOG=piow=Debug piow
```

## Configuration

`piow` looks for a configuration file in toml format in `${XDG_CONFIG_HOME}/piow/config.toml` and
`/etc/xdg/piow/config.toml` (former takes precedence). If it can't find this file, it loads the
default configuration contained in this repo. The configuration contains a map of application names
to icons, a default icon and a separator between workspaces number and icons. For getting started,
just copy `default.toml` over to `${XDG_CONFIG_HOME}/piow/config.toml` and start adding your own
icons.

| Configuration Key | Description                                                                                                                                            |
|-------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| `default_icon`    | Icon used for apps without a configured icon                                                                                                           |
| `format_str`      | Format string for generating names. Supported placeholders:<br/>  `%n`: Workspace number<br/>  `%i`: Icons                                             |
| `icon_separator`  | Literal between icons                                                                                                                                  |
| `[icons]`         | paris of `"app-id" = "icon"`, app id as reported by `swaymsg -t get_tree`, matches substrings, e.g. an entry 'libre' will be matched for 'libreoffice' |

Watch the log for messages like `[WARN piow::nodes] No icon for application 'app-name' in the
config.` to find applications without a config entry. Just add a line `"app-name" = "icon"` to the
end of the config file and restart piow. The crate [find_unicode] is awesome for finding suitable
icons on the command line.

## Setup with sway & waybar

Add the following to your sway config file at `${XDG_CONFIG_HOME}/sway/config`. By forwarding the
log you get access to application names without a icon in the config.

```bash
exec_always --no-startup-id piow --syslog
```

It's useful to turn of additional renaming in waybar, e.g. remove `"format"` and `"format-icon"`
from the `sway/workspaces` directive in the waybar config similar to this:

```json
sway/workspaces {
  "disable-scroll": true,
  "all-outputs": true
}
```

> Hint: If you remove the workspace number placeholder `%i` from the format string, the ordering of
> the workspaces may no longer work as intended.

## Similar Projects

[Workstyle] has a similar approach but uses the i3 IPC connection. The config file format was
inspired by this crate.

[sway]: https://github.com/swaywm/sway
[find_unicode]: https://crates.io/crates/find_unicode
[Workstyle]: https://github.com/pierrechevalier83/workstyle
