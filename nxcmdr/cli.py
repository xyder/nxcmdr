import os

from dotenv import load_dotenv, dotenv_values
from plumbum import cli, local, FG
from plumbum.commands.processes import CommandNotFound, ProcessExecutionError

from nxcmdr.printers import print_error, print_warning, print_info


ENV_PREFIX = 'NXCMDR_'


class CliApp(cli.Application):
    """Manager for the DSS server """

    def main(self, *args):
        if args:
            print_error(f'Unknown command {args[0]}.', check_doc=True)
            return 1

        if not self.nested_command:
            print_error('No command given.', check_doc=True)
            return 1


@CliApp.subcommand('run')
class RunCommand(cli.Application):
    """ Executes the given command with the selected environment variables """

    env_file = cli.SwitchAttr(['f', 'env-file'], str, mandatory=False, default='.env')
    bitwarden_key = cli.SwitchAttr('bw', str, mandatory=False, default='')

    def main(self, *args):
        if len(args) < 1:
            print_error(f'No command given.', check_doc=True)
            return 1

        if os.path.isfile(self.env_file):
            envs = dotenv_values(self.env_file)

            print_info(f'Loaded {len(envs)} vars from file {self.env_file} ..')
        else:
            print_warning(f'No environment file found ({self.env_file}).')
            envs = dict()

        try:
            cmd = local[args[0]]
        except CommandNotFound as e:
            print_error(f'Could not find {e.program} .')
            return 1

        cmd_args = args[1:]

        own_envs = {k: v for k, v in envs.items() if k.startswith(ENV_PREFIX)}
        envs = {k: v for k, v in envs.items() if k not in own_envs}

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
