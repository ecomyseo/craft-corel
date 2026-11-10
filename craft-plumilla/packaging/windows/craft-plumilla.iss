; Inno Setup script for Craft Plumilla (Windows 64-bit).
; Variables: /DVersion=x.y.z /DSrcDir=<the modular package folder, with
; craft-plumilla.exe, craft-plumilla-engine.dll and the other DLLs>
; /DOutDir=<where the setup goes>.
;
; The package folder's craft-plumilla-settings is left out on purpose: an
; installed copy keeps its preferences in %APPDATA%\Craft Plumilla, as the
; settings code does when there is no portable folder beside the program.
; No file type is taken from another program: only .tdraw drawings are
; offered, and only when the task is ticked.

#ifndef Version
  #define Version "0.0.0"
#endif
#ifndef SrcDir
  #error SrcDir must name the modular package folder
#endif
#ifndef OutDir
  #define OutDir "..\..\dist"
#endif

[Setup]
AppId={{3C8E5B21-7A4D-4E19-9C62-5F0B8D1A7E43}
AppName=Craft Plumilla
AppVersion={#Version}
AppVerName=Craft Plumilla {#Version}
AppPublisher=Ecom Experts
AppCopyright=Copyright (c) 2026 Craft Plumilla and TraceDraw contributors
AppPublisherURL=https://github.com/ecomyseo/craft-plumilla
AppSupportURL=https://github.com/ecomyseo/craft-plumilla/issues
DefaultDirName={autopf}\Craft Plumilla
DefaultGroupName=Craft Plumilla
UninstallDisplayIcon={app}\craft-plumilla.exe
UninstallDisplayName=Craft Plumilla
OutputDir={#OutDir}
OutputBaseFilename=craft-plumilla-{#Version}-windows-x86_64-setup
SetupIconFile=..\..\assets\craft-plumilla\craft-plumilla.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Installs for the current user without asking for administrator rights;
; the first page lets the user choose all users instead.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
ChangesAssociations=yes
DisableProgramGroupPage=yes
ShowLanguageDialog=auto
LicenseFile={#SrcDir}\LICENSE-MIT

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
spanish.AssociateTdraw=Abrir los dibujos .tdraw con Craft Plumilla
english.AssociateTdraw=Open .tdraw drawings with Craft Plumilla
spanish.FileTypes=Tipos de archivo:
english.FileTypes=File types:
spanish.TdrawDocument=Dibujo de Craft Plumilla
english.TdrawDocument=Craft Plumilla drawing
spanish.Manuals=Manuales de Craft Plumilla
english.Manuals=Craft Plumilla manuals

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "tdraw"; Description: "{cm:AssociateTdraw}"; GroupDescription: "{cm:FileTypes}"; Flags: unchecked

[Files]
Source: "{#SrcDir}\*"; DestDir: "{app}"; Excludes: "craft-plumilla-settings,craft-plumilla-settings\*"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Craft Plumilla"; Filename: "{app}\craft-plumilla.exe"; WorkingDir: "{app}"
Name: "{autoprograms}\{cm:Manuals}"; Filename: "{app}\Manuales"
Name: "{autodesktop}\Craft Plumilla"; Filename: "{app}\craft-plumilla.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\.tdraw"; ValueType: string; ValueName: ""; ValueData: "CraftPlumilla.Drawing"; Flags: uninsdeletevalue; Tasks: tdraw
Root: HKA; Subkey: "Software\Classes\CraftPlumilla.Drawing"; ValueType: string; ValueName: ""; ValueData: "{cm:TdrawDocument}"; Flags: uninsdeletekey; Tasks: tdraw
Root: HKA; Subkey: "Software\Classes\CraftPlumilla.Drawing\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\craft-plumilla.exe,0"; Tasks: tdraw
Root: HKA; Subkey: "Software\Classes\CraftPlumilla.Drawing\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\craft-plumilla.exe"" ""%1"""; Tasks: tdraw

[Run]
Filename: "{app}\craft-plumilla.exe"; Description: "{cm:LaunchProgram,Craft Plumilla}"; Flags: nowait postinstall skipifsilent unchecked
