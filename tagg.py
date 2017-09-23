import argparse
import collections
import os
import shutil
import sys
import traceback

from mutagen import apev2, id3, mp3
from prompt_toolkit import prompt
from prompt_toolkit.auto_suggest import AutoSuggestFromHistory
from prompt_toolkit.contrib.completers import WordCompleter
from prompt_toolkit.history import InMemoryHistory
from prompt_toolkit.shortcuts import print_tokens
from prompt_toolkit.styles import style_from_dict
from prompt_toolkit.validation import Validator, ValidationError
from pygments.token import Token


__version__ = '0.1.0'


ID3_FRAMES_MAP = {
    'artist': 'TPE1',
    'album_artist': 'TPE2',
    'album': 'TALB',
    'year': 'TDRC',
    'title': 'TIT2',
    'track': 'TRCK',
    'disc': 'TPOS'
}

ID3_FRAME_ENCODING = 3  # UTF-8
ID3_V2_VERSION = 4  # ID3v2.4


def find_files(path):
    result = []
    for filename in os.listdir(path):
        _, ext = os.path.splitext(filename)
        if ext.lower() != '.mp3':
            continue
        filepath = os.path.join(path, filename)
        result.append(filepath)
    return result


class ID3Reader(object):
    def __init__(self, path):
        try:
            tags = id3.ID3(path)
        except id3.ID3NoHeaderError:
            tags = None
        self.path = path
        self.__tags = tags

    def __getitem__(self, item):
        if not self.__tags:
            result = None
        else:
            # Handle track_number, disc_number, total_tracks, total_discs
            if item.endswith('number'):
                item, item_type = item.split('_')
            elif item.startswith('total'):
                item_type, item = item.split('_')
                item = item[:-1]
            else:
                item_type = None

            frame_name = ID3_FRAMES_MAP[item]
            try:
                result = str(self.__tags[frame_name])
            except KeyError:
                result = ''

            if result and item_type:
                parts = result.split('/')
                if len(parts) == 2:
                    if item_type == 'number':  # read track/disc number
                        result = parts[0]
                    elif item_type == 'total':  # read total tracks/discs
                        result = parts[1]
                else:
                    if item_type == 'total':
                        result = ''
                    # else: result already contains necessary value
        return result


class DefaultOrderedDict(collections.OrderedDict):
    """A minimal defaultdict + OrderedDict"""
    def __init__(self, factory, *args, **kwargs):
        self.__factory = factory
        super().__init__(*args, **kwargs)

    def __getitem__(self, item):
        try:
            result = super().__getitem__(item)
        except KeyError:
            result = self.__factory()
            self[item] = result
        return result


class Counter(object):
    """
    A dict-like object that yields most common value for each key

    ```python
    c = Counter()
    for _ in range(3):
        c['k'] = 'v1'
    for _ in range(5):
        c['k'] = 'v2'
    print(c['k'])  # v1

    # Iter:

    for k, v in c:
        print(k, v)

    # Convert to real dict:
    result = dict(c)
    ```
    """
    def __init__(self):
        self.__data = DefaultOrderedDict(collections.Counter)

    def __setitem__(self, key, value):
        self.__data[key][value] += 1

    def __getitem__(self, key):
        vals = self.__data[key].most_common(1)
        if vals:
            return vals[0][0]

    def __iter__(self):
        for key in self.__data:
            yield key, self[key]


def get_common_tags(tags):
    """Returns common tags for an album"""
    keys = ['artist', 'album_artist', 'album', 'year', 'total_tracks', 'total_discs']
    counter = Counter()
    for item in tags:
        for key in keys:
            counter[key] = item[key]
    return collections.OrderedDict(counter)


def get_items_tags(tags):
    """Returns unique tags for each track"""
    keys = ['title', 'track_number', 'disc_number']
    return {
        item.path: collections.OrderedDict(
            (key, item[key])
            for key in keys
        )
        for item in tags
    }


class RequiredValidator(Validator):
    def validate(self, document):
        if not document.text:
            raise ValidationError(message='Value is required')


class DigitValidator(RequiredValidator):
    def validate(self, document):
        super().validate(document)
        if not document.text.isdigit():
            raise ValidationError(message='Accepts numeric characters only')


