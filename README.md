# [![Voila](https://i.ibb.co/R2T5Tvb/voila.png)](https://shields.io/)   [![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)   [![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com)

![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)
[![GitHub license](https://img.shields.io/github/license/Alonely0/voila.svg)](https://github.com/Alonely0/voila/blob/master/LICENSE)
[![GitHub release](https://img.shields.io/github/release/Alonely0/voila.svg)](https://GitHub.com/Naereen/StrapDown.js/releases/)
[![Linux build](https://github.com/Alonely0/Voila/actions/workflows/linux-ci.yml/badge.svg)](https://github.com/Alonely0/Voila/actions/workflows/linux-ci.yml)
[![macOS build](https://github.com/Alonely0/Voila/actions/workflows/mac-ci.yml/badge.svg)](https://github.com/Alonely0/Voila/actions/workflows/mac-ci.yml)
[![Windows build](https://github.com/Alonely0/Voila/actions/workflows/windows-ci.yml/badge.svg)](https://github.com/Alonely0/Voila/actions/workflows/windows-ci.yml)

Voila is a domain-specific language designed for doing complex operations to folders & files. It is based on a CLI tool, although you can write your Voila code and do something like this `voila DIRECTORY "$(cat operations.vla)"`. Voila is mainly tested in Linux, so should work better in *nix (Linux,*BSD, macOS, etc) than in Windows-based operating systems.

## Syntax

`voila DIRECTORY "<@VARIABLE | STRING | #REGEXP#> OPERATOR <@VARIABLE | STRING | #REGEXP#> [|| | && ANOTHER_CONDITIONAL ...] {OPERATION1-CYCLE-1(ARG1 ARG1, ARG2) OPERATION2-CYCLE-1(ARG1 ARG2) ...; OPERATION1-CYCLE-2(ARG1, ARG2 ARG2, ARG3)...}"`

Voila's syntax is composed of a traditional conditional/multi-conditional statement, followed by the operations, delimited within curly brackets. These are separated into cycles. A cycle is an iteration between all directory files, the operations in every cycle are executed in parallel, and cycles are executed consecutively. cycles are separated with `;`, and operations/functions arguments are separated with `,`. Variables' prefix is `@`, and its value changes to the file that is evaluating. Regular expressions are delimited between `#`. For a more intuitive explanation, go to the "Examples" section.

These are the available conditional operators:

* `==`: true if the first value matches the second
* `!=`: true if the first value doesn't match the second
* `>`: true if the first value is greater than the second
* `>=`: true if the first value is equal or greater than the second
* `<`: true if the first value is less than the second
* `<=`: true if the first value is equal or less than the second
* `~=`: true if the a value matches the regex provided in the other value
* `~!`: true if the a value doesn't match the regex provided in the other value

These are the available variables:

* `name`: filename
* `path`: absolute path
* `parent`: absolute path to file's directory
* `ownerID`: file owner ID (unix-only)
* `empty`: true if the file size is less than 1 byte (else false)
* `readonly`: true if the file is ro (else false)
* `elf`: true if the file is compliant to the Executable & Linkable Format (else false)
* `txt`: true if the file is a valid text file (else false)
* `size=tb`: file size in terabytes (2 decimals)
* `size=gb`: file size in gigabytes (2 decimals)
* `size=mb`: file size in megabytes (2 decimals)
* `size=kb`: file size in kilobytes (2 decimals)
* `size=bs`: file size bytes (no decimals)
* `sum=md5`: md5 checksum ***this variable might be removed in the future, md5 is completely broken***
* `sum=sha1`: sha1 checksum ***this variable might be removed in the future, sha1 is completely broken***
* `sum=sha224`: sha224 checksum
* `sum=sha256`: sha256 checksum
* `sum=sha384`: sha384 checksum
* `sum=sha512`: sha256 checksum
* `creation=date`: date of file creation (yyyy-mm-dd)
* `creation=hour`: hour of file creation (hh:mm:ss)
* `lastChange=date`: date of the last modification to the file (yyyy-mm-dd)
* `lastChange=hour`: hour of the last modification to the file (hh:mm:ss)
* `lastAccess=date`: date of the last access to the file (yyyy-mm-dd)
* `lastAccess=hour`: hour of the last access to the file (hh:mm:ss)

These are the available operations/functions:

* `print`: prints something to the terminal (not to the printer lol)
* `create`: creates a file, with its content as second argument
* `mkdir`: cretes a folder/directory
* `delete`: deletes file/directory ⚠️
* `move`: moves a file or a folder/directory ⚠️
* `copy`: copies a file or a folder/directory ⚠️
* `gzc`: compress file using gzip. first argument is the file to compress, the second is the file to save the compressed file
* `gzd`: decompress file using gzip. first argument is the file to compress, the second is be file to save the compressed file
* `shell`: gives a command to the Bourne Shell (`sh`) in Unix systems (like Linux or macOS), and a command to PowerShell (`powershell`) in Windows systems. Exists for doing things Voila functions can't, for example, send a dbus message. ⚠️

**⚠️ WARNING: If you use functions that access and/or modify the same file/directory in the same cycle it could cause undefined behavior because the file would be accessed and overwritten at the same time. For avoiding that, consider splitting those functions into different cycles. A workaround is being discussed in [#5](https://github.com/Alonely0/Voila/issues/5)**

[![forthebadge](https://forthebadge.com/images/badges/not-a-bug-a-feature.svg)](https://forthebadge.com)

### Examples

* `voila /backup "@creation=date <= 2020-01-01 { print(@name has been deleted) delete(@path) }`: Voila will delete every file in /backup whose creation was earlier to 2020 printing a delete message.
* `voila /backup "@name ~= #(.*)-2020# { print(@name has been deleted) delete(@path) }`: Voila will delete every file in /backup ending in 2020 printing a delete message.
* `voila /something "@md5256sum == 308uyrp028y4hp079y2hv92gbf49 { mkdir(./sums); create(./sums/@name.sum, @sha256sum) }`: Voila will create a folder in the current directory named "sums", will search for a file with that md5 checksum, get its sha256 checksum and save it in the sums folder.
* `voila /backup "@size=gb >= 1 { print(@name has been deleted) delete(@path) }`: Voila will delete every file in /backup weighter than 1gb printing a delete message.

## CLI flags

* `-r, --recursive`: If provided, voila will be executed recursively, which means that will also affect the files in the folders of the directory provided.
* `-h, --help`: Displays help.
* `-v, --version`: Displays installed version of Voila, if any.

## Error types

Voila provides 2 main error types:

* `RUNTIME ERROR`: An error that occurred while running the Voila code. Code might have been already executed, and other code might not.
* `PARSE ERROR`: This error is triggered by a syntax error during the construction of the AST (what reads the interpreter), so no code was executed during the raise of this error.

# Installation

You can install voila by cloning the repository and compiling or by `cargo install voila`. I have planned to provide prebuilt binaries soon.

# Submitting

* Errors: file an error issue using [this link](https://github.com/Alonely0/voila/issues/new?assignees=Alonely0&labels=bug&template=bug_report.md&title=). Remember to check if that issue is [already registered](https://github.com/Alonely0/voila/labels/bug)!
* Feature requests: file a f-request issue using [this link](https://github.com/Alonely0/voila/issues/new?assignees=Alonely0&labels=enhancement&template=feature_request.md&title=). Remember to check if that f-request was [already submitted](https://github.com/Alonely0/voila/labels/enhancement)!
* Doubt: file a doubt issue using [this link](https://github.com/Alonely0/voila/issues/new?assignees=Alonely0&labels=question&template=doubt.md&title=). Remember to check if that doubt [was already resolved](https://github.com/Alonely0/voila/labels/question)!
* Wanna chat with me? You can talk with me on [Discord](https://discord.com), add me as friend (`NOT-Guillem#8042`) and we'll be able to start chatting!

# Message from the author

Voila has been coded & tested by only one person, so don't expect it to be perfect. I'm looking for more people interested in maintaining Voila and helping out, if you're interested, DM me on [discord](https://discord.com) (`NOT-Guillem#8042`). Voila's discord server [is this](https://discord.gg/RhTpYGbnXU)
