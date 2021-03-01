import os
from pathlib import Path

from plumbum import cli, local, FG
from plumbum.cmd import rm, mkdir, cp, tar, docker
import gitlab
from pip._vendor import requests

# This script requires docker to be running for the docker parts
# it also requires the following env vars: GITLAB_PROJECT_ID, GITLAB_TOKEN


# move to project root
os.chdir(Path('../..'))

def fail(message, code=1):
    print(f'ERROR: {message}')
    exit(code)


def info(message):
    print(f'INFO: {message}')


class BaseCommands:
    temp_dir = './releases/tmp'
    release_dir = './releases'

    TARGET_VOLUME = 'nxcmdr-target'

    BASE_GITLAB = 'https://gitlab.com'
    BASE_GITLAB_API = f'{BASE_GITLAB}/api/v4'
    PROJECT_URL = f'{BASE_GITLAB_API}/projects/{os.environ["GITLAB_PROJECT_ID"]}'
    PACKAGES_URL = f'{PROJECT_URL}/packages/generic/nxcmdr'

    @staticmethod
    def _build_linux_release():
        # todo: build using a docker image/container instead of local machine
        local['cargo']['build', '--release'] & FG
        info('Finished building linux release ..')

    @classmethod
    def _build_mac_release(cls):

        # build the builder image
        docker['build', '-f', 'Dockerfile.macosx', '-t', 'nxcmdr-macosx', '.'] & FG

        # delete the old build dest volume, if exists
        # todo: perhaps add a --clean flag and keep docker volume between builds
        # docker['volume', 'rm', TARGET_VOLUME](retcode = None)

        # re-create the volume
        docker['volume', 'create', cls.TARGET_VOLUME](retcode = None)

        # build the release
        docker['run', '--rm', '-v', f'{cls.TARGET_VOLUME}:/build/target', '-it', 'nxcmdr-macosx'] & FG

        info('Finished building mac release ..')

    @staticmethod
    def _reset_dir(dir_name):
        rm['-fr', dir_name]()
        mkdir[dir_name]()

    @classmethod
    def _clean(cls):
        cls._reset_dir(cls.release_dir)

    @classmethod
    def _create_linux_archive(cls, version):
        target = 'x86_64-unknown-linux-gnu'
        target_archive = f'{cls.release_dir}/nxc-{version}-{target}.tar.gz'

        cls._reset_dir(cls.temp_dir)

        cp['./target/release/nxc', cls.temp_dir]()
        cp['README.md', 'LICENSE', './target/release/nxc', cls.temp_dir]()

        tar['-czvf', target_archive, '-C', cls.temp_dir, '.']()

        # cleanup
        rm['-fr', cls.temp_dir]()

        info(f'Created release archive: {target_archive}')

    @classmethod
    def _create_mac_archive(cls, version):
        target = 'x86_64-apple-darwin'
        target_archive = f'{cls.release_dir}/nxc-{version}-{target}.tar.gz'

        cls._reset_dir(cls.temp_dir)

        docker[
            'container', 'create',
            '--name', 'interim',
            '-v', f'{cls.TARGET_VOLUME}:/root',
            'tianon/true'
        ]()

        docker['cp', f'interim:/root/{target}/release/nxc', cls.temp_dir]()

        docker['rm', 'interim']()

        cp['README.md', 'LICENSE', cls.temp_dir]()

        tar['-czvf', target_archive, '-C', cls.temp_dir, '.']()

        # cleanup
        rm['-fr', cls.temp_dir]()

        info(f'Created release archive: {target_archive}')

    @classmethod
    def _upload_file(cls, version, target_archive):
        url = f'{cls.PACKAGES_URL}/{version}/{target_archive}'

        resp = requests.put(
            url,
            headers={'PRIVATE-TOKEN': os.environ['GITLAB_TOKEN']},
            files={'upload_file': open(f'./releases/{target_archive}', 'rb')}
        )

        content = resp.json()
        if resp.status_code >= 400:
            fail(f'Upload package "{target_archive}" failed with: {content}')

        return content

    @classmethod
    def _upload_package(cls, project, version):
        target_archives = dict(
            linux=f'nxc-{version}-x86_64-unknown-linux-gnu.tar.gz',
            mac=f'nxc-{version}-x86_64-apple-darwin.tar.gz'
        )
        reverse_targets = {v: k for k, v in target_archives.items()}

        cls._upload_file(version, target_archive=target_archives['linux'])
        cls._upload_file(version, target_archive=target_archives['mac'])

        package = [p for p in project.packages.list() if p.version == version][0]

        package_files_url = f'{package._links["delete_api_path"]}/package_files'

        resp = requests.get(package_files_url, headers={'PRIVATE-TOKEN': os.environ['GITLAB_TOKEN']})
        content = resp.json()

        if resp.status_code >= 400:
            fail(f'Package retrieval failed with: {content}')

        files = {reverse_targets[v['file_name']]: v for v in content}

        return files

    @classmethod
    def _create_release(cls, project, version, description, files):
        # note: need to make sure everything is pushed beforehand
        project.releases.create(dict(
            name=f'{version}',
            tag_name=version,
            description=description,
            ref='master',
            assets=dict(links=[
                dict(
                    name='Linux',
                    url=f'https://gitlab.com/xyder/nxcmdr/-/package_files/{files["linux"]["id"]}/download',
                    filepath='/bin/linux'
                ),
                dict(
                    name='Apple/MacOS',
                    url=f'https://gitlab.com/xyder/nxcmdr/-/package_files/{files["mac"]["id"]}/download',
                    filepath='/bin/apple-macosx'
                )
            ])
        ))

    @classmethod
    def get_gitlab_client(cls):
        client = gitlab.Gitlab(url=cls.BASE_GITLAB, private_token=os.environ['GITLAB_TOKEN'])
        client.auth()
        return client

    @classmethod
    def build_all(cls):
        cls._build_linux_release()
        cls._build_mac_release()

    @classmethod
    def archive_all(cls, version):
        cls._clean()
        cls._create_linux_archive(version)
        cls._create_mac_archive(version)

    @classmethod
    def create_release(cls, version, description):
        if not version or not description:
            fail('both --release-version and --description must be specified.')

        client = cls.get_gitlab_client()
        project = client.projects.get(os.environ['GITLAB_PROJECT_ID'])

        info('Uploading package files ..')
        files = cls._upload_package(project, version)

        info(f'Creating release {version}..')
        cls._create_release(
            project, version, description, files
        )

        info('Finished release creation ..')

    @classmethod
    def delete_release(cls, version):
        client = cls.get_gitlab_client()
        project = client.projects.get(os.environ['GITLAB_PROJECT_ID'])

        project.releases.delete(version)

        project.tags.delete(version)

        package = [p for p in project.packages.list() if p.version == version][0]
        project.packages.delete(package.id)

        info(f'Deleted release {version}')


    @classmethod
    def run_all(cls, version, description):
        cls.build_all()
        cls.archive_all(version)
        cls.create_release(version, description)


