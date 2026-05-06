# Merge-only backup import

Noter supports local Backup export/import for the Flat Collection.

For the first backup flow, import is merge-only. A Merge Import adds Notes from the Backup and replaces same-identity Notes without destructively clearing the current Flat Collection. Replace import is out of scope for v1.

This keeps Backup useful for recovery and browser migration while avoiding accidental loss of existing Notes. A destructive replace flow can be reconsidered later if a clear workflow needs it, but it should require an explicit confirmation that names the Flat Collection impact.
