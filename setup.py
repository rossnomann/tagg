from setuptools import setup

version = None

with open('tagg.py') as f:
    for line in f:
        if line.startswith('__version__'):
            version = line.split(' ')[-1].strip("'\n")
            break

if not version:
    raise ValueError('Unable to get version')

setup(
    name='tagg',
    version=version,
    description='A tool to handle my mp3 collection',
    author='Ross Nomann',
    author_email='rossnomann@protonmail.com',
    url='https://github.com/rossnomann/tagg',
    license='MIT',
    py_modules=['tagg'],
    install_requires=[
        'Pygments',
        'mutagen',
        'prompt_toolkit'
    ],
    entry_points={
        'console_scripts': [
            'tagg=tagg:main'
        ]
    }
)
