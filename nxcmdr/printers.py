from plumbum import colors


def print_error(message: str, check_doc=False):
    if check_doc:
        message = f'{message} Run with -h/--help for usage information.'
    print(colors.red | f'ERROR: {message}')

def print_warning(message: str):
    print(colors.red | f'WARNING: {message}')

def print_info(message: str):
    print(colors.green | f'INFO: {message}')
