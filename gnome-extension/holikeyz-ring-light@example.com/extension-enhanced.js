import GObject from 'gi://GObject';
import St from 'gi://St';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';
import Pango from 'gi://Pango';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';

const DBUS_NAME = 'com.holikeyz.RingLight';
const DBUS_PATH = '/com/holikeyz/RingLight';
const DBUS_INTERFACE = 'com.holikeyz.RingLight.Control';

const HolikeyzDBusInterface = `
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
    <method name="TurnOnLight">
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="TurnOffLight">
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="ToggleLight">
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetBrightness">
      <arg type="y" direction="in" name="brightness"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetBrightnessLight">
      <arg type="y" direction="in" name="brightness"/>
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetTemperature">
      <arg type="u" direction="in" name="kelvin"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetTemperatureLight">
      <arg type="u" direction="in" name="kelvin"/>
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="GetStatus">
      <arg type="b" direction="out" name="is_on"/>
      <arg type="y" direction="out" name="brightness"/>
      <arg type="u" direction="out" name="temperature"/>
    </method>
    <method name="GetNumLights">
      <arg type="y" direction="out" name="num_lights"/>
    </method>
    <method name="GetAllLightsStatus">
      <arg type="ab" direction="out" name="is_on_array"/>
      <arg type="ay" direction="out" name="brightness_array"/>
      <arg type="au" direction="out" name="temperature_array"/>
    </method>
    <method name="GetLightStatus">
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="is_on"/>
      <arg type="y" direction="out" name="brightness"/>
      <arg type="u" direction="out" name="temperature"/>
    </method>
    <method name="GetAccessoryInfo">
      <arg type="s" direction="out" name="product_name"/>
      <arg type="s" direction="out" name="firmware_version"/>
      <arg type="s" direction="out" name="serial_number"/>
      <arg type="u" direction="out" name="firmware_build"/>
      <arg type="s" direction="out" name="display_name"/>
      <arg type="as" direction="out" name="features"/>
    </method>
    <method name="GetSettings">
      <arg type="y" direction="out" name="power_on_behavior"/>
      <arg type="y" direction="out" name="power_on_brightness"/>
      <arg type="u" direction="out" name="power_on_temperature"/>
      <arg type="u" direction="out" name="switch_on_ms"/>
      <arg type="u" direction="out" name="switch_off_ms"/>
      <arg type="u" direction="out" name="color_change_ms"/>
    </method>
    <method name="SetSettings">
      <arg type="y" direction="in" name="power_on_behavior"/>
      <arg type="y" direction="in" name="power_on_brightness"/>
      <arg type="u" direction="in" name="power_on_temperature"/>
      <arg type="u" direction="in" name="switch_on_ms"/>
      <arg type="u" direction="in" name="switch_off_ms"/>
      <arg type="u" direction="in" name="color_change_ms"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetPowerOnSettings">
      <arg type="y" direction="in" name="behavior"/>
      <arg type="y" direction="in" name="brightness"/>
      <arg type="u" direction="in" name="temperature"/>
      <arg type="b" direction="out" name="success"/>
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
    <property name="NumLights" type="y" access="read"/>
  </interface>
</node>
`;

const HolikeyzDBusProxy = Gio.DBusProxy.makeProxyWrapper(HolikeyzDBusInterface);

// Scene presets with their settings
const SCENE_PRESETS = [
    { id: 'daylight', name: 'Daylight', icon: '☀️', brightness: 80, temperature: 5600 },
    { id: 'warm', name: 'Warm', icon: '🕯️', brightness: 60, temperature: 3200 },
    { id: 'cool', name: 'Cool', icon: '❄️', brightness: 70, temperature: 6500 },
    { id: 'reading', name: 'Reading', icon: '📖', brightness: 90, temperature: 4500 },
    { id: 'video', name: 'Video', icon: '🎥', brightness: 75, temperature: 5000 },
    { id: 'relax', name: 'Relax', icon: '😌', brightness: 40, temperature: 2900 },
];

