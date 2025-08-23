// Legacy extension for GNOME 42-44
const { GObject, St, Gio, GLib, Clutter } = imports.gi;
const Main = imports.ui.main;
const PanelMenu = imports.ui.panelMenu;
const PopupMenu = imports.ui.popupMenu;
const Slider = imports.ui.slider;

const DBUS_NAME = 'com.elgato.RingLight';
const DBUS_PATH = '/com/elgato/RingLight';
const DBUS_INTERFACE = 'com.elgato.RingLight.Control';

const ElgatoDBusInterface = `
<node>
  <interface name="${DBUS_INTERFACE}">
    <method name="TurnOn">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="TurnOff">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="Toggle">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetBrightness">
      <arg type="y" direction="in" name="brightness"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetTemperature">
      <arg type="u" direction="in" name="kelvin"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="GetStatus">
      <arg type="b" direction="out" name="is_on"/>
      <arg type="y" direction="out" name="brightness"/>
      <arg type="u" direction="out" name="temperature"/>
    </method>
    <method name="Identify">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="ApplyScene">
      <arg type="s" direction="in" name="scene"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <property name="IsOn" type="b" access="read"/>
    <property name="Brightness" type="y" access="readwrite"/>
    <property name="Temperature" type="u" access="readwrite"/>
  </interface>
</node>`;

const ElgatoDBusProxy = Gio.DBusProxy.makeProxyWrapper(ElgatoDBusInterface);

