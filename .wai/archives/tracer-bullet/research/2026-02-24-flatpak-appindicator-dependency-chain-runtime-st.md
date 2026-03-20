Flatpak appindicator dependency chain + runtime status

## libayatana-appindicator dependency chain (for system tray icon)

Full chain required (none are in GNOME SDK 48):
  intltool → libdbusmenu → libayatana-indicator → ayatana-ido → libayatana-appindicator

### intltool
- Plain autotools, no issues.

### libdbusmenu 16.04.0
- Needs --disable-dumper (requires GTK2, not in SDK)
- Needs --disable-vala (requires introspection, not enabled)
- AM_CONDITIONAL(HAVE_VALGRIND) is inside the tests block; dropping
  --disable-tests and mocking DBUSMENUTESTSVALGRIND_CFLAGS/LIBS fixes it
- Uses deprecated G_TYPE_INSTANCE_GET_PRIVATE which triggers a GLib
  preprocessor #error (not a -Wdeprecated warning); fix is
  -DGLIB_DISABLE_DEPRECATION_WARNINGS in cflags build-option

### libayatana-indicator 0.9.4
- cmake, builddir: true
- Disable mono/vala/python bindings (-DENABLE_BINDINGS_*=NO)

### ayatana-ido 0.10.1
- cmake, builddir: true, no special flags needed

### libayatana-appindicator 0.5.93
- cmake, builddir: true
- Disable mono/vala/python/gtkdoc (-DENABLE_BINDINGS_*=NO -DENABLE_GTKDOC=NO)
- gtk-doc scangobj.sh step fails to find bundled libs at link time;
  -DENABLE_GTKDOC=NO skips it

## Runtime status (as of first working install)

- App launches successfully via: flatpak run io.github.charly.fotos
- System tray icon appears and works
- Screenshot capture does NOT work yet (under investigation)
