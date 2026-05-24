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
- Unexplained flags are bad for a cli tool. Because it requires knowledge about how to query/use them.
- instead different flags/etc are more suitable. Because these will get directly exposed by the cli iteself. And explained by it itself. While query always needs additional knowledge, that might not be in the context and hard to find. So stick to a list of flags or other elements that are directly exposed in the cli itself.
- Following is some help commands where you can see that it is currently impossible to self discover what  --notes-root <NOTES_ROOT>  does and how to use it
```
vincep@DESKTOP-MH64D1J:~/aitoolbox$ knowledge-cli query
error: the following required arguments were not provided:
  --db <DB>
  --notes-root <NOTES_ROOT>
  <QUERY>

Usage: knowledge-cli query --db <DB> --notes-root <NOTES_ROOT> <QUERY>

For more information, try '--help'.
vincep@DESKTOP-MH64D1J:~/aitoolbox$ knowledge-cli --help
Usage: knowledge-cli <COMMAND>

Commands:
  init     
  query    
  capture  
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
vincep@DESKTOP-MH64D1J:~/aitoolbox$ knowledge-cli query --help
Usage: knowledge-cli query --db <DB> --notes-root <NOTES_ROOT> <QUERY>

Arguments:
  <QUERY>  

Options:
      --db <DB>                  
      --notes-root <NOTES_ROOT>  
  -h, --help                     Print help
vincep@DESKTOP-MH64D1J:~/aitoolbox$ 
```

## pipeline to build releases via github actions
- I want tagged versioned releases of the cli. (For ubuntu aswell as mac)

