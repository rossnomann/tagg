from setuptools import find_packages, setup

setup(
    name='tagg',
    version='0.1.0',
    description='A tool to handle your mp3 collection',
    author='Ross Nomann',
    author_email='rossnomann@protonmail.com',
    url='https://github.com/rossnomann/tagg',
    packages=find_packages('source'),
    package_dir={'': 'source'},
    license='MIT',
    install_requires=[],
    entry_points={
        'console_scripts': [
            'tagg=tagg:run'
        ]
    }
)