const RingLightIndicator = GObject.registerClass(
class RingLightIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.0, 'Ring Light Controller');
        
        // Initialize properties
        this._dbusProxy = null;
        this._isOn = false;
        this._brightness = 50;
        this._temperature = 4500;
        this._numLights = 1;
        this._currentScene = null;
        this._sliderUpdateTimeout = null;
        this._ignoreSliderUpdate = false;
        this._lightItems = [];
        
        // Create panel icon
        let box = new St.BoxLayout({ style_class: 'panel-status-menu-box' });
        this._icon = new St.Icon({
            icon_name: 'dialog-information-symbolic',
            style_class: 'system-status-icon',
        });
        box.add_child(this._icon);
        this.add_child(box);
        
        // Connect to D-Bus
        this._connectToDbus();
        
        // Build menu
        this._buildMenu();
        
        // Initial status update
        this._updateStatus();
    }
    
    _connectToDbus() {
        try {
            this._dbusProxy = new HolikeyzDBusProxy(
                Gio.DBus.session,
                DBUS_NAME,
                DBUS_PATH
            );
            
            this._dbusProxy.connect('g-properties-changed', () => {
                this._updateStatus();
            });
        } catch (e) {
            log(`Failed to connect to D-Bus: ${e}`);
        }
    }
    
    _buildMenu() {
        // Clear existing menu
        this.menu.removeAll();
        
        // Title
        let title = new PopupMenu.PopupMenuItem('Ring Light Controller', { 
            reactive: false,
            can_focus: false 
        });
        title.label.add_style_class_name('ring-light-title');
        this.menu.addMenuItem(title);
        
        // Separator
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Power toggle
        this._powerSwitch = new PopupMenu.PopupSwitchMenuItem('Power', false);
        this._powerSwitch.connect('toggled', (item) => {
            this._togglePower(item.state);
        });
        this.menu.addMenuItem(this._powerSwitch);
        
        // Separator
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Check number of lights and create controls
        this._checkNumLights();
    }
    
    _checkNumLights() {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.GetNumLightsRemote((result) => {
            if (result && result[0]) {
                this._numLights = result[0];
                this._buildLightControls();
            }
        });
    }
    
    _buildLightControls() {
        // Remove old light controls if any
        this._lightItems.forEach(item => this.menu.box.remove_child(item));
        this._lightItems = [];
        
        if (this._numLights > 1) {
            // Multiple lights - show tabs or individual controls
            this._buildMultiLightControls();
        } else {
            // Single light - show simple controls
            this._buildSingleLightControls();
        }
        
        // Add common items at the end
        this._addCommonMenuItems();
    }
    
    _buildSingleLightControls() {
        // Brightness slider
        let brightnessItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        let brightnessBox = new St.BoxLayout({ vertical: false, x_expand: true });
        
        let brightnessLabel = new St.Label({ 
            text: 'Brightness',
            y_align: Clutter.ActorAlign.CENTER 
        });
        brightnessBox.add_child(brightnessLabel);
        
        this._brightnessSlider = new Slider.Slider(0.5);
        this._brightnessSlider.connect('notify::value', () => {
            this._onBrightnessChanged();
        });
        brightnessBox.add_child(this._brightnessSlider);
        
        this._brightnessValue = new St.Label({ 
            text: '50%',
            y_align: Clutter.ActorAlign.CENTER 
        });
        brightnessBox.add_child(this._brightnessValue);
        
        brightnessItem.add_child(brightnessBox);
        this.menu.addMenuItem(brightnessItem);
        this._lightItems.push(brightnessItem);
        
        // Temperature slider
        let tempItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        let tempBox = new St.BoxLayout({ vertical: false, x_expand: true });
        
        let tempLabel = new St.Label({ 
            text: 'Temperature',
            y_align: Clutter.ActorAlign.CENTER 
        });
        tempBox.add_child(tempLabel);
        
        this._tempSlider = new Slider.Slider(0.5);
        this._tempSlider.connect('notify::value', () => {
            this._onTemperatureChanged();
        });
        tempBox.add_child(this._tempSlider);
        
        this._tempValue = new St.Label({ 
            text: '4500K',
            y_align: Clutter.ActorAlign.CENTER 
        });
        tempBox.add_child(this._tempValue);
        
        tempItem.add_child(tempBox);
        this.menu.addMenuItem(tempItem);
        this._lightItems.push(tempItem);
    }
    
    _buildMultiLightControls() {
        // Header for multi-light control
        let headerItem = new PopupMenu.PopupMenuItem(`${this._numLights} Lights Detected`, { 
            reactive: false,
            can_focus: false 
        });
        this.menu.addMenuItem(headerItem);
        this._lightItems.push(headerItem);
        
        // Master controls for all lights
        let masterSection = new PopupMenu.PopupMenuSection();
        let masterLabel = new PopupMenu.PopupMenuItem('All Lights', { 
            reactive: false,
            can_focus: false 
        });
        masterSection.addMenuItem(masterLabel);
        
        // Master brightness
        let brightnessItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        let brightnessBox = new St.BoxLayout({ vertical: false, x_expand: true });
        
        let brightnessLabel = new St.Label({ 
            text: 'Brightness',
            y_align: Clutter.ActorAlign.CENTER 
        });
        brightnessBox.add_child(brightnessLabel);
        
        this._brightnessSlider = new Slider.Slider(0.5);
        this._brightnessSlider.connect('notify::value', () => {
            this._onBrightnessChanged();
        });
        brightnessBox.add_child(this._brightnessSlider);
        
        this._brightnessValue = new St.Label({ 
            text: '50%',
            y_align: Clutter.ActorAlign.CENTER 
        });
        brightnessBox.add_child(this._brightnessValue);
        
        brightnessItem.add_child(brightnessBox);
        masterSection.addMenuItem(brightnessItem);
        
        // Master temperature
        let tempItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        let tempBox = new St.BoxLayout({ vertical: false, x_expand: true });
        
        let tempLabel = new St.Label({ 
            text: 'Temperature',
            y_align: Clutter.ActorAlign.CENTER 
        });
        tempBox.add_child(tempLabel);
        
        this._tempSlider = new Slider.Slider(0.5);
        this._tempSlider.connect('notify::value', () => {
            this._onTemperatureChanged();
        });
        tempBox.add_child(this._tempSlider);
        
        this._tempValue = new St.Label({ 
            text: '4500K',
            y_align: Clutter.ActorAlign.CENTER 
        });
        tempBox.add_child(this._tempValue);
        
        tempItem.add_child(tempBox);
        masterSection.addMenuItem(tempItem);
        
        this.menu.addMenuItem(masterSection);
        this._lightItems.push(masterSection);
        
        // Separator
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Individual light toggles
        let individualSection = new PopupMenu.PopupMenuSection();
        let individualLabel = new PopupMenu.PopupMenuItem('Individual Lights', { 
            reactive: false,
            can_focus: false 
        });
        individualSection.addMenuItem(individualLabel);
        
        for (let i = 0; i < this._numLights; i++) {
            let lightSwitch = new PopupMenu.PopupSwitchMenuItem(`Light ${i + 1}`, false);
            lightSwitch.connect('toggled', (item) => {
                this._toggleLight(i, item.state);
            });
            individualSection.addMenuItem(lightSwitch);
            
            // Store reference for updating
            if (!this._individualLightSwitches) {
                this._individualLightSwitches = [];
            }
            this._individualLightSwitches[i] = lightSwitch;
        }
        
        this.menu.addMenuItem(individualSection);
        this._lightItems.push(individualSection);
    }
    
    _addCommonMenuItems() {
        // Separator before scenes
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Scene presets
        let sceneLabel = new PopupMenu.PopupMenuItem('Scene Presets', { 
            reactive: false,
            can_focus: false 
        });
        this.menu.addMenuItem(sceneLabel);
        
        this._sceneButtons = {};
        let sceneBox = new St.BoxLayout({ 
            vertical: false, 
            x_expand: true,
            style: 'spacing: 6px; padding: 10px;'
        });
        
        SCENE_PRESETS.forEach(scene => {
            let button = new St.Button({
                label: scene.icon,
                style_class: 'scene-button',
                x_expand: false,
                can_focus: true,
                track_hover: true
            });
            
            button.connect('clicked', () => {
                this._applyScene(scene);
            });
            
            this._sceneButtons[scene.id] = button;
            sceneBox.add_child(button);
        });
        
        let sceneItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        sceneItem.add_child(sceneBox);
        this.menu.addMenuItem(sceneItem);
        
        // Separator
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Settings section
        let settingsSection = new PopupMenu.PopupSubMenuMenuItem('Settings', true);
        
        // Power-on behavior
        let powerOnItem = new PopupMenu.PopupMenuItem('Power-On Behavior');
        powerOnItem.connect('activate', () => {
            this._showPowerOnSettings();
        });
        settingsSection.menu.addMenuItem(powerOnItem);
        
        // Device info
        let infoItem = new PopupMenu.PopupMenuItem('Device Info');
        infoItem.connect('activate', () => {
            this._showDeviceInfo();
        });
        settingsSection.menu.addMenuItem(infoItem);
        
        // Identify
        let identifyItem = new PopupMenu.PopupMenuItem('Identify Light');
        identifyItem.connect('activate', () => {
            this._identifyLight();
        });
        settingsSection.menu.addMenuItem(identifyItem);
        
        this.menu.addMenuItem(settingsSection);
    }
    
    _onBrightnessChanged() {
        if (this._ignoreSliderUpdate) return;
        
        let brightness = Math.round(this._brightnessSlider.value * 100);
        this._brightnessValue.text = `${brightness}%`;
        
        // Debounce the D-Bus call
        if (this._sliderUpdateTimeout) {
            GLib.source_remove(this._sliderUpdateTimeout);
        }
        
        this._sliderUpdateTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 50, () => {
            if (this._dbusProxy) {
                this._dbusProxy.SetBrightnessRemote(brightness, (result) => {
                    if (result && result[0]) {
                        this._updateSceneButtons(null);
                    }
                });
            }
            this._sliderUpdateTimeout = null;
            return GLib.SOURCE_REMOVE;
        });
    }
    
    _onTemperatureChanged() {
        if (this._ignoreSliderUpdate) return;
        
        let temp = Math.round(2900 + (this._tempSlider.value * 4100));
        this._tempValue.text = `${temp}K`;
        
        // Debounce the D-Bus call
        if (this._sliderUpdateTimeout) {
            GLib.source_remove(this._sliderUpdateTimeout);
        }
        
        this._sliderUpdateTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 50, () => {
            if (this._dbusProxy) {
                this._dbusProxy.SetTemperatureRemote(temp, (result) => {
                    if (result && result[0]) {
                        this._updateSceneButtons(null);
                    }
                });
            }
            this._sliderUpdateTimeout = null;
            return GLib.SOURCE_REMOVE;
        });
    }
    
    _togglePower(state) {
        if (!this._dbusProxy) return;
        
        if (state) {
            this._dbusProxy.TurnOnRemote((result) => {
                if (result && result[0]) {
                    this._updateStatus();
                }
            });
        } else {
            this._dbusProxy.TurnOffRemote((result) => {
                if (result && result[0]) {
                    this._updateStatus();
                }
            });
        }
    }
    
    _toggleLight(index, state) {
        if (!this._dbusProxy) return;
        
        if (state) {
            this._dbusProxy.TurnOnLightRemote(index, (result) => {
                if (result && result[0]) {
                    this._updateStatus();
                }
            });
        } else {
            this._dbusProxy.TurnOffLightRemote(index, (result) => {
                if (result && result[0]) {
                    this._updateStatus();
                }
            });
        }
    }
    
    _applyScene(scene) {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.ApplySceneRemote(scene.id, (result) => {
            if (result && result[0]) {
                this._currentScene = scene.id;
                this._updateSceneButtons(scene.id);
                
                // Update sliders immediately to match scene
                this._ignoreSliderUpdate = true;
                this._brightnessSlider.value = scene.brightness / 100;
                this._brightnessValue.text = `${scene.brightness}%`;
                this._tempSlider.value = (scene.temperature - 2900) / 4100;
                this._tempValue.text = `${scene.temperature}K`;
                this._ignoreSliderUpdate = false;
                
                this._updateStatus();
            }
        });
    }
    
    _updateSceneButtons(activeSceneId) {
        Object.keys(this._sceneButtons).forEach(sceneId => {
            let button = this._sceneButtons[sceneId];
            if (sceneId === activeSceneId) {
                button.add_style_class_name('scene-button-active');
            } else {
                button.remove_style_class_name('scene-button-active');
            }
        });
    }
    
    _showPowerOnSettings() {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.GetSettingsRemote((result) => {
            if (result) {
                let [behavior, brightness, temperature] = result;
                
                // Create dialog to edit settings
                let dialog = new PopupMenu.PopupMenuSection();
                
                let behaviorItem = new PopupMenu.PopupMenuItem(
                    `Power-On: ${behavior === 1 ? 'Last State' : 'Default'}`
                );
                behaviorItem.connect('activate', () => {
                    // Toggle behavior
                    let newBehavior = behavior === 1 ? 0 : 1;
                    this._dbusProxy.SetPowerOnSettingsRemote(
                        newBehavior, brightness, temperature,
                        (result) => {
                            if (result && result[0]) {
                                Main.notify('Ring Light', 'Power-on settings updated');
                            }
                        }
                    );
                });
                
                dialog.addMenuItem(behaviorItem);
                // Could add more UI for brightness/temperature settings
            }
        });
    }
    
    _showDeviceInfo() {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.GetAccessoryInfoRemote((result) => {
            if (result) {
                let [productName, firmwareVersion, serialNumber, firmwareBuild, displayName, features] = result;
                
                let message = `Product: ${productName}\n` +
                            `Name: ${displayName}\n` +
                            `Firmware: ${firmwareVersion} (build ${firmwareBuild})\n` +
                            `Serial: ${serialNumber}\n` +
                            `Features: ${features.join(', ')}`;
                
                Main.notify('Ring Light Info', message);
            }
        });
    }
    
    _identifyLight() {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.IdentifyRemote((result) => {
            if (result && result[0]) {
                Main.notify('Ring Light', 'Light should be flashing');
            }
        });
    }
    
    _updateStatus() {
        if (!this._dbusProxy) return;
        
        // Update status for backward compatibility with single light
        this._dbusProxy.GetStatusRemote((result) => {
            if (result) {
                let [isOn, brightness, temperature] = result;
                this._isOn = isOn;
                this._brightness = brightness;
                this._temperature = temperature;
                
                // Update UI
                this._powerSwitch.setToggleState(isOn);
                this._icon.icon_name = isOn ? 'dialog-information-symbolic' : 'dialog-information-symbolic';
                
                // Update sliders without triggering callbacks
                this._ignoreSliderUpdate = true;
                if (this._brightnessSlider) {
                    this._brightnessSlider.value = brightness / 100;
                    this._brightnessValue.text = `${brightness}%`;
                }
                if (this._tempSlider) {
                    this._tempSlider.value = (temperature - 2900) / 4100;
                    this._tempValue.text = `${temperature}K`;
                }
                this._ignoreSliderUpdate = false;
                
                // Check which scene matches current settings
                let matchingScene = SCENE_PRESETS.find(scene => 
                    Math.abs(scene.brightness - brightness) < 5 &&
                    Math.abs(scene.temperature - temperature) < 200
                );
                this._updateSceneButtons(matchingScene ? matchingScene.id : null);
            }
        });
        
        // If we have multiple lights, update their individual states
        if (this._numLights > 1 && this._individualLightSwitches) {
            this._dbusProxy.GetAllLightsStatusRemote((result) => {
                if (result) {
                    let [isOnArray, brightnessArray, temperatureArray] = result;
                    
                    for (let i = 0; i < Math.min(isOnArray.length, this._individualLightSwitches.length); i++) {
                        this._individualLightSwitches[i].setToggleState(isOnArray[i]);
                    }
                }
            });
        }
    }
    
    destroy() {
        if (this._sliderUpdateTimeout) {
            GLib.source_remove(this._sliderUpdateTimeout);
            this._sliderUpdateTimeout = null;
        }
        
        super.destroy();
    }
});

// Extension entry points
let ringLightIndicator;

export default class RingLightExtension {
    constructor(uuid) {
        this._uuid = uuid;
    }
    
    enable() {
        ringLightIndicator = new RingLightIndicator();
        Main.panel.addToStatusArea(this._uuid, ringLightIndicator);
    }
    
    disable() {
        if (ringLightIndicator) {
            ringLightIndicator.destroy();
            ringLightIndicator = null;
        }
    }
}