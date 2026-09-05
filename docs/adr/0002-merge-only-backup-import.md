# Merge-only backup import

Nota supports local Backup export/import for the Flat Collection.

For the first backup flow, import is merge-only. A Merge Import adds Notes from the Backup and replaces same-identity Notes without destructively clearing the current Flat Collection. Replace import is out of scope for v1.

Before applying a Merge Import, Nota shows a Backup Import Preview that validates the selected Backup and reports how many Notes will be added or replaced. Selecting a Backup file is therefore not itself a mutating action.

This keeps Backup useful for recovery and browser migration while avoiding accidental loss of existing Notes. A destructive replace flow can be reconsidered later if a clear workflow needs it, but it should require an explicit confirmation that names the Flat Collection impact.
