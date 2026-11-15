#!/usr/bin/env bash
set -euo pipefail

repo=$(realpath "$1")
debs=$(realpath "$2")
: "${APT_GPG_PRIVATE_KEY:?Missing APT_GPG_PRIVATE_KEY}"
: "${APT_GPG_FINGERPRINT:?Missing APT_GPG_FINGERPRINT}"

export GNUPGHOME
GNUPGHOME=$(mktemp -d)
chmod 700 "$GNUPGHOME"
trap 'rm -rf "$GNUPGHOME"' EXIT
printf '%s\n' "$APT_GPG_PRIVATE_KEY" | gpg --batch --import
gpg --batch --list-secret-keys "$APT_GPG_FINGERPRINT" >/dev/null

mkdir -p "$repo/pool/main" "$repo/dists/stable/main/binary-amd64"
shopt -s nullglob
packages=("$debs"/*.deb)
[[ ${#packages[@]} -eq 1 ]] || { echo "Expected exactly one deb" >&2; exit 1; }
[[ $(dpkg-deb -f "${packages[0]}" Package) == alquife ]]
[[ $(dpkg-deb -f "${packages[0]}" Architecture) == amd64 ]]
destination="$repo/pool/main/$(basename "${packages[0]}")"
if [[ -e "$destination" ]] && ! cmp -s "${packages[0]}" "$destination"; then
  echo "Refusing to replace an existing package with different contents" >&2
  exit 1
fi
cp "${packages[0]}" "$destination"
touch "$repo/.nojekyll"
gpg --batch --export "$APT_GPG_FINGERPRINT" > "$GNUPGHOME/public.gpg"
[[ -s "$GNUPGHOME/public.gpg" ]]
if [[ -s "$repo/key.gpg" ]] && ! cmp -s "$repo/key.gpg" "$GNUPGHOME/public.gpg"; then
  echo "Signing key differs from the published key; rotate it deliberately" >&2
  exit 1
fi
cp "$GNUPGHOME/public.gpg" "$repo/key.gpg"
cd "$repo"
apt-ftparchive packages pool/main > dists/stable/main/binary-amd64/Packages
gzip -n -9 -c dists/stable/main/binary-amd64/Packages > dists/stable/main/binary-amd64/Packages.gz
apt-ftparchive -o APT::FTPArchive::Release::Origin=Alquife \
  -o APT::FTPArchive::Release::Label=Alquife \
  -o APT::FTPArchive::Release::Suite=stable \
  -o APT::FTPArchive::Release::Codename=stable \
  -o APT::FTPArchive::Release::Architectures=amd64 \
  -o APT::FTPArchive::Release::Components=main \
  release dists/stable > dists/stable/Release
gpg --batch --yes --pinentry-mode loopback --passphrase "${APT_GPG_PASSPHRASE:-}" \
  --local-user "$APT_GPG_FINGERPRINT" --clearsign \
  --output dists/stable/InRelease dists/stable/Release
gpg --batch --yes --pinentry-mode loopback --passphrase "${APT_GPG_PASSPHRASE:-}" \
  --local-user "$APT_GPG_FINGERPRINT" --armor --detach-sign \
  --output dists/stable/Release.gpg dists/stable/Release
