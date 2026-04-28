Unicode true

!include "MUI2.nsh"

!ifndef APP_VERSION
  !define APP_VERSION "0.1.0"
!endif

!ifndef PRODUCT_VERSION
  !define PRODUCT_VERSION "0.1.0.0"
!endif

!ifndef APP_EXE_PATH
  !define APP_EXE_PATH "target\release\apex_footwork.exe"
!endif

!ifndef APP_ICON_PATH
  !define APP_ICON_PATH "apex-footwork.ico"
!endif

!ifndef OUT_FILE
  !define OUT_FILE "dist\ApexFootwork-${APP_VERSION}-setup.exe"
!endif

!define APP_NAME "Apex Footwork"
!define APP_PUBLISHER "Apex Footwork"
!define APP_EXE_NAME "apex_footwork.exe"
!define START_MENU_DIR "Apex Footwork"
!define UNINSTALL_REG_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\ApexFootwork"

Name "${APP_NAME}"
OutFile "${OUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\ApexFootwork"
InstallDirRegKey HKCU "${UNINSTALL_REG_KEY}" "InstallLocation"
RequestExecutionLevel user
BrandingText "${APP_NAME}"
Icon "${APP_ICON_PATH}"
UninstallIcon "${APP_ICON_PATH}"

VIProductVersion "${PRODUCT_VERSION}"
VIAddVersionKey "ProductName" "${APP_NAME}"
VIAddVersionKey "CompanyName" "${APP_PUBLISHER}"
VIAddVersionKey "FileDescription" "${APP_NAME} Setup"
VIAddVersionKey "FileVersion" "${APP_VERSION}"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"
VIAddVersionKey "LegalCopyright" "Copyright ${APP_PUBLISHER}"

!define MUI_ABORTWARNING
!define MUI_ICON "${APP_ICON_PATH}"
!define MUI_UNICON "${APP_ICON_PATH}"
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE_NAME}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetShellVarContext current
  SetOutPath "$INSTDIR"

  File /oname=${APP_EXE_NAME} "${APP_EXE_PATH}"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateDirectory "$SMPROGRAMS\${START_MENU_DIR}"
  CreateShortCut "$SMPROGRAMS\${START_MENU_DIR}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE_NAME}" "" "$INSTDIR\${APP_EXE_NAME}" 0
  CreateShortCut "$SMPROGRAMS\${START_MENU_DIR}\Uninstall ${APP_NAME}.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0

  WriteRegStr HKCU "${UNINSTALL_REG_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINSTALL_REG_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINSTALL_REG_KEY}" "Publisher" "${APP_PUBLISHER}"
  WriteRegStr HKCU "${UNINSTALL_REG_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_REG_KEY}" "DisplayIcon" "$INSTDIR\${APP_EXE_NAME},0"
  WriteRegStr HKCU "${UNINSTALL_REG_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKCU "${UNINSTALL_REG_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_REG_KEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  SetShellVarContext current

  Delete "$SMPROGRAMS\${START_MENU_DIR}\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${START_MENU_DIR}\Uninstall ${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${START_MENU_DIR}"

  Delete "$INSTDIR\${APP_EXE_NAME}"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "${UNINSTALL_REG_KEY}"
SectionEnd
