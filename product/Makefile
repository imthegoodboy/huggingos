PYTHON ?= python3

.PHONY: status doctor config capabilities run-status agent-ai-status agent-ai-plan agent-ai-run agent-secrets agent-smoke smoke

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

agent-ai-status:
	cd agent && cargo run -- ai status --json

agent-ai-plan:
	cd agent && cargo run -- ai plan "show product status" --json

agent-ai-run:
	cd agent && cargo run -- ai run "show product status" --json

agent-secrets:
	cd agent && cargo run -- secrets status --json

agent-smoke:
	cd agent && cargo test

smoke:
	$(PYTHON) -m unittest discover -s tests -p "test_*.py"
