PYTHON ?= python3

.PHONY: status doctor config capabilities run-status agent-ai-status agent-ai-plan agent-ai-run agent-secrets agent-desktop-status agent-apps-list agent-browser-dry-run agent-workspace-plan agent-screen-status agent-screen-capture-dry-run agent-context-snapshot agent-ocr-dry-run agent-memory-remember agent-memory-list agent-preference-set agent-semantic-index agent-semantic-search agent-resume-plan agent-agents-catalog agent-agents-plan agent-agents-orchestrate agent-agents-trace-list agent-workflow-detect agent-proactive-suggest agent-selfheal-diagnose agent-timeline-explain agent-smoke smoke

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

agent-memory-remember:
	cd agent && cargo run -- run memory.session.remember --param key=current-goal --param value=phase-six-seven --json

agent-memory-list:
	cd agent && cargo run -- run memory.session.list --json

agent-preference-set:
	cd agent && cargo run -- run memory.preference.set --param key=theme --param value=dark --json

agent-semantic-index:
	cd agent && cargo run -- run files.semantic.index --param root=../../docs --confirm --json

agent-semantic-search:
	cd agent && cargo run -- run files.semantic.search --param query=capability --json

agent-resume-plan:
	cd agent && cargo run -- run workspace.resume.plan --json

agent-agents-catalog:
	cd agent && cargo run -- run agents.catalog --json

agent-agents-plan:
	cd agent && cargo run -- run agents.plan --param "goal=daily brief" --json

agent-agents-orchestrate:
	cd agent && cargo run -- run agents.orchestrate --param "goal=daily brief" --confirm --json

agent-agents-trace-list:
	cd agent && cargo run -- run agents.trace.list --json

agent-workflow-detect:
	cd agent && cargo run -- run proactive.workflow.detect --json

agent-proactive-suggest:
	cd agent && cargo run -- run proactive.suggest --json

agent-selfheal-diagnose:
	cd agent && cargo run -- run selfheal.diagnose --param symptom=app_crashed --param target=editor --param simulated=true --json

agent-timeline-explain:
	cd agent && cargo run -- run timeline.explain --json

agent-smoke:
	cd agent && cargo test

smoke:
	$(PYTHON) -m unittest discover -s tests -p "test_*.py"