class NXCManager(cli.Application):
    def main(self, *args):
        if args:
            print(f'Unknown command {args[0]}')
            return 1
        if not self.nested_command:
            print('No command given.')
            return 1


@NXCManager.subcommand('build')
class Build(cli.Application):

    def main(self):
        # todo: maybe run in parallel
        BaseCommands.build_all()


@NXCManager.subcommand('archive')
class Archive(cli.Application):

    def main(self):
        BaseCommands.archive_all()


@NXCManager.subcommand('publish')
class Publish(cli.Application):
    release_version = cli.SwitchAttr('--release-version', str)
    description = cli.SwitchAttr('--description', str)

    def main(self):
        BaseCommands.create_release(self.release_version, self.description)


@NXCManager.subcommand('run-all')
class RunAll(cli.Application):
    release_version = cli.SwitchAttr('--release-version', str)
    description = cli.SwitchAttr('--description', str)

    def main(self):
        BaseCommands.run_all(self.release_version, self.description)


@NXCManager.subcommand('delete-release')
class RunAll(cli.Application):
    release_version = cli.SwitchAttr('--release-version', str)
    def main(self):
        BaseCommands.delete_release(self.release_version)


def main():
    # todo: impl wrapper for extract archive (for testing): `tar -xf archive_file -C dest_dir`
    NXCManager.run()


if __name__ == '__main__':
    main()
