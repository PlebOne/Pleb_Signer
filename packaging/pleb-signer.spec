Name:           pleb-signer
Version:        0.1.0
Release:        1%{?dist}
Summary:        Linux desktop Nostr signer with D-Bus interface

License:        MIT
URL:            https://github.com/PlebOne/Pleb_Signer
Source0:        %{name}-%{version}.tar.gz

Requires:       dbus
Requires:       libsecret

%description
Pleb Signer is a NIP-55 compatible Nostr signer for Linux,
similar to Amber for Android. It stores your private keys
securely in the OS keyring and provides a D-Bus interface
for other applications to request signatures.

Features:
- Secure key storage in OS keyring (GNOME Keyring/KWallet)
- System tray integration
- D-Bus API for application integration
- NIP-04 and NIP-44 encryption support
- NIP-46 Bunker mode for remote signing

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/dbus-1/services
mkdir -p %{buildroot}/usr/share/doc/%{name}

install -m 755 pleb-signer %{buildroot}/usr/bin/pleb-signer
install -m 644 pleb-signer.desktop %{buildroot}/usr/share/applications/pleb-signer.desktop
install -m 644 com.plebsigner.Signer.service %{buildroot}/usr/share/dbus-1/services/com.plebsigner.Signer.service
install -m 644 README.md %{buildroot}/usr/share/doc/%{name}/README.md
install -m 644 LICENSE %{buildroot}/usr/share/doc/%{name}/LICENSE

%files
%license /usr/share/doc/%{name}/LICENSE
%doc /usr/share/doc/%{name}/README.md
/usr/bin/pleb-signer
/usr/share/applications/pleb-signer.desktop
/usr/share/dbus-1/services/com.plebsigner.Signer.service

%post
update-desktop-database -q /usr/share/applications 2>/dev/null || true

%postun
update-desktop-database -q /usr/share/applications 2>/dev/null || true

%changelog
* Thu Nov 28 2024 PlebOne <plebone@protonmail.com> - 0.1.0-1
- Initial release
- D-Bus signing interface
- System tray integration
- NIP-46 Bunker mode support
