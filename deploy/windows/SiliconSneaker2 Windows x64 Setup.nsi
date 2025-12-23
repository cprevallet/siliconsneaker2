;Written by Craig S. Prevallet
;based on NSIS Modern User Interface
;NSIS Modern User Interface
;Welcome/Finish Page Example Script
;Written by Joost Verburg

;--------------------------------
;Include LogicLib to check we
; are admin

!include LogicLib.nsh

Function .onInit
UserInfo::GetAccountType
pop $0
${If} $0 != "admin" ;Require admin rights on NT4+
    MessageBox mb_iconstop "Administrator rights required!"
    SetErrorLevel 740 ;ERROR_ELEVATION_REQUIRED
    Quit
${EndIf}
FunctionEnd

;--------------------------------
;Include Modern UI

  !include "MUI2.nsh"

;--------------------------------
;General

  ;Name and file
  Name "SiliconSneaker2"
  OutFile "SiliconSneaker2 Windows x64 Setup.exe"

  ;Default installation folder
  InstallDir "$PROGRAMFILES64\SiliconSneaker2"

  ;Get installation folder from registry if available
  InstallDirRegKey HKCU "Software\SiliconSneaker2" ""

  ;Request application privileges for Windows Vista
  RequestExecutionLevel admin
;--------------------------------
;Variables

  Var StartMenuFolder
;--------------------------------
;Interface Settings

  !define MUI_ABORTWARNING

;--------------------------------
;Pages

  !insertmacro MUI_PAGE_WELCOME
  !insertmacro MUI_PAGE_LICENSE "LICENSE"
  !insertmacro MUI_PAGE_DIRECTORY

  ;Start Menu Folder Page Configuration
  !define MUI_STARTMENUPAGE_REGISTRY_ROOT "HKCU" 
  !define MUI_STARTMENUPAGE_REGISTRY_KEY "Software\SiliconSneaker2" 
  !define MUI_STARTMENUPAGE_REGISTRY_VALUENAME "Start Menu Folder"
  
  !insertmacro MUI_PAGE_STARTMENU Application $StartMenuFolder

  !insertmacro MUI_PAGE_INSTFILES
  !insertmacro MUI_PAGE_FINISH

  !insertmacro MUI_UNPAGE_WELCOME
  !insertmacro MUI_UNPAGE_CONFIRM
  !insertmacro MUI_UNPAGE_INSTFILES
  !insertmacro MUI_UNPAGE_FINISH

;--------------------------------
;Languages

  !insertmacro MUI_LANGUAGE "English"

;--------------------------------
;Installer Sections

Section "Components" Components

  CreateDirectory "$INSTDIR"
  SetOutPath "$INSTDIR"

  ;Install the following files
  CreateDirectory $INSTDIR\bin
  CreateDirectory $INSTDIR\etc
  CreateDirectory $INSTDIR\lib
  CreateDirectory $INSTDIR\share
  CreateDirectory $INSTDIR\icons
  SetOutPath "$INSTDIR\\bin"
  File /r "bin\"
  SetOutPath "$INSTDIR\\etc"
  File /r "etc\"
  SetOutPath "$INSTDIR\\lib"
  File /r "lib\"
  SetOutPath "$INSTDIR\\share"
  File /r "share\"
  SetOutPath "$INSTDIR\\icons"
  File /r "icons\"
  SetOutPath "$INSTDIR"
  File LICENSE
  
  ;Store installation folder
  WriteRegStr HKCU "Software\SiliconSneaker2" "" $INSTDIR
  
  ; Store file association
  WriteRegStr HKCR ".fit" "" "SiliconSneaker2"
  WriteRegStr HKCR "SiliconSneaker2" "" "SiliconSneaker2 File"
  WriteRegStr HKCR "SiliconSneaker2\DefaultIcon" "" "$INSTDIR\\icons\siliconsneaker2.ico,1"
  WriteRegStr HKCR "SiliconSneaker2\shell\view" "" "View with SiliconSneaker2"
  WriteRegStr HKCR "SiliconSneaker2\shell\view\command" "" '"$INSTDIR\\bin\siliconsneaker2.exe" "%1"'

  ;Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    
    ;Create shortcuts
    CreateDirectory "$SMPROGRAMS\$StartMenuFolder"
    CreateShortCut "$SMPROGRAMS\$StartMenuFolder\Uninstall.lnk" "$INSTDIR\Uninstall.exe"  "" "$INSTDIR\\icons\\siliconsneaker2.ico" 0
    CreateShortCut "$SMPROGRAMS\$StartMenuFolder\SiliconSneaker2.lnk" "$INSTDIR\\bin\\siliconsneaker2" "" "$INSTDIR\\icons\\siliconsneaker2.ico" 0
  
  !insertmacro MUI_STARTMENU_WRITE_END

SectionEnd

;Uninstaller Section

Section "Uninstall"

  RMDir /r "$INSTDIR"

  !insertmacro MUI_STARTMENU_GETFOLDER Application $StartMenuFolder
    
  Delete "$SMPROGRAMS\$StartMenuFolder\SiliconSneaker2.lnk" 
  Delete "$SMPROGRAMS\$StartMenuFolder\Uninstall.lnk"
  RMDir "$SMPROGRAMS\$StartMenuFolder"
  
  ;nsExec::Exec 'set STATIC_FILES ""'

  DeleteRegKey /ifempty HKCU "Software\SiliconSneaker2"
  ReadRegStr $R0 HKCR ".fit" ""
  StrCmp $R0 "SiliconSneaker2" 0 +2
    DeleteRegKey HKCR ".fit"
  DeleteRegKey HKCR "SiliconSneaker2"


SectionEnd
