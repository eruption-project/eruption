name Eruption

SetCompressor /solid lzma

!include MUI2.nsh

!define SOURCE_VERSION "0.5.0"

# Defines
!define URL https://eruption-project.org/
!define ShortName Eruption
!define Version ${SOURCE_VERSION}
!define REGKEY "SOFTWARE\$(^Name)"
!define SOURCEDIR "../../target/x86_64-pc-windows-gnu/release/"

# Version Information for the executable
VIProductVersion "${Version}.0"
VIAddVersionKey FileDescription  "Windows Installer for Eruption"
VIAddVersionKey ProductName      "Eruption Installer"
VIAddVersionKey FileVersion      "${Version}.0"
VIAddVersionKey ProductVersion   "${Version}.0"
VIAddVersionKey Comments         "Realtime RGB LED Software for Linux and Windows"
VIAddVersionKey CompanyName      "The Eruption Development Team"
VIAddVersionKey LegalCopyright   "Copyright (c) 2019-2023, The Eruption Development Team"
VIAddVersionKey InternalName     "eruption"
VIAddVersionKey LegalTrademarks  "GPLv3"
VIAddVersionKey OriginalFilename "Eruption.exe"

# MUI defines
!define MUI_FINISHPAGE_NOAUTOCLOSE
!define MUI_STARTMENUPAGE_REGISTRY_ROOT HKLM
!define MUI_STARTMENUPAGE_REGISTRY_KEY ${REGKEY}
!define MUI_STARTMENUPAGE_REGISTRY_VALUENAME StartMenuGroup
!define MUI_STARTMENUPAGE_DEFAULTFOLDER $(^Name)
#!define MUI_FINISHPAGE_SHOWREADME $INSTDIR\README.txt
!define MUI_UNFINISHPAGE_NOAUTOCLOSE
#!define MUI_ICON "eruption.ico"
#!define MUI_UNICON "uninstall-eruption.ico"
!define MUI_HEADERIMAGE
#!define MUI_HEADERIMAGE_BITMAP "../assets/eruption.bmp"
#!define MUI_HEADERIMAGE_BITMAP_NOSTRETCH
#!define MUI_WELCOMEFINISHPAGE_BITMAP "../assets/eruption.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP_NOSTRETCH

# Installer pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE ../../LICENSE
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
#!insertmacro MUI_PAGE_STARTMENU Application $StartMenuGroup
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

# Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
#!insertmacro MUI_UNPAGE_LICENSE textfile
#!insertmacro MUI_UNPAGE_COMPONENTS
#!insertmacro MUI_UNPAGE_DIRECTORY
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

# Installer languages
!insertmacro MUI_LANGUAGE English
!insertmacro MUI_LANGUAGE German

# Install Types
InstType "Full"
InstType "Standard"
InstType "Minimal"

OutFile ../../target/Eruption.exe
InstallDir $PROGRAMFILES\Eruption
CRCCheck on
XPStyle on
ShowInstDetails show

RequestExecutionLevel admin

# start default section
Section "!Eruption"
    SectionIn RO    # this section cannot be deselected
    SetOverwrite on

    # set the installation directory as the destination for the following actions
    SetOutPath $INSTDIR

    File "${SOURCEDIR}\eruption.exe"
    File "..\shell\windows\*.bat"

    # create the uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

SectionEnd

# start "Pyroclasm UI" section
Section "!Pyroclasm UI"
    SectionIn 1 2
    SetOverwrite on

    # set the installation directory as the destination for the following actions
    SetOutPath $INSTDIR

    File "${SOURCEDIR}\pyroclasm.exe"

SectionEnd

# start Eruption SDK section
Section "Eruption Software Development Kit (SDK)"
    SectionIn RO    # this section cannot be deselected
    SetOverwrite on

    # set the installation directory as the destination for the following actions
    SetOutPath $INSTDIR

    File "${SOURCEDIR}\eruption.dll"

SectionEnd

SectionGroup /e "Eruption GTK+3 GUI"
    # Eruption GUI GTK+3
    Section /o "Eruption GTK+3 GUI"
        SectionIn 1
        SetOverwrite on

        # set the installation directory as the destination for the following actions
        SetOutPath $INSTDIR

        File "${SOURCEDIR}\eruption-gui-gtk3.exe"

    SectionEnd

    # GTK+3 redist
    Section /o "GTK+3 Redistributable Package"
        SectionIn 1
        SetOverwrite on

        # set the installation directory as the destination for the following actions
        SetOutPath $INSTDIR

        File /r "..\redist\redist-x86_64-windows\*.*"

    SectionEnd
SectionGroupEnd

# uninstaller section start
Section "uninstall"

    # Remove the link from the start menu
    # Delete "$SMPROGRAMS\new shortcut.lnk"

    Delete /REBOOTOK $INSTDIR\*.exe
    Delete /REBOOTOK $INSTDIR\*.dll
    Delete /REBOOTOK $INSTDIR\*.bat

    # Delete the uninstaller
    Delete /REBOOTOK $INSTDIR\uninstall.exe

    # request for deletion of instdir; be safe, do not delete foreign files
    RMDir /REBOOTOK $INSTDIR

# uninstaller section end
SectionEnd
