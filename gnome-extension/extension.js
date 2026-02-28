import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import St from 'gi://St';
import {Extension, gettext as _} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

const FOTOS_BUS_NAME = 'io.github.charly.Fotos';
const FOTOS_OBJECT_PATH = '/io/github/charly/Fotos';
const FOTOS_DESKTOP_ID = 'io.github.charly.fotos.desktop';
const LAUNCH_POLL_INTERVAL_MS = 200;
const LAUNCH_POLL_TIMEOUT_MS = 5000;

// Inline interface XML — extensions cannot read files at runtime.
const FOTOS_IFACE_XML = `
<node>
  <interface name="io.github.charly.Fotos">
    <method name="Activate"/>
    <method name="TakeScreenshot">
      <arg type="s" direction="in" name="mode"/>
      <arg type="s" direction="out" name="status"/>
    </method>
    <property name="Version" type="s" access="read"/>
  </interface>
</node>`;

const FotosProxy = Gio.DBusProxy.makeProxyWrapper(FOTOS_IFACE_XML);

export default class FotosExtension extends Extension {
    enable() {
        this._settings = this.getSettings();
        this._fotosOnBus = false;
        this._proxy = null;
        this._pollSource = null;

        // Create proxy — watches for the service automatically.
        this._proxy = new FotosProxy(
            Gio.DBus.session,
            FOTOS_BUS_NAME,
            FOTOS_OBJECT_PATH,
            this._onProxyReady.bind(this),
        );

        // Track service presence via NameOwnerChanged.
        this._nameWatchId = Gio.DBus.session.signal_subscribe(
            'org.freedesktop.DBus',
            'org.freedesktop.DBus',
            'NameOwnerChanged',
            '/org/freedesktop/DBus',
            FOTOS_BUS_NAME,
            Gio.DBusSignalFlags.NONE,
            this._onNameOwnerChanged.bind(this),
        );

        // Build panel button.
        this._indicator = new PanelMenu.Button(0.0, this.metadata.name, false);
        const icon = new St.Icon({
            icon_name: 'camera-photo-symbolic',
            style_class: 'system-status-icon',
        });
        this._indicator.add_child(icon);

        // Menu items.
        const openItem = new PopupMenu.PopupMenuItem(_('Open Fotos'));
        openItem.connect('activate', () => this._launchAndThen(() => {
            this._proxy.ActivateRemote((_p, err) => {
                if (err) logError(err, 'Fotos: Activate failed');
            });
        }));
        this._indicator.menu.addMenuItem(openItem);

        this._indicator.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        this._regionItem = new PopupMenu.PopupMenuItem(_('Capture Region'));
        this._regionItem.connect('activate', () => this._launchAndThen(() => {
            this._proxy.TakeScreenshotRemote('region', (_p, err) => {
                if (err) logError(err, 'Fotos: TakeScreenshot region failed');
            });
        }));
        this._indicator.menu.addMenuItem(this._regionItem);

        this._fullscreenItem = new PopupMenu.PopupMenuItem(_('Capture Fullscreen'));
        this._fullscreenItem.connect('activate', () => this._launchAndThen(() => {
            this._proxy.TakeScreenshotRemote('fullscreen', (_p, err) => {
                if (err) logError(err, 'Fotos: TakeScreenshot fullscreen failed');
            });
        }));
        this._indicator.menu.addMenuItem(this._fullscreenItem);

        Main.panel.addToStatusArea(this.uuid, this._indicator);

        // Register keybindings.
        Main.wm.addKeybinding(
            'capture-region-shortcut',
            this._settings,
            Meta.KeyBindingFlags.NONE,
            Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
            () => this._launchAndThen(() => {
                this._proxy.TakeScreenshotRemote('region', (_p, err) => {
                    if (err) logError(err, 'Fotos: keybinding region failed');
                });
            }),
        );
        Main.wm.addKeybinding(
            'capture-fullscreen-shortcut',
            this._settings,
            Meta.KeyBindingFlags.NONE,
            Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
            () => this._launchAndThen(() => {
                this._proxy.TakeScreenshotRemote('fullscreen', (_p, err) => {
                    if (err) logError(err, 'Fotos: keybinding fullscreen failed');
                });
            }),
        );

        this._updateSensitivity();
    }

    disable() {
        Main.wm.removeKeybinding('capture-region-shortcut');
        Main.wm.removeKeybinding('capture-fullscreen-shortcut');

        if (this._nameWatchId) {
            Gio.DBus.session.signal_unsubscribe(this._nameWatchId);
            this._nameWatchId = null;
        }

        this._cancelPoll();

        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }

        this._proxy = null;
        this._regionItem = null;
        this._fullscreenItem = null;
        this._settings = null;
    }

    // ── Private ──────────────────────────────────────────────────────────────

    _onProxyReady(_proxy, error) {
        if (error) {
            logError(error, 'Fotos: proxy init error');
            return;
        }
        this._fotosOnBus = this._proxy.g_name_owner !== null;
        this._updateSensitivity();
    }

    _onNameOwnerChanged(_connection, _sender, _path, _iface, _signal, params) {
        const [_name, _oldOwner, newOwner] = params.deep_unpack();
        this._fotosOnBus = newOwner !== '';
        this._updateSensitivity();
    }

    _updateSensitivity() {
        if (!this._regionItem || !this._fullscreenItem)
            return;
        // Capture items always enabled — _launchAndThen handles cold start.
        // Open Fotos is always available too, but we dim captures when running
        // to give visual feedback that the service is live.
        this._regionItem.sensitive = true;
        this._fullscreenItem.sensitive = true;
    }

    /**
     * If Fotos is already on the bus call `callback` immediately.
     * Otherwise launch the app and poll until the bus name appears (5 s timeout).
     */
    _launchAndThen(callback) {
        if (this._fotosOnBus) {
            callback();
            return;
        }

        // Launch Fotos.
        try {
            const appInfo = Gio.DesktopAppInfo.new(FOTOS_DESKTOP_ID);
            if (appInfo)
                appInfo.launch([], null);
            else
                log('Fotos: desktop file not found, cannot launch');
        } catch (e) {
            logError(e, 'Fotos: launch failed');
            return;
        }

        // Poll for bus name (max 5 s).
        const deadline = GLib.get_monotonic_time() + LAUNCH_POLL_TIMEOUT_MS * 1000;
        this._cancelPoll();
        this._pollSource = GLib.timeout_add(GLib.PRIORITY_DEFAULT, LAUNCH_POLL_INTERVAL_MS, () => {
            if (this._fotosOnBus) {
                this._pollSource = null;
                callback();
                return GLib.SOURCE_REMOVE;
            }
            if (GLib.get_monotonic_time() > deadline) {
                log('Fotos: timed out waiting for D-Bus service after launch');
                this._pollSource = null;
                return GLib.SOURCE_REMOVE;
            }
            return GLib.SOURCE_CONTINUE;
        });
    }

    _cancelPoll() {
        if (this._pollSource) {
            GLib.source_remove(this._pollSource);
            this._pollSource = null;
        }
    }
}
