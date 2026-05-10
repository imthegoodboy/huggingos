PYTHON ?= python3

.PHONY: status doctor config capabilities run-status agent-smoke smoke

status:
	$(PYTHON) cli/huggingos.py status

doctor:
	$(PYTHON) cli/huggingos.py doctor

config:
	$(PYTHON) cli/huggingos.py config

capabilities:
	$(PYTHON) cli/huggingos.py capabilities

run-status:
	$(PYTHON) cli/huggingos.py run product.status

agent-smoke:
	cd agent && cargo test

smoke:
	$(PYTHON) -m unittest discover -s tests -p "test_*.py"
