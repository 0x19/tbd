# Thin shim: every target delegates to the mise task of the same name.
# `make` lists tasks; `make ci`, `make run:engine`, `make ship`, ...
.PHONY: help
help:
	@mise tasks

%:
	@mise run $@
