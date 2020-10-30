# nxcmdr

CLI utility to run applications using an env file or env variables fetched from Bitwarden.
([website](https://gitlab.com/xyder/nxcmdr))

## Installation
```bash
export NXCMDR_CONFIG_DIR=your/path/here
pip install nxcmdr --extra-index-url https://gitlab.com/api/v4/projects/22063064/packages/pypi/simple
```

## Usage

```
nxc --help-all
```
```
Executes a command with the selected environment variables

Example:
    nxc -c -b 'My App - Dev environment' -f .env.development -- ./run_my_app.sh

Usage:
    nxc [SWITCHES] args...

Meta-switches:
    -h, --help                    Prints this help message and quits
    --help-all                    Prints help messages of all sub-commands and quits
    -v, --version                 Prints the program's version and quits

Switches:
    -b, --bw-name VALUE:str       Load env vars from one or more Bitwarden secret notes. It will merge all fields of all secure notes that
                                  have a name which contains the VALUE (case-insensitive comparison). The merging is performed by
                                  overwriting the notes in alphabetical order of their names. Example: Having "MyApp.environment.a" and
                                  "MyApp.environment.b", for a VALUE of "myapp.environment", the first set of fields will be overwritten by
                                  the second.
    -c, --cumulative              If this is present, as well as an env file and a Bitwarden name, both sources will be taken and merged,
                                  with the Bitwarden secure note env vars (see `bw-name` for how multiple notes are merged) being
                                  overwritten by the .env file env vars.
    -f, --env-file VALUE:str      Load env vars from a .env file; the default is .env
```

## Development

TBD

## Note
> This project is a work in progress and not production ready yet. There may be bugs and vulnerabilities that may affect
> the overall experience or compromise the security of your data. That being said, there is no risk of corrupting data
> other than the application files itself. Use this application with that in mind and on a device that has additional
> security.

> The project is currently a mix of Python and Rust since we haven't found or implemented a good way to run a sub-process
> and have the stdin/stdout/stderr promoted to the parent process properly. Once that is found or implemented, the
> project will be converted to full Rust.
