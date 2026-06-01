from __future__ import annotations

from track_sources.manifest import (
    implementation_writer_allowed_scopes,
    implementation_writer_forbidden_scopes,
    implementation_writer_output_scopes,
    is_generated_or_derived_scope,
    manifest_write_scope_path,
    mentions_generated_or_derived_scope,
    new_scope_is_marked,
    scope_is_covered_by_wr,
)

__all__ = [
    "implementation_writer_allowed_scopes",
    "implementation_writer_forbidden_scopes",
    "implementation_writer_output_scopes",
    "is_generated_or_derived_scope",
    "manifest_write_scope_path",
    "mentions_generated_or_derived_scope",
    "new_scope_is_marked",
    "scope_is_covered_by_wr",
]
