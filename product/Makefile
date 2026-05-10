PYTHON ?= python3

.PHONY: status doctor config capabilities run-status agent-ai-status agent-ai-plan agent-ai-run agent-secrets agent-desktop-status agent-apps-list agent-browser-dry-run agent-workspace-plan agent-screen-status agent-screen-capture-dry-run agent-context-snapshot agent-ocr-dry-run agent-smoke smoke

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

agent-desktop-status:
	cd agent && cargo run -- run desktop.status --json

agent-apps-list:
	cd agent && cargo run -- run apps.list --json

agent-browser-dry-run:
	cd agent && cargo run -- run browser.open_url --param url=https://example.com --dry-run --json

agent-workspace-plan:
	cd agent && cargo run -- run workspace.mode.plan --param mode=coding --json

agent-screen-status:
	cd agent && cargo run -- run screen.status --json

agent-screen-capture-dry-run:
	cd agent && cargo run -- run screen.capture --dry-run --json

agent-context-snapshot:
	cd agent && cargo run -- run context.snapshot --confirm --json

agent-ocr-dry-run:
	cd agent && cargo run -- run screen.ocr_image --param path=../../README.md --dry-run --json

agent-smoke:
	cd agent && cargo test

smoke:
	$(PYTHON) -m unittest discover -s tests -p "test_*.py"
