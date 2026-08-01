; LiteMark NSIS hooks — use a solid document icon for Markdown file associations
; so Explorer does not show the transparent "hollow LM" app icon over the wallpaper.

!macro NSIS_HOOK_POSTINSTALL
  ; After APP_ASSOCIATE registers DefaultIcon as "$INSTDIR\litemark.exe,0",
  ; point the file class at our opaque multi-size document icon instead.
  ; Multi-size ICO path (no ",0") so Explorer picks correct resolution.
  WriteRegStr SHELL_CONTEXT "Software\Classes\Markdown\DefaultIcon" "" "$INSTDIR\resources\markdown-file.ico"
  !insertmacro UPDATEFILEASSOC
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Class key is removed by APP_UNASSOCIATE; just refresh the shell.
  !insertmacro UPDATEFILEASSOC
!macroend
