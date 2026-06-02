from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *  # noqa: F403
from truth.conformance.spec import ConformanceSpec
from truth.conformance.ui_program_architecture import verify_semantic_checks


def test_semantic_check_requires_behavior_probe_binding() -> None:
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "spec",
            "claim_ids": ["claim"],
            "final_owner_dirs": [],
            "required_files": [],
            "semantic_checks": [
                {
                    "check_id": "graph_family_semantic_contracts",
                    "description": "Symbol presence alone must not satisfy semantic truth.",
                    "subject_paths": ["domain/ui/ui_program/src/graphs/control.rs"],
                    "evidence_kinds": [],
                    "required_symbols": ["ControlGraph"],
                    "required_validation_fragments": [],
                }
            ],
        }
    )

    findings, _checks = verify_semantic_checks(spec, repo_root=REPO_ROOT)  # noqa: F405

    assert any("declares no behavior_probe_ids" in finding.message for finding in findings)


def test_semantic_check_behavior_probe_binding_accepts_existing_probe() -> None:
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "spec",
            "claim_ids": ["claim"],
            "final_owner_dirs": [],
            "required_files": [],
            "semantic_checks": [
                {
                    "check_id": "route_event_schema_payload_validation",
                    "description": "Route payload semantics are bound to executable tests.",
                    "subject_paths": [
                        "domain/ui/ui_program/src/events/packet.rs",
                        "domain/ui/ui_program/src/events/payload.rs",
                    ],
                    "behavior_probe_paths": ["domain/ui/ui_program/src/lib.rs"],
                    "behavior_probe_ids": [
                        "route_contract_uses_stable_ids_and_schema_payloads",
                        "route_contract_reports_invalid_payload_schema",
                    ],
                    "evidence_kinds": [],
                    "required_symbols": ["UiEventPacket", "UiEventPayload"],
                    "required_validation_fragments": [],
                }
            ],
        }
    )

    findings, _checks = verify_semantic_checks(spec, repo_root=REPO_ROOT)  # noqa: F405

    assert not any("behavior probe" in finding.message for finding in findings)
