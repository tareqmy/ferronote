$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (Test-Path "$scriptDir\scripts\install.ps1") {
    & "$scriptDir\scripts\install.ps1" @args
} else {
    irm https://raw.githubusercontent.com/tareqmy/ferronote/master/scripts/install.ps1 | iex
}
