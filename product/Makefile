PYTHON ?= python3

.PHONY: status doctor config smoke

status:
	$(PYTHON) cli/huggingos.py status

doctor:
	$(PYTHON) cli/huggingos.py doctor

config:
	$(PYTHON) cli/huggingos.py config

smoke:
	$(PYTHON) -m unittest discover -s tests -p "test_*.py"
