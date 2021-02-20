# nxcmdr

CLI utility to run applications using an env file or env variables fetched from Bitwarden.
([nxcmdr website](https://gitlab.com/xyder/nxcmdr))

## Features

### Implemented
- [x] load environment variables from .env files
- [x] load environment variables from Bitwarden secure notes
- [x] run commands with environment variables
- [x] encrypted session cache
- [x] docker build/run
- [x] list variables to be saved in a file or exported

### Planned
- [ ] Cleanup, better error handling, better messages
- [ ] Default to cached if Bitwarden server cannot be reached
- [ ] Partial sync (if possible)
- [ ] Command re-run on error or with filesystem watcher (if possible)

## Installation and running

### Using the installer script
```
# for linux
curl https://gitlab.com/xyder/nxcmdr/-/raw/master/bin/installer_linux.sh | bash

# for mac
curl https://gitlab.com/xyder/nxcmdr/-/raw/master/bin/installer_apple.sh | bash
```

### Using the binaries
Downloads page: [nxcmdr releases page](https://gitlab.com/xyder/nxcmdr/-/releases).

Download the appropriate package for your OS.
Example:
```
# download the archive for your OS
curl <package url> --output nxc-archive.tar.gz

# unpack it
tar -xf ./nxc-archive.tar.gz

# run the executable
./nxc -h
```

### With docker

```
# create the environment variable for nxcmdr variables
touch .docker.env

# optionally you can add the Bitwarden user, password and token there. See the environment variables section.

# build the image
docker-compose build

# run the command
docker-compose run nxcmdr <args>

# note: when prompted, you can save the session key in the .docker.env file
```

### With rust cargo

Note that this requires rust and cargo to be installed
```
git clone https://gitlab.com/xyder/nxcmdr
cd nxcmdr
cargo install --path .
```

## Usage and examples

> Note: When loading env vars from a file, use quotes if the values contain whitespace to avoid parsing errors.
>
> This does not work:
>
> `MY_ENV_VAR=the value`
>
> , but this does:
>
>`MY_ENV_VAR='the value'`

<br>

Run a command using collected variables:
```
# running a python script using zsh, an .env file and two secure notes called "env.test_app.development" and
# "env.test_app.development - aux" I've created

nxc -cb 'env.test_app.development' -f .env.test -s $(which zsh) -- python ./main.py arg
```

Write collected variables to an .env file:
```
nxc -clb 'env.test_app.development' -f .env.test >test.env
```

Export collected variables:
```
export $(nxc -clb 'env.test_app.development' -f .env.test | xargs -d '\n')

# or, to avoid tokenization problems with the single quotes
set -a
source <(nxc -clb 'env.test_app.development' -f .env.test)
set +a
```

Unset collected variables:
```
unset $(nxc -clb 'env.test_app.development' -f .env.test | sed -E 's/(.*)=.*/\1/' | xargs)
```

Get help:
```
nxc -h
```

Output of help:

```
Execute a command with environment variables from .env files or Bitwarden secure notes

USAGE:
    nxc [FLAGS] [OPTIONS] [command]...

ARGS:
    <command>...    the command to run

FLAGS:
    -c, --cumulative    If this is present all env sources will be merged, first loading Bitwarden,
                        then the .env file
    -h, --help          Prints help information
    -l, --list          If this is present, the environment variables will be printed to stdout and
                        the command will not be executed
    -q                  If this is present, no output will be printed (except for when printing
                        environment variables, if needed)
    -V, --version       Prints version information

OPTIONS:
    -b, --bitwarden-name <bitwarden-name>
            Load env vars from one or more Bitwarden secure notes. If multiple notes containing the
            same `bitwarden-name` are found, they will be merged in alphabetical order and identical
            fields overwritten. (Example: Having two notes named "MyApp environment A" and "MyApp
            environment B" will cause any identical fields to be taken from "MyApp environment B")

    -f, --file <file>                        Load env vars from an .env file [default: ./.env]
    -s, --shell <shell>                      The shell to run this command in [default: /bin/sh]
```

### nxcmdr environment variables

The app itself uses these environment variables:

```
# defines where the app will store its' configuration files and cache. They are encrypted and will not be editable by hand. Default: $HOME/.config/nxcmdr
NXCMDR_CONFIG_DIR=/your/path/here

# defines an automatically generated session key that will be used to decrypt the config files and cache.
# the app will prompt you to save this after login.
NXCMDR_SESSION_KEY=your_key_here

# Bitwarden credentials
NXCMDR_BW_USER=your_username
NXCMDR_BW_PASS=your_password
NXCMDR_BW_TFA=your_token
```

## Development

```
git clone https://gitlab.com/xyder/nxcmdr
cd nxcmdr
cargo run -- -h
```

## Building

### Local

```
git clone https://gitlab.com/xyder/nxcmdr
cd nxcmdr
cargo build --release
./target/release/nxc -h
```

### Docker

```
git clone https://gitlab.com/xyder/nxcmdr
cd nxcmdr
# note that this will currently build from the gitlab repo, not from local
docker-compose build
```

## Note
> This project is a work in progress and not production ready yet. There may be bugs and vulnerabilities that may affect
> the overall experience or compromise the security of your data. That being said, there is no risk of corrupting data
> other than the application files itself. Use this application with that in mind and on a device that has additional
> security.
