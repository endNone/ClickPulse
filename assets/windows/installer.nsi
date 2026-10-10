; Per-user setup. Program files only; user statistics are never an uninstall target.
Unicode true
RequestExecutionLevel user
SetCompressor /SOLID lzma
Name "ClickPulse"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\ClickPulse"
InstallDirRegKey HKCU "Software\ClickPulse" "InstallDir"
Icon "${ICON_PATH}"
UninstallIcon "${ICON_PATH}"
VIProductVersion "${NUMERIC_VERSION}"
VIAddVersionKey /LANG=1033 "ProductName" "ClickPulse"
VIAddVersionKey /LANG=1033 "ProductVersion" "${PUBLIC_VERSION}"
VIAddVersionKey /LANG=1033 "FileVersion" "${FILE_VERSION}"
VIAddVersionKey /LANG=1033 "FileDescription" "ClickPulse Setup"
VIAddVersionKey /LANG=1033 "LegalCopyright" "ClickPulse"

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "WinVer.nsh"
!include "x64.nsh"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\ClickPulse"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_TEXT "$(InstallHint)"
!define MUI_UNFINISHPAGE_TEXT "$(UninstallHint)"
!define MUI_LANGDLL_REGISTRY_ROOT HKCU
!define MUI_LANGDLL_REGISTRY_KEY "${UNINSTALL_KEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "InstallerLanguage"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"

LangString InstallHint ${LANG_ENGLISH} "Open ClickPulse from the Start menu.$\r$\n$\r$\nIf WebView2 Runtime is missing, install it from Microsoft's official website (see README.txt).$\r$\n$\r$\nHistory and settings are kept separately in your local ClickPulse data folder."
LangString InstallHint ${LANG_SIMPCHINESE} "从开始菜单打开 ClickPulse。$\r$\n$\r$\n若提示缺少 WebView2 Runtime，按 README.txt 的微软官方链接安装运行时。$\r$\n$\r$\n历史与设置单独保存在本机 ClickPulse 数据目录。"
LangString UninstallHint ${LANG_ENGLISH} "Program files were removed. Your recorded history and preferences were kept."
LangString UninstallHint ${LANG_SIMPCHINESE} "程序文件已移除，统计历史与偏好设置已保留。"
LangString StopApp ${LANG_ENGLISH} "ClickPulse is running or its files are locked. Quit ClickPulse from the system tray, then retry. Your history will be kept."
LangString StopApp ${LANG_SIMPCHINESE} "ClickPulse 正在运行或文件被占用。请从系统托盘退出 ClickPulse，再重试。统计历史会保留。"
LangString PlatformError ${LANG_ENGLISH} "This package requires Windows 10/11 with x64 application support."
LangString PlatformError ${LANG_SIMPCHINESE} "此安装包需要支持 x64 应用的 Windows 10/11。"
LangString DataDirectoryError ${LANG_ENGLISH} "Choose a program folder outside the ClickPulse statistics directory."
LangString DataDirectoryError ${LANG_SIMPCHINESE} "请选择统计数据目录以外的程序安装位置。"
LangString ExistingFilesError ${LANG_ENGLISH} "Choose an empty folder or an existing ClickPulse program folder. This folder contains unrelated files."
LangString ExistingFilesError ${LANG_SIMPCHINESE} "此目录包含其他文件，请选择空文件夹或现有的 ClickPulse 程序目录。"
LangString SetupRunning ${LANG_ENGLISH} "Another ClickPulse installation or uninstall is already in progress. Finish it before trying again."
LangString SetupRunning ${LANG_SIMPCHINESE} "另一个 ClickPulse 安装或卸载正在进行，请先完成后再试。"

!macro AcquireSetupGuard
  System::Call 'kernel32::CreateMutexW(p 0, i 0, w "Local\ClickPulse.Setup") p.r0 ?e'
  Pop $1
  StrCmp $1 183 0 +3
    MessageBox MB_OK|MB_ICONSTOP "$(SetupRunning)"
    Abort
!macroend

Function .onInit
  SetShellVarContext current
  !insertmacro MUI_LANGDLL_DISPLAY
  !insertmacro AcquireSetupGuard
  ${IfNot} ${AtLeastWin10}
    MessageBox MB_OK|MB_ICONSTOP "$(PlatformError)"
    Abort
  ${EndIf}
  ${IfNot} ${RunningX64}
    MessageBox MB_OK|MB_ICONSTOP "$(PlatformError)"
    Abort
  ${EndIf}
FunctionEnd

Function .onVerifyInstDir
  GetFullPathName $0 "$INSTDIR"
  check_parent:
    StrCmp $0 "$LOCALAPPDATA\ClickPulse" rejected_directory
    ${GetParent} "$0" $1
    StrCmp $1 "" directory_ok
    StrCmp $0 $1 directory_ok
    StrCpy $0 $1
    Goto check_parent
  rejected_directory:
    Abort
  directory_ok:
FunctionEnd

