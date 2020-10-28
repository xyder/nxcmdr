import os

from dotenv import load_dotenv, dotenv_values
from plumbum import cli, local, FG
from plumbum.commands.processes import CommandNotFound, ProcessExecutionError

import nx_core

from nxcmdr.printers import print_error, print_warning, print_info


ENV_PREFIX = 'NXCMDR_'


class CliApp(cli.Application):
    """ Executes a command with the selected environment variables

        Example:
            nxc -c -b 'My App - Dev environment' -f .env.development -- ./run_my_app.sh
    """

    env_file = cli.SwitchAttr(
        ['f', 'env-file'], str, mandatory=False, default='.env',
        help="Load env vars from a .env file")

    bitwarden_key = cli.SwitchAttr(
        ['b', 'bw-name'], str, mandatory=False, default='',
        help=(
            'Load env vars from one or more Bitwarden secret notes. It will merge all fields '
            'of all secure notes that have a name which contains the VALUE (case-insensitive comparison). '
            'The merging is performed by overwriting the notes in alphabetical order of their names. '
            'Example: Having "MyApp.environment.a" and "MyApp.environment.b", for a VALUE of '
            '"myapp.environment", the first set of fields will be overwritten by the second.'))
    cumulative = cli.Flag(
        ['c', 'cumulative'], default = False,
        help=("If this is present, as well as an env file and a Bitwarden name, both sources will be "
        "taken and merged, with the Bitwarden secure note env vars (see `bw-name` for how "
        "multiple notes are merged) being overwritten by the .env file env vars."))

    def main(self, *args):
        source = None

        if len(args) < 1:
            print_error(f'No command given.', check_doc=True)
            return 1

        envs = dict()
        if self.bitwarden_key:
            try:
                envs = nx_core.get_by_name(self.bitwarden_key)
            except:
                print_error('There was an error retrieving the Bitwarden data.'
                    'Please try again, more carefully.')
                exit(1)

            source = self.bitwarden_key

        if os.path.isfile(self.env_file):
            if not source:
                envs = dotenv_values(self.env_file)
                source = self.env_file
            elif self.cumulative:
                envs.update(dotenv_values(self.env_file))
                source = f'{source} and {self.env_file}'

        if not source:
            print_warning(f'No environment file found ({self.env_file}).')

        try:
            cmd = local[args[0]]
        except CommandNotFound as e:
            print_error(f'Could not find {e.program} .')
            return 1

        cmd_args = args[1:]

        own_envs = {k: v for k, v in envs.items() if k.startswith(ENV_PREFIX)}
        envs = {k: v for k, v in envs.items() if k not in own_envs}

        print_info(f'Loaded {len(envs)} vars from {source} ..')

        for k, v in own_envs.items():
            os.environ[k] = v

        with local.env(**envs):
            try:
                cmd[cmd_args] & FG
            except PermissionError as e:
                print_error(f'You do not have permission to run {cmd} .')
                return 1
            except ProcessExecutionError as e:
                print_error(f'Command exited with error code {e.retcode} .')
                if e.stderr:
                    print_error(e.stderr)
                return e.retcode
