Unicode true

!include "MUI2.nsh"
!include "Sections.nsh"

!include "nsProcess.nsh"
 
InstallDir $PROGRAMFILES64\xterminate

# retrieve xterminate version and add it to the installer's name
!getdllversion "..\target\release\xterminate.exe" EXEVERSION
Name "xterminate v${EXEVERSION1}.${EXEVERSION2}.${EXEVERSION3}"

# set installer executable name
OutFile "..\target\release\xterminate-setup.exe"

RequestExecutionLevel admin

!define MUI_WELCOMEPAGE_TEXT "This setup will guide you through the installation of xterminate v${EXEVERSION1}.${EXEVERSION2}.${EXEVERSION3}$\r$\n$\r$\nIf you already have an older version of xterminate installed it is recommended that you uninstall it before continuing this setup.$\r$\n$\r$\nClick Next to continue."
!define MUI_LICENSEPAGE_CHECKBOX

!define MUI_FINISHPAGE_RUN $INSTDIR\xterminate.exe

!define MUI_COMPONENTSPAGE_SMALLDESC

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE ..\LICENSE
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_COMPONENTS
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

# Close or kill xterminate if it is currently running (up to 0.5s delay)
!macro ExitAppIfRunning
    DetailPrint "Attempting to close xterminate..."
    ${nsProcess::FindProcess} "xterminate.exe" $R0
    Pop $R0
    ${If} $R0 = 0
        ${nsProcess::CloseProcess} "xterminate.exe" $R0
        Sleep 250
        
        ${If} $R0 != 0
            DetailPrint "Failed to close xterminate -- killing process instead"
            ${nsProcess::KillProcess} "xterminate.exe" $R0
            Sleep 250
        ${EndIf}
    ${EndIf}
!macroend

Section "Install" section_install

# used to set $APPDATA to be system-wide (i.e. %ProgramData%)
SetShellVarContext all

!insertmacro ExitAppIfRunning

# uninstall any previous version of xterminate if found
${If} ${FileExists} "$INSTDIR\xterminate.exe"
    DetailPrint "Uninstalling previous version of xterminate..."
    ExecWait '"$INSTDIR\uninstall.exe" /S _?=$INSTDIR'
${EndIf}

# define output path
SetOutPath $INSTDIR

# add files to uninstaller
File ..\LICENSE
File ..\target\release\xterminate.exe

SetOutPath $INSTDIR\res
File ..\res\icon.ico
File ..\res\cursor.cur

SetOutPath $INSTDIR

# create uninstaller
WriteUninstaller $INSTDIR\uninstall.exe

CreateShortcut "$SMPROGRAMS\xterminate.lnk" "$INSTDIR\xterminate.exe"
CreateShortcut "$SMPROGRAMS\Uninstall xterminate.lnk" "$INSTDIR\uninstall.exe"

# run xterminate on startup
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "xterminate" "$INSTDIR\xterminate.exe"

# add uninstaller to list of installed programs
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "DisplayName" "xterminate"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "Publisher" "Biixie <hi@biixie.com>"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "DisplayVersion" "${EXEVERSION1}.${EXEVERSION2}.${EXEVERSION3}"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "VersionMajor" "${EXEVERSION1}"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "VersionMinor" "${EXEVERSION2}"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "Version" "${EXEVERSION3}"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "InstallLocation" "$INSTDIR"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "URLInfoAbout" "https://github.com/ibiixie/xterminate"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "HelpLink" "https://github.com/ibiixie/xterminate"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "Readme" "$INSTDIR\README"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "QuietUninstallString" "$\"$INSTDIR\uninstall.exe$\" /S"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "DisplayIcon" "$INSTDIR\xterminate.exe"
WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "NoModify" 1
WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "NoRepair" 1

SectionGetSize ${section_install} $0
WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
               "EstimatedSize" "$0"

SectionEnd

Section "un.Uninstall xterminate" uninstall_xterminate_section
    SectionIn RO

    # used to set $APPDATA to be system-wide (i.e. %ProgramData%)
    SetShellVarContext all

    !insertmacro ExitAppIfRunning

    # remove xterminate program files
    Delete "$INSTDIR\xterminate.exe"
    Delete "$INSTDIR\xterminate.exe.old"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\uninstall.exe"
    Delete "$INSTDIR\res\icon.ico"
    Delete "$INSTDIR\res\cursor.cur"
    RMDir "$INSTDIR\res"
    RMDir "$INSTDIR\" # non-destructive - removes only if empty

    # remove xterminate shortcuts
    Delete $SMPROGRAMS\xterminate.lnk
    Delete "$SMPROGRAMS\Uninstall xterminate.lnk"

    # remove run on startup registry value
    DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "xterminate"

    # remove from list of installed programs
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate"
SectionEnd

Section "un.Remove settings" remove_settings_section
    SectionIn 1

    # used to set $APPDATA to be system-wide (i.e. %ProgramData%)
    SetShellVarContext all

    # remove program data
    Delete "$APPDATA\xterminate\config.toml"
    RMDir "$APPDATA\xterminate"

    # remove settings stored in the registry during runtime
    DeleteRegKey HKCU "Software\xterminate"
SectionEnd

Section "un.Remove logs" remove_logs_section
    SectionIn 2

    # used to set $APPDATA to be system-wide (i.e. %ProgramData%)
    SetShellVarContext all

    # remove logs
    RMDir /r "$APPDATA\xterminate\logs\"
    RMDir "$APPDATA\xterminate"
SectionEnd

!insertmacro MUI_UNFUNCTION_DESCRIPTION_BEGIN
!insertmacro MUI_DESCRIPTION_TEXT ${uninstall_xterminate_section} "Uninstall the xterminate application and most of its data."
!insertmacro MUI_DESCRIPTION_TEXT ${remove_settings_section} "Remove the configuration file used by xterminate to save application settings and preferences."
!insertmacro MUI_DESCRIPTION_TEXT ${remove_logs_section} "Remove log-files generated by xterminate during runtime."
!insertmacro MUI_UNFUNCTION_DESCRIPTION_END