!macro EnsureStopped Prefix
Function ${Prefix}EnsureStopped
  check_running:
    System::Call 'kernel32::OpenMutexW(i 0x100000, i 0, w "Local\ClickPulse.Running") p.r1'
    StrCmp $1 0 check_legacy_window
      System::Call 'kernel32::CloseHandle(p r1)'
      Goto app_running
  check_legacy_window:
    ; Older preview builds have no running marker; include their hidden windows.
    System::Call 'user32::FindWindowW(p 0, w "ClickPulse · 键鼠活动统计") p.r1'
    StrCmp $1 0 +2
      Goto app_running
    System::Call 'user32::FindWindowW(p 0, w "ClickPulse · Input statistics") p.r1'
    StrCmp $1 0 +2
      Goto app_running
    System::Call 'user32::FindWindowW(p 0, w "ClickPulse · 小屏") p.r1'
    StrCmp $1 0 +2
      Goto app_running
    System::Call 'user32::FindWindowW(p 0, w "ClickPulse · Mini") p.r1'
    StrCmp $1 0 inspect_file
  app_running:
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "$(StopApp)" IDRETRY check_running
    Abort
  inspect_file:
  IfFileExists "$INSTDIR\ClickPulse.exe" 0 stopped
  retry_stop:
    ; A loaded Windows EXE cannot be opened for writing. No write is performed.
    System::Call 'kernel32::CreateFileW(w "$INSTDIR\ClickPulse.exe", i 0x40000000, i 1, p 0, i 3, i 0, p 0) p.r0'
    StrCmp $0 -1 locked unlocked
  locked:
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "$(StopApp)" IDRETRY retry_stop
    Abort
  unlocked:
    System::Call 'kernel32::CloseHandle(p r0)'
  stopped:
FunctionEnd
!macroend
!insertmacro EnsureStopped ""
!insertmacro EnsureStopped "un."

Function CheckInstallDirectory
  ; Recheck even in silent mode: installation must not target a data folder.
  GetFullPathName $INSTDIR "$INSTDIR"
  StrCpy $0 "$INSTDIR"
  verify_parent:
    StrCmp $0 "$LOCALAPPDATA\ClickPulse" data_directory
    ${GetParent} "$0" $1
    StrCmp $1 "" check_files
    StrCmp $0 $1 check_files
    StrCpy $0 $1
    Goto verify_parent
  data_directory:
    MessageBox MB_OK|MB_ICONSTOP "$(DataDirectoryError)"
    Abort
  check_files:
    IfFileExists "$INSTDIR\ClickPulse.exe" done
    FindFirst $0 $1 "$INSTDIR\*"
  next_file:
    StrCmp $1 "" empty
    StrCmp $1 "." advance
    StrCmp $1 ".." advance
    FindClose $0
    MessageBox MB_OK|MB_ICONSTOP "$(ExistingFilesError)"
    Abort
  advance:
    FindNext $0 $1
    Goto next_file
  empty:
    FindClose $0
  done:
FunctionEnd

Section "ClickPulse"
  Call CheckInstallDirectory
  Call EnsureStopped
  SetOutPath "$INSTDIR"
  SetOverwrite on
  File /r "${PAYLOAD}\*"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\ClickPulse"
  CreateShortcut "$SMPROGRAMS\ClickPulse\ClickPulse.lnk" "$INSTDIR\ClickPulse.exe"
  WriteRegStr HKCU "Software\ClickPulse" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "ClickPulse"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${PUBLIC_VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\ClickPulse.exe"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Function un.onInit
  SetShellVarContext current
  !insertmacro MUI_UNGETLANGUAGE
  !insertmacro AcquireSetupGuard
  Call un.EnsureStopped
FunctionEnd

Section "Uninstall"
  ; Only remove this installation's startup entry and registered location.
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "ClickPulse"
  StrCmp $0 '$\"$INSTDIR\ClickPulse.exe$\" --background' 0 leave_startup
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "ClickPulse"
  leave_startup:
  ReadRegStr $0 HKCU "${UNINSTALL_KEY}" "InstallLocation"
  StrCpy $2 0
  StrCmp $0 "$INSTDIR" 0 leave_registration
    StrCpy $2 1
    DeleteRegKey HKCU "${UNINSTALL_KEY}"
  leave_registration:
  ReadRegStr $0 HKCU "Software\ClickPulse" "InstallDir"
  StrCmp $0 "$INSTDIR" 0 leave_location
    DeleteRegValue HKCU "Software\ClickPulse" "InstallDir"
  leave_location:
  !include "${UNINSTALL_FILES}"
  Delete "$INSTDIR\Uninstall.exe"
  StrCmp $2 1 0 leave_shortcut
    Delete "$SMPROGRAMS\ClickPulse\ClickPulse.lnk"
    RMDir "$SMPROGRAMS\ClickPulse"
  leave_shortcut:
  RMDir "$INSTDIR"
  ; No Delete/RMDir operation addresses $LOCALAPPDATA\ClickPulse.
SectionEnd