const ElgatoIndicator = GObject.registerClass(
class ElgatoIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.0, 'Elgato Ring Light');
        
        this._isConnected = false;
        this._isOn = false;
        this._brightness = 50;
        this._temperature = 4500;
        
        let icon = new St.Icon({
            icon_name: 'dialog-information-symbolic',
            style_class: 'system-status-icon',
        });
        this.add_child(icon);
        this._icon = icon;
        
        this._buildMenu();
        
        this._connectToDBus();
        
        this.connect('destroy', () => {
            this._onDestroy();
        });
    }
    
    _buildMenu() {
        this._powerSwitch = new PopupMenu.PopupSwitchMenuItem('Ring Light', false);
        this._powerSwitch.connect('toggled', (item, state) => {
            this._toggleLight(state);
        });
        this.menu.addMenuItem(this._powerSwitch);
        
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        let brightnessItem = new PopupMenu.PopupBaseMenuItem({ activate: false });
        let brightnessBox = new St.BoxLayout({ vertical: false, x_expand: true });
        
        let brightnessLabel = new St.Label({ text: 'Brightness:', y_align: Clutter.ActorAlign.CENTER });
        brightnessBox.add_child(brightnessLabel);
        
        this._brightnessSlider = new Slider.Slider(0.5);
        this._brightnessSlider.connect('notify::value', (slider) => {
            let brightness = Math.round(slider.value * 100);
            this._brightnessValueLabel.text = `${brightness}%`;
        });
        this._brightnessSlider.connect('drag-end', (slider) => {
            let brightness = Math.round(slider.value * 100);
            this._setBrightness(brightness);
        });
        brightnessBox.add_child(this._brightnessSlider);
        
        this._brightnessValueLabel = new St.Label({ text: '50%', y_align: Clutter.ActorAlign.CENTER });
        brightnessBox.add_child(this._brightnessValueLabel);
        
        brightnessItem.add_child(brightnessBox);
        this.menu.addMenuItem(brightnessItem);
        
        let temperatureItem = new PopupMenu.PopupBaseMenuItem({ activate: false });
        let temperatureBox = new St.BoxLayout({ vertical: false, x_expand: true });
        
        let temperatureLabel = new St.Label({ text: 'Temperature:', y_align: Clutter.ActorAlign.CENTER });
        temperatureBox.add_child(temperatureLabel);
        
        this._temperatureSlider = new Slider.Slider(0.5);
        this._temperatureSlider.connect('notify::value', (slider) => {
            let kelvin = Math.round(2900 + (slider.value * (7000 - 2900)));
            this._temperatureValueLabel.text = `${kelvin}K`;
        });
        this._temperatureSlider.connect('drag-end', (slider) => {
            let kelvin = Math.round(2900 + (slider.value * (7000 - 2900)));
            this._setTemperature(kelvin);
        });
        temperatureBox.add_child(this._temperatureSlider);
        
        this._temperatureValueLabel = new St.Label({ text: '4500K', y_align: Clutter.ActorAlign.CENTER });
        temperatureBox.add_child(this._temperatureValueLabel);
        
        temperatureItem.add_child(temperatureBox);
        this.menu.addMenuItem(temperatureItem);
        
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        let scenesMenu = new PopupMenu.PopupSubMenuMenuItem('Scenes');
        
        const scenes = [
            { name: 'Daylight', id: 'daylight' },
            { name: 'Warm', id: 'warm' },
            { name: 'Cool', id: 'cool' },
            { name: 'Reading', id: 'reading' },
            { name: 'Video', id: 'video' },
        ];
        
        for (let scene of scenes) {
            let sceneItem = new PopupMenu.PopupMenuItem(scene.name);
            sceneItem.connect('activate', () => {
                this._applyScene(scene.id);
            });
            scenesMenu.menu.addMenuItem(sceneItem);
        }
        
        this.menu.addMenuItem(scenesMenu);
        
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        let identifyItem = new PopupMenu.PopupMenuItem('Identify Light');
        identifyItem.connect('activate', () => {
            this._identifyLight();
        });
        this.menu.addMenuItem(identifyItem);
        
        this._statusItem = new PopupMenu.PopupMenuItem('Status: Disconnected', { reactive: false });
        this.menu.addMenuItem(this._statusItem);
    }
    
    _connectToDBus() {
        try {
            this._proxy = new ElgatoDBusProxy(
                Gio.DBus.session,
                DBUS_NAME,
                DBUS_PATH,
                (proxy, error) => {
                    if (error) {
                        log(`Failed to connect to Elgato D-Bus service: ${error}`);
                        this._setConnectionStatus(false);
                    } else {
                        this._setConnectionStatus(true);
                        this._updateStatus();
                    }
                }
            );
        } catch (e) {
            log(`Error creating D-Bus proxy: ${e}`);
            this._setConnectionStatus(false);
        }
    }
    
    _setConnectionStatus(connected) {
        this._isConnected = connected;
        if (connected) {
            this._statusItem.label.text = 'Status: Connected';
            this._icon.icon_name = 'weather-clear-symbolic';
            this._powerSwitch.setSensitive(true);
            this._brightnessSlider.reactive = true;
            this._temperatureSlider.reactive = true;
        } else {
            this._statusItem.label.text = 'Status: Disconnected';
            this._icon.icon_name = 'dialog-error-symbolic';
            this._powerSwitch.setSensitive(false);
            this._brightnessSlider.reactive = false;
            this._temperatureSlider.reactive = false;
        }
    }
    
    _updateStatus() {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.GetStatusRemote((result, error) => {
            if (error) {
                log(`Failed to get status: ${error}`);
                return;
            }
            
            let [is_on, brightness, temperature] = result;
            this._isOn = is_on;
            this._brightness = brightness;
            this._temperature = temperature;
            
            this._powerSwitch.setToggleState(is_on);
            this._brightnessSlider.value = brightness / 100;
            this._brightnessValueLabel.text = `${brightness}%`;
            
            let tempNorm = (temperature - 2900) / (7000 - 2900);
            this._temperatureSlider.value = tempNorm;
            this._temperatureValueLabel.text = `${temperature}K`;
            
            if (is_on) {
                this._icon.icon_name = 'weather-clear-symbolic';
            } else {
                this._icon.icon_name = 'weather-clear-night-symbolic';
            }
        });
    }
    
    _toggleLight(state) {
        if (!this._isConnected || !this._proxy) return;
        
        let method = state ? 'TurnOnRemote' : 'TurnOffRemote';
        this._proxy[method]((result, error) => {
            if (error) {
                log(`Failed to toggle light: ${error}`);
            } else {
                this._updateStatus();
            }
        });
    }
    
    _setBrightness(brightness) {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.SetBrightnessRemote(brightness, (result, error) => {
            if (error) {
                log(`Failed to set brightness: ${error}`);
            }
        });
    }
    
    _setTemperature(kelvin) {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.SetTemperatureRemote(kelvin, (result, error) => {
            if (error) {
                log(`Failed to set temperature: ${error}`);
            }
        });
    }
    
    _applyScene(scene) {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.ApplySceneRemote(scene, (result, error) => {
            if (error) {
                log(`Failed to apply scene: ${error}`);
            } else {
                this._updateStatus();
            }
        });
    }
    
    _identifyLight() {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.IdentifyRemote((result, error) => {
            if (error) {
                log(`Failed to identify light: ${error}`);
            }
        });
    }
    
    _onDestroy() {
        this._proxy = null;
    }
});

class Extension {
    constructor() {
        this._indicator = null;
    }
    
    enable() {
        this._indicator = new ElgatoIndicator();
        Main.panel.addToStatusArea('elgato-indicator', this._indicator);
    }
    
    disable() {
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }
}

function init() {
    return new Extension();
}