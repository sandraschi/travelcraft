; ==============================================================================
; TravelCraft — Production NSIS Current-User Installer
; SOTA Desktop Craft Standard: RequestExecutionLevel user (No Admin UAC required)
; Target install location: $LOCALAPPDATA\Programs\TravelCraft
; ==============================================================================

!define PRODUCT_NAME "TravelCraft"
!define PRODUCT_VERSION "0.2.0"
!define PRODUCT_PUBLISHER "Antigravity Craft Collective"
!define PRODUCT_WEB_SITE "https://github.com/sandraschi/travelcraft"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\travelcraft.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"

RequestExecutionLevel user
SetCompressor /SOLID lzma

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "..\target\travelcraft-v${PRODUCT_VERSION}-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\TravelCraft"
InstallDirRegKey HKCU "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show

Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  SetOverwrite ifnewer

  ; Primary binaries
  File /nonfatal "..\target\release\travelcraft.exe"
  File /nonfatal "..\target\release\travelcraft-cli.exe"
  File /nonfatal "..\target\release\travelcraft-mcp.exe"
  File /nonfatal "..\README.md"
  File /nonfatal "..\CHANGELOG.md"

  ; Start Menu Shortcuts
  CreateDirectory "$SMPROGRAMS\TravelCraft"
  CreateShortCut "$SMPROGRAMS\TravelCraft\TravelCraft Studio.lnk" "$INSTDIR\travelcraft.exe"
  CreateShortCut "$SMPROGRAMS\TravelCraft\TravelCraft CLI.lnk" "$INSTDIR\travelcraft-cli.exe"
  CreateShortCut "$SMPROGRAMS\TravelCraft\TravelCraft MCP Server.lnk" "$INSTDIR\travelcraft-mcp.exe"
  CreateShortCut "$SMPROGRAMS\TravelCraft\Uninstall TravelCraft.lnk" "$INSTDIR\uninst.exe"

  ; Write Uninstaller
  WriteUninstaller "$INSTDIR\uninst.exe"

  ; Registry App Path & Uninstall Entry (HKCU - No elevation required)
  WriteRegStr HKCU "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\travelcraft.exe"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninst.exe"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\travelcraft.exe"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegDWORD HKCU "${PRODUCT_UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${PRODUCT_UNINST_KEY}" "NoRepair" 1
SectionEnd

Section Uninstall
  ; Remove shortcuts
  Delete "$SMPROGRAMS\TravelCraft\TravelCraft Studio.lnk"
  Delete "$SMPROGRAMS\TravelCraft\TravelCraft CLI.lnk"
  Delete "$SMPROGRAMS\TravelCraft\TravelCraft MCP Server.lnk"
  Delete "$SMPROGRAMS\TravelCraft\Uninstall TravelCraft.lnk"
  RMDir "$SMPROGRAMS\TravelCraft"

  ; Remove binaries & files
  Delete "$INSTDIR\travelcraft.exe"
  Delete "$INSTDIR\travelcraft-cli.exe"
  Delete "$INSTDIR\travelcraft-mcp.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\CHANGELOG.md"
  Delete "$INSTDIR\uninst.exe"
  RMDir "$INSTDIR"

  ; Remove registry keys
  DeleteRegKey HKCU "${PRODUCT_UNINST_KEY}"
  DeleteRegKey HKCU "${PRODUCT_DIR_REGKEY}"
  SetAutoClose true
SectionEnd
