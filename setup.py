from setuptools import setup

setup(
    name='tagg',
    version='0.1.0',
    description='A tool to handle my mp3 collection',
    author='Ross Nomann',
    author_email='rossnomann@protonmail.com',
    url='https://github.com/rossnomann/tagg',
    license='MIT',
    py_modules=['tagg'],
    install_requires=[
    ],
    entry_points={
        'console_scripts': [
            'tagg=tagg:main'
        ]
    }
)
