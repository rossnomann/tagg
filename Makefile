.PHONY: build clean setup

BOOTSTRAP_URL = https://bootstrap.pypa.io/bootstrap-buildout.py

bootstrap-buildout.py:
	wget $(BOOTSTRAP_URL)

buildout: bootstrap-buildout.py
	python3 bootstrap-buildout.py

setup: buildout
	./buildout/bin/buildout

build: buildout
	./buildout/bin/pyinstaller -F tagg.py

clean:
	rm -f ./bootstrap-buildout.py
	rm -rf ./buildout
	rm -rf ./build
	rm -rf ./dist
	rm tagg.spec
