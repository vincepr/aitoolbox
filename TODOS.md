# Todos/Features missing
## Streamline the default's
`knowledge-cli init --db .local/knowledge.db --source config/knowledge/sources.example.json`
`knowledge-cli query MyCompanyName.Ebay.Custom.Client --db .local/knowledge.db --notes-root knowledge/notes`

It should not be required to add all these params. There should be sensible defaults
- So the default usage is streamlined
- init should also create missing default files if missing (again with sensible defaults) (MyCompanyName.Ebay.Custom.Client is no sensbile default! Its just a example)

## Streamline default usage
- knowledge-cli query MyCompanyName.Ebay.Custom.Client --db .local/knowledge.db --notes-root knowledge/notes

This is quite cluncy usage. I want to keep the most used clownledge-cli commands simple and fast. Also lets shorten knowledge-cli with suitable alias (that is not already commonly taken in linux/windows/mac).
- knowledge-cli query MyCompanyName.Ebay.Custom.Client -> this returns sensible defaults already

## Rework explorability, imporove self discoverability
- Current status: improved. `query` was replaced by `get`, defaults now reduce required flags, and help output has command/flag descriptions.
- Remaining gap: add shell completions and command aliases so discoverability extends beyond `--help`.
- Remaining gap: add a `quickstart` command to scaffold `.local/knowledge.sqlite3`, `knowledge/notes`, and example source file.
- Remaining gap: improve error guidance when users provide partial argument sets (show one-line fix examples directly in errors).

## pipeline to build releases via github actions
- I want tagged versioned releases of the cli. (For ubuntu aswell as mac)