class CLI(object):
    def __init__(self):
        self.style = style_from_dict({
            Token.Arrows: '#498f0c',
            Token.ConfirmMessage: '#c47566',
            Token.ConfirmValues: '#a35445',
            Token.Error: '#ff4444',
            Token.Label: '#69cf4c bold',
            Token.Success: '#69cf4c',
            Token.Value: '#ffffff'
        })
        self.__tags_history = InMemoryHistory()
        self.__tags_suggest = AutoSuggestFromHistory()

    def __prompt(self, *args, **kwargs):
        try:
            return prompt(*args, **kwargs)
        except EOFError:
            self.print_tokens((Token.Error, 'Interrupted'))
            sys.exit(1)

    def set_tags_completions(self, common_tags, items_tags):
        words = []
        for value in common_tags.values():
            words += value.split(' ')
        for item in items_tags.values():
            for value in item.values():
                words += value.split(' ')
        self.__tags_completer = WordCompleter(set(words))

    def edit_tag(self, tag_name, default_value):
        def get_prompt_tokens(cli):
            return [
                (Token.Label, tag_name.upper().replace('_', ' ')),
                (Token.Space, ' '),
                (Token.Arrows, '>>>'),
                (Token.Space, ' ')
            ]

        if 'number' in tag_name or 'total' in tag_name:
            validator = DigitValidator()
        else:
            validator = RequiredValidator()

        return self.__prompt(
            auto_suggest=self.__tags_suggest,
            completer=self.__tags_completer,
            default=default_value,
            get_prompt_tokens=get_prompt_tokens,
            history=self.__tags_history,
            style=self.style,
            validator=validator
        )

    def edit_tags(self, data):
        self.print_line()
        return {key: self.edit_tag(key, value) for key, value in data.items()}

    def ask_confirm(self, message):
        def get_prompt_tokens(cli):
            return [
                (Token.ConfirmMessage, message),
                (Token.Space, ' '),
                (Token.ConfirmValues, '[y/n]'),
                (Token.Space, ' ')
            ]

        while True:
            answer = prompt(get_prompt_tokens=get_prompt_tokens, style=self.style)
            if answer == 'y':
                return True
            if answer == 'n':
                return False

    def print(self, *tokens):
        print_tokens(tokens, style=self.style)

    def println(self, *tokens):
        self.print(*tokens)
        self.print_line()

    @staticmethod
    def print_line():
        print('')


def copy_files(dest, common_tags, items_tags):
    dest = os.path.join(
        dest,
        common_tags['artist'],
        '{} - {}'.format(
            common_tags['year'],
            common_tags['album']
        )
    )
    os.makedirs(dest)

    result = {}
    for src_path, tags in items_tags.items():
        if int(common_tags['total_discs']) > 1:
            number = '{}-{}'.format(tags['disc_number'], tags['track_number'])
        else:
            number = tags['track_number']
        dest_path = os.path.join(dest, '{} - {}.mp3'.format(number, tags['title']))
        shutil.copy(src_path, dest_path)
        result[dest_path] = tags

    return result


def write_tags(cli, common_tags, items_tags):
    for path, tags in items_tags.items():
        cli.print((Token.Text, 'Writing tags for {} ... '.format(path)))
        try:
            ape_tags = apev2.APEv2(path)
        except apev2.APENoHeaderError:
            pass
        else:
            ape_tags.delete()

        try:
            id3_tags = id3.ID3(path)
        except id3.ID3NoHeaderError:
            track = mp3.MP3(path)
            track.add_tags()
            id3_tags = track.tags
        else:
            id3_tags.delete(filename=path, delete_v1=True, delete_v2=True)

        for key, frame_name in ID3_FRAMES_MAP.items():
            frame_factory = getattr(id3, frame_name)
            if key in tags:
                value = tags[key]
            elif key in common_tags:
                value = common_tags[key]
            else:
                assert key in ['disc', 'track'], 'Unexpected key: {}'.format(key)
                pos_key, total_key = '{}_number'.format(key), 'total_{}s'.format(key)
                value = '{}/{}'.format(tags[pos_key], common_tags[total_key])
            id3_tags[frame_name] = frame_factory(text=value, encoding=ID3_FRAME_ENCODING)

        try:
            id3_tags.save(filename=path, v1=id3.ID3v1SaveOptions.REMOVE, v2_version=ID3_V2_VERSION)
        except Exception:
            cli.println((Token.Error, 'FAILED'))
            traceback.print_exc()
            sys.exit(1)
        else:
            cli.println((Token.Success, 'OK'))


def run(source, dest):
    """
    :param str source: Path to source directory
    :param str dest: Path to destination directory
    """
    cli = CLI()

    if not os.path.isdir(source):
        cli.println((Token.Error, 'No such directory: {0}'.format(source)))
        return 1

    if os.path.exists(dest):
        cli.println((Token.Error, '{0} already exists'.format(dest)))
        return 1

    files = find_files(source)
    if not files:
        cli.println((Token.Error, 'There are no mp3 files in {0}'.format(source)))
        return 1

    cli.println((Token.Label, 'Source:'), (Token.Space, ' '), (Token.Value, source))
    cli.println((Token.Label, 'Destination:'), (Token.Space, ' '), (Token.Value, dest))

    tags = [ID3Reader(path) for path in files]
    common_tags = get_common_tags(tags)
    items_tags = get_items_tags(tags)

    cli.set_tags_completions(common_tags, items_tags)
    common_tags = cli.edit_tags(common_tags)
    items_tags = {path: cli.edit_tags(tags) for path, tags in items_tags.items()}

    cli.print_line()
    if not cli.ask_confirm('Continue?'):
        cli.println((Token.Error, 'Cancelled!'))
        return 0

    cli.print_line()
    items_tags = copy_files(dest, common_tags, items_tags)

    cli.print_line()
    write_tags(cli, common_tags, items_tags)

    cli.print_line()
    cli.println((Token.Success, 'Done!'))
    return 0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-s', '--source', help='Source directory', default='.')
    parser.add_argument('-d', '--dest', help='Destination directory', default='res')
    parser.add_argument('--version', action='version', version='%(prog)s {}'.format(__version__))
    args = parser.parse_args()

    source = os.path.abspath(args.source)
    dest = os.path.abspath(args.dest)

    try:
        exitcode = run(source, dest)
    except KeyboardInterrupt:
        print('^C')
        exitcode = 0

    sys.exit(exitcode)


if __name__ == '__main__':
    main()
