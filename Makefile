VERSION = awk '/__version__ = .+/{print substr(substr($$3, 2), 0, length($$3) - 2)}' tagg.py
TARGET_TRIPLET = gcc -dumpmachine

.PHONY: build clean setup

BOOTSTRAP_URL = https://bootstrap.pypa.io/bootstrap-buildout.py

bootstrap-buildout.py:
	wget $(BOOTSTRAP_URL)

buildout: bootstrap-buildout.py
	python3 bootstrap-buildout.py

setup: buildout
	./buildout/bin/buildout

build: buildout
	rm -rf ./build ./dist tagg.spec
	./buildout/bin/pyinstaller -F tagg.py
	cp dist/tagg dist/tagg-$(shell $(VERSION))_$(shell $(TARGET_TRIPLET))

clean:
	rm -f ./bootstrap-buildout.py
	rm -rf ./buildout
	rm -rf ./build
	rm -rf ./dist
	rm tagg.spec
