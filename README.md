# Put Icons On Workspaces

Rename sway workspaces with icons according to the applications running on them. It constructs the
name in the format `<num><sep><icon1><icon2>..` where `<num>` is the workspace number, `<sep>` is a
configurable separator followed by icons of all applications in that workspace. Duplicate icons are
stripped from the list.

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
```

## Configuration

`piow` looks for a configuration file in toml format in `${XDG_CONFIG_HOME}/piow/config.toml`. If it
can't find this file, it loads the default configuration contained in this repo. The configuration
contains a map of application names to icons, a default icon and a separator between workspaces
number and icons.

Watch the log for messages like 
`[WARN  piow::nodes] No icon for application 'app-name' in the config.` to find applications
without a config entry. Just add a line `"app-name" = "icon"` to the end of the config file and
restart piow.

## Setup with sway & waybar

Add the following to your sway config file at `${XDG_CONFIG_HOME}/sway/config`. By fowarding the
log you get access to application names without a icon in the config.

```bash
exec_always --no-startup-id piow > /tmp/piow.log
```

It's usefule to turn of additional renaming in waybar, e.g. remove `"format"` and `"format-icon"`
from the `sway/workspaces` directive in the waybar config similar to this:

```json
sway/workspaces {
  "disable-scroll": true,
  "all-outputs": true
}
```

