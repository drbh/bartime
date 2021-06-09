# Bartime 🍻


[![crates.io](https://meritbadge.herokuapp.com/bartime)](https://crates.io/crates/bartime)


## Installation

```bash
cargo install bartime
```

## Add OSX Application
```bash
bartime install
# Installed bartime.app at /Applications/Bartime
# Configuration file at ~/.bartime/config.toml
```

## As a CLI tool
```bash
bartime
```


## Update times

The configuration file is located in `~/.bartime/config.toml`. The application will update while running as long as you enter a valid location.

```toml
[[location]]
	name = "CAL 🏄‍♀️"
	tz = "America/Los_Angeles"

[[location]]
	name = "NYC 🗽"
	tz = "America/New_York"

[[location]]
	name = "ENG 🇬🇧"
	tz = "Europe/London"

[[location]]
	name = "SHI 🇨🇳"
	tz = "Asia/Shanghai"

[[location]]
	name = "NZE 🇳🇿"
	tz = "Pacific/Auckland"
```

# Resources

### Helpful time stuff

https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
https://24timezones.com/current_world_time.php
