# MavLang VS Code — установка

1. Скопируй папку `vscode-mavlang` в расширения:
  - Windows: `%USERPROFILE%\.vscode\extensions\mavlang-0.1.0\`
2. Перезапусти VS Code, открой `source/main.mavl` — должна быть подсветка.
3. Проверка JSON: файлы `package.json` и `syntaxes/mavlang.tmGrammar.json` должны быть валидным JSON.

Упаковка в .vsix (если нужен файл):
```powershell
npm i -g @vscode/vsce
vsce package --out mavlang-0.1.0.vsix
```
