#!/usr/bin/env bash
set -euo pipefail

nix-shell -p curl msitools unzip --run "
set -euo pipefail

echo 'Fetching MSI URL from updates.txt...'
url=\$(curl -s https://ltspice.analog.com/download/updates.txt | sed -n 's/^URL = //p')
if [[ -z \"\$url\" ]]; then
  echo 'ERROR: could not extract MSI URL from updates.txt' >&2
  exit 1
fi
echo \"MSI URL: \$url\"

echo 'Downloading MSI installer...'
curl -sL \"\$url\" -o /tmp/LTspice64.msi
if [[ ! -s /tmp/LTspice64.msi ]]; then
  echo 'ERROR: download failed or file is empty' >&2
  exit 1
fi

echo 'Extracting lib.zip from MSI...'
msiextract /tmp/LTspice64.msi
if [[ ! -f 'LocalAppDataFolder/LTspice/lib.zip' ]]; then
  echo 'ERROR: lib.zip not found in MSI' >&2
  rm -rf /tmp/LTspice64.msi 'LocalAppDataFolder'
  exit 1
fi

echo 'Unpacking lib.zip...'
mkdir -p ltspice-lib
unzip -o 'LocalAppDataFolder/LTspice/lib.zip' -d ltspice-lib
rm -rf /tmp/LTspice64.msi 'LocalAppDataFolder'
echo 'Done — extracted to ./ltspice-lib/'
"